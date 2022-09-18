pub mod args;
pub mod branch;
pub mod cli;
pub mod context;
pub mod git_commands;
pub mod template;
pub mod try_convert;

use clap::Parser;
use context::Context;
use git_commands::{Git, GitCommands};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let context = Context::new(Git)?;

    // Could move into build script ?
    context.connection.execute(
        "CREATE TABLE IF NOT EXISTS branch (
            name TEXT NOT NULL PRIMARY KEY,
            ticket TEXT,
            data BLOB,
            created TEXT NOT NULL
        )",
        (),
    )?;

    match args.command {
        cli::Command::Commit(template) => {
            let contents = template.commit(&context)?;
            context.commands.commit(&contents)?;
        }
        cli::Command::Checkout(checkout) => {
            checkout.checkout(&context)?;
        }
    };

    context.close()?;

    Ok(())
}
