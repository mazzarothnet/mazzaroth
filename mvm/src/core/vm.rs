use std::collections::{BTreeMap, BTreeSet};

use consensus::{
    STO_ACCOUNT_MIN_BALANCE, TRANSFER_GAS, get_now_block_reward,
    types::{AccountKey, Hash},
};
use log::{error, info};
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
    ) -> Result<Hash> {
        let transfer_hash = sha256_hash_rlp(&transfer.inner);
        verify_message(
            &transfer_hash,
            &transfer.from_signature.0,
            &transfer.inner.from.0,
        )?;
        let cast = transfer.inner.amount + transfer.inner.gas_price * TRANSFER_GAS;
        let min_need = cast + STO_ACCOUNT_MIN_BALANCE;

        {
            let from_account =
                now_state_map
                    .get(&transfer.inner.from)
                    .ok_or_else(|| Error::AccountNotFound {
                        message: format!("from account not found: {:?}", transfer.inner.from),
                    })?;
            if from_account.balance < min_need {
                return Err(Error::AccountBalanceNotEnough {
                    message: format!("from account balance not enough: {:?}", transfer.inner.from),
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
        if !now_state_map.contains_key(&transfer.inner.to) && cast < STO_ACCOUNT_MIN_BALANCE {
            return Err(Error::AccountBalanceNotEnough {
                message: format!("from account balance not enough: {:?}", transfer.inner.from),
            });
        }
        if !now_state_map.contains_key(minner) {
            return Err(Error::AccountNotFound {
                message: format!("minner account not found: {:?}", minner),
            });
        }
        Ok(Hash(transfer_hash))
    }

    pub fn verify_merge(
        merge: &Merge,
        minner: &AccountKey,
        now_state_map: &BTreeMap<AccountKey, Account>,
    ) -> Result<(Hash, u128)> {
        let merge_hash = sha256_hash_rlp(&merge.inner);
        verify_message(&merge_hash, &merge.from_signature.0, &merge.inner.from.0)?;
        verify_message(&merge_hash, &merge.to_signature.0, &merge.inner.to.0)?;

        let from_account =
            now_state_map
                .get(&merge.inner.from)
                .ok_or_else(|| Error::AccountNotFound {
                    message: format!("from account not found: {:?}", merge.inner.from),
                })?;
        let merge_amount = from_account.balance;
        if merge_amount != from_account.balance {
            return Err(Error::AccountBalanceNotEnough {
                message: format!("from account balance not enough: {:?}", merge.inner.from),
            });
        }
        if merge_amount < merge.inner.gas_price * TRANSFER_GAS {
            return Err(Error::AccountBalanceNotEnough {
                message: format!("from account balance not enough: {:?}", merge.inner.from),
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

        Ok((Hash(merge_hash), merge_amount))
    }

    fn do_minner_reward(
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        block: &Block,
    ) -> Result<()> {
        let now_reward = get_now_block_reward(block.inner.header.part_sort_header.size);
        if !now_state_map.contains_key(&block.inner.miner) && now_reward >= STO_ACCOUNT_MIN_BALANCE
        {
            now_state_map.insert(
                block.inner.miner,
                Account {
                    key: block.inner.miner,
                    balance: 0,
                    action_hash: Hash([0; 32]),
                },
            );
        }
        let minner_account =
            now_state_map
                .get_mut(&block.inner.miner)
                .ok_or_else(|| Error::AccountNotFound {
                    message: format!("minner account not found: {:?}", block.inner.miner),
                })?;

        if minner_account.action_hash != block.inner.minner_last_action_hash {
            return Err(Error::AccountHashNotMatch {
                message: format!(
                    "minner account action hash not match: {:?}",
                    block.inner.miner
                ),
            });
        }
        minner_account.balance += now_reward;
        let block_hash: [u8; 32] = block.key.0.to_be_bytes();
        minner_account.action_hash = Hash(block_hash);
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
        for transfer in &block.inner.transfers {
            if let Err(e) = Self::do_transfer(transfer, &block.inner.miner, now_state_map) {
                if let Error::Impossible { message } = e {
                    return Err(Error::Impossible { message });
                } else {
                    info!("do transfer error: {:?}", e);
                }
            }
        }
        for merge in &block.inner.merges {
            if let Err(e) = Self::do_merge(merge, &block.inner.miner, now_state_map, delete_set) {
                if let Error::Impossible { message } = e {
                    return Err(Error::Impossible { message });
                } else {
                    info!("do merge error: {:?}", e);
                }
            }
        }

        Ok(())
    }

    pub fn do_block(&mut self, block: &Block) -> Result<()> {
        let account_set = Self::get_account_set(block);
        let mut account_transaction = self.account_db.begin_transaction()?;
        let mut now_state_map = account_transaction
            .batch_read::<AccountKey, Account>(account_set.into_iter().collect())?
            .into_iter()
            .map(|account| (account.key, account))
            .collect::<BTreeMap<AccountKey, Account>>();
        let mut delete_set = BTreeSet::new();

        if let Err(e) = Self::do_minner_reward(&mut now_state_map, block) {
            error!("do minner reward error: {:?}", e);
        }
        Self::do_all_transfer_and_merge(block, &mut now_state_map, &mut delete_set)?;

        for key in &delete_set {
            account_transaction.delete_data(key)?;
        }
        let mut set_account_hash = Vec::new();
        for (key, account) in now_state_map {
            let account_hash = sha256_hash_rlp(&account);
            account_transaction.set_data(key, account)?;
            set_account_hash.push((key, Hash(account_hash)));
        }
        account_transaction.commit()?;
        self.merkle_tree
            .update_tree(set_account_hash, delete_set.into_iter().collect())?;

        Ok(())
    }

    fn do_merge(
        merge: &Merge,
        minner: &AccountKey,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
        delete_set: &mut BTreeSet<AccountKey>,
    ) -> Result<()> {
        let (merge_hash, merge_amount) = Self::verify_merge(merge, minner, now_state_map)?;
        now_state_map.remove(&merge.inner.from);
        delete_set.insert(merge.inner.from);
        let to_account =
            now_state_map
                .get_mut(&merge.inner.to)
                .ok_or_else(|| Error::Impossible {
                    message: format!("to account not found: {:?}", merge.inner.to),
                })?;
        to_account.balance += merge_amount - merge.inner.gas_price * TRANSFER_GAS;
        to_account.action_hash = merge_hash;

        let minner_account = now_state_map
            .get_mut(minner)
            .ok_or_else(|| Error::Impossible {
                message: format!("minner account not found: {:?}", minner),
            })?;
        minner_account.balance += merge.inner.gas_price * TRANSFER_GAS;

        Ok(())
    }

    fn do_transfer(
        transfer: &Transfer,
        minner: &AccountKey,
        now_state_map: &mut BTreeMap<AccountKey, Account>,
    ) -> Result<()> {
        let transfer_hash = Self::verify_transfer(transfer, minner, now_state_map)?;
        let cast = transfer.inner.amount + transfer.inner.gas_price * TRANSFER_GAS;
        let from_account =
            now_state_map
                .get_mut(&transfer.inner.from)
                .ok_or_else(|| Error::Impossible {
                    message: format!("from account not found: {:?}", transfer.inner.from),
                })?;

        from_account.balance -= cast;
        from_account.action_hash = transfer_hash;

        let to_account = now_state_map.entry(transfer.inner.to).or_insert(Account {
            key: transfer.inner.to,
            balance: 0,
            action_hash: Hash([0; 32]),
        });
        to_account.balance += transfer.inner.amount;

        let minner_account = now_state_map
            .get_mut(minner)
            .ok_or_else(|| Error::Impossible {
                message: format!("minner account not found: {:?}", minner),
            })?;
        minner_account.balance += transfer.inner.gas_price * TRANSFER_GAS;

        Ok(())
    }

    // fn verify_merge_rollback(
    //     merge: &Merge,
    //     now_state_map: &BTreeMap<AccountKey, Account>,
    // ) -> Result<(Hash, u128)> {
    //     let merge_hash = sha256_hash_rlp(&merge.inner);
    //     verify_message(&merge_hash, &merge.from_signature.0, &merge.inner.from.0)?;
    //     verify_message(&merge_hash, &merge.to_signature.0, &merge.inner.to.0)?;

    //     let from_account =
    //         now_state_map
    //             .get(&merge.inner.from)
    //             .ok_or_else(|| Error::AccountNotFound {
    //                 message: format!("from account not found: {:?}", merge.inner.from),
    //             })?;
    //     let merge_amount = from_account.balance;
    //     if merge_amount != from_account.balance {
    //         return Err(Error::AccountBalanceNotEnough {
    //             message: format!("from account balance not enough: {:?}", merge.inner.from),
    //         });
    //     }
    //     if merge_amount < merge.inner.gas_price * TRANSFER_GAS {
    //         return Err(Error::AccountBalanceNotEnough {
    //             message: format!("from account balance not enough: {:?}", merge.inner.from),
    //         });
    //     }
    //     if from_account.action_hash != merge.inner.from_last_action_hash {
    //         return Err(Error::AccountHashNotMatch {
    //             message: format!("from account action hash not match: {:?}", merge.inner.from),
    //         });
    //     }

    //     let to_account =
    //         now_state_map
    //             .get(&merge.inner.to)
    //             .ok_or_else(|| Error::AccountNotFound {
    //                 message: format!("to account not found: {:?}", merge.inner.to),
    //             })?;
    //     if to_account.action_hash != merge.inner.to_last_action_hash {
    //         return Err(Error::AccountHashNotMatch {
    //             message: format!("to account action hash not match: {:?}", merge.inner.to),
    //         });
    //     }

    //     if !now_state_map.contains_key(minner) {
    //         return Err(Error::AccountNotFound {
    //             message: format!("minner account not found: {:?}", minner),
    //         });
    //     }

    //     Ok((Hash(merge_hash), merge_amount))
    // }
}
