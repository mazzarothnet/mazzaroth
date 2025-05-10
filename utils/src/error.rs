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
}

pub type Result<T> = std::result::Result<T, Error>;
