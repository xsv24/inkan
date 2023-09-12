use clap::Subcommand;

use crate::{
    app_context::AppContext,
    domain::{
        adapters::{prompt::Prompter, Git, Store},
        errors::Errors,
    },
};

use super::{checkout, commit, context, template};

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Commit staged changes via git with a template message.
    Commit(commit::Arguments),
    /// Checkout an existing branch or create a new branch and add a ticket number as context for future commits.
    Checkout(checkout::Arguments),
    /// Add or update the ticket number related to the current branch.
    Context(context::Arguments),
    /// Get or Set active template.
    #[clap(subcommand)]
    Template(template::SubCommands),
}

impl Commands {
    pub fn execute<G: Git, S: Store, P: Prompter>(
        self,
        context: &mut AppContext<G, S>,
        prompt: P,
    ) -> Result<(), Errors> {
        match self {
            Commands::Checkout(args) => checkout::handler(context, args, prompt),
            Commands::Context(args) => context::handler(context, args, prompt),
            Commands::Commit(args) => commit::handler(context, args, prompt),
            Commands::Template(args) => template::handler(
                &mut context.store,
                &context.config,
                args,
                prompt,
                &context.interactive,
            ),
        }
    }
}
