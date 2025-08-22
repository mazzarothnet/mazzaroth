use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::types::{AccountKey, Hash, Signature};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone, PartialEq, Eq, Hash)]
pub struct Transfer {
    pub inner: TransferInner,
    pub from_signature: Signature,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone, PartialEq, Eq, Hash)]
pub struct TransferInner {
    pub from: AccountKey,
    pub to: AccountKey,
    pub amount: u128,
    pub from_last_action_hash: Hash,
    pub gas_price: u128,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
pub struct Merge {
    pub inner: MergeInner,
    pub from_signature: Signature,
    pub to_signature: Signature,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
pub struct MergeInner {
    pub from: AccountKey,
    pub to: AccountKey,
    pub balance: u128,
    pub from_last_action_hash: Hash,
    pub to_last_action_hash: Hash,
    pub gas_price: u128,
}
