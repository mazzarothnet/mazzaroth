use std::collections::{BTreeMap, BTreeSet};

use crate::core::storage::{DbStorage, DbStorageTransaction};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::types::{ACCOUNT_KEY_LEN, AccountKey, StateHash};
use log::info;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use utils::error::Result;
const ZERO_STATE_HASH: StateHash = StateHash([0; 32]);

pub struct MerkleTree<S: DbStorage> {
    storage: S,
    state_root: Option<StateHash>,
}

impl<S: DbStorage> MerkleTree<S> {
    pub fn new(storage: S) -> Result<Self> {
        Ok(Self {
            storage,
            state_root: None,
        })
    }

    pub fn update_tree(
        &mut self,
        set_account: BTreeMap<AccountKey, StateHash>,
        delete_account: BTreeSet<AccountKey>,
    ) -> Result<S::Transaction<'_>> {
        let mut transaction: S::Transaction<'_> = self.storage.begin_transaction()?;
        let set_account = set_account
            .into_iter()
            .map(|(key, state_hash)| {
                let tree_key = TreeKey { mask_num: 0, key };
                let tree_node = TreeNode::new(state_hash);
                (tree_key, tree_node)
            })
            .collect::<BTreeMap<_, _>>();
        let delete_account = delete_account
            .into_iter()
            .map(|key| TreeKey { mask_num: 0, key })
            .collect::<BTreeSet<_>>();
        let mut rotate_set_account = vec![set_account, BTreeMap::new()];
        let mut rotate_delete_account = vec![delete_account, BTreeSet::new()];
        let mut now_set_account = 0;
        let mut now_delete_account = 0;
        for i in 0..ACCOUNT_KEY_LEN {
            let mut new_set_account: BTreeMap<TreeKey, TreeNode> = BTreeMap::new();
            for (key, node) in rotate_set_account[now_set_account].iter() {
                transaction.set_data(key, node)?;
                //let (new_key, new_node) = Self::get_next_node(&mut transaction, key)?;
            }
            for key in rotate_delete_account[now_delete_account].iter() {
                transaction.delete_data(key)?;
                //Self::get_next_node(&mut transaction, key, &mut new_set_account)?;
            }
        }

        unimplemented!()
    }

    fn get_next_node(
        key: &TreeKey,
        new_set_account: &mut BTreeMap<TreeKey, TreeNode>,
        transaction: &mut S::Transaction<'_>,
    ) -> Result<(TreeKey, TreeNode)> {
        let mut new_account_key = key.key;
        new_account_key.0[key.mask_num] = 0;
        let new_key = TreeKey {
            mask_num: key.mask_num + 1,
            key: new_account_key,
        };
        
        let new_node: TreeNode = transaction.get_data(&new_key)?.unwrap_or_default();
        Ok((new_key, new_node))
    }
}

#[derive(
    Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone, Ord, PartialOrd, Eq, PartialEq,
)]
struct TreeKey {
    mask_num: usize,
    key: AccountKey,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
struct TreeNode {
    hash: StateHash,
    children: Vec<TreeNodeChildren>,
}

impl TreeNode {
    fn new(hash: StateHash) -> Self {
        Self {
            hash,
            children: vec![],
        }
    }
}

impl Default for TreeNode {
    fn default() -> Self {
        Self {
            hash: ZERO_STATE_HASH,
            children: vec![],
        }
    }
}

#[derive(
    Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone, Ord, PartialOrd, Eq, PartialEq,
)]
struct TreeNodeChildren {
    key: TreeKey,
    state_hash: StateHash,
}
