use crate::state::{mz_state::MzState, tips::push_block};
use alloy_rlp::{Decodable, Encodable};
use anyhow::Context;
use futures::stream::StreamExt;
use libp2p::{
    Multiaddr, gossipsub, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use log::info;
use mvm::models::block::Block;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{io, io::AsyncBufReadExt, select, sync::mpsc};
use utils::time::get_current_time_ms;

#[derive(NetworkBehaviour)]
struct MBehaviour {
    gossipsub: gossipsub::Behaviour,
}

// todo: check block pow and expired
#[allow(clippy::unwrap_used)]
pub async fn spawn_gossip_thread(mz_state: MzState) -> mpsc::Sender<Block> {
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

            Ok(MBehaviour { gossipsub })
        })
        .unwrap()
        .build();

    let (tx, mut rx) = mpsc::channel::<Block>(64);

    let message_topic = gossipsub::IdentTopic::new("test-message");
    let message_topic_hash = message_topic.hash();

    let block_topic = gossipsub::IdentTopic::new("block");
    let block_topic_hash = block_topic.hash();

    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&message_topic)
        .unwrap();
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&block_topic)
        .unwrap();

    if let (Some(addr), Some(peer_id)) = (
        mz_state.config.bootstrap_addr.clone(),
        mz_state.config.bootstrap_peer_id.clone(),
    ) {
        let bootstrap_addr: Multiaddr = addr.parse().unwrap();
        let bootstrap_peer_id = peer_id.parse().unwrap();
        swarm.dial(bootstrap_addr).unwrap();
        swarm
            .behaviour_mut()
            .gossipsub
            .add_explicit_peer(&bootstrap_peer_id);
    }

    // 监听地址（引导节点用固定地址，普通节点用动态地址）
    let listen_addr = format!("/ip6/::/tcp/{}", mz_state.config.gossip_tcp_port)
        .parse::<Multiaddr>()
        .unwrap();
    swarm.listen_on(listen_addr).unwrap();
    let quic_listen_addr = format!("/ip6/::/udp/{}/quic-v1", mz_state.config.gossip_udp_port)
        .parse::<Multiaddr>()
        .unwrap();
    swarm.listen_on(quic_listen_addr).unwrap();

    info!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    tokio::spawn(async move {
        loop {
            select! {
                Some(block) = rx.recv() => {
                    let mut block_bytes = Vec::new();
                    block.encode(&mut block_bytes);
                    if let Err(e) = swarm
                        .behaviour_mut().gossipsub
                        .publish(block_topic.clone(), block_bytes) {
                            info!("Publish error: {e:?}");
                    }
                }
                Ok(Some(line)) = stdin.next_line() => {
                    if let Err(e) = swarm
                        .behaviour_mut().gossipsub
                        .publish(message_topic.clone(), line.as_bytes()) {
                            info!("Publish error: {e:?}");
                    }
                }
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(MBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => {
                        if message.topic == message_topic_hash {
                            info!(
                                "Got message: '{}' with id: {id} from peer: {peer_id}",
                                String::from_utf8_lossy(&message.data),
                            );
                        }
                        if message.topic == block_topic_hash {
                            if let Err(e) = process_block(&message.data, &mz_state) {
                                info!("Process block error: {e:?}");
                            }
                        }
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Local node is listening on {address}");
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Connected to peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        info!("Disconnected from peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                    _ => {}
                }
            }
        }
    });

    tx
}

fn process_block(mut block_bytes: &[u8], mz_state: &MzState) -> anyhow::Result<()> {
    let block: Block =
        Decodable::decode(&mut block_bytes).with_context(|| "Failed to decode block")?;
    let now = get_current_time_ms();
    if (now as i64 - block.inner.header.pow_header.now_timestamp_ms as i64).abs() > 1000 * 300 {
        return Err(anyhow::anyhow!(
            "Block timestamp is too old: {}",
            block.inner.header.pow_header.now_timestamp_ms
        ));
    }
    push_block(block, mz_state).with_context(|| "Failed to push block")?;

    Ok(())
}
