use std::collections::{BTreeMap, BTreeSet};

#[cfg(not(feature = "disable_storage_limit"))]
use consensus::{
    STO_ACCOUNT_MIN_BALANCE, TRANSFER_GAS, get_now_block_reward,
    types::{AccountKey, Hash},
};
#[cfg(feature = "disable_storage_limit")]
use consensus::{
    TRANSFER_GAS, get_now_block_reward,
    types::{AccountKey, Hash},
};
use log::{debug, info};
use utils::{
    error::{Error, Result},
    secp::verify_message,
    sha256::sha256_hash_rlp,
};

use crate::{
    core::{
        merkle_tree::MerkleTree,
        storage::{DbStorage, DbStorageTransaction},
    },
    models::{
        account::Account,
        block::Block,
        transfer::{Merge, Transfer},
    },
};

pub struct Mvm<S: DbStorage> {
    account_db: S,
    merkle_tree: MerkleTree<S>,
}

impl<S: DbStorage> Mvm<S> {
    pub fn new(account_db: S, merkle_tree: MerkleTree<S>) -> Self {
        Self {
            account_db,
            merkle_tree,
        }
    }

    pub fn verify_transfer(
        transfer: &Transfer,
        minner: &AccountKey,
        now_state_map: &BTreeMap<AccountKey, Account>,
        transfer_hash: Hash,
    ) -> Result<u128> {
        if transfer.inner.from == transfer.inner.to {
            return Err(Error::MergeFromAndToIsTheSame {
                message: format!(
                    "transfer from and to is the same: {:?}",
                    transfer.inner.from
                ),
            });
        }
        verify_message(
            &transfer_hash.0,
            &transfer.from_signature.0,
            &transfer.inner.from.0,
        )?;
        let cast = transfer.inner.amount + transfer.inner.gas_price * TRANSFER_GAS;
        #[cfg(not(feature = "disable_storage_limit"))]
        let min_need = cast + STO_ACCOUNT_MIN_BALANCE;
        #[cfg(feature = "disable_storage_limit")]
        let min_need = cast;

        {
            let from_account =
                now_state_map
                    .get(&transfer.inner.from)
                    .ok_or_else(|| Error::AccountNotFound {
                        message: format!("from account not found: {:?}", transfer.inner.from),
                    })?;
            if from_account.balance < min_need {
                return Err(Error::AccountBalanceNotEnough {
                    message: format!(
                        "from account balance not enough: {:?} balance: {:?} min_need: {:?}",
                        transfer.inner.from, from_account.balance, min_need
                    ),
                });
            }
            if from_account.action_hash != transfer.inner.from_last_action_hash {
                return Err(Error::AccountHashNotMatch {
                    message: format!(
                        "from account action hash not match: {:?}",
                        transfer.inner.from
                    ),
                });
            }
        }
        #[cfg(not(feature = "disable_storage_limit"))]
        if !now_state_map.contains_key(&transfer.inner.to)
            && transfer.inner.amount < STO_ACCOUNT_MIN_BALANCE
        {
            return Err(Error::AccountBalanceNotEnough {
                message: format!("to account balance not enough: {:?}", transfer.inner.from),
            });
        }
        if !now_state_map.contains_key(minner) {
            return Err(Error::AccountNotFound {
                message: format!("minner account not found: {:?}", minner),
            });
        }
        Ok(min_need)
    }

    pub fn verify_merge(
        merge: &Merge,
        minner: &AccountKey,
        now_state_map: &BTreeMap<AccountKey, Account>,
        merge_hash: Hash,
    ) -> Result<()> {
        if merge.inner.from == merge.inner.to || merge.inner.from == *minner {
            return Err(Error::MergeFromAndToIsTheSame {
                message: format!("merge from and to is the same: {:?}", merge.inner.from),
            });
        }
        verify_message(&merge_hash.0, &merge.from_signature.0, &merge.inner.from.0)?;
        verify_message(&merge_hash.0, &merge.to_signature.0, &merge.inner.to.0)?;

        let from_account =
            now_state_map
                .get(&merge.inner.from)
                .ok_or_else(|| Error::AccountNotFound {
                    message: format!("from account not found: {:?}", merge.inner.from),
                })?;
        if merge.inner.balance != from_account.balance {
            return Err(Error::AccountBalanceNotEnough {
                message: format!(
                    "from account balance not enough: {:?} merge amount: {:?} balance: {:?}",
                    merge.inner.from, merge.inner.balance, from_account.balance
                ),
            });
        }
        if merge.inner.balance < merge.inner.gas_price * TRANSFER_GAS {
            return Err(Error::AccountBalanceNotEnough {
                message: format!(
                    "from account balance not enough: {:?} merge amount {} gas price {}",
                    merge.inner.from,
                    merge.inner.balance,
                    merge.inner.gas_price * TRANSFER_GAS
                ),
            });
        }
        if from_account.action_hash != merge.inner.from_last_action_hash {
            return Err(Error::AccountHashNotMatch {
                message: format!("from account action hash not match: {:?}", merge.inner.from),
            });
        }

        let to_account =
            now_state_map
                .get(&merge.inner.to)
                .ok_or_else(|| Error::AccountNotFound {
                    message: format!("to account not found: {:?}", merge.inner.to),
                })?;
        if to_account.action_hash != merge.inner.to_last_action_hash {
            return Err(Error::AccountHashNotMatch {
                message: format!("to account action hash not match: {:?}", merge.inner.to),
            });
        }

        if !now_state_map.contains_key(minner) {
            return Err(Error::AccountNotFound {
                message: format!("minner account not found: {:?}", minner),
            });
        }

        Ok(())
    }

    pub fn do_block(&mut self, block: &Block) -> Result<()> {
        let mut account_transaction = self.account_db.begin_transaction()?;
        let mut now_state_map = Self::get_now_state_map(&mut account_transaction, block)?;
        let mut delete_set = BTreeSet::new();

        #[cfg(debug_assertions)]
        {
            use utils::file::write_to_json;

            write_to_json(
                &format!(
                    "test_mvm/forward_before_{}.json",
                    get_key_from_block_key(&block.key.0.to_be_bytes())
                ),
                &get_map(&now_state_map),
            )?;
        }

        Self::do_all_transfer_and_merge(block, &mut now_state_map, &mut delete_set)?;

        #[cfg(debug_assertions)]
        {
            use utils::file::write_to_json;
            write_to_json(
                &format!(
                    "test_mvm/forward_after_{}.json",
                    get_key_from_block_key(&block.key.0.to_be_bytes())
                ),
                &get_map(&now_state_map),
            )?;
        }

        let set_account_hash =
            Self::save_now_state_map(&mut account_transaction, &now_state_map, &delete_set)?;
        let transaction = self
            .merkle_tree
            .update_tree(set_account_hash, delete_set.into_iter().collect())?;
        transaction.commit()?;
        account_transaction.commit()?;

        Ok(())
    }

    pub fn do_block_rollback(&mut self, block: &Block) -> Result<()> {
        let mut account_transaction = self.account_db.begin_transaction()?;
        let mut now_state_map = Self::get_now_state_map(&mut account_transaction, block)?;
        let mut delete_set = BTreeSet::new();

        #[cfg(debug_assertions)]
        {
            use utils::file::write_to_json;

            write_to_json(
                &format!(
                    "test_mvm/rollback_before_{}.json",
                    get_key_from_block_key(&block.key.0.to_be_bytes())
                ),
                &get_map(&now_state_map),
            )?;
        }

        Self::rollback_all_transfer_and_merge(block, &mut now_state_map, &mut delete_set)?;

        #[cfg(debug_assertions)]
        {
            use utils::file::write_to_json;

            write_to_json(
                &format!(
                    "test_mvm/rollback_after_{}.json",
                    get_key_from_block_key(&block.key.0.to_be_bytes())
                ),
                &get_map(&now_state_map),
            )?;
        }

        let set_account_hash =
            Self::save_now_state_map(&mut account_transaction, &now_state_map, &delete_set)?;
        let transaction = self
            .merkle_tree
            .update_tree(set_account_hash, delete_set.into_iter().collect())?;
        transaction.commit()?;
        account_transaction.commit()?;

        Ok(())
    }

    pub fn get_state_root(&self) -> Result<Hash> {
        self.merkle_tree.get_state_root()
    }

    fn rollback_all_transfer_and_merge(
        block: &Block,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        delete_set: &mut BTreeSet<AccountKey>,
    ) -> Result<()> {
        // 逆序
        for merge in block.inner.merges.iter().rev() {
            let merge_hash = Hash(sha256_hash_rlp(&merge.inner));
            if let Err(e) =
                Self::do_merge_rollback(merge, &block.inner.miner, now_state_map, merge_hash)
            {
                if let Error::Impossible { message } = e {
                    return Err(Error::Impossible { message });
                } else {
                    info!(
                        "error rollback merge merge_hash: {:?} merge: {:?} e: {:?}",
                        merge_hash, merge, e
                    );
                }
            } else {
                debug!(
                    "success rollback merge merge_hash: {:?} merge: {:?}",
                    merge_hash, merge
                );
            }
        }
        for transfer in block.inner.transfers.iter().rev() {
            let transfer_hash = Hash(sha256_hash_rlp(&transfer.inner));
            if let Err(e) = Self::do_transfer_rollback(
                transfer,
                &block.inner.miner,
                now_state_map,
                delete_set,
                transfer_hash,
            ) {
                if let Error::Impossible { message } = e {
                    return Err(Error::Impossible { message });
                } else {
                    info!(
                        "error rollback transfer transfer_hash: {:?} transfer: {:?} e: {:?}",
                        transfer_hash, transfer, e
                    );
                }
            } else {
                debug!(
                    "success rollback transfer transfer_hash: {:?} transfer: {:?}",
                    transfer_hash, transfer
                );
            }
        }
        if let Err(e) =
            Self::do_minner_reward_rollback(block, now_state_map, delete_set, &block.inner.miner)
        {
            if let Error::Impossible { message } = e {
                return Err(Error::Impossible { message });
            } else {
                info!(
                    "error rollback minner reward key: {:?} e: {:?}",
                    block.key, e
                );
            }
        } else {
            debug!("success rollback minner reward key: {:?}", block.key);
        }
        Ok(())
    }

    fn get_now_state_map<'a>(
        transaction: &mut S::Transaction<'a>,
        block: &Block,
    ) -> Result<BTreeMap<AccountKey, Account>> {
        let account_set = Self::get_account_set(block);
        let now_state_map = transaction
            .batch_read::<AccountKey, Account>(account_set.into_iter().collect())?
            .into_iter()
            .map(|account| (account.key, account))
            .collect::<BTreeMap<AccountKey, Account>>();
        Ok(now_state_map)
    }

    fn save_now_state_map<'a>(
        transaction: &mut S::Transaction<'a>,
        now_state_map: &BTreeMap<AccountKey, Account>,
        delete_set: &BTreeSet<AccountKey>,
    ) -> Result<Vec<(AccountKey, Hash)>> {
        for key in delete_set {
            transaction.delete_data(key)?;
        }
        let mut set_account_hash = Vec::new();
        for (key, account) in now_state_map {
            let account_hash = sha256_hash_rlp(&account);
            transaction.set_data(key, account)?;
            set_account_hash.push((*key, Hash(account_hash)));
        }
        Ok(set_account_hash)
    }

    fn verify_minner_reward(
        now_state_map: &BTreeMap<AccountKey, Account>,
        block: &Block,
    ) -> Result<()> {
        #[cfg(not(feature = "disable_storage_limit"))]
        {
            let now_reward = get_now_block_reward(block.inner.header.part_sort_header.size);
            if !now_state_map.contains_key(&block.inner.miner)
                && now_reward < STO_ACCOUNT_MIN_BALANCE
            {
                return Err(Error::AccountBalanceNotEnough {
                    message: format!("minner account balance not enough: {:?}", block.inner.miner),
                });
            }
        }
        let minner_account_action_hash = now_state_map
            .get(&block.inner.miner)
            .map(|v| v.action_hash)
            .unwrap_or(Hash([0; 32]));
        if minner_account_action_hash != block.inner.miner_last_action_hash {
            return Err(Error::AccountHashNotMatch {
                message: format!(
                    "minner account action hash not match: {:?} {:?} {:?}",
                    block.inner.miner,
                    minner_account_action_hash,
                    block.inner.miner_last_action_hash
                ),
            });
        }
        Ok(())
    }

    fn do_minner_reward(
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        block: &Block,
    ) -> Result<()> {
        Self::verify_minner_reward(now_state_map, block)?;
        let now_reward = get_now_block_reward(block.inner.header.part_sort_header.size);

        let minner_account = now_state_map.entry(block.inner.miner).or_insert(Account {
            key: block.inner.miner,
            balance: 0,
            action_hash: Hash([0; 32]),
        });

        if minner_account.balance == 0 {
            debug!(
                "mining create account {:?} {:?}",
                minner_account.key, block.key
            )
        }

        minner_account.balance += now_reward;
        minner_account.action_hash = Hash(block.key.0.to_be_bytes());
        debug!(
            "do miner update account: {:?} {:?} {:?}",
            minner_account.key, minner_account.action_hash, minner_account.balance
        );
        Ok(())
    }

    fn get_account_set(block: &Block) -> Vec<AccountKey> {
        let mut account_set = BTreeSet::new();
        account_set.insert(block.inner.miner);
        for transfer in &block.inner.transfers {
            account_set.insert(transfer.inner.from);
            account_set.insert(transfer.inner.to);
        }
        for merge in &block.inner.merges {
            account_set.insert(merge.inner.from);
            account_set.insert(merge.inner.to);
        }
        account_set.into_iter().collect()
    }

    fn do_all_transfer_and_merge(
        block: &Block,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        delete_set: &mut BTreeSet<AccountKey>,
    ) -> Result<()> {
        if let Err(e) = Self::do_minner_reward(now_state_map, block) {
            info!(
                "error do minner reward key: {:?}, minner:{:?} e: {:?}",
                block.key, block.inner.miner, e,
            );
        } else {
            debug!(
                "success do minner reward key: {:?} miner: {:?} reward: {:?}",
                block.key,
                block.inner.miner,
                get_now_block_reward(block.inner.header.part_sort_header.size),
            );
        }
        for transfer in &block.inner.transfers {
            let transfer_hash = Hash(sha256_hash_rlp(&transfer.inner));
            if let Err(e) =
                Self::do_transfer(transfer, &block.inner.miner, now_state_map, transfer_hash)
            {
                if let Error::Impossible { message } = e {
                    return Err(Error::Impossible { message });
                } else {
                    info!(
                        "error do transfer transfer_hash: {:?} transfer: {:?} e: {:?}",
                        transfer_hash, transfer, e,
                    );
                }
            } else {
                debug!(
                    "success do transfer transfer_hash: {:?} transfer: {:?}",
                    transfer_hash, transfer
                );
            }
        }
        for merge in &block.inner.merges {
            let merge_hash = Hash(sha256_hash_rlp(&merge.inner));
            if let Err(e) = Self::do_merge(
                merge,
                &block.inner.miner,
                now_state_map,
                delete_set,
                merge_hash,
            ) {
                if let Error::Impossible { message } = e {
                    return Err(Error::Impossible { message });
                } else {
                    info!(
                        "error do merge merge_hash: {:?} merge: {:?} e: {:?}",
                        merge_hash, merge, e,
                    );
                }
            } else {
                debug!(
                    "success do merge merge_hash: {:?} merge: {:?}",
                    merge_hash, merge
                );
            }
        }

        Ok(())
    }

    fn do_merge(
        merge: &Merge,
        minner: &AccountKey,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        delete_set: &mut BTreeSet<AccountKey>,
        merge_hash: Hash,
    ) -> Result<()> {
        Self::verify_merge(merge, minner, now_state_map, merge_hash)?;
        now_state_map.remove(&merge.inner.from);
        delete_set.insert(merge.inner.from);
        let to_account =
            now_state_map
                .get_mut(&merge.inner.to)
                .ok_or_else(|| Error::Impossible {
                    message: format!("do_merge to account not found: {:?}", merge.inner.to),
                })?;
        to_account.balance += merge.inner.balance - merge.inner.gas_price * TRANSFER_GAS;
        to_account.action_hash = merge_hash;
        debug!(
            "do merge to update account: {:?} {:?} {:?}",
            to_account.key, to_account.action_hash, to_account.balance
        );
        let minner_account = now_state_map
            .get_mut(minner)
            .ok_or_else(|| Error::Impossible {
                message: format!("minner account not found: {:?}", minner),
            })?;
        minner_account.balance += merge.inner.gas_price * TRANSFER_GAS;
        debug!(
            "do merge minner update account: {:?} {:?} {:?}",
            minner_account.key, minner_account.action_hash, minner_account.balance
        );

        Ok(())
    }

    fn do_transfer(
        transfer: &Transfer,
        minner: &AccountKey,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        transfer_hash: Hash,
    ) -> Result<()> {
        let need = Self::verify_transfer(transfer, minner, now_state_map, transfer_hash)?;
        let cast = transfer.inner.amount + transfer.inner.gas_price * TRANSFER_GAS;
        let from_account =
            now_state_map
                .get_mut(&transfer.inner.from)
                .ok_or_else(|| Error::Impossible {
                    message: format!("from account not found: {:?}", transfer.inner.from),
                })?;
        let old_balance = from_account.balance;
        from_account.balance -= cast;
        from_account.action_hash = transfer_hash;
        debug!(
            "do transfer from update account: {:?} {:?} {:?} balance: {:?} need: {}",
            from_account.key, from_account.action_hash, from_account.balance, old_balance, need
        );

        let to_account = now_state_map.entry(transfer.inner.to).or_insert(Account {
            key: transfer.inner.to,
            balance: 0,
            action_hash: Hash([0; 32]),
        });
        if to_account.balance == 0 {
            debug!(
                "transfer create account: {:?} {:?}",
                to_account.key, transfer_hash
            )
        }
        to_account.balance += transfer.inner.amount;

        debug!(
            "do transfer to update account: {:?} {:?} {:?}",
            to_account.key, to_account.action_hash, to_account.balance
        );

        let minner_account = now_state_map
            .get_mut(minner)
            .ok_or_else(|| Error::Impossible {
                message: format!("minner account not found: {:?}", minner),
            })?;
        minner_account.balance += transfer.inner.gas_price * TRANSFER_GAS;

        debug!(
            "do transfer minner update account: {:?} {:?} {:?}",
            minner_account.key, minner_account.action_hash, minner_account.balance
        );

        Ok(())
    }

    fn do_minner_reward_rollback(
        block: &Block,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        delete_set: &mut BTreeSet<AccountKey>,
        minner: &AccountKey,
    ) -> Result<()> {
        let minner_account =
            now_state_map
                .get_mut(minner)
                .ok_or_else(|| Error::AccountNotFound {
                    message: format!("minner account not found: {:?}", minner),
                })?;
        if minner_account.action_hash != Hash(block.key.0.to_be_bytes()) {
            return Err(Error::AccountHashNotMatch {
                message: format!(
                    "minner account action hash not match: {:?}",
                    block.inner.miner
                ),
            });
        }
        minner_account.balance -= get_now_block_reward(block.inner.header.part_sort_header.size);
        minner_account.action_hash = block.inner.miner_last_action_hash;
        if minner_account.balance == 0 {
            now_state_map.remove(minner);
            delete_set.insert(*minner);
        }
        Ok(())
    }

    fn do_transfer_rollback(
        transfer: &Transfer,
        minner: &AccountKey,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        delete_set: &mut BTreeSet<AccountKey>,
        transfer_hash: Hash,
    ) -> Result<()> {
        Self::verify_transfer_rollback(transfer, now_state_map, transfer_hash)?;
        let from_account =
            now_state_map
                .get_mut(&transfer.inner.from)
                .ok_or_else(|| Error::Impossible {
                    message: format!("from account not found: {:?}", transfer.inner.from),
                })?;
        from_account.balance += transfer.inner.amount + transfer.inner.gas_price * TRANSFER_GAS;
        from_account.action_hash = transfer.inner.from_last_action_hash;
        let to_account =
            now_state_map
                .get_mut(&transfer.inner.to)
                .ok_or_else(|| Error::Impossible {
                    message: format!(
                        "do_transfer_rollback to account not found: {:?}",
                        transfer.inner.to
                    ),
                })?;
        to_account.balance -= transfer.inner.amount;
        if to_account.balance == 0 {
            now_state_map.remove(&transfer.inner.to);
            delete_set.insert(transfer.inner.to);
        }
        let minner_account = now_state_map
            .get_mut(minner)
            .ok_or_else(|| Error::Impossible {
                message: format!("minner account not found: {:?}", minner),
            })?;
        minner_account.balance -= transfer.inner.gas_price * TRANSFER_GAS;
        Ok(())
    }

    fn do_merge_rollback(
        merge: &Merge,
        minner: &AccountKey,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        merge_hash: Hash,
    ) -> Result<()> {
        Self::verify_merge_rollback(merge, now_state_map, merge_hash)?;
        now_state_map.insert(
            merge.inner.from,
            Account {
                key: merge.inner.from,
                balance: merge.inner.balance,
                action_hash: merge.inner.from_last_action_hash,
            },
        );
        let to_account =
            now_state_map
                .get_mut(&merge.inner.to)
                .ok_or_else(|| Error::Impossible {
                    message: format!(
                        "do_merge_rollback to account not found: {:?}",
                        merge.inner.to
                    ),
                })?;
        to_account.balance -= merge.inner.balance - merge.inner.gas_price * TRANSFER_GAS;
        to_account.action_hash = merge.inner.to_last_action_hash;
        let minner_account = now_state_map
            .get_mut(minner)
            .ok_or_else(|| Error::Impossible {
                message: format!("minner account not found: {:?}", minner),
            })?;
        minner_account.balance -= merge.inner.gas_price * TRANSFER_GAS;

        Ok(())
    }

    fn verify_transfer_rollback(
        transfer: &Transfer,
        now_state_map: &BTreeMap<AccountKey, Account>,
        transfer_hash: Hash,
    ) -> Result<()> {
        verify_message(
            &transfer_hash.0,
            &transfer.from_signature.0,
            &transfer.inner.from.0,
        )?;
        let from_account =
            now_state_map
                .get(&transfer.inner.from)
                .ok_or_else(|| Error::AccountNotFound {
                    message: format!("from account not found: {:?}", transfer.inner.from),
                })?;
        if from_account.action_hash != transfer_hash {
            return Err(Error::AccountHashNotMatch {
                message: format!(
                    "from account action hash not match: {:?}",
                    transfer.inner.from
                ),
            });
        }
        Ok(())
    }

    fn verify_merge_rollback(
        merge: &Merge,
        now_state_map: &BTreeMap<AccountKey, Account>,
        merge_hash: Hash,
    ) -> Result<()> {
        verify_message(&merge_hash.0, &merge.from_signature.0, &merge.inner.from.0)?;
        verify_message(&merge_hash.0, &merge.to_signature.0, &merge.inner.to.0)?;

        let to_account =
            now_state_map
                .get(&merge.inner.to)
                .ok_or_else(|| Error::AccountNotFound {
                    message: format!(
                        "verify_merge_rollback to account not found: {:?}",
                        merge.inner.to
                    ),
                })?;
        if to_account.action_hash != merge_hash {
            return Err(Error::AccountHashNotMatch {
                message: format!("to account action hash not match: {:?}", merge.inner.to),
            });
        }

        Ok(())
    }
}

pub fn get_key_from_block_key(block_key: &[u8]) -> i32 {
    let mut key: i32 = 0;
    for i in block_key {
        key += i32::from(*i);
    }
    key
}

pub fn get_map(map: &BTreeMap<AccountKey, Account>) -> BTreeMap<String, Account> {
    let mut new_map = BTreeMap::new();
    for (key, value) in map {
        new_map.insert(get_key_from_block_key(&key.0).to_string(), value.clone());
    }
    new_map
}
