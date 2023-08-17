use clap::{Parser, Subcommand};
use libgitcash::Repo;
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

    let repo = Repo::open(&args.repo_path)?;
    println!("Accounts:");
    for account in repo.accounts() {
        println!("- Account: {} ({:?})", account.name, account.account_type);
    }

    Ok(())
}
