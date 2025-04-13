use consensus::traits::Key;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimKey(i64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimBlock {
    pub key: SimKey,
    pub ts: i64,
    pub x: f64,
    pub y: f64,
    pub parent_keys: Vec<SimKey>,
}

impl Key for SimKey {
    fn is_genesis(&self) -> bool {
        self.0 == 0
    }
}
