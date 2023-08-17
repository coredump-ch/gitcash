#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repo error: {0}")]
    RepoError(String),
    #[error("Libgit error: {0}")]
    LibgitError(#[from] git2::Error),
    #[error("Could not parse transaction: {0}")]
    TransactionParseError(String),
}
