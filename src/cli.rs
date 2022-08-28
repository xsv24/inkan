use crate::template::Template;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "git-kit")]
#[clap(about = "git cli containing templates & utilities.", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(subcommand)]
    Commit(Template),
    #[clap(subcommand)]
    Pr(Template),
}
