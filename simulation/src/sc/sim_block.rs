use anyhow::Context;
use consensus::traits::Key;
use serde::{Deserialize, Serialize};
use utils::error::Result;

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
    fn serde_to_string(&self) -> String {
        self.0.to_string()
    }
    fn from_string(s: &str) -> Result<Self> {
        Ok(SimKey(
            s.parse().context(format!("SimKey from_string: {}", s))?,
        ))
    }
}
