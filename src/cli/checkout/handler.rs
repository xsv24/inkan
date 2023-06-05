use crate::{
    app_context::AppContext,
    domain::{
        adapters::{prompt::Prompter, Git, Store},
        commands::checkout,
        errors::Errors,
    },
    template_config::TemplateConfig,
};

use super::Arguments;

pub fn handler<G: Git, S: Store, P: Prompter>(
    context: &AppContext<G, S>,
    args: Arguments,
    prompt: P,
) -> Result<(), Errors> {
    let checkout = args
        .try_into_domain(prompt, &context.interactive)
        .map_err(Errors::UserInput)?;

    let template = TemplateConfig::new(&context.config.path)?;

    checkout::handler(&context.git, &context.store, template, checkout)?;

    Ok(())
}
