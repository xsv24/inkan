pub mod args;
pub mod branch;
pub mod cli;
pub mod template;

use anyhow::Context;
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
            let args = template.args();
            let template = template.read_file()?;
            let contents = args.commit_message(template, &conn)?;

            let _ = Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(contents)
                .arg("-e")
                .status()?;
        }
        cli::Command::Checkout { name, ticket } => {
            // We want to store the branch name against and ticket number
            // So whenever we commit we get the ticket number from the branch
            let branch = Branch::new(&name, ticket)?;
            branch.insert_into_db(&conn)?;

            let _ = Command::new("git")
                .arg("checkout")
                .arg("-b")
                .arg(name)
                .status()?;
        }
    };

    conn.close()
        .map_err(|_| anyhow::anyhow!("Failed to close 'git-kit' connection"))?;

    Ok(())
}
