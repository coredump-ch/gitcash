use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use git2::{Signature, Sort};
use tracing::debug;

mod config;
mod error;
mod transaction;

use crate::{error::Error, transaction::extract_transaction};

pub use crate::{
    config::{Currency, RepoConfig},
    transaction::{Account, AccountType, Transaction},
};

/// A GitCash repository and all its transactions
pub struct Repo {
    repository: git2::Repository,
    config: RepoConfig,
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

        // Read config
        let config = RepoConfig::load(repo_path)?;

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

        Ok(Repo {
            repository: repo,
            config,
            transactions,
        })
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

    /// Convert a floating-point currency amount into an integer based value.
    pub fn convert_amount(&self, amount: f32) -> i32 {
        let amount = amount * self.config.currency.divisor as f32;
        amount as i32
    }

    pub fn create_transaction(&self, transaction: &Transaction) -> Result<(), Error> {
        let summary = transaction.summary(&self.config);
        debug!("Creating commit: {}", &summary);

        // Encode transaction
        let transaction_toml = toml::to_string(transaction)
            .map_err(|e| Error::TransactionSerializeError(e.to_string()))?;
        let commit_message = format!("{}\n\n---\n{}\n---", &summary, transaction_toml.trim());

        // Create signature (for both committer and author)
        let sig = Signature::now("GitCash CLI", "gitcash@coredump.ch")?;

        // Create tree object
        let head = self
            .repository
            .find_commit(self.repository.head()?.target().unwrap())?;
        let tree_id = self.repository.index()?.write_tree()?;
        let tree = self.repository.find_tree(tree_id)?;

        // Create commit
        let commit =
            self.repository
                .commit(Some("HEAD"), &sig, &sig, &commit_message, &tree, &[&head])?;

        debug!("Created commit: {commit}");
        Ok(())
    }
}
