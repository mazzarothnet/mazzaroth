use crate::state::{block_storage::get_block, mz_state::MzState, tips::get_tips};
use alloy_rlp::Encodable;
use axum::extract::{Query, State};
use consensus::types::BlockKey;
use serde::Deserialize;
use utils::error::{BinaryRes, Error, Res, Result};

#[derive(Deserialize)]
pub struct BlockKeyParam {
    block_key: BlockKey,
}

pub async fn get_block_api(
    State(state): State<MzState>,
    Query(block_key): Query<BlockKeyParam>,
) -> Result<BinaryRes> {
    let block_key = block_key.block_key;
    //let block_key = BlockKey(U256::from_be_hex(block_key.as_str()));
    let block =
        get_block(&state.block_storage, &block_key)?.ok_or_else(|| Error::BlockNotFound {
            key: block_key.to_string(),
        })?;
    let mut block_bytes = Vec::new();
    block.encode(&mut block_bytes);
    Ok(BinaryRes { data: block_bytes })
}

pub async fn get_tips_api(State(state): State<MzState>) -> Result<Res<Vec<BlockKey>>> {
    let tips = get_tips(&state)?.into_iter().collect();
    Ok(Res { data: tips })
}
