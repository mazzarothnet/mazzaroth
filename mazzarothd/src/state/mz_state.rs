use crate::{
    config::Config,
    state::{
        account_manager::AccountManager,
        block_storage::{BlockStorage, get_block_storage},
        mvm::get_mvm_storage,
        tips::TempBlock,
        transfer::PendingTransfer,
    },
};
use consensus::types::BlockKey;
use database::rocksdb_no_batch::RocksDbStorage;
use mvm::core::vm::Mvm;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct MzState {
    pub mvm: Arc<Mutex<Mvm<RocksDbStorage>>>,
    pub block_storage: Arc<Mutex<BlockStorage>>,
    pub tips: Arc<Mutex<BTreeMap<BlockKey, u64>>>,
    pub temp_blocks: Arc<Mutex<TempBlock>>,
    pub config: Arc<Config>,
    pub account_manager: Arc<Mutex<AccountManager>>,
    pub pending_transfers: Arc<Mutex<PendingTransfer>>,
}

pub fn clear_path(path: &str) -> anyhow::Result<()> {
    let os_path = Path::new(path);
    if os_path.exists() {
        std::fs::remove_dir_all(os_path)?;
    }
    Ok(())
}

pub fn get_mz_state(path: &str) -> anyhow::Result<MzState> {
    let block_path = format!("{}/block", path);
    let mvm_path = format!("{}/mvm", path);
    let block_storage = Arc::new(Mutex::new(get_block_storage(&block_path)?));
    let mvm = Arc::new(Mutex::new(get_mvm_storage(&mvm_path)?));
    let tips = Arc::new(Mutex::new(BTreeMap::new()));
    let temp_blocks = Arc::new(Mutex::new(TempBlock::new(block_storage.clone())));
    let config_path = format!("{}/config.toml", path);
    let config = Config::init(&config_path)?;
    let account_manager = Arc::new(Mutex::new(AccountManager::init(&format!(
        "{}/account_manager.json",
        path
    ))?));
    let pending_transfers = Arc::new(Mutex::new(PendingTransfer::default()));
    Ok(MzState {
        mvm,
        block_storage,
        tips,
        temp_blocks,
        config: Arc::new(config),
        account_manager,
        pending_transfers,
    })
}
