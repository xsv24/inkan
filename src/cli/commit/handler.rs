use crate::{
    app_context::AppContext,
    domain::{
        adapters::{prompt::Prompter, Git, Store},
        commands::commit,
        errors::Errors,
        models::Branch,
    },
    entry::Interactive,
    template_config::TemplateConfig,
};

use super::Arguments;

pub fn handler<G: Git, S: Store, P: Prompter>(
    context: &AppContext<G, S>,
    args: Arguments,
    prompter: P,
) -> Result<(), Errors> {
    let branch = match context.interactive {
        Interactive::Enable => get_branch(context)?,
        Interactive::Disable => None,
    };

    let templates = TemplateConfig::new(&context.config.path)?;
    let commit = args
        .try_into_domain(&templates, &branch, &prompter, &context.interactive)
        .map_err(Errors::UserInput)?;

    commit::handler(&context.git, branch, commit)?;

    Ok(())
}

fn get_branch<G: Git, S: Store>(context: &AppContext<G, S>) -> Result<Option<Branch>, Errors> {
    let branch_name = context.git.branch_name().map_err(Errors::Git)?;
    let repo_name = context.git.repository_name().map_err(Errors::Git)?;
    let branch = context.store.get_branch(&branch_name, &repo_name).ok();

    Ok(branch)
}
