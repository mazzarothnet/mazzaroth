use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};
use futures::stream::StreamExt;
use libp2p::{
    Multiaddr, gossipsub, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::{io, io::AsyncBufReadExt, select};

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
#[allow(clippy::expect_used)]
#[tokio::main]
async fn main() {
    // 解析命令行参数
    let is_bootstrap = std::env::args().any(|arg| arg == "--bootstrap");
    let bootstrap_addr = std::env::var("BOOTSTRAP_ADDR").ok();
    let bootstrap_peer_id = std::env::var("BOOTSTRAP_PEER_ID").ok();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .unwrap()
        .with_quic()
        .with_behaviour(|key| {
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(io::Error::other)
                .unwrap();

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .unwrap();

            Ok(MyBehaviour { gossipsub })
        })
        .unwrap()
        .build();

    let topic = gossipsub::IdentTopic::new("test-net");
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

    // 根据节点类型配置
    if !is_bootstrap {
        // 普通节点：连接到引导节点
        if let (Some(addr), Some(peer_id)) = (bootstrap_addr, bootstrap_peer_id) {
            let bootstrap_addr: Multiaddr = addr.parse().expect("Invalid bootstrap address");
            let bootstrap_peer_id = peer_id.parse().expect("Invalid PeerId");
            swarm.dial(bootstrap_addr).unwrap();
            swarm
                .behaviour_mut()
                .gossipsub
                .add_explicit_peer(&bootstrap_peer_id);
        } else {
            println!(
                "Warning: BOOTSTRAP_ADDR and BOOTSTRAP_PEER_ID must be set for non-bootstrap nodes"
            );
        }
    }

    // 监听地址（引导节点用固定地址，普通节点用动态地址）
    let listen_addr = if is_bootstrap {
        "/ip6/::/tcp/4001".parse::<Multiaddr>().unwrap()
    } else {
        "/ip6/::/tcp/0".parse::<Multiaddr>().unwrap()
    };
    swarm.listen_on(listen_addr).unwrap();
    let quic_listen_addr = if is_bootstrap {
        "/ip6/::/udp/4001/quic-v1".parse::<Multiaddr>().unwrap()
    } else {
        "/ip6/::/udp/0/quic-v1".parse::<Multiaddr>().unwrap()
    };
    swarm.listen_on(quic_listen_addr).unwrap();

    if is_bootstrap {
        println!("Bootstrap node PeerId: {}", swarm.local_peer_id());
    }

    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), line.as_bytes()) {
                    println!("Publish error: {e:?}");
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => println!(
                    "Got message: '{}' with id: {id} from peer: {peer_id}",
                    String::from_utf8_lossy(&message.data),
                ),
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("Connected to peer: {peer_id}");
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    println!("Disconnected from peer: {peer_id}");
                    swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                }
                _ => {}
            }
        }
    }
}
