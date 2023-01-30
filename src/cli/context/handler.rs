use crate::{
    adapters::prompt::Prompt,
    app_context::AppContext,
    domain::{
        adapters::{Git, Store},
        commands::context,
    },
};

use super::Arguments;

pub fn handler<G: Git, S: Store>(
    context: &AppContext<G, S>,
    args: Arguments,
) -> anyhow::Result<()> {
    context::handler(&context.git, &context.store, args.try_into_domain(Prompt)?)?;

    Ok(())
}
