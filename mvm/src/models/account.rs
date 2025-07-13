use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::types::{AccountKey, Hash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
pub struct Account {
    pub key: AccountKey,
    pub balance: u128,
    pub action_hash: Hash,
}
