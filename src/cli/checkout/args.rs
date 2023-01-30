use clap::Args;

use crate::domain::{adapters::prompt::Prompter, commands::checkout::Checkout};

#[derive(Debug, Args, Clone)]
pub struct Arguments {
    /// Name of the branch to checkout or create.
    #[clap(value_parser)]
    pub name: String,

    /// Issue ticket number related to the branch.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    /// Short describing a section of the codebase the changes relate to.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,

    /// Issue ticket number link.
    #[clap(short, long, value_parser)]
    pub link: Option<String>,
}

impl Arguments {
    pub fn try_into_domain<P: Prompter>(&self, _prompt: P) -> anyhow::Result<Checkout> {
        Ok(Checkout {
            name: self.name.clone(),
            ticket: self.ticket.clone(),
            scope: self.scope.clone(),
            link: self.link.clone(),
        })
    }
}
