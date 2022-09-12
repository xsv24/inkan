use crate::template::Template;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "git-kit")]
#[clap(bin_name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
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
    Checkout {
        /// Name of the branch to checkout or create.
        #[clap(value_parser)]
        name: String,

        /// Issue ticket number related to the branch.
        #[clap(short, long, value_parser)]
        ticket: Option<String>,
    },
}
