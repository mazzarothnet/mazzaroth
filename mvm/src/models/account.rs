use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::types::ActionHash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Account {
    pub balance: u128,
    pub action_hash: ActionHash,
}
