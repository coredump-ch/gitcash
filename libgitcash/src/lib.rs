use std::{collections::HashSet, path::Path};

use git2::{Repository, Sort};
use tracing::debug;

#[derive(Debug, serde::Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: usize,
    pub description: Option<String>,
    pub meta: Option<TransactionMeta>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TransactionMeta {
    pub class: String,
    pub ean: u64,
}

fn extract_transaction(message: &str) -> Transaction {
    let mut lines = Vec::new();
    let mut in_transaction = false;
    for line in message.lines() {
        match in_transaction {
            false if line == "---" => in_transaction = true,
            true if line == "---" => break,
            false => continue,
            true => lines.push(line.to_string()),
        }
    }
    toml::from_str(&lines.join("\n")).expect("Invalid transaction data")
}

pub fn get_transactions(repo_path: &Path) -> Vec<Transaction> {
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open repo: {}", e),
    };
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    revwalk
        .set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)
        .unwrap();
    let mut transactions = Vec::new();
    for commit in revwalk {
        let commit_oid = commit.unwrap();
        let commit = repo.find_commit(commit_oid).unwrap();
        let message = match commit.message_raw() {
            Some(msg) => msg,
            None => continue,
        };
        if !message.starts_with("Transaction: ") {
            continue;
        }
        debug!("Processing commit {}", commit.id());
        transactions.push(extract_transaction(message));
    }
    transactions
}

pub fn get_accounts(repo_path: &Path) -> HashSet<String> {
    let mut accounts = HashSet::new();
    for transaction in get_transactions(repo_path) {
        accounts.insert(transaction.from);
        accounts.insert(transaction.to);
    }
    accounts
}
