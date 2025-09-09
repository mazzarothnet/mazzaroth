use crate::state::{account_manager::AccountKeyPair, transfer::SelfTransfer};
use consensus::types::{Hash, Signature};
use mvm::models::transfer::{Transfer, TransferInner};
use std::collections::HashSet;
use utils::{secp::sign_message, sha256::sha256_hash_rlp};

pub fn self_transfers_to_transfers(
    self_transfers: HashSet<SelfTransfer>,
    miner_account_pair: &AccountKeyPair,
    mut miner_action_hash: Hash,
) -> anyhow::Result<Vec<Transfer>> {
    let mut transfers = Vec::new();
    for self_transfer in self_transfers {
        let transfer_inner = TransferInner {
            from: miner_account_pair.public_key,
            to: self_transfer.to,
            amount: self_transfer.amount,
            from_last_action_hash: miner_action_hash,
            gas_price: 0,
        };
        let now_hash = sha256_hash_rlp(&transfer_inner);
        let signature = sign_message(&now_hash, &miner_account_pair.private_key)?;
        transfers.push(Transfer {
            inner: transfer_inner,
            from_signature: Signature(signature),
        });
        miner_action_hash = Hash(now_hash);
    }
    Ok(transfers)
}
