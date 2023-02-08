use crate::{
    app_context::AppContext,
    domain::{
        adapters::{prompt::Prompter, Git, Store},
        commands::context,
    },
};

use super::Arguments;

pub fn handler<G: Git, S: Store, P: Prompter>(
    context: &AppContext<G, S>,
    args: Arguments,
    prompt: P,
) -> anyhow::Result<()> {
    context::handler(
        &context.git,
        &context.store,
        args.try_into_domain(prompt, &context.interactive)?,
    )?;

    Ok(())
}
