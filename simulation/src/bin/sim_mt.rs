#![allow(clippy::unwrap_used)]
use database::rocksdb_no_batch::RocksDbStorage;
use log::info;
use mvm::core::{
    merkle_tree::MerkleTree,
    storage::{DbStorage, DbStorageTransaction},
};
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use simulation::smvm::rand_account::{gen_merkle_tree, gen_rand_account};
use std::time::Instant;
use utils::{get_u8_vec_sum, log::init_log};

#[allow(clippy::cast_lossless, clippy::unwrap_used)]
fn main() {
    init_log();
    let mts = gen_merkle_tree("tmp_storage");
    let mut mt = MerkleTree::default();
    let mut account_vec = Vec::new();
    let mut rand = StdRng::seed_from_u64(23);
    let account_len = 200;
    let account_num = 1000;
    for _i in 0..account_len {
        let accounts = gen_rand_account(&mut rand, account_num);
        account_vec.push(accounts);
    }
    let now = Instant::now();
    for accounts in &account_vec {
        info!("update_tree, accounts: {:?}", accounts.len());
        let mut transaction = mts.begin_transaction().unwrap();
        mt.update_tree::<RocksDbStorage>(&mut transaction, accounts.clone(), vec![])
            .unwrap();
        transaction.commit().unwrap();
    }
    let tm_ms1 = now.elapsed().as_millis();
    let mut transaction = mts.begin_transaction().unwrap();
    let state_root1 = mt
        .get_state_root::<RocksDbStorage>(&mut transaction)
        .unwrap();
    transaction.commit().unwrap();
    account_vec.shuffle(&mut rand);
    let now = Instant::now();
    for accounts in account_vec {
        info!("update_tree, accounts: {:?}", accounts.len());
        let mut transaction = mts.begin_transaction().unwrap();
        mt.update_tree::<RocksDbStorage>(&mut transaction, accounts, vec![])
            .unwrap();
        transaction.commit().unwrap();
    }
    let tm_ms2 = now.elapsed().as_millis();
    let mut transaction = mts.begin_transaction().unwrap();
    let state_root2 = mt
        .get_state_root::<RocksDbStorage>(&mut transaction)
        .unwrap();
    transaction.commit().unwrap();
    info!("state_root1: {:?}", state_root1);
    info!("state_root2: {:?}", state_root2);
    info!(
        "s1 sum: {}, s2 sum: {} cost: {} {}",
        get_u8_vec_sum(&state_root1.0),
        get_u8_vec_sum(&state_root2.0),
        tm_ms1 as f64 / account_len as f64,
        tm_ms2 as f64 / account_len as f64
    );
}
