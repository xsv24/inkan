use crate::{
    adapters::prompt::Prompt,
    app_context::AppContext,
    domain::{
        adapters::{Git, Store},
        commands::checkout,
    },
};

use super::Arguments;

pub fn handler<G: Git, S: Store>(
    context: &AppContext<G, S>,
    args: Arguments,
) -> anyhow::Result<()> {
    let checkout = args.try_into_domain(Prompt)?;
    checkout::handler(&context.git, &context.store, checkout)?;

    Ok(())
}
