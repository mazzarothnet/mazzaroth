use crate::models::transfer::{Merge, Transfer};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::{
    block_header::ConsensusHeader,
    types::{AccountKey, BlockKey},
};
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
    pub merges: Vec<Merge>,
    pub miner: AccountKey,
}
