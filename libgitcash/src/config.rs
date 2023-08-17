use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoConfig {
    pub name: String,
    pub currency: Currency,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Currency {
    pub code: String,
    pub divisor: usize,
}
