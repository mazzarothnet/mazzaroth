
pub fn work_hash_to_package_to_u8_vec(
    block_hash: [u8; 32], work_id: u64
) -> [u8; 48] {
    let mut ans = [0u8; 48];
    for i in 0..32 {
        ans[i] = block_hash[i];
    }
    let work_id_bytes = work_id.to_le_bytes();
    for i in 0..8 {
        ans[i + 32] = work_id_bytes[i];
    }
    let tpnonce: u64 = 0;
    let tpnonce_bytes = tpnonce.to_ne_bytes();
    for i in 0..8 {
        ans[i + 40] = tpnonce_bytes[i];
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
