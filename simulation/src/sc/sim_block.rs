use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::{block_header::ConsensusHeader, types::BlockKey};
use serde::{Deserialize, Serialize};

use super::sim_miner::Position;

#[derive(Clone, Serialize, Deserialize, RlpDecodable, RlpEncodable)]
pub struct SimBlock {
    pub key: BlockKey,
    pub header: ConsensusHeader,
    pub creator_position: Position,
}
