use crate::{
    app_context::AppContext,
    domain::{
        adapters::{prompt::Prompter, Git, Store},
        commands::context,
        errors::Errors,
    },
};

use super::Arguments;

pub fn handler<G: Git, S: Store, P: Prompter>(
    context: &AppContext<G, S>,
    args: Arguments,
    prompt: P,
) -> Result<(), Errors> {
    let repo_name = context.git.repository_name().map_err(Errors::Git)?;
    let branch_name = context.git.branch_name().map_err(Errors::Git)?;

    let branch = context.store.get_branch(&branch_name, &repo_name).ok();

    let args = args
        .try_into_domain(prompt, &context.interactive, branch)
        .map_err(Errors::UserInput)?;

    context::handler(&context.git, &context.store, args)?;

    Ok(())
}
