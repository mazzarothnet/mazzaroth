use consensus::traits::Key;
use serde::{Deserialize, Serialize};

use super::sim_miner::Position;

// key is the block id, in the simulation, it is the same as timestamp
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimKey(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimBlock {
    pub key: SimKey,
    pub creator_position: Position,
    pub parent_keys: Vec<SimKey>,
}

impl Key for SimKey {
    fn is_genesis(&self) -> bool {
        self.0 == 0
    }
}
