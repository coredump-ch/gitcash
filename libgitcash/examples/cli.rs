use std::path::PathBuf;

use clap::{Parser, Subcommand};
use libgitcash::get_accounts;
use tracing::metadata::LevelFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    repo_path: std::path::PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ListAccounts,
}

pub fn main() -> anyhow::Result<()> {
    // Initialize logging subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set tracing subscriber");

    // Parse args
    let args = Args::parse();

    let path = PathBuf::from(&args.repo_path);
    tracing::info!("Loading repository at {:?}", path);
    println!("Accounts:");
    for account in get_accounts(path.as_ref())? {
        println!("- Account: {} ({:?})", account.name, account.account_type);
    }

    Ok(())
}
