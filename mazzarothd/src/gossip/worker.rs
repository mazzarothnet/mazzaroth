use crate::gossip::{UDP_RECV, UDP_SEND, channel_block::ChannelBlock, ipv6_udp::SendAction};
use crossbeam::channel::{Receiver, Sender};
use std::net::{SocketAddr, UdpSocket};

pub struct GossipBlock {
    pub data: ChannelBlock,
    pub src: SocketAddr,
}

fn recv_inner(
    udp_socket: &UdpSocket,
    buf: &mut [u8],
    tx: &Sender<GossipBlock>,
) -> anyhow::Result<()> {
    let (len, src) = udp_socket
        .recv_from(buf)
        .map_err(|e| anyhow::anyhow!("Failed to receive from udp_socket: {:?}", e))?;
    let data = &buf[..len];
    let ans = UDP_RECV
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock UDP_RECV: {:?}", e))?
        .recv(src, data);
    if let Some(ans) = ans {
        tx.send(GossipBlock { data: ans, src })
            .map_err(|e| anyhow::anyhow!("Failed to send GossipBlock: {:?}", e))?;
    }

    Ok(())
}

pub fn spawn_std_thread_recv_loop(udp_socket: UdpSocket) -> anyhow::Result<Receiver<GossipBlock>> {
    let (tx, rx) = crossbeam::channel::bounded(1024);
    std::thread::spawn(move || {
        let mut buf = [0; 65535];
        loop {
            if let Err(e) = recv_inner(&udp_socket, &mut buf, &tx) {
                log::error!("Failed to recv: {:?}", e);
            }
        }
    });
    Ok(rx)
}

fn send_action(rx: &Receiver<SendAction>, udp_socket: &mut UdpSocket) -> anyhow::Result<()> {
    let action = rx
        .recv()
        .map_err(|e| anyhow::anyhow!("Failed to receive action: {:?}", e))?;
    let mut send = UDP_SEND
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock UDP_SEND: {:?}", e))?;
    send.action(action, udp_socket)
        .map_err(|e| anyhow::anyhow!("Failed to send action: {:?}", e))?;

    Ok(())
}

pub fn spawn_std_thread_send_loop(udp_socket: UdpSocket) -> anyhow::Result<Sender<SendAction>> {
    let (action_tx, action_rx) = crossbeam::channel::bounded(1024);
    std::thread::spawn(move || {
        let mut udp_socket = udp_socket;
        loop {
            if let Err(e) = send_action(&action_rx, &mut udp_socket) {
                log::error!("Failed to send action: {:?}", e);
            }
        }
    });
    Ok(action_tx)
}
