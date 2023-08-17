use std::{collections::HashSet, path::Path};

use git2::{Repository, Sort};
use tracing::debug;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repo error: {0}")]
    RepoError(String),
    #[error("Libgit error: {0}")]
    LibgitError(#[from] git2::Error),
    #[error("Could not parse transaction: {0}")]
    TransactionParseError(String),
}

#[derive(Debug, serde::Deserialize)]
pub struct Transaction {
    pub from: Account,
    pub to: Account,
    pub amount: usize,
    pub description: Option<String>,
    pub meta: Option<TransactionMeta>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TransactionMeta {
    pub class: String,
    pub ean: u64,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum AccountType {
    /// A user can both receive and send money
    User,
    /// A point of payment can only receive money
    PointOfPayment,
    /// A cash source is used to deposit money into the system
    Source,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct Account {
    pub account_type: AccountType,
    pub name: String,
}

impl TryFrom<String> for Account {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        let mut parts = value.split(':');

        let raw_account_type = parts.next().ok_or_else(|| {
            Error::TransactionParseError(format!("Account does not contain ':': {}", value))
        })?;
        let account_type = match raw_account_type {
            "user" => AccountType::User,
            "pop" => AccountType::PointOfPayment,
            "source" => AccountType::Source,
            other => {
                return Err(Error::TransactionParseError(format!(
                    "Invalid account type: {}",
                    other
                )))
            }
        };

        let name = parts
            .next()
            .ok_or_else(|| {
                Error::TransactionParseError(format!(
                    "Account does not contain a name after ':': {}",
                    value
                ))
            })?
            .to_string();

        Ok(Self { account_type, name })
    }
}

/// Extract a transaction from a commit message
fn extract_transaction(commit_message: &str) -> Result<Transaction, Error> {
    let mut lines = Vec::new();
    let mut in_transaction = false;
    for line in commit_message.lines() {
        match in_transaction {
            false if line == "---" => in_transaction = true,
            true if line == "---" => break,
            false => continue,
            true => lines.push(line.to_string()),
        }
    }
    toml::from_str(&lines.join("\n"))
        .map_err(|e| Error::TransactionParseError(format!("Invalid TOML transaction data: {}", e)))
}

pub fn get_transactions(repo_path: &Path) -> Result<Vec<Transaction>, Error> {
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => return Err(Error::RepoError(format!("Failed to open repo: {}", e))),
    };
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;
    let mut transactions = Vec::new();
    for commit_oid in revwalk {
        let commit = repo.find_commit(commit_oid?)?;
        let message = match commit.message_raw() {
            Some(msg) => msg,
            None => continue,
        };
        if !message.starts_with("Transaction: ") {
            continue;
        }
        debug!("Processing commit {}", commit.id());
        transactions.push(extract_transaction(message)?);
    }
    Ok(transactions)
}

pub fn get_accounts(repo_path: &Path) -> Result<HashSet<Account>, Error> {
    let mut accounts = HashSet::new();
    for transaction in get_transactions(repo_path)? {
        accounts.insert(transaction.from);
        accounts.insert(transaction.to);
    }
    Ok(accounts)
}
