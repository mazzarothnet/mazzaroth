use axum::{Router, routing::get};
use serde::{Deserialize, Serialize};
use utils::error::{Res, Result};

use crate::gossip::{UDP_RECV, UDP_SEND};

pub fn api_router() -> Router {
    Router::new()
        .route("/recv_addr", get(list_recv_addr))
        .route("/send_addr", get(list_send_addr))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecvAddrItem {
    pub addr: String,
    pub listen_topic_len: u16,
}

pub async fn list_recv_addr() -> Result<Res<Vec<RecvAddrItem>>> {
    let recv = UDP_RECV
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock UDP_RECV: {:?}", e))?;
    let items = recv
        .node_map
        .iter()
        .map(|(addr, buf)| RecvAddrItem {
            addr: addr.to_string(),
            listen_topic_len: buf.listen_topic_len,
        })
        .collect();
    Ok(Res { data: items })
}

pub async fn list_send_addr() -> Result<Res<Vec<String>>> {
    let send = UDP_SEND
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock UDP_SEND: {:?}", e))?;
    let items = send
        .send_set
        .iter()
        .map(|(addr, _v)| addr.to_string())
        .collect();
    Ok(Res { data: items })
}
