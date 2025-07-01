use consensus::types::{AccountKey, ActionHash};
use serde::{Deserialize, Serialize};
use alloy_rlp::{RlpDecodable, RlpEncodable};

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Account {
    pub key: AccountKey,
    pub balance: u128,
    pub action_hash: ActionHash,
}

