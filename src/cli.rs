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
    #[clap(subcommand)]
    Commit(Template),
    Checkout {
        #[clap(value_parser)]
        name: String,

        #[clap(short, long, value_parser)]
        ticket: Option<String>,
    },
}
