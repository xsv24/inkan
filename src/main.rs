mod adapters;
mod app_config;
mod app_context;
mod cli;
mod domain;
mod migrations;
mod utils;

use std::fmt::Debug;

use cli::{checkout, commit, context, log::LogLevel, templates};

use adapters::{sqlite::Sqlite, Git};
use anyhow::{Context, Ok};
use app_context::AppContext;
use clap::{Parser, Subcommand};
use domain::{
    adapters::{Git as _, Store},
    commands::Actions,
    models::{Config, ConfigKey, ConfigStatus},
};

use crate::app_config::AppConfig;

#[derive(Debug, Parser)]
#[clap(name = "git-kit")]
#[clap(bin_name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
#[clap(version)]
pub struct Cli {
    /// File path to your 'git-kit' config file
    #[clap(short, long)]
    config: Option<String>,

    /// Log level
    #[clap(arg_enum, long, default_value_t=LogLevel::None)]
    log: LogLevel,

    /// Commands
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Commit staged changes via git with a template message.
    Commit(commit::Arguments),
    /// Checkout an existing branch or create a new branch and add a ticket number as context for future commits.
    Checkout(checkout::Arguments),
    /// Add or update the ticket number related to the current branch.
    Context(context::Arguments),
    /// Get or Set persisted configuration file path.
    #[clap(subcommand)]
    Config(cli::config::Arguments),
    /// Display a list of configured templates.
    Templates,
}

impl Cli {
    fn init(&self) -> anyhow::Result<AppContext<Git, Sqlite>> {
        self.log.init_logger();

        let store =
            Sqlite::new(AppConfig::db_connection()?).context("Failed to initialize 'Sqlite'")?;

        let git = Git;

        let config = match &self.config {
            Some(path) => Config::new(ConfigKey::Once, path.into(), ConfigStatus::Active),
            None => store.get_config(None),
        }?;

        // use custom user config if provided or default.
        let config = AppConfig::new(config, git.root_directory()?)?;

        let context = AppContext::new(git, store, config)?;

        Ok(context)
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut context = cli.init()?;
    let actions = Actions::new(&context);

    let result = match cli.commands {
        Commands::Checkout(args) => checkout::handler(&actions, args),
        Commands::Context(args) => context::handler(&actions, args),
        Commands::Commit(args) => commit::handler(&actions, &context.config, args),
        Commands::Config(args) => cli::config::handler(&mut context.store, args),
        Commands::Templates => templates::handler(&context.config),
    };

    // close the connection no matter if we error or not.
    context.close()?;

    Ok(result?)
}

#[test]
fn verify_app() {
    // Simple test to assure cli builds correctly
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
