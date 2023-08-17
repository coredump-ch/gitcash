use crate::{error::Error, RepoConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub from: Account,
    pub to: Account,
    pub amount: i32,
    pub description: Option<String>,
    pub meta: Option<TransactionMeta>,
}

impl Transaction {
    pub fn summary(&self, config: &RepoConfig) -> String {
        format!(
            "Transaction: {:?} {} pays {:.2} {} to {:?} {}",
            self.from.account_type,
            self.from.name,
            self.amount as f32 / config.currency.divisor as f32,
            config.currency.code,
            self.to.account_type,
            self.to.name,
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionMeta {
    pub class: String,
    pub ean: u64,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Clone, Copy)]
pub enum AccountType {
    /// A user can both receive and send money
    User,
    /// A point of sale can only receive money
    PointOfSale,
    /// A cash source is used to deposit money into the system
    Source,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Clone, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct Account {
    pub account_type: AccountType,
    pub name: String,
}

impl Account {
    pub fn user<S: Into<String>>(name: S) -> Self {
        Self {
            account_type: AccountType::User,
            name: name.into(),
        }
    }

    pub fn point_of_sale<S: Into<String>>(name: S) -> Self {
        Self {
            account_type: AccountType::PointOfSale,
            name: name.into(),
        }
    }

    pub fn source<S: Into<String>>(name: S) -> Self {
        Self {
            account_type: AccountType::Source,
            name: name.into(),
        }
    }
}

impl Into<String> for Account {
    fn into(self) -> String {
        format!(
            "{}:{}",
            match self.account_type {
                AccountType::User => "user",
                AccountType::PointOfSale => "pos",
                AccountType::Source => "source",
            },
            self.name,
        )
    }
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
            "pos" => AccountType::PointOfSale,
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
pub(crate) fn extract_transaction(commit_message: &str) -> Result<Transaction, Error> {
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
