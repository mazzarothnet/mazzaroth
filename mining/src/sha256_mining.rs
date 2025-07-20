
pub fn work_hash_to_package_to_u8_vec(
    block_hash: [u8; 32], work_id: u64
) -> [u8; 48] {
    let mut ans = [0u8; 48];
    for i in 0..32 {
        ans[i] = block_hash[i];
    }
    let work_id_bytes = work_id.to_be_bytes();
    for i in 0..8 {
        ans[i + 32] = work_id_bytes[i];
    }
    let tpnonce: u64 = 0;
    let tpnonce_bytes = tpnonce.to_be_bytes();
    for i in 0..8 {
        ans[i + 40] = tpnonce_bytes[i];
    }
    ans
}

pub fn vec_to_nonce(nonce_vec: [u32; 4]) -> u128 {
    let mut nonce = 0u128;
    for i in 0..4 {
        nonce = nonce << 32;
        nonce = nonce | nonce_vec[i] as u128;
    }
    nonce
}

pub fn nonce_hash_to_package_to_u8_vec(block_hash: [u8; 32], nonce: u128) -> [u8; 48] {
    let mut ans = [0u8; 48];
    for i in 0..32 {
        ans[i] = block_hash[i];
    }
    let nonce_bytes = nonce.to_be_bytes();
    for i in 0..16 {
        ans[i + 32] = nonce_bytes[i];
    }
    ans
}

pub fn work_hash_to_package(block_hash: [u8; 32], work_id: u64) -> [u32; 16] {
    let package = work_hash_to_package_to_u8_vec(block_hash, work_id);
    let mut ans: [u8; 64] = [0; 64];
    for i in 0..48 {
        ans[i] = package[i];
    }
    ans[48] = 0x80;
    let bitlen: u64 = 48 * 8;
    let bitlen_bytes = bitlen.to_be_bytes();
    for i in 0..8 {
        ans[i + 56] = bitlen_bytes[i];
    }
    let mut real_ans = [0u32; 16];
    for i in 0..16 {
        real_ans[i] = u32::from_be_bytes([
            ans[i * 4], ans[i * 4 + 1], ans[i * 4 + 2], ans[i * 4 + 3]
        ]);
    }
    real_ans
}

pub fn bytes_to_hex(bytes: &[u8; 32]) -> String {
    let mut result = String::new();
    for &byte in bytes {
        result.push_str(&format!("{:02x}", byte));
    }
    result
}