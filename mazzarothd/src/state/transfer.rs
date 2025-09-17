use consensus::types::AccountKey;
use std::collections::HashSet;

use crate::state::mz_state::MzState;

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct SelfTransfer {
    pub to: AccountKey,
    pub amount: u128,
}

#[derive(Debug, Clone, Default)]
pub struct PendingSelfTransfer {
    pub transfers: HashSet<SelfTransfer>,
}

pub fn take_pending_transfers(mz_state: &MzState) -> anyhow::Result<HashSet<SelfTransfer>> {
    let mut pending_transfers_lock = mz_state.pending_transfers.lock().map_err(|e| {
        anyhow::anyhow!(
            "get_pending_transfers Failed to lock pending_transfers: {}",
            e
        )
    })?;
    let pending_transfers = std::mem::take(&mut pending_transfers_lock.transfers);
    Ok(pending_transfers)
}

pub fn clear_pending_transfers(mz_state: &MzState) -> anyhow::Result<()> {
    let mut pending_transfers_lock = mz_state.pending_transfers.lock().map_err(|e| {
        anyhow::anyhow!(
            "clear_pending_transfers Failed to lock pending_transfers: {}",
            e
        )
    })?;
    pending_transfers_lock.transfers.clear();
    Ok(())
}

pub fn insert_pending_transfers(mz_state: &MzState, transfers: SelfTransfer) -> anyhow::Result<()> {
    let mut pending_transfers_lock = mz_state.pending_transfers.lock().map_err(|e| {
        anyhow::anyhow!(
            "insert_pending_transfers Failed to lock pending_transfers: {}",
            e
        )
    })?;
    pending_transfers_lock.transfers.insert(transfers);
    Ok(())
}
