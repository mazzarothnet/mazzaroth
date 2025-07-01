use crate::models::transfer::Transfer;
use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::{block_header::ConsensusHeader, types::{AccountKey, BlockKey, StateHash}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Block {
    pub key: BlockKey,
    pub nonce: u128,
    pub inner: BlockInner,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct BlockInner {
    pub version: u32,
    pub header: ConsensusHeader,
    pub transfers: Vec<Transfer>,
    pub state_hash: StateHash,
    pub miner: AccountKey,
}
