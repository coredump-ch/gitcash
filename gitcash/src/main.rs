use std::{fs::write, path::PathBuf};

use anyhow::{anyhow, bail, Context};
use clap::{Parser, Subcommand};
use config::Config;
use inquire::{Autocomplete, InquireError};
use libgitcash::{Account, AccountType, Repo, Transaction};
use tracing::metadata::LevelFilter;

use crate::validators::{NewUsernameValidator, UsernameValidator};

mod config;
mod validators;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, PartialEq, Eq)]
enum Command {
    /// List all accounts
    Accounts,
    /// List all account balances
    Balances,
    /// List all user accounts with negative balances
    Shame,

    /// Interactive CLI
    Cli,

    /// Generate an example config
    GenerateConfig,
}

#[derive(Clone)]
struct CommandSuggester {
    commands: Vec<&'static str>,
}

impl CommandSuggester {
    pub fn new(commands: &[CliCommand]) -> Self {
        Self {
            commands: commands
                .iter()
                .map(|command| command.command())
                .collect::<Vec<_>>(),
        }
    }
}

impl Autocomplete for CommandSuggester {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        if input.is_empty() {
            return Ok(vec![]);
        }
        Ok(self
            .commands
            .iter()
            .filter(|acc| acc.to_lowercase().contains(&input.to_lowercase()))
            .map(|value| value.to_string())
            .collect::<Vec<_>>())
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(highlighted_suggestion)
    }
}

#[derive(Debug, Clone, Copy)]
enum CliCommand {
    AddUser,
    Help,
}

impl CliCommand {
    fn command(&self) -> &'static str {
        match self {
            CliCommand::AddUser => "adduser",
            CliCommand::Help => "help",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            CliCommand::AddUser => "Add a new user",
            CliCommand::Help => "Show this help",
        }
    }
}

impl TryFrom<&str> for CliCommand {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_ref() {
            "adduser" => Ok(CliCommand::AddUser),
            "help" => Ok(CliCommand::Help),
            other => Err(anyhow!("Invalid command: {}", other)),
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    // Initialize logging subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set tracing subscriber");

    // Parse args
    let args = Args::parse();

    if args.command == Command::GenerateConfig {
        if args.config.exists() {
            bail!(format!(
                "Config file {:?} already exists!",
                args.config.display()
            ));
        }

        let example = include_str!("../../config.toml.example");

        write(&args.config, example).with_context(|| {
            format!(
                "unable to write example config to {:?}",
                args.config.display()
            )
        })?;
        println!("✅ Wrote example config to {:?}", args.config.display());
        return Ok(());
    }

    // Parse config
    let config = Config::load(&args.config)?;

    // Open repo
    let mut repo = Repo::open(&config.repo_path)?;

    // Run command
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
            println!("Welcome to the GitCash CLI for {}!", config.git_name);
            loop {
                if let Err(e) = handle_cli_input(&mut repo, &config) {
                    match e.downcast::<InquireError>() {
                        Ok(e) => return Err(e.into()),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
        }
        Command::GenerateConfig => {
            unreachable!("handled above");
        }
    }

    Ok(())
}

// Valid commands
const COMMANDS: [CliCommand; 2] = [CliCommand::AddUser, CliCommand::Help];

fn handle_cli_input(repo: &mut Repo, config: &Config) -> anyhow::Result<()> {
    // Get list of valid user account names
    let usernames = repo
        .accounts()
        .into_iter()
        .filter(|acc| acc.account_type == AccountType::User)
        .map(|acc| acc.name)
        .collect::<Vec<_>>();

    // Autocompletion: All names that contain the current input as
    // substring (case-insensitive)
    let name_suggester = {
        let usernames = usernames.clone();
        move |val: &str| {
            Ok(usernames
                .iter()
                .filter(|acc| acc.to_lowercase().contains(&val.to_lowercase()))
                .cloned()
                .collect::<Vec<_>>())
        }
    };

    // First, ask for command, product or amount
    let target = inquire::Text::new("Amount, EAN or command:")
        .with_placeholder("e.g. 2.50 CHF")
        .with_autocomplete(CommandSuggester::new(&COMMANDS))
        .prompt()?;

    // Check whether it's a command
    match CliCommand::try_from(&*target) {
        Ok(CliCommand::AddUser) => {
            println!("Adding user");
            let new_name = inquire::Text::new("Name:")
                .with_validator(NewUsernameValidator::new(usernames.clone()))
                .prompt()?;
            repo.create_transaction(Transaction {
                from: Account::source("cash")?,
                to: Account::user(new_name.clone())?,
                amount: 0,
                description: Some(format!("Create user {}", new_name)),
                meta: None,
            })?;
            println!("Successfully added user {}", new_name);
            return Ok(());
        }
        Ok(CliCommand::Help) => {
            println!("Available commands:");
            for command in COMMANDS {
                println!("- {}: {}", command.command(), command.description());
            }
            return Ok(());
        }
        Err(_) => {}
    };

    // Not a command, treat it as amount if below a reasonable limit
    let amount: f32 = target
        .parse()
        .context(format!("Invalid amount: {}", target))?;
    if amount > 1337.0 {
        bail!("Neither a valid command nor a known EAN, and definitely not a reasonable amount either");
    }
    let name = inquire::Text::new("Name:")
        .with_autocomplete(name_suggester.clone())
        .with_validator(UsernameValidator::new(usernames))
        .prompt()?;
    println!("Creating transaction: {} pays {:.2} CHF", name, amount);
    repo.create_transaction(Transaction {
        from: Account::user(name)?,
        to: config.account.clone(),
        amount: repo.convert_amount(amount),
        description: None,
        meta: None,
    })?;

    Ok(())
}
