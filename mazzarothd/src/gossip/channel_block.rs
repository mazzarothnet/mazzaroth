use alloy_rlp::{RlpDecodable, RlpEncodable};

#[derive(Debug, RlpEncodable, RlpDecodable)]
pub struct ChannelBlock {
    pub topic_id: u16,
    pub key: u16,
    pub data: Vec<u8>,
}

