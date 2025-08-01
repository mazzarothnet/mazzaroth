use log::debug;

use crate::gossip::{
    channel_block::ChannelBlock,
    ipv6_udp::buf::{Ipv6UdpBuf, gen_reed_solomon_block},
};
use std::{
    collections::{HashMap, HashSet},
    net::{SocketAddr, UdpSocket},
};

pub mod buf;

pub struct Ipv6UdpRecv {
    pub socket: UdpSocket,
    pub node_map: HashMap<SocketAddr, Ipv6UdpBuf>,
    pub recv_buf: Vec<u8>,
}

impl Ipv6UdpRecv {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            node_map: HashMap::new(),
            recv_buf: Vec::with_capacity(65535),
        }
    }

    pub fn add_node(&mut self, addr: SocketAddr, listen_topic_len: u16) {
        self.node_map
            .insert(addr, Ipv6UdpBuf::new(listen_topic_len));
    }

    pub fn remove_node(&mut self, addr: SocketAddr) {
        self.node_map.remove(&addr);
    }

    /// it will block until receive a message
    pub fn recv(&mut self) -> Option<ChannelBlock> {
        let mut buf = [0; 65535];
        let (len, src) = self
            .socket
            .recv_from(&mut buf)
            .map_err(|e| {
                debug!("Ipv6UdpSwarm::recv error: {}", e);
                e
            })
            .ok()?;
        let data = &buf[..len];
        if self.node_map.contains_key(&src) {
            self.node_map.get_mut(&src)?.try_add_data(data)
        } else {
            debug!("Ipv6UdpSwarm::recv node not found: {}", src);
            None
        }
    }
}

pub struct Ipv6UdpSend {
    pub socket: UdpSocket,
    pub send_set: HashSet<SocketAddr>,
}

impl Ipv6UdpSend {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            send_set: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, addr: SocketAddr) {
        self.send_set.insert(addr);
    }

    pub fn remove_node(&mut self, addr: SocketAddr) {
        self.send_set.remove(&addr);
    }

    pub fn send(
        &mut self,
        channel_block: ChannelBlock,
        sender: Option<SocketAddr>,
    ) -> anyhow::Result<()> {
        let data = gen_reed_solomon_block(channel_block)?;
        for dst in self.send_set.iter() {
            if Some(*dst) == sender {
                continue;
            }
            for d in data.iter() {
                self.socket.send_to(&d, dst)?;
            }
        }
        Ok(())
    }
}
