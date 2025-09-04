#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::smvm::random_block::{gen_rand_blocks, new_test_mvm};
    use mvm::core::vm::Mvm;
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
            let mut transaction = mvm.begin_transaction().unwrap();
            Mvm::do_block(&mut transaction, block).unwrap();
            let now_state_map = Mvm::get_state_root(&mut transaction).unwrap();
            transaction.commit(block.key).unwrap();
            forward_map.insert(block.key, now_state_map);
        }
        let mut backward_map = BTreeMap::new();
        for block in blocks.iter().rev() {
            let mut transaction = mvm.begin_transaction().unwrap();
            let now_state_map = Mvm::get_state_root(&mut transaction).unwrap();
            Mvm::do_block_rollback(&mut transaction, block).unwrap();
            transaction.commit(block.key).unwrap();
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
                179, 30, 246, 119, 165, 202, 193, 203, 225, 137, 251, 47, 155, 197, 236, 16, 108,
                213, 23, 157, 44, 221, 30, 167, 161, 243, 115, 20, 245, 134, 224, 63
            ]
        );
        eprintln!("backward_hash: {:?}", backward_hash);
        assert_eq!(forward_hash, backward_hash);
        eprintln!("test_mvm done");
    }
}
