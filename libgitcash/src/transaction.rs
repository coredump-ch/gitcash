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
        if self.amount == 0 && self.to.account_type == AccountType::User {
            return format!("Transaction: Add user {}", self.to.name);
        }

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
    pub class: Option<String>,
    pub ean: Option<u64>,
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

/// Validate a string for usage as account name
fn validate_account_name(name: &str) -> Result<(), Error> {
    if !name.chars().all(|char| char.is_ascii_alphanumeric()) {
        return Err(Error::ValidationError(
            "Invalid account name, must consist only of ascii characters or digits".into(),
        ));
    }
    if name.is_empty() {
        return Err(Error::ValidationError(
            "Invalid account name, may not be empty".into(),
        ));
    }
    Ok(())
}

impl Account {
    pub fn new<S: Into<String>>(account_type: AccountType, name: S) -> Result<Self, Error> {
        let name = name.into();
        validate_account_name(&name)?;
        Ok(Self { account_type, name })
    }

    pub fn user<S: Into<String>>(name: S) -> Result<Self, Error> {
        Self::new(AccountType::User, name)
    }

    pub fn point_of_sale<S: Into<String>>(name: S) -> Result<Self, Error> {
        Self::new(AccountType::PointOfSale, name)
    }

    pub fn source<S: Into<String>>(name: S) -> Result<Self, Error> {
        Self::new(AccountType::Source, name)
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
        validate_account_name(&name)?;

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
