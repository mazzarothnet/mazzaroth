use std::collections::HashSet;
use mvm::models::transfer::Transfer;

#[derive(Debug, Clone, Default)]
pub struct PendingTransfer {
    pub transfers: HashSet<Transfer>
}


