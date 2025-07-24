use std::{collections::BTreeMap, path::Path};

use consensus::{
    TRANSFER_GAS,
    block_header::ConsensusHeader,
    get_now_block_reward,
    types::{AccountKey, BlockKey, Hash, Signature},
};
use crypto_bigint::U256;
use database::rocksdb_no_batch::RocksDbStorage;
use log::debug;
use mvm::{
    core::{merkle_tree::MerkleTree, vm::Mvm},
    models::{
        account::Account,
        block::{Block, BlockInner},
        transfer::{Merge, MergeInner, Transfer, TransferInner},
    },
};
use rand::{Rng, rngs::StdRng};
use utils::{
    secp::{gen_keypair, sign_message},
    sha256::sha256_hash_rlp,
};

pub fn new_test_mvm(path: &str) -> Mvm<RocksDbStorage> {
    if Path::new(path).exists() {
        // delete
        std::fs::remove_dir_all(path).unwrap();
    }
    load_mvm(path)
}

pub fn load_mvm(path: &str) -> Mvm<RocksDbStorage> {
    let merkle_path = format!("{}/merkle", path);
    let account_path = format!("{}/account", path);
    let merkle_tree_db = RocksDbStorage::new(&merkle_path).unwrap();
    let merkle_tree = MerkleTree::new(merkle_tree_db).unwrap();
    let account_db = RocksDbStorage::new(&account_path).unwrap();
    Mvm::new(account_db, merkle_tree)
}

#[derive(Debug, Clone)]
struct AccountPackage {
    account: Account,
    secret_key: [u8; 32],
}

pub fn gen_empty_block(account_num: u64) -> Vec<Block> {
    let mut blocks = Vec::new();
    for i in 0..account_num {
        blocks.push(Block {
            key: BlockKey(U256::from_u64(i)),
            nonce: 0,
            inner: BlockInner {
                version: 0,
                header: ConsensusHeader::default(),
                transfers: vec![],
                merges: vec![],
                miner: AccountKey([i as u8; 33]),
                miner_last_action_hash: Hash([0; 32]),
            },
        });
    }
    blocks
}

pub fn gen_rand_blocks(rng: &mut StdRng, block_num: u64, account_num: u64) -> Vec<Block> {
    let mut account_map = BTreeMap::new();
    for i in 0..account_num {
        let (secret_key, public_key) = gen_keypair(rng);
        let account_key = AccountKey(public_key);
        account_map.insert(
            i,
            AccountPackage {
                account: Account {
                    key: account_key,
                    balance: 0,
                    action_hash: Hash([0; 32]),
                },
                secret_key,
            },
        );
    }
    let mut blocks = Vec::new();
    for i in 0..block_num {
        let block_key = BlockKey(U256::from_u64(i));
        let miner_index = get_rand_exist_index(rng, &account_map);
        let (miner_key, miner_last_action_hash) = {
            let miner_package = account_map.get_mut(&miner_index).unwrap();
            let last_action_hash = miner_package.account.action_hash;
            miner_package.account.balance += get_now_block_reward(0);
            miner_package.account.action_hash = Hash(block_key.0.to_be_bytes());
            debug!(
                "random miner update account: {:?} {:?} {:?}",
                miner_package.account.key,
                miner_package.account.action_hash,
                miner_package.account.balance
            );
            (miner_package.account.key, last_action_hash)
        };
        let mut block = Block {
            key: block_key,
            nonce: 0,
            inner: BlockInner {
                version: 0,
                header: ConsensusHeader::default(),
                transfers: vec![],
                merges: vec![],
                miner: miner_key,
                miner_last_action_hash,
            },
        };

        for _i in 0..600 {
            block.inner.transfers.push(gen_rand_transfer(
                rng,
                &mut account_map,
                account_num,
                miner_index,
            ));
        }

        for _i in 0..30 {
            block.inner.merges.push(gen_rand_merge(
                rng,
                &mut account_map,
                account_num,
                miner_index,
                miner_key,
            ));
        }

        blocks.push(block);
    }
    blocks
}

fn gen_rand_transfer(
    rng: &mut StdRng,
    account_map: &mut BTreeMap<u64, AccountPackage>,
    account_num: u64,
    miner_index: u64,
) -> Transfer {
    let mut accepted_sign = true;
    let from_index = get_rand_from_index(rng, account_map);
    let to_index = rng.random_range(0..account_num);
    let to_package = account_map
        .entry(to_index)
        .or_insert(AccountPackage {
            account: Account {
                key: AccountKey([0; 33]),
                balance: 0,
                action_hash: Hash([0; 32]),
            },
            secret_key: [0; 32],
        })
        .clone();
    let from_package = account_map.get(&from_index).unwrap();
    let transfer_amount = rng.random_range(0..from_package.account.balance / 2);
    let transfer_real_hash = rng.random_range(0..10);
    let transfer_hash = if transfer_real_hash == 0 {
        Hash([0; 32])
    } else {
        from_package.account.action_hash
    };
    let gas_price = rng.random_range(0..2);
    let transfer_inner = TransferInner {
        from: from_package.account.key,
        to: to_package.account.key,
        amount: transfer_amount,
        from_last_action_hash: transfer_hash,
        gas_price,
    };
    let transfer_inner_hash = sha256_hash_rlp(&transfer_inner);
    let signature =
        sign_message(&transfer_inner_hash, &from_package.secret_key).unwrap_or_else(|_| {
            accepted_sign = false;
            [0; 64]
        });
    let transfer = Transfer {
        inner: transfer_inner,
        from_signature: Signature(signature),
    };
    if transfer.inner.amount + transfer.inner.gas_price <= from_package.account.balance
        && transfer.inner.from_last_action_hash == from_package.account.action_hash
        && transfer.inner.from != transfer.inner.to
        && account_map.contains_key(&from_index)
        && account_map.contains_key(&miner_index)
        && accepted_sign
    {
        let from_package = account_map.get_mut(&from_index).unwrap();
        from_package.account.balance -=
            transfer.inner.amount + transfer.inner.gas_price * TRANSFER_GAS;
        from_package.account.action_hash = Hash(transfer_inner_hash);
        debug!(
            "random transfer from update account: {:?} {:?} {:?}",
            from_package.account.key,
            from_package.account.action_hash,
            from_package.account.balance
        );
        let to_package = account_map.get_mut(&to_index).unwrap();
        to_package.account.balance += transfer.inner.amount;
        debug!(
            "random transfer to update account: {:?} {:?} {:?}",
            to_package.account.key, to_package.account.action_hash, to_package.account.balance
        );
        let miner_package = account_map.get_mut(&miner_index).unwrap();
        miner_package.account.balance += transfer.inner.gas_price * TRANSFER_GAS;

        debug!(
            "random transfer minner update account: {:?} {:?} {:?}",
            miner_package.account.key,
            miner_package.account.action_hash,
            miner_package.account.balance
        );
    }

    transfer
}

fn get_rand_from_index(rng: &mut StdRng, account_map: &BTreeMap<u64, AccountPackage>) -> u64 {
    let mut has_set = Vec::new();
    for (i, v) in account_map {
        if v.account.balance > 0 {
            has_set.push(*i);
        }
    }
    if has_set.is_empty() {
        debug!(
            "get_rand_from_index has_set is empty use {:?}",
            account_map.get(&0).unwrap().account.key
        );
        return 0;
    }
    let index = rng.random_range(0..has_set.len());
    has_set[index]
}

fn get_rand_exist_index(rng: &mut StdRng, account_map: &BTreeMap<u64, AccountPackage>) -> u64 {
    let mut has_set = Vec::new();
    for (i, _v) in account_map {
        has_set.push(*i);
    }
    if has_set.is_empty() {
        debug!(
            "get_rand_from_index has_set is empty use {:?}",
            account_map.get(&0).unwrap().account.key
        );
        return 0;
    }
    let index = rng.random_range(0..has_set.len());
    has_set[index]
}

fn gen_rand_merge(
    rng: &mut StdRng,
    account_map: &mut BTreeMap<u64, AccountPackage>,
    account_num: u64,
    miner_index: u64,
    miner_key: AccountKey,
) -> Merge {
    let mut accepted_sign = true;
    let from_index = get_rand_from_index(rng, account_map);
    let to_index = rng.random_range(0..account_num);
    let from_package = account_map.get(&from_index).unwrap();
    let to_package = account_map.get(&to_index).unwrap_or(&AccountPackage {
        account: Account {
            key: AccountKey([0; 33]),
            balance: 0,
            action_hash: Hash([0; 32]),
        },
        secret_key: [0; 32],
    });
    let merge_amount_real_amount = rng.random_range(0..10);
    let merge_amount = if merge_amount_real_amount == 9 {
        debug!("gen_rand_merge rand");
        rng.random_range(0..from_package.account.balance)
    } else {
        debug!(
            "gen_rand_merge from_package.account.balance: {:?}",
            from_package.account.balance
        );
        from_package.account.balance
    };
    let merge_from_real_hash = rng.random_range(0..2);
    let merge_from_hash = if merge_from_real_hash == 0 {
        Hash([0; 32])
    } else {
        from_package.account.action_hash
    };
    let merge_to_real_hash = rng.random_range(0..2);
    let merge_to_hash = if merge_to_real_hash == 0 {
        Hash([0; 32])
    } else {
        to_package.account.action_hash
    };
    let gas_price = rng.random_range(0..2);
    let merge_inner = MergeInner {
        from: from_package.account.key,
        to: to_package.account.key,
        balance: merge_amount,
        from_last_action_hash: merge_from_hash,
        to_last_action_hash: merge_to_hash,
        gas_price,
    };
    let merge_inner_hash = sha256_hash_rlp(&merge_inner);
    let from_signature =
        sign_message(&merge_inner_hash, &from_package.secret_key).unwrap_or_else(|_| {
            accepted_sign = false;
            [0; 64]
        });
    let to_signature =
        sign_message(&merge_inner_hash, &to_package.secret_key).unwrap_or_else(|_| {
            accepted_sign = false;
            [0; 64]
        });
    let merge = Merge {
        inner: merge_inner,
        from_signature: Signature(from_signature),
        to_signature: Signature(to_signature),
    };
    if merge.inner.balance == from_package.account.balance
        && merge.inner.balance >= merge.inner.gas_price * TRANSFER_GAS
        && merge.inner.to_last_action_hash == to_package.account.action_hash
        && merge.inner.from_last_action_hash == from_package.account.action_hash
        && merge.inner.from != merge.inner.to
        && merge.inner.from != miner_key
        && account_map.contains_key(&to_index)
        && account_map.contains_key(&from_index)
        && account_map.contains_key(&miner_index)
        && accepted_sign
    {
        account_map.remove(&from_index);
        let to_package = account_map.get_mut(&to_index).unwrap();
        to_package.account.balance += merge.inner.balance - merge.inner.gas_price * TRANSFER_GAS;
        to_package.account.action_hash = Hash(merge_inner_hash);
        debug!(
            "random merge to update account: {:?} {:?} {:?}",
            to_package.account.key, to_package.account.action_hash, to_package.account.balance
        );
        let miner_package = account_map.get_mut(&miner_index).unwrap();
        miner_package.account.balance += merge.inner.gas_price * TRANSFER_GAS;
        debug!(
            "random merge minner update account: {:?} {:?} {:?}",
            miner_package.account.key,
            miner_package.account.action_hash,
            miner_package.account.balance
        );
    }

    merge
}
