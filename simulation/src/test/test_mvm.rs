#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::smvm::random_block::{gen_rand_blocks, new_test_mvm};
    use rand::{SeedableRng, rngs::StdRng};
    use utils::sha256::sha256_hash_rlp;

    #[test]
    fn test_mvm() {
        eprintln!("test_mvm");
        let mut mvm = new_test_mvm("test_mvm");
        let mut rng = StdRng::seed_from_u64(11121291);
        let block_num = 30;
        let account_num = 50;
        let blocks = gen_rand_blocks(&mut rng, block_num, account_num);
        let mut forward_map = BTreeMap::new();
        for block in &blocks {
            mvm.do_block(block).unwrap();
            let now_state_map = mvm.get_state_root().unwrap();
            forward_map.insert(block.key, now_state_map);
        }
        let mut backward_map = BTreeMap::new();
        for block in blocks.iter().rev() {
            let now_state_map = mvm.get_state_root().unwrap();
            mvm.do_block_rollback(block).unwrap();
            backward_map.insert(block.key, now_state_map);
        }
        let forward_vec = forward_map.into_values().collect::<Vec<_>>();
        let backward_vec = backward_map.into_values().collect::<Vec<_>>();
        let forward_hash = sha256_hash_rlp(&forward_vec);
        eprintln!("forward_hash: {:?}", forward_hash);
        let backward_hash = sha256_hash_rlp(&backward_vec);
        assert_eq!(
            forward_hash,
            [
                222, 22, 176, 46, 125, 190, 106, 202, 129, 151, 150, 175, 158, 172, 33, 11, 91,
                147, 244, 149, 246, 98, 128, 156, 27, 104, 113, 207, 122, 214, 9, 237
            ]
        );
        eprintln!("backward_hash: {:?}", backward_hash);
        assert_eq!(forward_hash, backward_hash);
        eprintln!("test_mvm done");
    }
}
