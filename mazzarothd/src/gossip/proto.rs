use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::types::{BlockKey, Hash};
use mvm::models::block::Block;
use serde::{Deserialize, Serialize};

pub const NOTIFY_BLOCK_TOPIC: u16 = 0;
pub const REQ_BLOCK_KEY_TOPIC: u16 = 1;
pub const BLOCK_TOPIC: u16 = 2;
pub const TRY_CONNECT_TOPIC: u16 = 3;
pub const ALIVE_TOPIC: u16 = 4;
pub const MESSAGE_TOPIC: u16 = 5;
pub const PING_TOPIC: u16 = 6;
pub const PONG_TOPIC: u16 = 7;

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct NotifyBlock {
    pub key: BlockKey,
    pub nonce: u128,
    pub inner_hash: Hash,
}

pub type ReqBlock = BlockKey;
pub type RespBlock = Block;

pub const TRY_CONNECT_TOPIC_MSG: &str = "TRY CONNECT";
pub const ALIVE_TOPIC_MSG: &str = "ALIVE";
