use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use log::debug;
use reed_solomon_erasure::galois_8::ReedSolomon;

use crate::gossip::channel_block::ChannelBlock;

const CHANNEL_BLOCK_SIZE: usize = 1024;
const PARITY_SHARDS: usize = 10;

#[derive(Debug, Clone, Hash)]
pub struct UdpBuf {
    pub listen_topic_len: u16,
    topic_buf: Vec<ReedSolomonBuf>,
}

impl UdpBuf {
    pub fn new(listen_topic_len: u16) -> Self {
        let mut topic_buf = Vec::new();
        for _ in 0..listen_topic_len {
            topic_buf.push(ReedSolomonBuf::new());
        }
        Self {
            listen_topic_len,
            topic_buf,
        }
    }

    pub fn try_add_data(&mut self, mut data: &[u8]) -> Option<ChannelBlock> {
        let block: UdpBlock = Decodable::decode(&mut data)
            .map_err(|e| {
                debug!("UdpBlock::decode error: {}", e);
            })
            .ok()?;
        if block.topic_id >= self.listen_topic_len {
            return None;
        }
        self.topic_buf[block.topic_id as usize].try_add_data(block)
    }
}

pub fn gen_reed_solomon_block(
    channel_block: &ChannelBlock,
) -> anyhow::Result<(Vec<Vec<u8>>, u32, u16)> {
    let mut raw_data = Vec::new();
    channel_block.encode(&mut raw_data);
    let total_len = raw_data.len();
    let mut master_copy = Vec::new();
    for block in raw_data.chunks(CHANNEL_BLOCK_SIZE) {
        if block.len() != CHANNEL_BLOCK_SIZE {
            let mut buf = vec![0; CHANNEL_BLOCK_SIZE];
            buf[..block.len()].copy_from_slice(block);
            master_copy.push(buf);
        } else {
            master_copy.push(block.to_vec());
        }
    }
    let master_copy_len = master_copy.len();
    let parity_shards = get_parity_len(master_copy_len);
    for _i in 0..parity_shards {
        master_copy.push(vec![0; CHANNEL_BLOCK_SIZE]);
    }

    debug!("master_copy_len: {}", master_copy_len);
    debug!("parity_shards: {}", parity_shards);

    if parity_shards > 0 {
        let erasure_code = ReedSolomon::new(master_copy_len, parity_shards)?;
        erasure_code.encode(&mut master_copy)?;
    }

    // let mut blocks = Vec::new();
    // for (index, block) in master_copy.into_iter().enumerate() {
    //     let b = UdpBlock {
    //         topic_id: channel_block.topic_id,
    //         key: channel_block.key,
    //         index: index as u16,
    //         total_len: total_len as u32,
    //         channel_block_len: master_copy_len as u16,
    //         data: block,
    //     };
    //     let mut buf = Vec::new();
    //     b.encode(&mut buf);
    //     blocks.push(buf);
    // }

    Ok((master_copy, total_len as u32, master_copy_len as u16))
}

pub fn gen_udp_block(
    data: &[u8],
    topic_id: u16,
    key: u16,
    index: u16,
    total_len: u32,
    channel_block_len: u16,
) -> anyhow::Result<Vec<u8>> {
    let b = UdpBlock {
        topic_id,
        key,
        index,
        total_len,
        channel_block_len,
        data: data.to_vec(),
    };

    let mut buf = Vec::new();
    b.encode(&mut buf);
    Ok(buf)
}

#[derive(Debug, RlpEncodable, RlpDecodable)]
struct UdpBlock {
    topic_id: u16,
    key: u16,
    index: u16,
    total_len: u32,
    channel_block_len: u16,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Hash)]
struct ReedSolomonBuf {
    key: u16,
    channel_block_len: u16,
    now_size: u16,
    total_len: u32,
    inited: bool,
    buf: Vec<Option<Vec<u8>>>,
}

impl ReedSolomonBuf {
    fn new() -> Self {
        Self {
            key: 0,
            channel_block_len: 0,
            now_size: 0,
            total_len: 0,
            inited: false,
            buf: Vec::new(),
        }
    }

    fn try_add_data(&mut self, data: UdpBlock) -> Option<ChannelBlock> {
        if data.key != self.key || !self.inited {
            if need_update_key(self.key, data.key) || !self.inited {
                self.inited = true;
                self.key = data.key;
                self.now_size = 0;
                self.channel_block_len = data.channel_block_len;
                let buf_len = self.channel_block_len as usize
                    + get_parity_len(self.channel_block_len as usize);
                self.total_len = data.total_len;
                self.buf.clear();
                self.buf.resize(buf_len, None);
            } else {
                return None;
            }
        }
        if self.buf[data.index as usize].is_some() {
            return None;
        }
        self.buf[data.index as usize] = Some(data.data);
        self.now_size += 1;
        debug!("ReedSolomonBuf::try_add_data now_size: {}", self.now_size);
        debug!(
            "ReedSolomonBuf::try_add_data channel_block_len: {}",
            self.channel_block_len
        );
        if self.now_size == self.channel_block_len {
            // return Some(ChannelBlock {
            //     topic_id: 0,
            //     key: self.key,
            //     data: Vec::new(),
            // });
            if self.channel_block_len != 1 {
                let parity_shards = get_parity_len(self.channel_block_len as usize);
                let erasure_code = ReedSolomon::new(self.channel_block_len as usize, parity_shards)
                    .map_err(|e| {
                        debug!("ReedSolomon::new error: {}", e);
                    })
                    .ok()?;
                erasure_code
                    .reconstruct(&mut self.buf)
                    .map_err(|e| {
                        debug!("ReedSolomon::reconstruct error: {}", e);
                    })
                    .ok()?;
            }
            let mut block_buf =
                Vec::with_capacity(CHANNEL_BLOCK_SIZE * self.channel_block_len as usize);
            for i in 0..self.channel_block_len as usize {
                if let Some(data) = self.buf[i].take() {
                    block_buf.extend_from_slice(&data);
                }
            }
            block_buf.truncate(self.total_len as usize);
            let cb: ChannelBlock = Decodable::decode(&mut block_buf.as_slice())
                .map_err(|e| {
                    debug!("ChannelBlock::decode error: {}", e);
                })
                .ok()?;
            return Some(cb);
        }

        None
    }
}

fn need_update_key(old_key: u16, new_key: u16) -> bool {
    if old_key == new_key {
        return false;
    }
    let old_key_i32 = i32::from(old_key);
    let mut new_key_i32 = i32::from(new_key);
    if new_key_i32 < old_key_i32 {
        new_key_i32 += i32::from(u16::MAX);
    }
    let gap = new_key_i32 - old_key_i32;
    gap < (i32::from(u16::MAX)) / 2
}

fn get_parity_len(len: usize) -> usize {
    if len == 1 {
        0
    } else {
        std::cmp::max(len / PARITY_SHARDS, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_need_update_key() {
        assert!(!need_update_key(0, 0));
        assert!(need_update_key(0, 1));
        assert!(!need_update_key(1, 0));
        assert!(!need_update_key(0, u16::MAX));
        assert!(need_update_key(u16::MAX, 0));
    }
}
