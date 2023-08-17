use std::sync::Arc;

use clap::{Parser, Subcommand};
use inquire::validator::{ErrorMessage, Validation};
use libgitcash::{Account, AccountType, Repo, Transaction};
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

    /// Interactive CLI
    Cli,
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
        Command::Cli => {
            println!("Welcome to the GitCash CLI!");

            // Get list of valid user account names
            let usernames = Arc::new(
                repo.accounts()
                    .into_iter()
                    .filter(|acc| acc.account_type == AccountType::User)
                    .map(|acc| acc.name)
                    .collect::<Vec<_>>(),
            );

            // Validators
            let amount_validator = move |value: &str| {
                Ok(if let Ok(_) = value.parse::<f64>() {
                    Validation::Valid
                } else {
                    Validation::Invalid(ErrorMessage::Default)
                })
            };
            let username_validator = {
                let usernames = usernames.clone();
                move |value: &str| {
                    Ok(if usernames.iter().any(|name| name == value) {
                        Validation::Valid
                    } else {
                        Validation::Invalid(ErrorMessage::Custom(format!(
                            "Not a known username: {}",
                            value
                        )))
                    })
                }
            };

            // Autocompletion: All names that contain the current input as
            // substring (case-insensitive)
            let suggester = {
                move |val: &str| {
                    Ok(usernames
                        .iter()
                        .filter(|acc| acc.to_lowercase().contains(&val.to_lowercase()))
                        .cloned()
                        .collect())
                }
            };

            loop {
                // First, ask for amount, then for name
                let amount: f32 = inquire::Text::new("Amount?")
                    .with_placeholder("e.g. 2.50 CHF")
                    .with_validator(amount_validator)
                    .prompt()?
                    .parse()
                    .expect("Invalid float (even after validation)");
                let name = inquire::Text::new("Name?")
                    .with_autocomplete(suggester.clone())
                    .with_validator(username_validator.clone())
                    .prompt()?;
                println!("Creating transaction: {} pays {:.2} CHF", name, amount);

                repo.create_transaction(&Transaction {
                    from: Account::user(name),
                    to: Account::point_of_sale("TODO"),
                    amount: repo.convert_amount(amount),
                    description: None,
                    meta: None,
                })
                .unwrap();
            }
        }
    }

    Ok(())
}
