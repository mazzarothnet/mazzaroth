use crate::models::transfer::{Merge, Transfer};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::{
    block_header::ConsensusHeader,
    types::{AccountKey, BlockKey, Hash},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
pub struct Block {
    pub key: BlockKey,
    pub nonce: u128,
    pub inner: BlockInner,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone, Default)]
pub struct BlockInner {
    pub version: u32,
    pub header: ConsensusHeader,
    pub transfers: Vec<Transfer>,
    pub merges: Vec<Merge>,
    pub miner: AccountKey,
    pub miner_last_action_hash: Hash,
}

impl BlockInner {
    pub fn is_less_than_max_block_size(&self) -> bool {
        let transfer_gas = self.transfers.len() as u128 * consensus::TRANSFER_GAS;
        let merge_gas = self.merges.len() as u128 * consensus::MERGE_GAS;
        let total_gas = transfer_gas + merge_gas;
        total_gas < consensus::BLOCK_GAS_LIMIT
    }
}
