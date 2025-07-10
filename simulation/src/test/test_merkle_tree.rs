#[cfg(test)]
mod tests {
    use crate::smvm::sim_state_root::{gen_merkle_tree, gen_rand_account};
    use mvm::core::storage::DbStorageTransaction;
    use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};

    #[test]
    fn test_merkle_tree() {
        let mut mt = gen_merkle_tree("tmp_storage");
        let mut account_vec = Vec::new();
        let mut rand = StdRng::seed_from_u64(23);
        let account_len = 20;
        let account_num = 100;
        for _i in 0..account_len {
            let accounts = gen_rand_account(&mut rand, account_num);
            account_vec.push(accounts);
        }
        for accounts in &account_vec {
            let ts = mt.update_tree(accounts.clone(), vec![]).unwrap();
            ts.commit().unwrap();
        }
        let state_root1 = mt.get_state_root().unwrap();
        account_vec.shuffle(&mut rand);
        for accounts in account_vec {
            let ts = mt.update_tree(accounts, vec![]).unwrap();
            ts.commit().unwrap();
        }
        let state_root2 = mt.get_state_root().unwrap();
        assert_eq!(state_root1, state_root2);
    }
}
