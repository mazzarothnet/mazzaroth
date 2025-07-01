use consensus::types::{AccountKey, ActionHash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub key: AccountKey,
    pub balance: u128,
    pub action_hash: ActionHash,
}

