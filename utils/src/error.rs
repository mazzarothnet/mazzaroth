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

    #[error("isolate block error: {message}")]
    IsolateBlock { message: String },

    #[error("parent not sorted: {message}")]
    ParentNotSorted { message: String },

    #[error("unknown block: {message}")]
    UnknownBlock { message: String },

    #[error("not well connected block: {message}")]
    NotWellConnectedBlock { message: String },

    #[error("cycle dependency: {message}")]
    CycleDependency { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;
