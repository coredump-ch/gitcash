use clap::{Parser, Subcommand};
use libgitcash::{AccountType, Repo};
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
    /// List all accounts
    Accounts,
    /// List all account balances
    Balances,
    /// List all user accounts with negative balances
    Shame,
}

pub fn main() -> anyhow::Result<()> {
    // Initialize logging subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set tracing subscriber");

    // Parse args
    let args = Args::parse();

    // Open repo
    let repo = Repo::open(&args.repo_path)?;

    match args.command {
        Command::Accounts => {
            println!("Accounts:");
            for account in repo.accounts() {
                println!("- Account: {} ({:?})", account.name, account.account_type);
            }
        }
        Command::Balances => {
            println!("Balances:");
            for (account, balance) in repo.balances() {
                println!(
                    "- {}: {:.2} CHF [{:?}]",
                    account.name,
                    balance as f32 / 100.0,
                    account.account_type
                );
            }
        }
        Command::Shame => {
            println!("Wall of shame (negative user balances):");
            let negative_balance_accounts = repo
                .balances()
                .into_iter()
                .filter(|(account, balance)| {
                    account.account_type == AccountType::User && *balance < 0
                })
                .collect::<Vec<_>>();
            for (account, balance) in &negative_balance_accounts {
                println!(
                    "- {}: {:.2} CHF [{:?}]",
                    account.name,
                    *balance as f32 / 100.0,
                    account.account_type
                );
            }
            if negative_balance_accounts.is_empty() {
                println!("None at all! 🎉");
            }
        }
    }

    Ok(())
}
