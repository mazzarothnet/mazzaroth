use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::types::{AccountKey, ActionHash, Signature};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Transfer {
    pub inner: TransferInner,
    pub from_signature: Signature,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct TransferInner {
    pub from: AccountKey,
    pub to: AccountKey,
    pub amount: u128,
    pub from_last_action_hash: ActionHash,
    pub gas_price: u128,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct Merge {
    pub inner: MergeInner,
    pub from_signature: Signature,
    pub to_signature: Signature,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct MergeInner {
    pub from: AccountKey,
    pub to: AccountKey,
    pub amount: u128,
    pub from_last_action_hash: ActionHash,
    pub to_last_action_hash: ActionHash,
    pub gas_price: u128,
}
