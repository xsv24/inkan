use std::{collections::HashMap, fmt::Debug};

use clap::Args;

use crate::{
    domain::{
        adapters::prompt::{Prompter, SelectItem},
        commands::commit::Commit,
    },
    template_config::{Template, TemplateConfig},
};

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct Arguments {
    /// Name of the commit template to be used.
    pub template: Option<String>,

    /// Issue ticket number related to the commit.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    /// Message for the commit.
    #[clap(short, long, value_parser)]
    pub message: Option<String>,

    /// Short describing a section of the codebase the changes relate to.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,
}

impl Arguments {
    pub fn try_into_domain<P: Prompter>(
        &self,
        config: &TemplateConfig,
        prompter: P,
    ) -> anyhow::Result<Commit> {
        let template = match &self.template {
            Some(template) => template.into(),
            None => Self::prompt_template_select(config.commit.templates.clone(), prompter)?,
        };

        // TODO: Could we do a prompt if no ticket / args found ?
        Ok(Commit {
            template: config.get_template_config(&template)?.clone(),
            ticket: self.ticket.clone(),
            message: self.message.clone(),
            scope: self.scope.clone(),
        })
    }

    fn prompt_template_select<P: Prompter>(
        templates: HashMap<String, Template>,
        prompter: P,
    ) -> anyhow::Result<String> {
        let items = templates
            .into_iter()
            .map(|(name, template)| SelectItem {
                name: name.clone(),
                value: name,
                description: Some(template.description),
            })
            .collect::<Vec<_>>();

        let selected = prompter.select("Template:", items)?;

        Ok(selected.name)
    }
}
