use axum::Router;

pub mod peer;

pub fn api_router() -> Router {
    Router::new().nest("/peer", peer::api_router())
}
