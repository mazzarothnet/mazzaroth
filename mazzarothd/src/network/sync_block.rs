use crate::{
    config::CFG,
    network::req::{req_block, req_tips},
    state::{block_storage::has_block, tips::push_block},
};
use consensus::{traits::GENESIS_BLOCK_KEY, types::BlockKey};
use log::info;

/// this fn will block
pub async fn sync_block() -> anyhow::Result<()> {
    let host = CFG.block_sync_host.clone();
    let tips = req_tips(&host).await?;
    for tip in tips {
        sync_begin_with_key(tip, &host).await?;
    }
    Ok(())
}

async fn sync_begin_with_key(mut key: BlockKey, host: &str) -> anyhow::Result<()> {
    let mut head_link: Vec<BlockKey> = Vec::new();
    while key != BlockKey::from(GENESIS_BLOCK_KEY) {
        head_link.push(key);
        if has_block(&key)? {
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
        req_and_push_block(host, *key).await?;
    }

    Ok(())
}

async fn req_and_push_block(host: &str, key: BlockKey) -> anyhow::Result<()> {
    let block = req_block(host, key).await?;
    for pb in &block.inner.header.part_sort_header.part_sort {
        let pb_block = req_block(host, *pb).await?;
        info!("sync_begin_with_key, sync pb key: {:?}", pb_block.key);
        push_block(pb_block)?;
    }
    push_block(block)?;

    Ok(())
}
