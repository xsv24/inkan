pub mod actions;
pub mod args;
pub mod branch;
pub mod cli;
pub mod context;
pub mod git_commands;
pub mod template;
pub mod try_convert;

use actions::{Actions, CommandActions};
use clap::Parser;
use context::Context;
use git_commands::Git;

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let context = Context::new(Git)?;
    let actions = CommandActions::new(&context)?;

    match args.command {
        cli::Command::Commit(template) => {
            actions.commit(template)?;
        }
        cli::Command::Checkout(checkout) => {
            actions.checkout(checkout)?;
        }
        cli::Command::Context(current) => {
            actions.current(current)?;
        }
    };

    context.close()?;

    Ok(())
}
