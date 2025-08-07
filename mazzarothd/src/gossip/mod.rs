use anyhow::Context;
use consensus::types::BlockKey;
use crossbeam::channel::Receiver;
use log::warn;
use mvm::models::block::Block;

use crate::{
    gossip::{
        udp::{SendAction, UdpRecv, UdpSend},
        worker::{spawn_std_thread_recv_loop, spawn_std_thread_send_loop, GossipBlock},
    }, MAZZAROTH_UDP_PORT, MAZZAROTH_UDP_PORT_DEFAULT
};
use std::{net::UdpSocket, sync::Mutex};

pub mod channel_block;
pub mod proto;
/// @ fawkes There are tow point can be optimized:
/// 1. send many udp packet at once
/// 2. optimize the reed solomon algorithm
pub mod udp;
pub mod worker;

lazy_static::lazy_static! {
    pub static ref UDP_RECV: Mutex<UdpRecv> = Mutex::new(UdpRecv::new());
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
    let (tx_send, rx_send) = crossbeam::channel::bounded(1024);

    std::thread::spawn(move || {
        loop {
            let mut has_action = false;
            let gossip_block = recv_udp_recv.try_recv();
            match gossip_block {
                Ok(gossip_block) => {
                    has_action = true;
                    match process_gossip_block(gossip_block) {
                        Ok(Some(send_action)) => {
                            tx_send.send(send_action).unwrap();
                        }
                        Ok(None) => {}
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


fn process_gossip_block(gossip_block: GossipBlock) -> anyhow::Result<Option<SendAction>> {
    

    Ok(None)
}