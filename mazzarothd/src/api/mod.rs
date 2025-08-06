use crate::{MAZZAROTH_HTTP_PORT, MAZZAROTH_HTTP_PORT_DEFAULT};
use axum::Router;
use tokio::net::TcpListener;

pub mod peer;

pub fn api_router() -> Router {
    Router::new().nest("/peer", peer::api_router())
}

pub async fn serve() -> anyhow::Result<()> {
    let port = std::env::var(MAZZAROTH_HTTP_PORT)
        .unwrap_or_else(|_| MAZZAROTH_HTTP_PORT_DEFAULT.to_string());
    let app = api_router();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
