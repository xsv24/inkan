use crate::template::Template;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "git-kit")]
#[clap(bin_name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
#[clap(version)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Commit staged changes via git with a template message.
    #[clap(subcommand)]
    Commit(Template),
    /// Checkout an existing branch or create a new branch and add a ticket number as context for future commits.
    Checkout(Checkout),
    /// Add or update the ticket number related to the current branch.
    Context(Current),
}

#[derive(Debug, Args, Clone)]
pub struct Current {
    /// Issue ticket number related to the current branch.
    #[clap(value_parser)]
    pub ticket: String,
}

#[derive(Debug, Args, Clone)]
pub struct Checkout {
    /// Name of the branch to checkout or create.
    #[clap(value_parser)]
    pub name: String,

    /// Issue ticket number related to the branch.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,
}
