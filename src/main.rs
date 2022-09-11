pub mod args;
pub mod branch;
pub mod cli;
pub mod git_commands;
pub mod template;
pub mod try_convert;

use anyhow::{anyhow, Context};
use branch::Branch;
use clap::Parser;
use directories::ProjectDirs;
use rusqlite::Connection;
use std::process::Command;

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
        .context("Failed to retrieve 'git-kit' config")?;

    let conn = Connection::open(project_dir.config_dir().join("db"))?;

    // Could move into build script ?
    conn.execute(
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
            // TODO: Move this into a separate function.
            let args = template.args();
            let template = template.read_file(&project_dir)?;
            let contents = args.commit_message(template, &conn)?;

            let _ = Command::new("git")
                .args(["commit", "-m", &contents, "-e"])
                .status()?;
        }
        cli::Command::Checkout { name, ticket } => {
            // TODO: Move this into a separate function.
            // We want to store the branch name against and ticket number
            // So whenever we commit we get the ticket number from the branch
            let branch = Branch::new(&name, ticket)?;
            branch.insert_or_update(&conn)?;

            // Attempt to create branch
            let create = Command::new("git")
                .args(["checkout", "-b", &name])
                .output()?;

            // If the branch exists check it out
            if !create.status.success() {
                Command::new("git").args(["checkout", &name]).status()?;
            }
        }
    };

    conn.close()
        .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

    Ok(())
}
