use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use log::info;
use thiserror::Error;

/// error type
/// There are two types of errors:
/// 1. anyhow error, it mean some error only in this machine, like file not found, etc.
/// 2. custom error, it mean some error in the consensus, like invalid block, etc.
#[derive(Error, Debug)]
pub enum Error {
    #[error("anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("custom error: {message}")]
    Custom { message: String },

    #[error("isolate block error: {key}")]
    IsolateBlock { key: String },

    #[error("parent not sorted: {key}")]
    ParentNotSorted { key: String },

    #[error("unknown block: {key}")]
    UnknownBlock { key: String },

    #[error("not well connected block: {key}")]
    NotWellConnectedBlock { key: String },

    #[error("cycle dependency: {key}")]
    CycleDependency { key: String },

    #[error("empty parent keys")]
    EmptyParentKeys,

    #[error("no lca found for tips")]
    NoLcaFoundForTips,

    #[error("top sort error")]
    TopSortError,

    #[error("block not found: {key}")]
    BlockNotFound { key: String },

    #[error("merkle tree error: {message}")]
    MerkleTree { message: String },

    #[error("account not found: {message}")]
    AccountNotFound { message: String },

    #[error("account balance not enough: {message}")]
    AccountBalanceNotEnough { message: String },

    #[error("account action hash not match: {message}")]
    AccountHashNotMatch { message: String },

    #[error("impossible error: {message}")]
    Impossible { message: String },

    #[error("merge from and to is the same: {message}")]
    MergeFromAndToIsTheSame { message: String },

    #[error("tips not found")]
    TipsNotFound,

    #[error("mining failed")]
    MiningFailed,

    #[error("invalid state root")]
    InvalidStateRoot,
}

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        let body = Body::from(message);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap_or_default()
    }
}

pub struct Res<T: serde::Serialize> {
    pub data: T,
}

impl<T: serde::Serialize> IntoResponse for Res<T> {
    fn into_response(self) -> axum::response::Response {
        let body = Body::from(serde_json::to_string(&self.data).unwrap_or_default());
        Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .unwrap_or_default()
    }
}

pub struct BinaryRes {
    pub data: Vec<u8>,
}

impl IntoResponse for BinaryRes {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .body(Body::from(self.data))
            .unwrap_or_else(|e| {
                info!("Failed to build response: {e:?}");
                Default::default()
            })
    }
}
