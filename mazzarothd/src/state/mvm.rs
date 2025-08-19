use anyhow::Context;
use database::rocksdb_no_batch::RocksDbStorage;
use mvm::core::{merkle_tree::MerkleTree, vm::Mvm};
use std::{path::Path, sync::Mutex};

use crate::state::app_data::get_mvm_db_path;

lazy_static::lazy_static! {
    static ref MVM_STORAGE: Mutex<Mvm<RocksDbStorage>> = Mutex::new(get_mvm_storage());
}

#[allow(clippy::unwrap_used)]
fn get_mvm_storage() -> Mvm<RocksDbStorage> {
    let path = get_mvm_db_path().unwrap();
    get_mvm_storage_inner(&path, false).unwrap()
}

fn get_mvm_storage_inner(path: &str, need_reset: bool) -> anyhow::Result<Mvm<RocksDbStorage>> {
    if need_reset {
        std::fs::remove_dir_all(&path)?;
    }
    {
        let path = Path::new(path);
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
    }

    let merkle_path = format!("{}/merkle", path);
    let account_path = format!("{}/account", path);
    let state_path = format!("{}/state", path);
    let merkle_tree_db =
        RocksDbStorage::new(&merkle_path).with_context(|| "Failed to create merkle tree db")?;
    let merkle_tree =
        MerkleTree::new(merkle_tree_db).with_context(|| "Failed to create merkle tree")?;
    let account_db =
        RocksDbStorage::new(&account_path).with_context(|| "Failed to create account db")?;
    let state_db = RocksDbStorage::new(&state_path).with_context(|| "Failed to create state db")?;
    Ok(Mvm::new(account_db, merkle_tree, state_db))
}
