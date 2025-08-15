#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::run_gpu::{get_test_sha256_cpu, get_test_sha256_gpu};
    use utils::sha256::sha256_hash;

    #[test]
    fn test_sha256_gpu() {
        let block_hash = sha256_hash(b"123hqw1123123");
        let work_id = 12312312;
        let ans1 = get_test_sha256_gpu(block_hash, work_id).unwrap();
        let ans2 = get_test_sha256_cpu(block_hash, work_id);
        eprintln!("ans1: {:?}", ans1);
        eprintln!("ans2: {:?}", ans2);
        assert_eq!(ans1, ans2);
    }
}
