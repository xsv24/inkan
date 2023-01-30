use std::fmt::Debug;

use crate::adapters::{sqlite::Sqlite, Git};
use crate::app_config::AppConfig;
use crate::app_context::AppContext;
use crate::cli::{commands::Commands, log::LogLevel};
use anyhow::{Context, Ok};
use clap::Parser;

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
    pub commands: Commands,
}

impl Cli {
    pub fn init(&self) -> anyhow::Result<AppContext<Git, Sqlite>> {
        self.log.init_logger();

        let git = Git;

        let connection = AppConfig::db_connection()?;
        let store = Sqlite::new(connection).context("Failed to initialize 'Sqlite'")?;

        let app_config = AppConfig::new(self.config.clone(), &git, &store)?;

        let context = AppContext::new(git, store, app_config.config)?;

        Ok(context)
    }
}

#[test]
fn verify_app() {
    // Simple test to assure cli builds correctly
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
