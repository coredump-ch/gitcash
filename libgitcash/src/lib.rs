use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use git2::Sort;
use tracing::debug;

mod error;
mod transaction;

use crate::{
    error::Error,
    transaction::{extract_transaction, Account, Transaction},
};

/// A GitCash repository and all its transactions
pub struct Repo {
    transactions: Vec<Transaction>,
}

impl Repo {
    /// Open a GitCash repository at the specified path and parse all transactions
    pub fn open(repo_path: &Path) -> Result<Self, Error> {
        // Open git repo
        tracing::debug!("Loading repository at {:?}", repo_path);
        let repo = match git2::Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(e) => return Err(Error::RepoError(format!("Failed to open repo: {}", e))),
        };

        // Traverse commits from oldest to newest, extract transactions
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
            let transaction = extract_transaction(message)?;
            transactions.push(transaction);
        }

        Ok(Repo { transactions })
    }

    /// Return set of all acounts
    pub fn accounts(&self) -> HashSet<Account> {
        self.transactions
            .iter()
            .flat_map(|t| [t.from.clone(), t.to.clone()])
            .collect()
    }

    /// Return all accounts and their balances
    pub fn balances(&self) -> HashMap<Account, i32> {
        let mut accounts = HashMap::new();
        for transaction in &self.transactions {
            let source = accounts.entry(transaction.from.clone()).or_default();
            *source -= transaction.amount;
            let destination = accounts.entry(transaction.to.clone()).or_default();
            *destination += transaction.amount;
        }
        accounts
    }
}
