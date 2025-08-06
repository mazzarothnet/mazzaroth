#![allow(clippy::unwrap_used)]
use log::info;
use mvm::core::storage::DbStorageTransaction;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use simulation::smvm::rand_account::{gen_merkle_tree, gen_rand_account};
use std::time::Instant;
use utils::{get_u8_vec_sum, log::init_log};

#[allow(clippy::cast_lossless, clippy::unwrap_used)]
fn main() {
    init_log();
    let mut mt = gen_merkle_tree("tmp_storage");
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
        let ts = mt.update_tree(accounts.clone(), vec![]).unwrap();
        ts.commit().unwrap();
    }
    let tm_ms1 = now.elapsed().as_millis();
    let state_root1 = mt.get_state_root().unwrap();
    account_vec.shuffle(&mut rand);
    let now = Instant::now();
    for accounts in account_vec {
        let ts = mt.update_tree(accounts, vec![]).unwrap();
        ts.commit().unwrap();

    }
    let tm_ms2 = now.elapsed().as_millis();
    let state_root2 = mt.get_state_root().unwrap();
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
