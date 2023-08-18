use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use libgitcash::{Account, AccountType};
use serde::{Deserialize, Serialize};

/// GitCash configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Path to the GitCash repository to use
    pub repo_path: PathBuf,

    /// Account corresponding to this PoS
    pub account: Account,

    /// Name to use for git commits
    pub git_name: String,
    /// E-mail to use for git commits
    pub git_email: String,
}

impl Config {
    /// Load config from the specified config path
    pub fn load(config_path: &Path) -> anyhow::Result<Self> {
        let config_string: String = std::fs::read_to_string(config_path)
            .context(format!("Could not read config from {:?}", config_path))?;
        let config: Self = toml::from_str(&config_string)
            .context(format!("Could not parse config at {:?}", config_path))?;
        if config.account.account_type != AccountType::PointOfSale {
            bail!(
                "Account type must be {:?}, not {:?}",
                AccountType::PointOfSale,
                config.account.account_type
            );
        }
        Ok(config)
    }
}
