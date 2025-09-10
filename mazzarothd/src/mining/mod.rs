use crate::{
    mining::self_transfer::self_transfers_to_transfers,
    state::{
        account_manager::get_miner_account,
        block_storage::{gen_consensus_header_with_global_storage, get_block_hard},
        mvm::{get_mvm_now_key, move_mvm_to_next_key, mvm_get_account},
        mz_state::MzState,
        tips::{force_remove_tips, get_tips, push_block},
        transfer::get_pending_transfers,
    },
};
use anyhow::Context;
use consensus::{
    block_header::ConsensusHeader,
    traits::GENESIS_BLOCK_KEY,
    types::{BlockKey, Hash, block_key_to_hash},
};
use crypto_bigint::U256;
use log::info;
use mining::{
    run_gpu::{Sha256Context, mining_gpu_sha256},
    sha256_mining::gen_sha256_by_block_hash_and_nonce,
};
use mvm::{
    core::vm::Mvm,
    models::block::{Block, BlockInner},
};
use std::time::Duration;
use utils::{
    error::{Error, Result},
    sha256::sha256_hash_rlp,
    time::get_current_time_ms,
};

pub mod self_transfer;

#[allow(clippy::unwrap_used)]
pub fn spawn_mining_thread(mz_state: MzState, block_sender: tokio::sync::mpsc::Sender<Block>) {
    std::thread::spawn(move || {
        let sha256_context = Sha256Context::new().unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            loop {
                match update_mvm_and_try_mining(&mz_state, &sha256_context) {
                    Ok(block) => {
                        info!(
                            "mining new block: {:?} target: {:?}",
                            block.key, block.inner.header.pow_header.target
                        );
                        block_sender.send(block.clone()).await.unwrap();
                        push_block(block, &mz_state).unwrap();
                    }
                    Err(Error::TipsNotFound) => {
                        info!("tips not found");
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                    Err(Error::MiningFailed) => {
                        info!("mining new block failed");
                    }
                    Err(e) => {
                        panic!("mining new block failed: {:?}", e);
                    }
                }
            }
        });
    });
}

fn update_mvm_and_try_mining(mz_state: &MzState, sha256_context: &Sha256Context) -> Result<Block> {
    let tips = get_tips(mz_state)?.into_iter().collect::<Vec<_>>();
    if tips.is_empty() {
        return Err(Error::TipsNotFound);
    }
    let now_time = get_current_time_ms();
    let consensus_header =
        gen_consensus_header_with_global_storage(&mz_state.block_storage, &tips, now_time)?;
    let head_key = consensus_header.part_sort_header.head_key;
    let now_key = get_mvm_now_key(mz_state)?;
    move_mvm_to_next_key(now_key, head_key, mz_state)?;
    force_remove_tips(mz_state, head_key)?;
    check_head_state_root(mz_state, head_key)?;
    let target = consensus_header.pow_header.target;
    let block_inner = gen_new_block_inner(mz_state, consensus_header)?;
    let block_inner_hash = sha256_hash_rlp(&block_inner);
    let nonce = if let Some(nonce) =
        mining_gpu_sha256(sha256_context, block_inner_hash, now_time, target)?
    {
        nonce
    } else {
        return Err(Error::MiningFailed);
    };
    let block_key = BlockKey(U256::from_be_slice(&gen_sha256_by_block_hash_and_nonce(
        block_inner_hash,
        nonce,
    )));
    let block = Block {
        key: block_key,
        nonce,
        inner: block_inner.clone(),
    };
    Ok(block)
}

fn check_head_state_root(mz_state: &MzState, head_key: BlockKey) -> Result<()> {
    let head_block = get_block_hard(&mz_state.block_storage, &head_key)?;
    let now_state_root = get_now_mvm_state_root(mz_state)?;
    if head_block.inner.state_root != now_state_root {
        return Err(Error::InvalidStateRoot);
    }
    Ok(())
}

fn get_now_mvm_state_root(mz_state: &MzState) -> Result<Hash> {
    let mut mvm_lock = mz_state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock mvm: {}", e))?;
    let mut mvm_transaction = mvm_lock
        .begin_transaction()
        .with_context(|| "Failed to begin transaction")?;
    let state_root = Mvm::get_state_root(&mut mvm_transaction)?;
    Ok(state_root)
}

fn gen_new_block_inner(
    mz_state: &MzState,
    consensus_header: ConsensusHeader,
) -> Result<BlockInner> {
    let miner_account_pair = get_miner_account(mz_state)?;
    let miner_action_hash = mvm_get_account(mz_state, miner_account_pair.public_key)?.action_hash;
    let self_transfers = get_pending_transfers(mz_state)?;
    let transfers = self_transfers_to_transfers(
        self_transfers.transfers,
        &miner_account_pair,
        block_key_to_hash(consensus_header.part_sort_header.head_key),
    )?;
    let head_key = consensus_header.part_sort_header.head_key;
    let block_inner = BlockInner {
        version: 0,
        header: consensus_header,
        transfers,
        merges: Vec::new(),
        miner: miner_account_pair.public_key,
        miner_last_action_hash: miner_action_hash,
        state_root: Hash::default(),
    };
    let mock_block = Block {
        key: GENESIS_BLOCK_KEY,
        nonce: 0,
        inner: block_inner,
    };
    let state_root = get_block_state_root(&mock_block, mz_state, head_key)?;
    let mut block_inner = mock_block.inner;
    block_inner.state_root = state_root;

    Ok(block_inner)
}

fn get_block_state_root(block: &Block, mz_state: &MzState, head_key: BlockKey) -> Result<Hash> {
    let mut mvm_lock = mz_state
        .mvm
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock mvm: {}", e))?;
    let mut mvm_transaction = mvm_lock
        .begin_transaction()
        .with_context(|| "Failed to begin transaction")?;
    let part_sort = &block.inner.header.part_sort_header.part_sort;
    for key in part_sort.iter().rev() {
        if *key == head_key {
            continue;
        }
        let block = get_block_hard(&mz_state.block_storage, key)?;
        Mvm::do_block(&mut mvm_transaction, &block)?;
    }
    Mvm::do_block(&mut mvm_transaction, block)?;
    let state_root = Mvm::get_state_root(&mut mvm_transaction)?;
    Ok(state_root)
}
