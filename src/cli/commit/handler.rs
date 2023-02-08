use crate::{
    app_context::AppContext,
    domain::{
        adapters::{prompt::Prompter, Git, Store},
        commands::commit,
    },
    template_config::TemplateConfig,
};

use super::Arguments;

pub fn handler<G: Git, S: Store, P: Prompter>(
    context: &AppContext<G, S>,
    args: Arguments,
    prompter: P,
) -> anyhow::Result<()> {
    let templates = TemplateConfig::new(&context.config.path)?;
    let commit = args.try_into_domain(&templates, prompter, &context.interactive)?;
    commit::handler(&context.git, &context.store, commit)?;

    Ok(())
}
