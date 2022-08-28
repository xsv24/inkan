pub mod cli;
pub mod template;

use anyhow::Context;
use clap::Parser;
use std::fs;
use std::{os::unix::process::CommandExt, process::Command};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    match args.command {
        cli::Command::Commit(template) => {
            let args = template.args();
            println!("{:?}", args);

            let template = format!("templates/commit/{}", template.file_name());

            let contents: String = fs::read_to_string(&template)
                .with_context(|| format!("Failed to read template '{}'", template))?
                .parse()?;

            let contents = match &args.ticket {
                Some(num) => contents.replace("{ticket_num}", &format!("[{}]", num)),
                None => contents.replace("{ticket_num}", ""),
            };

            let contents = match &args.message {
                Some(message) => contents.replace("{message}", &message),
                None => contents.replace("{message}", ""),
            };

            println!("contents: {}", contents);

            let _ = Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(contents)
                .arg("-e")
                .exec();
        }
        cli::Command::Pr(template) => {
            let template = format!("templates/pr/{}", template.file_name());
        }
    };

    Ok(())
}
