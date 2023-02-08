use clap::Args;
use std::result::Result::Ok;

use crate::{
    domain::{adapters::prompt::Prompter, commands::checkout::Checkout},
    entry::Interactive,
    utils::or_else_try::OrElseTry,
};

#[derive(Debug, Args, Clone)]
pub struct Arguments {
    /// Name of the branch to checkout or create.
    #[clap(value_parser)]
    pub name: String,

    /// Issue ticket number related to the branch.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    /// Short describing a section of the codebase the changes relate to.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,

    /// Issue ticket number link.
    #[clap(short, long, value_parser)]
    pub link: Option<String>,
}

impl Arguments {
    pub fn try_into_domain<P: Prompter>(
        &self,
        prompt: P,
        interactive: &Interactive,
    ) -> anyhow::Result<Checkout> {
        let domain = match interactive {
            Interactive::Enable => Checkout {
                name: self.name.clone(),
                ticket: self.ticket.clone().or_else_try(|| prompt.text("Ticket:"))?,
                scope: self.scope.clone().or_else_try(|| prompt.text("Scope:"))?,
                link: self.link.clone().or_else_try(|| prompt.text("Link:"))?,
            },
            Interactive::Disable => Checkout {
                name: self.name.clone(),
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
    use anyhow::Context;
    use fake::{Fake, Faker};

    use crate::domain::adapters::prompt::SelectItem;

    #[test]
    fn try_into_domain_with_no_interactive_prompts() -> anyhow::Result<()> {
        let args = fake_args();

        let prompt = PromptTest {
            select_index: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let actual = args
            .clone()
            .try_into_domain(prompt, &Interactive::Disable)?;

        let expected = Checkout {
            name: args.name.clone(),
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

        let actual = args.clone().try_into_domain(prompt, &Interactive::Enable)?;

        let expected = Checkout {
            name: args.name.clone(),
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
            ..fake_args()
        };

        let prompt = PromptTest {
            select_index: Err(anyhow::anyhow!("select should not be called")),
            text_result: Err(anyhow::anyhow!("text should not be called")),
        };

        let actual = args.clone().try_into_domain(prompt, &Interactive::Enable)?;

        let expected = Checkout {
            name: args.name.clone(),
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
        fn text(&self, _: &str) -> anyhow::Result<Option<String>> {
            match &self.text_result {
                Ok(option) => Ok(option.clone()),
                Err(_) => Err(anyhow::anyhow!("Text prompt failed")),
            }
        }

        fn select<T>(&self, _: &str, options: Vec<SelectItem<T>>) -> anyhow::Result<SelectItem<T>> {
            match &self.select_index {
                Ok(index) => Ok(options
                    .into_iter()
                    .nth(index.clone())
                    .context("Failed to get item")?),
                Err(_) => Err(anyhow::anyhow!("Select prompt failed")),
            }
        }
    }

    fn fake_args() -> Arguments {
        Arguments {
            name: Faker.fake(),
            ticket: Faker.fake(),
            scope: Faker.fake(),
            link: Faker.fake(),
        }
    }
}
