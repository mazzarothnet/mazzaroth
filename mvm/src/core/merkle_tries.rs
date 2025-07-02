use crate::core::storage::{DbStorage, DbStorageTransaction};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::types::{AccountKey, StateHash};
use log::info;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use utils::error::Result;

const TIRE_STATE_KEY: u128 = 0u128;
const TIRE_ROOT_KEY: u128 = 1u128;
const ZERO_STATE_HASH: StateHash = StateHash([0; 32]);

pub struct Tire<S: DbStorage> {
    state: TireState,
    storage: S,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
struct TireState {
    counter: u128,
    state_hash: StateHash,
}

impl<S: DbStorage> Tire<S> {
    pub fn new(storage: S) -> Result<Self> {
        let mut transaction: S::Transaction<'_> = storage.begin_transaction()?;
        let mut need_save = false;
        let state: TireState = transaction.get_data(TIRE_STATE_KEY)?.unwrap_or_else(|| {
            need_save = true;
            info!("TireState not found, creating new one");
            TireState {
                counter: 20,
                state_hash: ZERO_STATE_HASH,
            }
        });

        if need_save {
            transaction.set_data(TIRE_STATE_KEY, state.clone())?;
        }
        transaction.commit()?;
        Ok(Self { state, storage })
    }

    pub fn get_state_hash(&self) -> StateHash {
        self.state.state_hash
    }

    pub fn set_state_hash(
        &mut self,
        account_key: AccountKey,
        state_hash: StateHash,
    ) -> Result<StateHash> {
        let mut transaction: S::Transaction<'_> = self.storage.begin_transaction()?;
        let mut counter = self.state.counter;
        let ans = Self::set_state_hash_inner(
            &mut counter,
            &mut transaction,
            account_key,
            state_hash,
            0,
            TIRE_ROOT_KEY,
        )?;
        self.state.state_hash = ans;
        self.state.counter = counter;
        transaction.set_data(TIRE_ROOT_KEY, self.state.clone())?;
        transaction.commit()?;
        Ok(ans)
    }

    pub fn delete_state_hash(&mut self, account_key: AccountKey) -> Result<StateHash> {
        let mut transaction: S::Transaction<'_> = self.storage.begin_transaction()?;
        let ans = Self::delete_state_hash_inner(&mut transaction, account_key, 0, TIRE_ROOT_KEY)?;
        self.state.state_hash = ans;
        transaction.set_data(TIRE_ROOT_KEY, self.state.clone())?;
        transaction.commit()?;
        Ok(ans)
    }

    fn get_new_node_index(counter: &mut u128) -> u128 {
        let node_index = *counter;
        *counter += 1;
        node_index
    }

    fn delete_state_hash_inner(
        transaction: &mut S::Transaction<'_>,
        account_key: AccountKey,
        index: usize,
        node_index: u128,
    ) -> Result<StateHash> {
        if index == account_key.len() {
            return Ok(ZERO_STATE_HASH);
        }
        let node: TireNode = transaction
            .get_data(node_index)?
            .unwrap_or(TireNode { children: vec![] });
        let mut new_children = vec![];
        for mut i in node.children {
            if i.value == account_key[index] {
                let new_hash =
                    Self::delete_state_hash_inner(transaction, account_key, index + 1, i.id)?;
                if new_hash != ZERO_STATE_HASH {
                    i.state_hash = new_hash;
                    new_children.push(i);
                }
            } else {
                new_children.push(i);
            }
        }
        if new_children.is_empty() {
            transaction.delete_data(node_index)?;
            return Ok(ZERO_STATE_HASH);
        }
        let mut hasher = sha2::Sha256::new();
        for i in new_children.iter() {
            hasher.update(i.state_hash.0);
        }
        let state_hash = StateHash(hasher.finalize().into());
        transaction.set_data(
            node_index,
            TireNode {
                children: new_children,
            },
        )?;
        Ok(state_hash)
    }

    fn set_state_hash_inner(
        counter: &mut u128,
        transaction: &mut S::Transaction<'_>,
        account_key: AccountKey,
        state_hash: StateHash,
        index: usize,
        node_index: u128,
    ) -> Result<StateHash> {
        if index == account_key.len() {
            return Ok(state_hash);
        }
        let mut node: TireNode = transaction
            .get_data(node_index)?
            .unwrap_or(TireNode { children: vec![] });

        let mut need_new_node = true;
        for i in node.children.iter_mut() {
            if i.value == account_key[index] {
                need_new_node = false;
                i.state_hash = Self::set_state_hash_inner(
                    counter,
                    transaction,
                    account_key,
                    state_hash,
                    index + 1,
                    i.id,
                )?;
                break;
            }
        }
        if need_new_node {
            let new_node_index = Self::get_new_node_index(counter);
            let state_hash = Self::set_state_hash_inner(
                counter,
                transaction,
                account_key,
                state_hash,
                index + 1,
                new_node_index,
            )?;
            node.children.push(TireNodeChildren {
                value: account_key[index],
                state_hash,
                id: new_node_index,
            });
        }
        node.children.sort_by(|a, b| a.value.cmp(&b.value));
        let mut hasher = sha2::Sha256::new();
        for i in node.children.iter() {
            hasher.update(i.state_hash.0);
        }
        let state_hash = StateHash(hasher.finalize().into());
        transaction.set_data(node_index, node)?;
        Ok(state_hash)
    }
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
struct TireNode {
    children: Vec<TireNodeChildren>,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
struct TireNodeChildren {
    value: u8,
    state_hash: StateHash,
    id: u128,
}
