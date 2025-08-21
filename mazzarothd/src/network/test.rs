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
        api::spawn_api_thread,
        network::{
            req::{req_block, req_tips},
            test::push_random_transfers,
        },
        state::{
            block_storage::set_block,
            mz_state::{clear_path, get_mz_state},
            tips::{force_set_tips, gen_test_block, u32_to_block_key},
        },
    };
    use std::collections::HashSet;
    use utils::{file::write_to_json, sha256::sha256_hash_rlp};

    #[allow(clippy::unwrap_used)]
    #[tokio::test]
    async fn test_get_block_api() {
        clear_path("test_get_block_api").unwrap();
        let mz_state = get_mz_state("test_get_block_api").unwrap();
        spawn_api_thread(mz_state.clone(), 8081);
        //tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let block_key = u32_to_block_key(2);
        let mut block = gen_test_block(2, &HashSet::new());
        push_random_transfers(&mut block, 1500);
        set_block(&mz_state.block_storage, &block_key, &block).unwrap();
        let block_hash = sha256_hash_rlp(&block);
        let new_block = req_block("localhost:8081", block_key).await.unwrap();
        let new_block_hash = sha256_hash_rlp(&new_block);
        if new_block_hash != block_hash {
            write_to_json("new_block.json", &new_block).unwrap();
            write_to_json("block.json", &block).unwrap();
        }
        assert_eq!(new_block_hash, block_hash);
    }

    #[allow(clippy::unwrap_used)]
    #[tokio::test]
    async fn test_set_test_tips() {
        clear_path("test_set_test_tips").unwrap();
        let mz_state = get_mz_state("test_set_test_tips").unwrap();
        spawn_api_thread(mz_state.clone(), 8082);
        // tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let key = u32_to_block_key(2);
        force_set_tips(vec![key], &mz_state).unwrap();
        let tips = req_tips("localhost:8082").await.unwrap();
        assert_eq!(tips.len(), 1);
        assert_eq!(tips[0], key);
    }
}
