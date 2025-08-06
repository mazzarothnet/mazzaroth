use axum::{
    Router,
    extract::Query,
    routing::{delete, get, put},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use utils::error::{Res, Result};

use crate::gossip::{UDP_RECV, UDP_SEND};

pub fn api_router() -> Router {
    Router::new()
        .route("/recv_addr", put(add_recv_addr))
        .route("/recv_addr", delete(remove_recv_addr))
        .route("/recv_addr", get(list_recv_addr))
        .route("/send_addr", put(add_send_addr))
        .route("/send_addr", delete(remove_send_addr))
        .route("/send_addr", get(list_send_addr))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecvAddrItem {
    pub addr: String,
    pub listen_topic_len: u16,
}

pub async fn add_recv_addr(req: Query<RecvAddrItem>) -> Result<()> {
    let req = req.0;
    let addr = req
        .addr
        .parse::<SocketAddr>()
        .map_err(|e| anyhow::anyhow!("Invalid address: {:?}", e))?;
    let mut recv = UDP_RECV
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock UDP_RECV: {:?}", e))?;
    recv.add_node(addr, req.listen_topic_len);
    Ok(())
}

pub async fn remove_recv_addr(req: Query<RecvAddrItem>) -> Result<()> {
    let req = req.0;
    let addr = req
        .addr
        .parse::<SocketAddr>()
        .map_err(|e| anyhow::anyhow!("Invalid address: {:?}", e))?;
    let mut recv = UDP_RECV
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock UDP_RECV: {:?}", e))?;
    recv.remove_node(addr);
    Ok(())
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
        .keys()
        .map(|addr| addr.to_string())
        .collect();
    Ok(Res { data: items })
}

pub async fn add_send_addr(req: Query<String>) -> Result<()> {
    let req = req.0;
    let addr = req
        .parse::<SocketAddr>()
        .map_err(|e| anyhow::anyhow!("Invalid address: {:?}", e))?;
    let mut send = UDP_SEND
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock UDP_SEND: {:?}", e))?;
    send.add_node(addr);
    Ok(())
}

pub async fn remove_send_addr(req: Query<String>) -> Result<()> {
    let req = req.0;
    let addr = req
        .parse::<SocketAddr>()
        .map_err(|e| anyhow::anyhow!("Invalid address: {:?}", e))?;
    let mut send = UDP_SEND
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock UDP_SEND: {:?}", e))?;
    send.remove_node(addr);
    Ok(())
}
