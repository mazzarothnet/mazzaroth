use consensus::types::{AccountKey, Hash};
use database::rocksdb_no_batch::RocksDbStorage;
use rand::{Rng, rngs::StdRng};
use std::path::Path;

pub fn gen_rand_account(rng: &mut StdRng, num: u64) -> Vec<(AccountKey, Hash)> {
    let mut accounts = Vec::new();
    for _ in 0..num {
        let mut account_key = [0; 33];
        for i in &mut account_key {
            *i = rng.random_range(0..255);
        }
        let mut state_hash = [0; 32];
        for i in &mut state_hash {
            *i = rng.random_range(0..255);
        }
        accounts.push((AccountKey(account_key), Hash(state_hash)));
    }

    accounts
}

pub fn gen_merkle_tree(path: &str) -> RocksDbStorage {
    if Path::new(path).exists() {
        std::fs::remove_dir_all(path).unwrap();
    }

    RocksDbStorage::new(&format!("{path}/mt")).unwrap()
}
