use crate::{
    mining::self_transfer::self_transfers_to_transfers,
    state::{
        account_manager::get_miner_account,
        block_storage::gen_consensus_header_with_global_storage,
        mvm::{get_mvm_now_key, move_mvm_to_next_key, mvm_get_account},
        mz_state::MzState,
        tips::{get_tips, push_block},
        transfer::get_pending_transfers,
    },
};
use consensus::types::{BlockKey, block_key_to_hash};
use crypto_bigint::U256;
use log::info;
use mining::{
    run_gpu::{Sha256Context, mining_gpu_sha256},
    sha256_mining::gen_sha256_by_block_hash_and_nonce,
};
use mvm::models::block::{Block, BlockInner};
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
    let now_key = get_mvm_now_key(mz_state)?;
    move_mvm_to_next_key(
        now_key,
        consensus_header.part_sort_header.head_key,
        mz_state,
    )?;
    let miner_account_pair = get_miner_account(mz_state)?;
    let miner_action_hash = mvm_get_account(mz_state, miner_account_pair.public_key)?.action_hash;
    let self_transfers = get_pending_transfers(mz_state)?;
    let transfers = self_transfers_to_transfers(
        self_transfers.transfers,
        &miner_account_pair,
        block_key_to_hash(consensus_header.part_sort_header.head_key),
    )?;
    let target = consensus_header.pow_header.target;
    let block_inner = BlockInner {
        version: 0,
        header: consensus_header,
        transfers,
        merges: Vec::new(),
        miner: miner_account_pair.public_key,
        miner_last_action_hash: miner_action_hash,
    };
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
