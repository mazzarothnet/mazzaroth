use crate::state::{
    account_manager::AccountKeyPair,
    mz_state::MzState,
    transfer::{SelfTransfer, insert_pending_transfers},
};
use anyhow::Context;
use axum::extract::{Query, State};
use consensus::types::AccountKey;
use mvm::{core::vm::Mvm, models::account::Account};
use serde::Deserialize;
use utils::error::{Res, Result};

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
    let account = get_account_by_mz_state(&mz_state, req.account_key)?;
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
    let self_trans = SelfTransfer {
        to: req.account_key,
        amount,
    };
    insert_pending_transfers(&mz_state, self_trans)?;

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
    let mut mvm = mz_state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("get_account Failed to lock mvm: {}", e))?;
    let mut mvm_transaction = mvm
        .begin_transaction()
        .map_err(|e| anyhow::anyhow!("get_account Failed to begin transaction: {}", e))?;
    let account = Mvm::get_account(&mut mvm_transaction, account_key)
        .map_err(|e| anyhow::anyhow!("get_account Failed to get account: {}", e))?;
    Ok(account)
}
