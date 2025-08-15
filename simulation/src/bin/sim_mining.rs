#![allow(clippy::unwrap_used)]
use consensus::{block_header::MAX_TARGET, types::BlockKey};
use crypto_bigint::U256;
use mining::{run_gpu::mining_gpu_sha256, sha256_mining::gen_sha256_by_block_hash_and_nonce};
use utils::sha256::sha256_hash;

fn main() {
    let mt = MAX_TARGET;
    println!("mt: {:x}", mt.0);
    let block_hash = sha256_hash(b"12123112asd");
    let work_id = 1123123;
    let mut target = [0u32; 8];
    target[0] = 0x00000fff;
    target[1] = 0xffffffff;
    target[2] = 0xffffffff;
    target[3] = 0xffffffff;
    target[4] = 0xffffffff;
    target[5] = 0xffffffff;
    target[6] = 0xffffffff;
    target[7] = 0xffffffff;
    let nonce_vec = mining_gpu_sha256(block_hash, work_id, target).unwrap();
    for nonce in nonce_vec {
        let hash = gen_sha256_by_block_hash_and_nonce(block_hash, nonce);
        let block_key = BlockKey(U256::from_be_slice(&hash));
        println!("nonce: {:x}, hash: {}", nonce, block_key);
    }
}
