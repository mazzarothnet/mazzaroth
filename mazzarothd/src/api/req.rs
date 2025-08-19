use alloy_rlp::Decodable;
use anyhow::Context;
use consensus::types::BlockKey;
use mvm::models::block::Block;
use reqwest::{Client, RequestBuilder};
use std::time::Duration;

lazy_static::lazy_static!(
    static ref REQWEST_CLIENT: Option<Client> = Client::builder()
                                            .pool_idle_timeout(Duration::from_secs(30))
                                            .build()
                                            .ok();
);

fn reqwest_client_get(url: &str) -> anyhow::Result<RequestBuilder> {
    Ok(REQWEST_CLIENT
        .as_ref()
        .ok_or(anyhow::anyhow!("reqwest client not initialized"))?
        .get(url))
}

pub async fn req_block(host: &str, block_key: BlockKey) -> anyhow::Result<Block> {
    let block_key_str = block_key.to_string();
    let url = format!("http://{}/block?block_key={}", host, block_key_str);
    let res = reqwest_client_get(&url)
        .with_context(|| format!("Failed to get block from {}", url))?
        .send()
        .await
        .with_context(|| format!("Failed to send request to {}", url))?;
    let block_bytes = res
        .bytes()
        .await
        .with_context(|| format!("Failed to get block bytes from {}", url))?;
    let block = Block::decode(&mut block_bytes.as_ref())
        .with_context(|| format!("Failed to decode block from {}", url))?;
    Ok(block)
}
