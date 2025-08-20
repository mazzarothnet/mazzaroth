use alloy_rlp::Encodable;
use consensus::types::{AccountKey, Hash, Signature};
use mvm::models::{
    block::Block,
    transfer::{Transfer, TransferInner},
};
use rand::{Rng, SeedableRng, rngs::StdRng};

pub fn push_random_transfers(block: &mut Block, num: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(1112331);
    for _ in 0..num {
        let from = gen_rand_account_key(&mut rng);
        let to = gen_rand_account_key(&mut rng);
        let amount = rng.random_range(0..1000);
        block.inner.transfers.push(Transfer {
            inner: TransferInner {
                from,
                to,
                amount,
                from_last_action_hash: Hash::default(),
                gas_price: 0,
            },
            from_signature: Signature::default(),
        });
    }
}

pub fn gen_rand_account_key(rng: &mut StdRng) -> AccountKey {
    let mut account_key = [0; 33];
    for i in &mut account_key {
        *i = rng.random_range(0..255);
    }
    AccountKey(account_key)
}

pub fn get_block_size(block: &Block) -> usize {
    let mut bytes = Vec::new();
    block.encode(&mut bytes);
    bytes.len() / 1024
}

#[cfg(test)]
mod tests {
    use crate::{
        network::{
            req::{req_block, req_tips},
            test::push_random_transfers,
        },
        state::{
            app_data::set_test_data_path_and_clean,
            block_storage::set_block,
            tips::{gen_test_block, set_test_tips, u32_to_block_key},
        },
    };
    use std::collections::HashSet;
    use utils::sha256::sha256_hash_rlp;

    #[allow(clippy::unwrap_used)]
    #[tokio::test]
    async fn test_get_block_api() {
        set_test_data_path_and_clean("test_get_block_api").unwrap();
        tokio::spawn(async {
            crate::api::serve().await.unwrap();
        });

        let block_key = u32_to_block_key(2);
        let mut block = gen_test_block(2, &HashSet::new());
        push_random_transfers(&mut block, 1500);
        set_block(&block_key, &block).unwrap();
        let block_hash = sha256_hash_rlp(&block);
        let new_block = req_block("localhost:8080", block_key).await.unwrap();
        let new_block_hash = sha256_hash_rlp(&new_block);
        assert_eq!(new_block_hash, block_hash);
    }

    #[allow(clippy::unwrap_used)]
    #[tokio::test]
    async fn test_set_test_tips() {
        tokio::spawn(async {
            crate::api::serve().await.unwrap();
        });
        let key = u32_to_block_key(2);
        set_test_tips(vec![key]).unwrap();
        let tips = req_tips("localhost:8080").await.unwrap();
        assert_eq!(tips.len(), 1);
        assert_eq!(tips[0], key);
    }
}

