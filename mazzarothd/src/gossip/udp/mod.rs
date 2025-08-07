use crate::gossip::{
    channel_block::ChannelBlock,
    udp::buf::{UdpBuf, gen_reed_solomon_block, gen_udp_block},
};
use lru::LruCache;
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    num::NonZero,
};

pub mod buf;

pub struct UdpRecv {
    // use lru_cache to store the node_map
    pub listen_topic_len: u16,
    pub node_map: LruCache<SocketAddr, UdpBuf>,
}

impl UdpRecv {
    pub fn new(listen_topic_len: u16, cap: NonZero<usize>) -> Self {
        Self {
            listen_topic_len,
            node_map: LruCache::new(cap),
        }
    }

    /// it will block until receive a message
    pub fn recv(&mut self, src: SocketAddr, data: &[u8]) -> Option<ChannelBlock> {
        let entry = self
            .node_map
            .get_or_insert_mut(src, || UdpBuf::new(self.listen_topic_len));
        entry.try_add_data(data)
    }
}

pub struct UdpSend {
    pub high_send_set: HashMap<SocketAddr, HashMap<u16, u16>>,
    pub low_send_set: HashMap<SocketAddr, HashMap<u16, u16>>,
}

pub enum SendAction {
    AddNode(SocketAddr),
    Send(ChannelBlock, SocketAddr),
    Broadcast(ChannelBlock, Option<SocketAddr>),
}

impl Default for UdpSend {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpSend {
    pub fn new() -> Self {
        Self {
            high_send_set: HashMap::new(),
            low_send_set: HashMap::new(),
        }
    }

    pub fn action(&mut self, action: SendAction, socket: &mut UdpSocket) -> anyhow::Result<()> {
        match action {
            SendAction::AddNode(addr) => {
                self.add_node(addr);
            }
            SendAction::Send(cb, dst) => {
                self.send_to(&cb, dst, socket)?;
            }
            SendAction::Broadcast(cb, sender) => {
                self.broadcast(&cb, sender, socket)?;
            }
        }

        Ok(())
    }

    pub fn add_node(&mut self, addr: SocketAddr) {
        self.high_send_set.insert(addr, HashMap::new());
        
    }

    fn broadcast(
        &mut self,
        channel_block: &ChannelBlock,
        sender: Option<SocketAddr>,
        socket: &mut UdpSocket,
    ) -> anyhow::Result<()> {
        let (data, total_len, channel_block_len) = gen_reed_solomon_block(channel_block)?;
        for (dst, topic_to_key) in self.high_send_set.iter_mut() {
            if Some(*dst) == sender {
                continue;
            }
            Self::send_to_inner(
                socket,
                channel_block,
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
        data: &[Vec<u8>],
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
                data,
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
        cb: &ChannelBlock,
        dst: SocketAddr,
        socket: &mut UdpSocket,
    ) -> anyhow::Result<()> {
        let (data, total_len, channel_block_len) = gen_reed_solomon_block(cb)?;
        let topic_to_key = self.high_send_set.entry(dst).or_default();
        Self::send_to_inner(
            socket,
            cb,
            &data,
            dst,
            total_len,
            channel_block_len,
            topic_to_key,
        )?;
        Ok(())
    }
}
