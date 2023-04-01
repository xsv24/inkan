use std::fmt::Debug;

use crate::adapters::sqlite::Sqlite;
use crate::adapters::{Git, GitCommand};
use crate::app_config::AppConfig;
use crate::app_context::AppContext;
use crate::cli::{commands::Commands, log::LogLevel};
use anyhow::Ok;
use clap::clap_derive::ArgEnum;
use clap::Parser;

#[derive(Clone, Debug, ArgEnum, Default, PartialEq, Eq)]
pub enum Interactive {
    #[default]
    Enable,
    Disable,
}

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

    /// Interactive prompts
    #[clap(arg_enum, short, long, default_value_t=Interactive::Enable)]
    prompt: Interactive,

    /// Commands
    #[clap(subcommand)]
    pub commands: Commands,
}

impl Cli {
    pub fn init(&self) -> anyhow::Result<AppContext<Git<GitCommand>, Sqlite>> {
        self.log.init_logger();

        let git = Git { git: GitCommand };

        let connection = AppConfig::db_connection()?;
        let store = Sqlite::new(connection);

        let app_config = AppConfig::new(self.config.clone(), &git, &store)?;

        let context = AppContext::new(git, store, app_config.config, self.prompt.clone())?;

        Ok(context)
    }
}

#[test]
fn verify_app() {
    // Simple test to assure cli builds correctly
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
