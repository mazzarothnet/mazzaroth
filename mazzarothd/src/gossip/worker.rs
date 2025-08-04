use crossbeam::channel::{Receiver, Sender};
use std::net::{SocketAddr, UdpSocket};

use crate::gossip::{
    channel_block::ChannelBlock,
    ipv6_udp::{Ipv6UdpRecv, Ipv6UdpSend, RecvAction, SendAction},
};

pub struct GossipBlock {
    pub data: ChannelBlock,
    pub src: SocketAddr,
}

pub fn spawn_std_thread_recv_loop(
    udp_socket: UdpSocket,
) -> anyhow::Result<(Receiver<GossipBlock>, Sender<RecvAction>)> {
    let (action_tx, action_rx) = crossbeam::channel::bounded(1024);
    let (tx, rx) = crossbeam::channel::bounded(1024);
    std::thread::spawn(move || {
        let mut recv = Ipv6UdpRecv::new();
        let mut buf = [0; 65535];
        loop {
            let (len, src) = udp_socket.recv_from(&mut buf).unwrap();
            if let Ok(action) = action_rx.try_recv() {
                recv.action(action);
            }
            let data = &buf[..len];
            let ans = recv.recv(src, data);
            if let Some(ans) = ans {
                tx.send(GossipBlock { data: ans, src }).unwrap();
            }
        }
    });
    Ok((rx, action_tx))
}

pub fn spawn_std_thread_send_loop(udp_socket: UdpSocket) -> anyhow::Result<Sender<SendAction>> {
    let (action_tx, action_rx) = crossbeam::channel::bounded(1024);
    std::thread::spawn(move || {
        let mut send = Ipv6UdpSend::new(udp_socket);
        loop {
            let action = action_rx.recv().unwrap();
            send.action(action).unwrap();
        }
    });
    Ok(action_tx)
}
