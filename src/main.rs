pub mod args;
pub mod branch;
pub mod cli;
pub mod template;

use branch::Branch;
use clap::Parser;
use rusqlite::Connection;
use std::{os::unix::process::CommandExt, process::Command};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let conn = Connection::open("db")?;

    match args.command {
        cli::Command::Commit(template) => {
            let args = template.args();
            let template = template.read_file()?;
            let contents = args.commit_message(template, &conn)?;

            dbg!("contents: {}", &contents);

            let _ = Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(contents)
                .arg("-e")
                .exec();
        }
        cli::Command::Checkout { name, ticket } => {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS branch (
                    name TEXT NOT NULL PRIMARY KEY,
                    ticket TEXT,
                    data BLOB,
                    created TEXT NOT NULL
                )",
                (),
            )?;

            // We want to store the branch name against and ticket number
            // So whenever we commit we get the ticket number from the branch
            let branch = Branch::new(&name, ticket)?;
            branch.insert_into_db(&conn)?;

            let _ = Command::new("git")
                .arg("checkout")
                .arg("-b")
                .arg(name)
                .exec();
        }
    };

    Ok(())
}
