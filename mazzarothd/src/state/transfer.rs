use mvm::models::transfer::Transfer;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct PendingTransfer {
    pub transfers: HashSet<Transfer>,
}
