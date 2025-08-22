use crate::state::{account_manager::AccountKeyPair, mz_state::MzState};
use anyhow::Context;
use axum::extract::{Query, State};
use consensus::types::{AccountKey, Signature};
use mvm::models::{
    account::Account,
    transfer::{Transfer, TransferInner},
};
use serde::Deserialize;
use utils::{
    error::{Res, Result},
    secp::sign_message,
    sha256::sha256_hash_rlp,
};

pub async fn get_current_account(State(mz_state): State<MzState>) -> Result<Res<Account>> {
    let account_key = get_account_pair_by_mz_state(&mz_state)?.public_key;
    let account = get_account_by_mz_state(&mz_state, account_key)?;
    Ok(Res { data: account })
}

#[derive(Debug, Deserialize)]
pub struct GetAccountReq {
    pub account_key: AccountKey,
}

pub async fn get_account(
    State(mz_state): State<MzState>,
    Query(req): Query<GetAccountReq>,
) -> Result<Res<Account>> {
    let account_key = req.account_key;
    let account = mz_state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("get_account Failed to lock mvm: {}", e))?
        .get_account(account_key)
        .map_err(|e| anyhow::anyhow!("get_account Failed to get account: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("get_account Failed to get account"))?;
    Ok(Res { data: account })
}

#[derive(Deserialize)]
pub struct TransToReq {
    pub account_key: AccountKey,
    pub amount: String,
}

pub async fn transfer(
    State(mz_state): State<MzState>,
    Query(req): Query<TransToReq>,
) -> Result<Res<()>> {
    let amount: u128 = req
        .amount
        .parse()
        .with_context(|| "transfer amount parse error")?;
    let account_pair = get_account_pair_by_mz_state(&mz_state)?;
    let account = get_account_by_mz_state(&mz_state, account_pair.public_key)?;
    let trans_inner = TransferInner {
        from: account_pair.public_key,
        to: req.account_key,
        amount: amount,
        from_last_action_hash: account.action_hash,
        gas_price: 0,
    };
    let hash = sha256_hash_rlp(&trans_inner);
    let sign = sign_message(&hash, &account_pair.private_key)
        .with_context(|| "transfer sign_message error")?;
    let trans = Transfer {
        inner: trans_inner,
        from_signature: Signature(sign),
    };
    let mut pending_transfers_lock = mz_state
        .pending_transfers
        .lock()
        .map_err(|e| anyhow::anyhow!("transfer Failed to lock pending_transfers: {}", e))?;
    pending_transfers_lock.transfers.insert(trans);

    Ok(Res { data: () })
}

fn get_account_pair_by_mz_state(mz_state: &MzState) -> Result<AccountKeyPair> {
    let account_pair = mz_state
        .account_manager
        .lock()
        .map_err(|e| anyhow::anyhow!("get_current_account Failed to lock account_manager: {}", e))?
        .now_selected_account
        .clone();
    Ok(account_pair)
}

fn get_account_by_mz_state(mz_state: &MzState, account_key: AccountKey) -> Result<Account> {
    let account = mz_state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("get_account Failed to lock mvm: {}", e))?
        .get_account(account_key)
        .map_err(|e| anyhow::anyhow!("get_account Failed to get account: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("get_account Failed to get account"))?;
    Ok(account)
}
