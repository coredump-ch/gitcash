use std::path::PathBuf;

use libgitcash::get_accounts;
use tracing::metadata::LevelFilter;

pub fn main() {
    // Initialize logging subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set tracing subscriber");

    let path = PathBuf::from("../../gitcash-demo-repo");
    tracing::info!("Loading repository at {:?}", path);
    for account in get_accounts(path.as_ref()) {
        println!("Account: {}", account);
    }
}
