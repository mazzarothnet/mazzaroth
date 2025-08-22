use crate::state::{
    block_storage::gen_consensus_header_with_global_storage,
    mz_state::MzState,
    tips::{get_tips, push_block},
};
use consensus::types::BlockKey;
use crypto_bigint::U256;
use log::info;
use mining::{
    run_gpu::{Sha256Context, mining_gpu_sha256},
    sha256_mining::gen_sha256_by_block_hash_and_nonce,
};
use mvm::models::block::{Block, BlockInner};
use std::time::Duration;
use utils::{sha256::sha256_hash_rlp, time::get_current_time_ms};

#[allow(clippy::unwrap_used)]
pub fn spawn_mining_thread(mz_state: MzState, block_sender: tokio::sync::mpsc::Sender<Block>) {
    std::thread::spawn(move || {
        let sha256_context = Sha256Context::new().unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all() // 启用 I/O、时间等驱动（按需配置）
            .build()
            .unwrap();
        rt.block_on(async move {
            loop {
                let block = try_gen_new_block(&mz_state, &sha256_context).unwrap();
                if let Some(block) = block {
                    info!("mining new block: {:?}", block.key);
                    block_sender.send(block.clone()).await.unwrap();
                    push_block(block, &mz_state).unwrap();
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                } else {
                    info!("mining new block failed");
                }
            }
        });
    });
}

fn try_gen_new_block(
    mz_state: &MzState,
    sha256_context: &Sha256Context,
) -> anyhow::Result<Option<Block>> {
    let now_time = get_current_time_ms();
    let tips = get_tips(mz_state)?.into_iter().collect::<Vec<_>>();
    let consensus_header =
        gen_consensus_header_with_global_storage(&mz_state.block_storage, &tips, now_time)?;
    let miner_account = mz_state
        .account_manager
        .lock()
        .map_err(|e| anyhow::anyhow!("try_gen_new_block Failed to lock account_manager: {}", e))?
        .now_selected_account
        .clone();
    let pending_transfers = {
        let mut pending_transfers_lock = mz_state.pending_transfers.lock().map_err(|e| {
            anyhow::anyhow!("try_gen_new_block Failed to lock pending_transfers: {}", e)
        })?;
        let transfers = std::mem::take(&mut pending_transfers_lock.transfers);
        transfers
    }
    .into_iter()
    .collect::<Vec<_>>();
    let target = consensus_header.pow_header.target;
    let block_inner = BlockInner {
        version: 0,
        header: consensus_header,
        transfers: pending_transfers,
        merges: Vec::new(),
        miner: miner_account.public_key,
    };
    let block_inner_hash = sha256_hash_rlp(&block_inner);
    let nonce = if let Some(nonce) =
        mining_gpu_sha256(sha256_context, block_inner_hash, now_time, target)?
    {
        nonce
    } else {
        return Ok(None);
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
    Ok(Some(block))
}
