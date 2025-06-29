use consensus::block_header::BlockHeader;
use serde::{Deserialize, Serialize};

use super::sim_miner::Position;

#[derive(Clone, Serialize, Deserialize)]
pub struct SimBlock {
    pub header: BlockHeader,
    pub creator_position: Position,
}
