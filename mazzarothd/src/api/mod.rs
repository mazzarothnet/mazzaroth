use crate::config::CFG;
use anyhow::Context;
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use utils::error::{Res, Result};

pub mod block;

pub fn api_router() -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/block", get(block::get_block_api))
        .route("/tips", get(block::get_tips_api))
}

async fn hello() -> Result<Res<String>> {
    Ok(Res {
        data: "Hello mazzaroth".to_string(),
    })
}

pub async fn serve() -> anyhow::Result<()> {
    let port = CFG.http_port;
    let app = api_router();
    let listener = TcpListener::bind(format!("[::]:{}", port))
        .await
        .with_context(|| format!("Failed to bind to port {}", port))?;
    axum::serve(listener, app)
        .await
        .with_context(|| "Failed to serve API".to_string())?;
    Ok(())
}

