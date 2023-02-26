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
    let branch = context
        .store
        .get_branch(&context.git.branch_name()?, &context.git.repository_name()?)
        .ok();

    let args = args.try_into_domain(prompt, &context.interactive, branch)?;

    context::handler(&context.git, &context.store, args)?;

    Ok(())
}
