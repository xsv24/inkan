use crate::{
    app_context::AppContext,
    domain::{
        adapters::{prompt::Prompter, Git, Store},
        commands::checkout,
    },
};

use super::Arguments;

pub fn handler<G: Git, S: Store, P: Prompter>(
    context: &AppContext<G, S>,
    args: Arguments,
    prompt: P,
) -> anyhow::Result<()> {
    let checkout = args.try_into_domain(prompt, &context.interactive)?;
    checkout::handler(&context.git, &context.store, checkout)?;

    Ok(())
}
