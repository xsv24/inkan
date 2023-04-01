use clap::Args;

use crate::{
    domain::{
        adapters::prompt::Prompter, commands::context::Context, errors::UserInputError,
        models::Branch,
    },
    entry::Interactive,
    utils::or_else_try::OrElseTry,
};

#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Issue ticket number related to the current branch.
    #[clap(value_parser)]
    pub ticket: Option<String>,

    /// Short describing a section of the codebase the changes relate to.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,

    /// Issue ticket number link.
    #[clap(short, long, value_parser)]
    pub link: Option<String>,
}

impl Arguments {
    pub fn try_prompt_with_defaults<P: Prompter>(
        &self,
        branch: Option<Branch>,
        prompt: P,
    ) -> Result<Context, UserInputError> {
        let ticket = self
            .ticket
            .clone()
            .or_else_try(|| prompt.text("Ticket:", branch.clone().map(|b| b.ticket)))?;

        let scope = self
            .scope
            .clone()
            .or_else_try(|| prompt.text("Scope:", branch.clone().and_then(|b| b.scope)))?;

        let link = self
            .link
            .clone()
            .or_else_try(|| prompt.text("Link:", branch.and_then(|b| b.link)))?;

        Ok(Context {
            ticket,
            scope,
            link,
        })
    }

    pub fn try_into_domain<P: Prompter>(
        &self,
        prompt: P,
        interactive: &Interactive,
        branch: Option<Branch>,
    ) -> Result<Context, UserInputError> {
        let domain = match interactive {
            Interactive::Enable => self.try_prompt_with_defaults(branch, prompt)?,
            Interactive::Disable => Context {
                ticket: self.ticket.clone(),
                scope: self.scope.clone(),
                link: self.link.clone(),
            },
        };

        Ok(domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context as _;
    use fake::{Fake, Faker};

    use crate::domain::{
        adapters::prompt::SelectItem, commands::context::Context, errors::UserInputError,
    };

    #[test]
    fn try_into_domain_with_no_interactive_prompts() -> anyhow::Result<()> {
        let args = fake_args();

        let prompt = PromptTest {
            select_index: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let actual = args
            .clone()
            .try_into_domain(prompt, &Interactive::Disable, None)?;

        let expected = Context {
            ticket: args.ticket.clone(),
            scope: args.scope.clone(),
            link: args.link.clone(),
        };

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn try_into_domain_with_interactive_prompt_is_used_if_none() -> anyhow::Result<()> {
        let args = Arguments {
            ticket: None,
            scope: None,
            link: None,
            ..fake_args()
        };

        let text_prompt = Faker.fake::<Option<String>>();

        let prompt = PromptTest {
            select_index: Err(anyhow::anyhow!("select should not be called")),
            text_result: Ok(text_prompt.clone()),
        };

        let actual = args
            .clone()
            .try_into_domain(prompt, &Interactive::Enable, None)?;

        let expected = Context {
            ticket: text_prompt.clone(),
            scope: text_prompt.clone(),
            link: text_prompt.clone(),
        };

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn try_into_domain_with_interactive_prompt_is_not_used_if_value_is_already_provided(
    ) -> anyhow::Result<()> {
        let args = Arguments {
            ticket: Some(Faker.fake()),
            scope: Some(Faker.fake()),
            link: Some(Faker.fake()),
        };

        let prompt = PromptTest {
            select_index: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let actual = args
            .clone()
            .try_into_domain(prompt, &Interactive::Enable, None)?;

        let expected = Context {
            ticket: args.ticket.clone(),
            scope: args.scope.clone(),
            link: args.link.clone(),
        };

        assert_eq!(expected, actual);

        Ok(())
    }

    pub struct PromptTest {
        select_index: anyhow::Result<usize>,
        text_result: anyhow::Result<Option<String>>,
    }

    impl Prompter for PromptTest {
        fn text(&self, name: &str, _: Option<String>) -> Result<Option<String>, UserInputError> {
            match &self.text_result {
                Ok(option) => Ok(option.clone()),
                Err(_) => Err(UserInputError::Validation {
                    name: name.into(),
                    message: "An error occurred within the mock prompter".into(),
                }),
            }
        }

        fn select<T>(
            &self,
            name: &str,
            options: Vec<SelectItem<T>>,
        ) -> Result<SelectItem<T>, UserInputError> {
            match &self.select_index {
                Ok(index) => options
                    .into_iter()
                    .nth(index.clone())
                    .context("Failed to get item")
                    .map_err(|_| UserInputError::Validation {
                        name: name.into(),
                        message: "An error occurred within the mock prompter".into(),
                    }),
                Err(_) => Err(UserInputError::Validation {
                    name: name.into(),
                    message: "An error occurred within the mock prompter".into(),
                }),
            }
        }
    }

    fn fake_args() -> Arguments {
        Arguments {
            ticket: Faker.fake(),
            scope: Faker.fake(),
            link: Faker.fake(),
        }
    }
}
