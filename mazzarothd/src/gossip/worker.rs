use crossbeam::channel::{Receiver, Sender};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Mutex,
};

use crate::gossip::{
    channel_block::ChannelBlock,
    ipv6_udp::{Ipv6UdpRecv, Ipv6UdpSend, SendAction},
};

lazy_static::lazy_static! {
    pub static ref UDP_RECV: Mutex<Ipv6UdpRecv> = Mutex::new(Ipv6UdpRecv::new());
    pub static ref UDP_SEND: Mutex<Ipv6UdpSend> = Mutex::new(Ipv6UdpSend::new());
}

pub struct GossipBlock {
    pub data: ChannelBlock,
    pub src: SocketAddr,
}

pub fn spawn_std_thread_recv_loop(udp_socket: UdpSocket) -> anyhow::Result<Receiver<GossipBlock>> {
    let (tx, rx) = crossbeam::channel::bounded(1024);
    std::thread::spawn(move || {
        let mut buf = [0; 65535];
        loop {
            let (len, src) = udp_socket.recv_from(&mut buf).unwrap();
            let data = &buf[..len];
            let ans = UDP_RECV.lock().unwrap().recv(src, data);
            if let Some(ans) = ans {
                tx.send(GossipBlock { data: ans, src }).unwrap();
            }
        }
    });
    Ok(rx)
}

pub fn spawn_std_thread_send_loop(udp_socket: UdpSocket) -> anyhow::Result<Sender<SendAction>> {
    let (action_tx, action_rx) = crossbeam::channel::bounded(1024);
    std::thread::spawn(move || {
        let mut udp_socket = udp_socket;
        loop {
            let action = action_rx.recv().unwrap();
            UDP_SEND
                .lock()
                .unwrap()
                .action(action, &mut udp_socket)
                .unwrap();
        }
    });
    Ok(action_tx)
}
