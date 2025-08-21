use anyhow::Context;
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use utils::error::{Res, Result};
use crate::state::mz_state::MzState;

pub mod block;

pub fn api_router(mz_state: MzState) -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/block", get(block::get_block_api))
        .route("/tips", get(block::get_tips_api))
        .with_state(mz_state)
}

async fn hello() -> Result<Res<String>> {
    Ok(Res {
        data: "Hello Mazzaroth".to_string(),
    })
}

pub async fn serve(mz_state: MzState, http_port: u16) -> anyhow::Result<()> {
    let app = api_router(mz_state);
    let listener = TcpListener::bind(format!("[::]:{}", http_port))
        .await
        .with_context(|| format!("Failed to bind to port {}", http_port))?;
    axum::serve(listener, app)
        .await
        .with_context(|| "Failed to serve API".to_string())?;
    Ok(())
}
