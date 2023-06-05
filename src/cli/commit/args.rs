use std::{collections::HashMap, fmt::Debug};

use clap::Args;

use crate::{
    cli::context,
    domain::{
        adapters::prompt::{Prompter, SelectItem},
        commands::commit::Commit,
        errors::UserInputError,
        models::Branch,
    },
    entry::Interactive,
    template_config::{Template, TemplateConfig},
};

#[derive(Debug, Args, PartialEq, Eq, Clone)]
#[group(skip)]
pub struct Arguments {
    /// Name of the commit template to be used.
    pub template: Option<String>,

    /// Message for the commit.
    #[clap(short, long, value_parser)]
    pub message: Option<String>,

    #[clap(flatten)]
    pub context: context::Arguments,
}

impl Arguments {
    pub fn try_into_domain<P: Prompter>(
        &self,
        config: &TemplateConfig,
        branch: &Option<Branch>,
        prompter: &P,
        interactive: &Interactive,
    ) -> Result<Commit, UserInputError> {
        let template = match &self.template {
            Some(template) => template.into(),
            None => Self::prompt_template_select(
                config.commit.templates.clone(),
                prompter,
                interactive,
            )?,
        };

        let context = self
            .context
            .try_into_domain(prompter, interactive, branch)?;

        Ok(Commit {
            template: config.get_template_config(&template)?.clone(),
            message: self.message.clone(),
            ticket: context.ticket,
            scope: context.scope,
            link: context.link,
        })
    }

    fn prompt_template_select<P: Prompter>(
        templates: HashMap<String, Template>,
        prompter: &P,
        interactive: &Interactive,
    ) -> Result<String, UserInputError> {
        if interactive == &Interactive::Disable {
            return Err(UserInputError::Required {
                name: "template".into(),
            });
        }

        let items = templates
            .into_iter()
            .map(|(name, template)| SelectItem {
                name: name.clone(),
                value: name,
                description: Some(template.description),
            })
            .collect::<Vec<_>>();

        let selected = prompter.select("Template", items)?;

        Ok(selected.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use fake::{Fake, Faker};

    use crate::{
        domain::{adapters::prompt::SelectItem, errors::UserInputError},
        template_config::CommitConfig,
    };

    #[test]
    fn try_into_domain_with_no_interactive_prompts() -> anyhow::Result<()> {
        let key = Faker.fake::<String>();
        let value = fake_template(&key);

        let args = Arguments {
            template: Some(key.clone()),
            ..fake_args()
        };

        let config = fake_template_config(Some((key.clone(), value.clone())));

        let prompt = PromptTest {
            select_item_name: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let actual =
            args.clone()
                .try_into_domain(&config, &None, &prompt, &Interactive::Disable)?;

        let expected = Commit {
            template: value,
            ticket: args.context.ticket.clone(),
            scope: args.context.scope.clone(),
            message: args.message.clone(),
            link: args.context.link,
        };

        assert_eq!(expected.template.content, actual.template.content);
        assert_eq!(expected.message, actual.message);
        assert_eq!(expected.scope, actual.scope);
        assert_eq!(expected.ticket, actual.ticket);
        assert_eq!(expected.link, actual.link);

        Ok(())
    }

    #[test]
    fn try_into_domain_with_interactive_prompt_is_used_if_none() -> anyhow::Result<()> {
        let key = Faker.fake::<String>();
        let value = fake_template(&key);

        let args = Arguments {
            template: None,
            message: None,
            context: context::Arguments {
                ticket: None,
                scope: None,
                link: None,
            },
        };

        let config = fake_template_config(Some((key.clone(), value.clone())));

        let text_prompt = Faker.fake::<Option<String>>();

        let prompt = PromptTest {
            select_item_name: Ok(key.clone()),
            text_result: Ok(text_prompt.clone()),
        };

        let actual = args
            .clone()
            .try_into_domain(&config, &None, &prompt, &Interactive::Enable)?;

        let expected = Commit {
            template: value,
            ticket: text_prompt.clone(),
            scope: text_prompt.clone(),
            message: text_prompt.clone(),
            link: text_prompt.clone(),
        };

        assert_eq!(expected.template.description, actual.template.description);
        assert_eq!(expected.scope, actual.scope);
        assert_eq!(expected.ticket, actual.ticket);
        assert_eq!(expected.link, actual.link);

        assert_eq!(args.message, actual.message);

        Ok(())
    }

    #[test]
    fn try_into_domain_with_interactive_prompt_is_not_used_if_value_is_already_provided(
    ) -> anyhow::Result<()> {
        let key = Faker.fake::<String>();
        let value = fake_template(&key);

        let args = Arguments {
            template: Some(key.clone()),
            message: Some(Faker.fake()),
            context: context::Arguments {
                ticket: Some(Faker.fake()),
                scope: Some(Faker.fake()),
                link: Some(Faker.fake()),
            },
            ..fake_args()
        };

        let config = fake_template_config(Some((key.clone(), value.clone())));

        let prompt = PromptTest {
            select_item_name: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let actual = args
            .clone()
            .try_into_domain(&config, &None, &prompt, &Interactive::Enable)?;

        let expected = Commit {
            template: value,
            ticket: args.context.ticket.clone(),
            scope: args.context.scope.clone(),
            message: args.message.clone(),
            link: args.context.link,
        };

        assert_eq!(expected.template.description, actual.template.description);
        assert_eq!(expected.message, actual.message);
        assert_eq!(expected.scope, actual.scope);
        assert_eq!(expected.ticket, actual.ticket);
        assert_eq!(expected.link, actual.link);

        Ok(())
    }

    #[test]
    fn try_into_domain_with_interactive_prompt_disabled_and_no_template_provided_an_error_is_thrown(
    ) {
        let args = Arguments {
            template: None,
            ..fake_args()
        };

        let config = fake_template_config(None);

        let prompt = PromptTest {
            select_item_name: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let error = args
            .clone()
            .try_into_domain(&config, &None, &prompt, &Interactive::Disable)
            .unwrap_err();

        assert_eq!(error.to_string(), "Missing required \"template\" input");
    }

    pub struct PromptTest {
        select_item_name: anyhow::Result<String>,
        text_result: anyhow::Result<Option<String>>,
    }

    impl Prompter for PromptTest {
        fn text(&self, name: &str, _: Option<String>) -> Result<Option<String>, UserInputError> {
            match &self.text_result {
                Ok(option) => Ok(option.clone()),
                Err(_) => Err(UserInputError::Validation {
                    name: name.into(),
                    message: "error".into(),
                }),
            }
        }

        fn select<T>(
            &self,
            name: &str,
            options: Vec<SelectItem<T>>,
        ) -> Result<SelectItem<T>, UserInputError> {
            match &self.select_item_name {
                Ok(name) => options
                    .into_iter()
                    .find(|i| i.name == name.clone())
                    .context("Failed to get item")
                    .map_err(|_| UserInputError::Validation {
                        name: name.into(),
                        message: "error".into(),
                    }),
                Err(_) => Err(UserInputError::Validation {
                    name: name.into(),
                    message: "error".into(),
                }),
            }
        }
    }

    fn fake_template(description: &str) -> Template {
        Template {
            description: description.into(),
            content: Faker.fake(),
        }
    }

    fn fake_template_config(selected: Option<(String, Template)>) -> TemplateConfig {
        let mut map = HashMap::from([
            ("option-1".into(), fake_template("option-1")),
            ("option-2".into(), fake_template("option-2")),
            ("option-3".into(), fake_template("option-3")),
        ]);

        if let Some((key, item)) = selected {
            map.insert(key, item);
        }

        let config = CommitConfig { templates: map };

        TemplateConfig {
            commit: config,
            version: 1,
            branch: None,
        }
    }

    fn fake_args() -> Arguments {
        Arguments {
            template: Faker.fake(),
            context: context::Arguments {
                ticket: Faker.fake(),
                scope: Faker.fake(),
                link: Faker.fake(),
            },
            message: Faker.fake(),
        }
    }
}
