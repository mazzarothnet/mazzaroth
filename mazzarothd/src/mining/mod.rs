use crate::state::{
    account_manager::{AccountKeyPair, get_miner_account},
    block_storage::gen_consensus_header_with_global_storage,
    mvm::{get_mvm_now_key, move_mvm_to_next_key, mvm_get_account},
    mz_state::MzState,
    tips::{get_tips, push_block},
    transfer::{SelfTransfer, get_pending_transfers},
};
use consensus::types::{BlockKey, Hash, Signature, block_key_to_hash};
use crypto_bigint::U256;
use log::info;
use mining::{
    run_gpu::{Sha256Context, mining_gpu_sha256},
    sha256_mining::gen_sha256_by_block_hash_and_nonce,
};
use mvm::models::{
    block::{Block, BlockInner},
    transfer::{Transfer, TransferInner},
};
use std::{collections::HashSet, time::Duration};
use utils::{secp::sign_message, sha256::sha256_hash_rlp, time::get_current_time_ms};

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
                let block = update_mvm_and_try_mining(&mz_state, &sha256_context).unwrap();
                if let Some(block) = block {
                    info!(
                        "mining new block: {:?} target: {:?}",
                        block.key, block.inner.header.pow_header.target
                    );
                    block_sender.send(block.clone()).await.unwrap();
                    push_block(block, &mz_state).unwrap();
                } else {
                    info!("mining new block failed");
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        });
    });
}

fn update_mvm_and_try_mining(
    mz_state: &MzState,
    sha256_context: &Sha256Context,
) -> anyhow::Result<Option<Block>> {
    let tips = get_tips(mz_state)?.into_iter().collect::<Vec<_>>();
    if tips.is_empty() {
        return Ok(None);
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
    let miner_account = mvm_get_account(mz_state, miner_account_pair.public_key)?;
    let miner_action_hash = miner_account.action_hash;
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

fn self_transfers_to_transfers(
    self_transfers: HashSet<SelfTransfer>,
    miner_account_pair: &AccountKeyPair,
    mut miner_action_hash: Hash,
) -> anyhow::Result<Vec<Transfer>> {
    let mut transfers = Vec::new();
    for self_transfer in self_transfers {
        let transfer_inner = TransferInner {
            from: miner_account_pair.public_key,
            to: self_transfer.to,
            amount: self_transfer.amount,
            from_last_action_hash: miner_action_hash,
            gas_price: 0,
        };
        let now_hash = sha256_hash_rlp(&transfer_inner);
        let signature = sign_message(&now_hash, &miner_account_pair.private_key)?;
        transfers.push(Transfer {
            inner: transfer_inner,
            from_signature: Signature(signature),
        });
        miner_action_hash = Hash(now_hash);
    }
    Ok(transfers)
}
