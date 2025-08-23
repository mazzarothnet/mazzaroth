#![allow(clippy::unwrap_used)]
use consensus::{block_header::MAX_TARGET, types::BlockKey};
use crypto_bigint::U256;
use mining::{
    run_gpu::{Sha256Context, mining_gpu_sha256},
    sha256_mining::gen_sha256_by_block_hash_and_nonce,
};
use utils::sha256::sha256_hash;

fn main() {
    let context = Sha256Context::new().unwrap();
    let mt = MAX_TARGET;
    println!("mt: {:x}", mt.0);
    let block_hash = sha256_hash(b"12123112asd");
    let mut work_id = 112323;
    let now = std::time::Instant::now();
    for i in 0..10 {
        work_id += i;
        let nonce_vec = mining_gpu_sha256(&context, block_hash, work_id, mt).unwrap();
        if let Some(nonce) = nonce_vec {
            let hash = gen_sha256_by_block_hash_and_nonce(block_hash, nonce);
            let block_key = BlockKey(U256::from_be_slice(&hash));
            println!("nonce: {:x}, hash: {}", nonce, block_key);
        } else {
            println!("no nonce found");
        }
    }
    println!("time: {:?}", now.elapsed().as_millis());
}
