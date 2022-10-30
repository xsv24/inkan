pub mod adapters;
pub mod app_context;
pub mod cli;
pub mod domain;
pub mod utils;

use crate::cli::{checkout, commit, context};
use adapters::{sqlite::Sqlite, Git};
use anyhow::Context;
use app_context::AppContext;
use clap::Parser;
use directories::ProjectDirs;
use domain::commands::{CommandActions, Commands};
use rusqlite::Connection;

#[derive(Debug, Parser)]
#[clap(name = "git-kit")]
#[clap(bin_name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
#[clap(version)]
pub enum Cli {
    /// Commit staged changes via git with a template message.
    #[clap(subcommand)]
    Commit(commit::Template),
    /// Checkout an existing branch or create a new branch and add a ticket number as context for future commits.
    Checkout(checkout::Arguments),
    /// Add or update the ticket number related to the current branch.
    Context(context::Arguments),
}

fn init() -> anyhow::Result<(ProjectDirs, Sqlite)> {
    let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
        .context("Failed to retrieve 'git-kit' config")?;

    let connection = Connection::open(project_dir.config_dir().join("db"))?;
    let store = Sqlite::new(connection)?;

    Ok((project_dir, store))
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let (project_dir, store) = init()?;
    let context = AppContext::new(Git, store, project_dir)?;
    let actions = CommandActions::new(&context)?;

    match args {
        Cli::Commit(template) => {
            actions.commit(template)?;
        }
        Cli::Checkout(checkout) => {
            actions.checkout(checkout)?;
        }
        Cli::Context(current) => {
            actions.current(current)?;
        }
    };

    context.close()?;

    Ok(())
}
