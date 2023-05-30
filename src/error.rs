#[derive(Debug, thiserror::Error)]
pub enum Err {
    #[error("{0}")]
    ExtServer(#[from] ExtServerErrors),
}

#[derive(Debug, thiserror::Error)]
pub enum ExtServerErrors {
    #[error("invalid extension server command: {0}")]
    InvalidCmd(String),
}
pub type Result<T> = std::result::Result<T, Err>;
