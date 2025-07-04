use std::path::Path;

use database::rocksdb_storage::RocksDbStorage;
use mvm::core::storage::VmStorage;

pub fn gen_new_vm(path: &str) -> VmStorage<RocksDbStorage> {
    // create dir if not exists
    if !Path::new(path).exists() {
        std::fs::create_dir_all(path).unwrap();
    }

    let account_storage = RocksDbStorage::new(&format!("{path}/account")).unwrap();
    let mt_storage = RocksDbStorage::new(&format!("{path}/mt")).unwrap();

    VmStorage::new(account_storage, mt_storage).unwrap()
}
