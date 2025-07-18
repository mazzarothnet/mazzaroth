#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::smvm::random_block::{gen_rand_blocks, new_test_mvm};
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn test_mvm() {
        let mut mvm = new_test_mvm("test_mvm");
        let mut rng = StdRng::seed_from_u64(11121291);
        let block_num = 33;
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
        assert_eq!(forward_vec, backward_vec);
        println!("test_mvm done");
    }
}
