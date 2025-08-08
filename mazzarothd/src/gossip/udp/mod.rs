use crate::gossip::{
    channel_block::ChannelBlock,
    proto::{MAX_SHARDING_LEN, UDP_LRU_CAP},
    udp::buf::{ReedSolomonBuf, UdpBlock, UdpBuf, gen_reed_solomon_block, gen_udp_block},
};
use alloy_rlp::Decodable;
use log::debug;
use lru::LruCache;
use std::{
    collections::{HashMap, HashSet},
    net::{SocketAddr, UdpSocket},
};

pub mod buf;

pub struct UdpRecv {
    // use lru_cache to store the node_map
    pub listen_topic_len: u16,
    pub node_map: LruCache<SocketAddr, UdpBuf>,
}

impl UdpRecv {
    pub fn new(listen_topic_len: u16) -> Self {
        Self {
            listen_topic_len,
            node_map: LruCache::new(UDP_LRU_CAP),
        }
    }

    /// it will block until receive a message
    pub fn recv(&mut self, src: SocketAddr, mut data: &[u8]) -> Option<ChannelBlock> {
        let block: UdpBlock = Decodable::decode(&mut data)
            .map_err(|e| {
                debug!("UdpBlock::decode error: {}", e);
            })
            .ok()?;
        if block.channel_block_len == 1 {
            return ReedSolomonBuf::decode_to_channel_block(block.data, block.total_bytes as usize);
        }
        let entry = self
            .node_map
            .get_or_insert_mut(src, || UdpBuf::new(self.listen_topic_len));
        entry.try_add_data(block)
    }
}

pub struct UdpSend {
    pub send_set: LruCache<SocketAddr, HashMap<u16, u16>>,
}

pub enum SendAction {
    Send(ChannelBlock, SocketAddr),
    ShardingSend(ChannelBlock, SocketAddr),
    Broadcast(ChannelBlock, HashSet<SocketAddr>),
}

impl Default for UdpSend {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpSend {
    pub fn new() -> Self {
        Self {
            send_set: LruCache::new(UDP_LRU_CAP),
        }
    }

    pub fn action(&mut self, action: SendAction, socket: &mut UdpSocket) -> anyhow::Result<()> {
        match action {
            SendAction::Send(cb, dst) => {
                self.send_to(&cb, dst, socket)?;
            }
            SendAction::ShardingSend(cb, dst) => {
                self.sharding_send(&cb, dst, socket)?;
            }
            SendAction::Broadcast(cb, dsts) => {
                self.broadcast(&cb, dsts, socket)?;
            }
        }

        Ok(())
    }

    fn broadcast(
        &mut self,
        channel_block: &ChannelBlock,
        dsts: HashSet<SocketAddr>,
        socket: &mut UdpSocket,
    ) -> anyhow::Result<()> {
        let (data, total_bytes, channel_block_len) = gen_reed_solomon_block(channel_block)?;
        if channel_block_len != 1 {
            return Err(anyhow::anyhow!("channel_block_len != 1"));
        }
        for dst in dsts {
            let mut topic_to_key = HashMap::new();
            Self::send_to_inner(
                socket,
                channel_block,
                &data,
                dst,
                total_bytes,
                channel_block_len,
                &mut topic_to_key,
            )?;
        }
        Ok(())
    }

    fn send_to_inner(
        socket: &mut UdpSocket,
        channel_block: &ChannelBlock,
        data: &[Vec<u8>],
        dst: SocketAddr,
        total_bytes: u32,
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
                total_bytes,
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
        let (data, total_bytes, channel_block_len) = gen_reed_solomon_block(cb)?;
        if channel_block_len != 1 {
            return Err(anyhow::anyhow!("channel_block_len != 1"));
        }
        let mut topic_to_key = HashMap::new();
        Self::send_to_inner(
            socket,
            cb,
            &data,
            dst,
            total_bytes,
            channel_block_len,
            &mut topic_to_key,
        )?;
        Ok(())
    }

    fn sharding_send(
        &mut self,
        cb: &ChannelBlock,
        dst: SocketAddr,
        socket: &mut UdpSocket,
    ) -> anyhow::Result<()> {
        let (data, total_bytes, channel_block_len) = gen_reed_solomon_block(cb)?;
        if channel_block_len > MAX_SHARDING_LEN {
            return Err(anyhow::anyhow!("channel_block_len > MAX_SHARDING_LEN"));
        }
        let topic_to_key = self.send_set.get_or_insert_mut(dst, || HashMap::new());
        Self::send_to_inner(
            socket,
            cb,
            &data,
            dst,
            total_bytes,
            channel_block_len,
            topic_to_key,
        )?;
        Ok(())
    }
}
