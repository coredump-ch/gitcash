use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct RepoConfig {
    pub name: String,
    pub currency: Currency,
}

impl RepoConfig {
    /// Load repo config in the specified repo path
    pub fn load(repo_path: &Path) -> Result<Self, Error> {
        let config_string = std::fs::read_to_string(repo_path.join("gitcash.toml"))
            .map_err(|e| Error::RepoError(format!("Could not read gitcash.toml: {}", e)))?;
        Self::from_str(&config_string)
    }

    pub fn from_str(config_string: &str) -> Result<Self, Error> {
        let config: RepoConfig = toml::from_str(&config_string)
            .map_err(|e| Error::RepoError(format!("Could not parse gitcash.toml: {}", e)))?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Currency {
    pub code: String,
    pub divisor: usize,
}

#[cfg(test)]
mod tests {
    use crate::{Currency, RepoConfig};

    #[test]
    fn test_from_str() {
        let repo_config_str = r#"name = "foo"
               [currency]
               code = "CHF"
               divisor = 100"#;
        println!("{}", repo_config_str);
        let repo_config = RepoConfig::from_str(repo_config_str).unwrap();
        assert_eq!(
            RepoConfig {
                name: "foo".to_owned(),
                currency: Currency {
                    code: "CHF".to_owned(),
                    divisor: 100
                }
            },
            repo_config
        );
    }
}
