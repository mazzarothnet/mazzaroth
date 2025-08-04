use log::debug;

use crate::gossip::{
    channel_block::ChannelBlock,
    ipv6_udp::buf::{Ipv6UdpBuf, gen_reed_solomon_block, gen_udp_block},
};
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
};

pub mod buf;

pub struct Ipv6UdpRecv {
    pub node_map: HashMap<SocketAddr, Ipv6UdpBuf>,
}

pub enum RecvAction {
    AddNode(SocketAddr, u16),
    RemoveNode(SocketAddr),
}

impl Ipv6UdpRecv {
    pub fn new() -> Self {
        Self {
            node_map: HashMap::new(),
        }
    }

    pub fn action(&mut self, action: RecvAction) {
        match action {
            RecvAction::AddNode(addr, listen_topic_len) => {
                self.add_node(addr, listen_topic_len);
            }
            RecvAction::RemoveNode(addr) => {
                self.remove_node(addr);
            }
        }
    }

    /// it will block until receive a message
    pub fn recv(&mut self, src: SocketAddr, data: &[u8]) -> Option<ChannelBlock> {
        if self.node_map.contains_key(&src) {
            self.node_map.get_mut(&src)?.try_add_data(data)
        } else {
            debug!("Ipv6UdpSwarm::recv node not found: {}", src);
            None
        }
    }

    pub fn add_node(&mut self, addr: SocketAddr, listen_topic_len: u16) {
        self.node_map
            .insert(addr, Ipv6UdpBuf::new(listen_topic_len));
    }

    pub fn remove_node(&mut self, addr: SocketAddr) {
        self.node_map.remove(&addr);
    }
}

pub struct Ipv6UdpSend {
    pub send_set: HashMap<SocketAddr, HashMap<u16, u16>>,
}

pub enum SendAction {
    AddNode(SocketAddr),
    RemoveNode(SocketAddr),
    Send(ChannelBlock, SocketAddr),
    Broadcast(ChannelBlock, Option<SocketAddr>),
}

impl Ipv6UdpSend {
    pub fn new() -> Self {
        Self {
            send_set: HashMap::new(),
        }
    }

    pub fn action(&mut self, action: SendAction, socket: &mut UdpSocket) -> anyhow::Result<()> {
        match action {
            SendAction::AddNode(addr) => {
                self.add_node(addr);
            }
            SendAction::RemoveNode(addr) => {
                self.remove_node(addr);
            }
            SendAction::Send(cb, dst) => {
                self.send_to(cb, dst, socket)?;
            }
            SendAction::Broadcast(cb, sender) => {
                self.broadcast(cb, sender, socket)?;
            }
        }

        Ok(())
    }

    pub fn add_node(&mut self, addr: SocketAddr) {
        self.send_set.insert(addr, HashMap::new());
    }

    pub fn remove_node(&mut self, addr: SocketAddr) {
        self.send_set.remove(&addr);
    }

    fn broadcast(
        &mut self,
        channel_block: ChannelBlock,
        sender: Option<SocketAddr>,
        socket: &mut UdpSocket,
    ) -> anyhow::Result<()> {
        let (data, total_len, channel_block_len) = gen_reed_solomon_block(&channel_block)?;
        for (dst, topic_to_key) in self.send_set.iter_mut() {
            if Some(*dst) == sender {
                continue;
            }
            Self::send_to_inner(
                socket,
                &channel_block,
                &data,
                *dst,
                total_len,
                channel_block_len,
                topic_to_key,
            )?;
        }
        Ok(())
    }

    fn send_to_inner(
        socket: &mut UdpSocket,
        channel_block: &ChannelBlock,
        data: &Vec<Vec<u8>>,
        dst: SocketAddr,
        total_len: u32,
        channel_block_len: u16,
        topic_to_key: &mut HashMap<u16, u16>,
    ) -> anyhow::Result<()> {
        let key = topic_to_key.entry(channel_block.topic_id).or_insert(0);
        if *key == u16::MAX {
            *key = 0;
        }
        *key += 1;
        for (index, data) in data.iter().enumerate() {
            let d = gen_udp_block(
                &data,
                channel_block.topic_id,
                *key,
                index as u16,
                total_len,
                channel_block_len,
            )?;
            socket.send_to(&d, dst)?;
        }
        Ok(())
    }

    fn send_to(
        &mut self,
        cb: ChannelBlock,
        dst: SocketAddr,
        socket: &mut UdpSocket,
    ) -> anyhow::Result<()> {
        let (data, total_len, channel_block_len) = gen_reed_solomon_block(&cb)?;
        let topic_to_key = self
            .send_set
            .get_mut(&dst)
            .ok_or_else(|| anyhow::anyhow!("node not found"))?;
        Self::send_to_inner(
            socket,
            &cb,
            &data,
            dst,
            total_len,
            channel_block_len,
            topic_to_key,
        )?;
        Ok(())
    }
}
