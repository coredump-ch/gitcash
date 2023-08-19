use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoConfig {
    pub name: String,
    pub currency: Currency,
}

impl RepoConfig {
    /// Load repo config in the specified repo path
    pub fn load(repo_path: &Path) -> Result<Self, Error> {
        let config_string = std::fs::read_to_string(repo_path.join("gitcash.toml"))
            .map_err(|e| Error::RepoError(format!("Could not read gitcash.toml: {}", e)))?;
        let config: RepoConfig = toml::from_str(&config_string)
            .map_err(|e| Error::RepoError(format!("Could not parse gitcash.toml: {}", e)))?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Currency {
    pub code: String,
    pub divisor: usize,
}
