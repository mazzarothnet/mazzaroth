use mining::{
    run_gpu::mining_gpu_sha256,
    sha256_mining::{bytes_to_hex, nonce_hash_to_package_to_u8_vec},
};
use utils::sha256::sha256_hash;

fn main() {
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
    let nonce_vec = mining_gpu_sha256(block_hash, work_id, target);
    for i in 0..nonce_vec.len() {
        let nonce = nonce_vec[i];
        let msg = nonce_hash_to_package_to_u8_vec(block_hash, nonce);
        let hash = sha256_hash(&msg);
        println!("nonce: {:x}, hash: {}", nonce, bytes_to_hex(&hash));
    }
}
