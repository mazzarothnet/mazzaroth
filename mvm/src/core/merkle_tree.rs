use crate::core::storage::{DbStorage, DbStorageTransaction};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use consensus::types::{ACCOUNT_KEY_LEN, AccountKey, Hash};
use log::{debug, info};
// use log::info;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::{BTreeMap, BTreeSet};
use utils::error::{Error, Result};
const ZERO_STATE_HASH: Hash = Hash([0; 32]);

pub struct MerkleTree<S: DbStorage> {
    storage: S,
}

impl<S: DbStorage> MerkleTree<S> {
    pub fn new(storage: S) -> Result<Self> {
        Ok(Self { storage })
    }

    pub fn get_state_root(&self) -> Result<Hash> {
        let mut transaction: S::Transaction<'_> = self.storage.begin_transaction()?;
        let node = transaction
            .batch_read::<TreeKey, TreeNode>(vec![TreeKey {
                mask_num: 33,
                key: AccountKey([0; 33]),
            }])?
            .first()
            .map(|node| node.hash)
            .unwrap_or_else(|| {
                info!("get_state_root: TreeKey not found, creating new one");
                ZERO_STATE_HASH
            });
        transaction.commit()?;
        Ok(node)
    }

    pub fn update_tree(
        &mut self,
        set_account: Vec<(AccountKey, Hash)>,
        delete_account: Vec<AccountKey>,
    ) -> Result<S::Transaction<'_>> {
        let mut transaction: S::Transaction<'_> = self.storage.begin_transaction()?;
        let mut set_account = set_account
            .into_iter()
            .map(|(key, state_hash)| {
                let tree_key = TreeKey { mask_num: 0, key };
                TreeNode::new(tree_key, state_hash)
            })
            .collect::<Vec<_>>();
        let mut delete_account = delete_account
            .into_iter()
            .map(|key| TreeKey { mask_num: 0, key })
            .collect::<Vec<_>>();
        let mut total_account_map =
            Self::read_nodes(&mut transaction, &set_account, &delete_account)?;
        for i in 0..ACCOUNT_KEY_LEN {
            let mut account_map = total_account_map.remove(&(i + 1)).unwrap_or_default();
            let mut new_set_account = vec![];
            let mut new_delete_account = vec![];
            for node in set_account.iter() {
                #[cfg(debug_assertions)]
                {
                    let key = utils::get_u8_vec_sum(&node.key.key.0);
                    let value = utils::get_u8_vec_sum(&node.hash.0);
                    debug!("key: {:?}, value: {:?}", key, value);
                }
                transaction.set_data(node.key, node)?;
                let new_node = Self::get_next_node(&node.key, &mut account_map);
                Self::update_next_node(node, new_node);
            }
            for key in delete_account.iter() {
                transaction.delete_data(key)?;
                let new_node = Self::get_next_node(key, &mut account_map);
                Self::delete_node(key, new_node);
            }

            for mut node in account_map.into_values() {
                if node.children.is_empty() {
                    new_delete_account.push(node.key);
                } else {
                    Self::update_node_by_children(&mut node);
                    new_set_account.push(node);
                }
            }
            set_account = new_set_account;
            delete_account = new_delete_account;
        }
        for node in set_account.iter() {
            transaction.set_data(node.key, node)?;
        }
        for key in delete_account.iter() {
            transaction.delete_data(key)?;
        }
        if set_account.len() + delete_account.len() > 1 {
            return Err(Error::MerkleTree {
                message: "panic set_account.len() + delete_account.len() > 1".to_string(),
            });
        }

        Ok(transaction)
    }

    fn read_nodes(
        transaction: &mut S::Transaction<'_>,
        set_account: &[TreeNode],
        delete_account: &[TreeKey],
    ) -> Result<BTreeMap<usize, BTreeMap<TreeKey, TreeNode>>> {
        let mut real_keys = BTreeSet::new();
        for node in set_account.iter() {
            let mut now_key = node.key;
            for i in 0..ACCOUNT_KEY_LEN {
                now_key.key.0[i] = 0;
                now_key.mask_num = i + 1;
                real_keys.insert(now_key);
            }
        }
        for key in delete_account.iter() {
            let mut now_key = *key;
            for i in 0..ACCOUNT_KEY_LEN {
                now_key.key.0[i] = 0;
                now_key.mask_num = i + 1;
                real_keys.insert(now_key);
            }
        }
        //info!("batch_read len: {:?}", real_keys.len());
        let nodes: Vec<TreeNode> =
            transaction.batch_read(real_keys.into_iter().collect::<Vec<_>>())?;
        let mut ans = BTreeMap::new();
        debug!("read_nodes {:?}", nodes.len());
        for node in nodes.into_iter() {
            #[cfg(debug_assertions)]
            {
                let key = utils::get_u8_vec_sum(&node.key.key.0);
                let value = utils::get_u8_vec_sum(&node.hash.0);
                debug!("key: {:?}, value: {:?}", key, value);
            }
            let entry = ans.entry(node.key.mask_num).or_insert(BTreeMap::new());
            entry.insert(node.key, node);
        }
        debug!("read_nodes end\n");
        Ok(ans)
    }

    fn delete_node(key: &TreeKey, next_node: &mut TreeNode) {
        next_node.children.retain(|child| child.key != *key);
    }

    fn update_node_by_children(node: &mut TreeNode) {
        node.children.sort_by_key(|child| child.key);
        let mut hasher = sha2::Sha256::new();
        for child in node.children.iter() {
            hasher.update(child.state_hash.0);
        }
        node.hash = Hash(hasher.finalize().into());
    }

    fn update_next_node(now_node: &TreeNode, next_node: &mut TreeNode) {
        let mut need_push = true;
        for child in next_node.children.iter_mut() {
            if now_node.key == child.key {
                child.state_hash = now_node.hash;
                need_push = false;
                break;
            }
        }
        if need_push {
            next_node.children.push(TreeNodeChildren {
                key: now_node.key,
                state_hash: now_node.hash,
            });
        }
    }

    #[allow(clippy::unwrap_used)]
    fn get_next_node<'a>(
        key: &TreeKey,
        new_set_account: &'a mut BTreeMap<TreeKey, TreeNode>,
    ) -> &'a mut TreeNode {
        let mut new_account_key = key.key;
        new_account_key.0[key.mask_num] = 0;
        let new_key = TreeKey {
            mask_num: key.mask_num + 1,
            key: new_account_key,
        };
        new_set_account
            .entry(new_key)
            .or_insert(TreeNode::new(new_key, ZERO_STATE_HASH))
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    RlpEncodable,
    RlpDecodable,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Copy,
)]
struct TreeKey {
    mask_num: usize,
    key: AccountKey,
}

#[derive(Debug, Serialize, Deserialize, RlpEncodable, RlpDecodable, Clone)]
struct TreeNode {
    key: TreeKey,
    hash: Hash,
    children: Vec<TreeNodeChildren>,
}

impl TreeNode {
    fn new(key: TreeKey, hash: Hash) -> Self {
        Self {
            key,
            hash,
            children: vec![],
        }
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    RlpEncodable,
    RlpDecodable,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Copy,
)]
struct TreeNodeChildren {
    key: TreeKey,
    state_hash: Hash,
}
