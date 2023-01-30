use clap::Args;

use crate::domain::{adapters::prompt::Prompter, commands::context::Context};

#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Issue ticket number related to the current branch.
    #[clap(value_parser)]
    pub ticket: Option<String>,

    /// Short describing a section of the codebase the changes relate to.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,

    /// Issue ticket number link.
    #[clap(short, long, value_parser)]
    pub link: Option<String>,
}

impl Arguments {
    pub fn try_into_domain<P: Prompter>(&self, _prompt: P) -> anyhow::Result<Context> {
        Ok(Context {
            ticket: self.ticket.clone(),
            scope: self.scope.clone(),
            link: self.link.clone(),
        })
    }
}
