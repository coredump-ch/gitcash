use std::{collections::HashSet, path::Path};

use git2::{Repository, Sort};
use tracing::debug;

mod error;
mod transaction;

use crate::{
    error::Error,
    transaction::{Account, Transaction},
};

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
