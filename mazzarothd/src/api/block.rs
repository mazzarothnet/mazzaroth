use alloy_rlp::Encodable;
use axum::extract::Query;
use consensus::types::BlockKey;
use crypto_bigint::U256;
use utils::error::{BinaryRes, Error, Res, Result};

use crate::state::{block_storage::get_block, tips::get_tips};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BlockKeyParam {
    block_key: String,
}

pub async fn get_block_api(Query(block_key): Query<BlockKeyParam>) -> Result<BinaryRes> {
    let block_key = block_key.block_key;
    let block_key = BlockKey(U256::from_be_hex(block_key.as_str()));
    let block = get_block(&block_key)?.ok_or_else(|| Error::BlockNotFound {
        key: block_key.to_string(),
    })?;
    let mut block_bytes = Vec::new();
    block.encode(&mut block_bytes);
    Ok(BinaryRes { data: block_bytes })
}

pub async fn get_tips_api() -> Result<Res<Vec<BlockKey>>> {
    let tips = get_tips()?.into_iter().collect();
    Ok(Res { data: tips })
}
