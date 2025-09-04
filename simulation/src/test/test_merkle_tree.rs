#[cfg(test)]
mod tests {
    use crate::smvm::rand_account::{gen_merkle_tree, gen_rand_account};
    use database::rocksdb_no_batch::RocksDbStorage;
    use mvm::core::{
        merkle_tree::MerkleTree,
        storage::{DbStorage, DbStorageTransaction},
    };
    use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};

    #[test]
    fn test_merkle_tree() {
        eprintln!("test_merkle_tree");
        let mts = gen_merkle_tree("tmp_storage");
        let mut mt = MerkleTree::default();
        let mut account_vec = Vec::new();
        let mut rand = StdRng::seed_from_u64(23);
        let account_len = 20;
        let account_num = 100;
        for _i in 0..account_len {
            let accounts = gen_rand_account(&mut rand, account_num);
            account_vec.push(accounts);
        }
        for accounts in &account_vec {
            let mut transaction = mts.begin_transaction().unwrap();
            mt.update_tree::<RocksDbStorage>(&mut transaction, accounts.clone(), vec![])
                .unwrap();
            transaction.commit().unwrap();
        }
        let mut transaction = mts.begin_transaction().unwrap();
        let state_root1 = mt
            .get_state_root::<RocksDbStorage>(&mut transaction)
            .unwrap();
        transaction.commit().unwrap();
        account_vec.shuffle(&mut rand);
        for accounts in account_vec {
            let mut transaction = mts.begin_transaction().unwrap();
            mt.update_tree::<RocksDbStorage>(&mut transaction, accounts, vec![])
                .unwrap();
            transaction.commit().unwrap();
        }
        let mut transaction = mts.begin_transaction().unwrap();
        let state_root2 = mt
            .get_state_root::<RocksDbStorage>(&mut transaction)
            .unwrap();
        transaction.commit().unwrap();
        eprintln!("state_root1: {:?}", state_root1);
        eprintln!("state_root2: {:?}", state_root2);
        assert_eq!(state_root1, state_root2);
    }
}
