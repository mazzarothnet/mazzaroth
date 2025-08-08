use anyhow::Context;
use consensus::types::BlockKey;
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use log::{info, warn};
use mvm::models::block::Block;

use crate::{
    MAZZAROTH_UDP_PORT, MAZZAROTH_UDP_PORT_DEFAULT, SEED_NODE_ADDR, SEED_NODE_ADDR_DEFAULT,
    gossip::{
        channel_block::ChannelBlock,
        proto::{LISTEN_TOPIC_LEN, PING_TOPIC, PONG_TOPIC, REQ_LISTEN_LIST_TOPIC},
        udp::{SendAction, UdpRecv, UdpSend},
        worker::{GossipBlock, spawn_std_thread_recv_loop, spawn_std_thread_send_loop},
    },
};
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::Mutex,
};

pub mod channel_block;
pub mod proto;
/// @ fawkes There are tow point can be optimized:
/// 1. send many udp packet at once
/// 2. optimize the reed solomon algorithm
pub mod udp;
pub mod worker;

lazy_static::lazy_static! {
    pub static ref UDP_RECV: Mutex<UdpRecv> = Mutex::new(UdpRecv::new(LISTEN_TOPIC_LEN));
    pub static ref UDP_SEND: Mutex<UdpSend> = Mutex::new(UdpSend::new());
}

pub enum GossipAction {
    Send(Block),
    Req(BlockKey),
}

pub fn spawn_gossip_logic() -> anyhow::Result<(Receiver<Block>, Sender<GossipAction>)> {
    let udp_port = std::env::var(MAZZAROTH_UDP_PORT)
        .unwrap_or_else(|_| MAZZAROTH_UDP_PORT_DEFAULT.to_string());
    let udp_port = udp_port
        .parse::<u16>()
        .with_context(|| "Failed to parse UDP port")?;
    let ipv6_addr = format!("[::]:{}", udp_port);
    let udp_socket = UdpSocket::bind(ipv6_addr).with_context(|| "Failed to bind UDP socket")?;
    let recv_udp_socket = udp_socket
        .try_clone()
        .with_context(|| "Failed to clone UDP socket")?;
    let recv_udp_recv = spawn_std_thread_recv_loop(recv_udp_socket)
        .with_context(|| "Failed to spawn UDP recv thread")?;
    let send_udp_send = spawn_std_thread_send_loop(udp_socket)
        .with_context(|| "Failed to spawn UDP send thread")?;
    let (tx_recv, rx_recv) = crossbeam::channel::bounded(1024);
    let (tx_send, rx_send): (Sender<GossipAction>, Receiver<GossipAction>) =
        crossbeam::channel::bounded(1024);
    let init_ping = get_init_ping()?;
    send_udp_send.send(init_ping).unwrap();

    std::thread::spawn(move || {
        loop {
            let mut listen_list = HashMap::new();
            let mut has_action = false;
            let gossip_block = recv_udp_recv.try_recv();
            match gossip_block {
                Ok(gossip_block) => {
                    has_action = true;
                    match process_gossip_block(gossip_block, &mut listen_list) {
                        Ok(send_actions) => {
                            for send_action in send_actions {
                                send_udp_send.send(send_action).unwrap();
                            }
                        }
                        Err(e) => {
                            warn!("Failed to process gossip block: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    if e != TryRecvError::Empty {
                        warn!("Failed to receive gossip block: {:?}", e);
                    }
                }
            }

            if !has_action {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    });

    Ok((rx_recv, tx_send))
}

fn get_init_ping() -> anyhow::Result<SendAction> {
    let seed_node_addr =
        std::env::var(SEED_NODE_ADDR).unwrap_or_else(|_| SEED_NODE_ADDR_DEFAULT.to_string());
    let seed_node_addr = seed_node_addr
        .parse::<SocketAddr>()
        .with_context(|| "Failed to parse seed node address")?;

    let send_action = SendAction::Send(
        ChannelBlock {
            topic_id: PING_TOPIC,
            data: "ping".as_bytes().to_vec(),
        },
        seed_node_addr,
    );

    Ok(send_action)
}

fn process_gossip_block(
    gossip_block: GossipBlock,
    listen_list: &mut HashMap<SocketAddr, u32>,
) -> anyhow::Result<Vec<SendAction>> {
    let mut send_actions = Vec::new();
    match gossip_block.data.topic_id {
        PING_TOPIC => {
            info!("PING from {}", gossip_block.src);
            let addr = gossip_block.src.to_string();
            let send_action = SendAction::Send(
                ChannelBlock {
                    topic_id: PONG_TOPIC,
                    data: addr.as_bytes().to_vec(),
                },
                gossip_block.src,
            );
            send_actions.push(send_action);
        }
        PONG_TOPIC => {
            let from_addr = gossip_block.src;
            let me_addr = String::from_utf8(gossip_block.data.data)?;
            info!("PONG from {} to {}", from_addr, me_addr);
            if listen_list.is_empty() {
                send_actions.push(SendAction::Send(
                    ChannelBlock {
                        topic_id: REQ_LISTEN_LIST_TOPIC,
                        data: Vec::new(),
                    },
                    from_addr,
                ))
            }
        }
        REQ_LISTEN_LIST_TOPIC => {
            info!("REQ_LISTEN_LIST_TOPIC from {}", gossip_block.src);
        }
        _ => {}
    };

    Ok(send_actions)
}
