use crate::config::CFG;
use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;

pub fn api_router() -> Router {
    Router::new()
}

pub async fn serve() -> anyhow::Result<()> {
    let port = CFG.http_port;
    let app = api_router();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .with_context(|| format!("Failed to bind to port {}", port))?;
    axum::serve(listener, app)
        .await
        .with_context(|| "Failed to serve API".to_string())?;
    Ok(())
}
