#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repo error: {0}")]
    RepoError(String),
    #[error("Libgit error: {0}")]
    LibgitError(#[from] git2::Error),
    #[error("Could not parse transaction: {0}")]
    TransactionParseError(String),
    #[error("Could not serialize transaction: {0}")]
    TransactionSerializeError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}
