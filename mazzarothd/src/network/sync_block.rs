use crate::{
    network::req::{req_block, req_tips},
    state::{block_storage::has_block, mz_state::MzState, tips::push_block},
};
use consensus::{traits::GENESIS_BLOCK_KEY, types::BlockKey};
use log::info;

/// this fn will block
pub async fn sync_block(mz_state: &MzState, host: &str) -> anyhow::Result<()> {
    let tips = req_tips(host).await?;
    info!("sync_block, tips: {:?}", tips);
    for tip in tips {
        info!("sync_block, sync tip: {:?}", tip);
        sync_begin_with_key(tip, host, mz_state).await?;
    }
    Ok(())
}

async fn sync_begin_with_key(
    mut key: BlockKey,
    host: &str,
    mz_state: &MzState,
) -> anyhow::Result<()> {
    let mut head_link: Vec<BlockKey> = Vec::new();
    while key != BlockKey::from(GENESIS_BLOCK_KEY) {
        head_link.push(key);
        if has_block(&mz_state.block_storage, &key)? {
            break;
        }
        let block = req_block(host, key).await?;
        info!(
            "sync_begin_with_key, sync head key: {:?}",
            block.inner.header.part_sort_header.head_key
        );
        key = block.inner.header.part_sort_header.head_key;
    }
    for key in head_link.iter().rev() {
        req_and_push_block(host, *key, mz_state).await?;
    }

    Ok(())
}

async fn req_and_push_block(host: &str, key: BlockKey, mz_state: &MzState) -> anyhow::Result<()> {
    let block = req_block(host, key).await?;
    for pb in &block.inner.header.part_sort_header.part_sort {
        let pb_block = req_block(host, *pb).await?;
        info!("sync_begin_with_key, sync pb key: {:?}", pb_block.key);
        push_block(pb_block, mz_state)?;
    }
    push_block(block, mz_state)?;

    Ok(())
}
