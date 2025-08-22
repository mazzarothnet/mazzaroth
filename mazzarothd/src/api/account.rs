use crate::state::mz_state::MzState;
use axum::extract::State;
use mvm::models::account::Account;
use utils::error::{Res, Result};

pub async fn get_current_account(State(state): State<MzState>) -> Result<Res<Account>> {
    let current_key = state
        .account_manager
        .lock()
        .map_err(|e| anyhow::anyhow!("get_current_account Failed to lock account_manager: {}", e))?
        .now_selected_account
        .public_key;
    let account = state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("get_current_account Failed to lock mvm: {}", e))?
        .get_account(current_key)
        .map_err(|e| anyhow::anyhow!("get_current_account Failed to get account: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("get_current_account Failed to get account"))?;
    Ok(Res { data: account })
}
