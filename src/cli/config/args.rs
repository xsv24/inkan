use clap::Args;

use crate::{
    domain::{
        adapters::{
            prompt::{Prompter, SelectItem},
            Store,
        },
        models::{Config, ConfigKey, ConfigStatus},
    },
    entry::Interactive,
};

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Arguments {
    /// Add / register a custom config file.
    Add(ConfigAdd),
    /// Switch to another config file.
    Set(ConfigSet),
    /// Display the current config in use.
    Show,
    /// Reset to the default config.
    Reset,
}

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct ConfigAdd {
    /// Name used to reference the config file.
    pub name: String,
    /// File path to the config file.
    pub path: String,
}

impl ConfigAdd {
    pub fn try_into_domain(self) -> anyhow::Result<Config> {
        Config::new(self.name.into(), self.path, ConfigStatus::Active)
    }
}

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct ConfigSet {
    /// Name used to reference the config file.
    name: Option<String>,
}

impl ConfigSet {
    pub fn try_into_domain<S: Store, P: Prompter>(
        self,
        store: &S,
        prompt: P,
        interactive: &Interactive,
    ) -> anyhow::Result<ConfigKey> {
        Ok(match self.name {
            Some(name) => name.into(),
            None => prompt_configuration_select(
                store.get_configurations()?,
                prompt,
                interactive.to_owned(),
            )?,
        })
    }
}

fn prompt_configuration_select<P: Prompter>(
    configurations: Vec<Config>,
    selector: P,
    interactive: Interactive,
) -> anyhow::Result<ConfigKey> {
    if interactive == Interactive::Disable {
        anyhow::bail!(clap::Error::raw(
            clap::ErrorKind::MissingRequiredArgument,
            "'name' is required"
        ))
    }

    let configurations: Vec<SelectItem<ConfigKey>> = configurations
        .iter()
        .map(|config| SelectItem {
            name: config.key.clone().into(),
            value: config.key.clone(),
            description: None,
        })
        .collect();

    let selected = selector.select("Configuration:", configurations)?;

    Ok(selected.value)
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use fake::{Fake, Faker};
    use std::path::PathBuf;

    use super::*;
    use crate::domain::adapters::prompt::{Prompter, SelectItem};

    #[test]
    fn with_interactive_enabled_select_prompt_is_used() {
        // Arrange
        let config = fake_config();
        let selector = PromptTest {
            select_item_name: Ok(config.key.clone().into()),
        };
        let configurations = vec![fake_config(), config.clone(), fake_config()];

        // Act
        let selected =
            prompt_configuration_select(configurations, selector, Interactive::Enable).unwrap();

        // Assert
        assert_eq!(config.key, selected);
    }

    #[test]
    fn with_interactive_disabled_select_prompt_errors() {
        // Arrange
        let config = fake_config();
        let selector = PromptTest {
            select_item_name: Ok(config.key.clone().into()),
        };
        let configurations = vec![fake_config(), config.clone(), fake_config()];

        // Act
        let error = prompt_configuration_select(configurations, selector, Interactive::Disable)
            .unwrap_err();

        // Assert
        assert_eq!(error.to_string(), "error: 'name' is required");
    }

    pub fn fake_config() -> Config {
        Config {
            key: ConfigKey::User(Faker.fake()),
            path: PathBuf::new(),
            status: ConfigStatus::Active,
        }
    }

    pub struct PromptTest {
        select_item_name: anyhow::Result<String>,
    }

    impl Prompter for PromptTest {
        fn text(&self, _: &str, _: Option<String>) -> anyhow::Result<Option<String>> {
            Err(anyhow::anyhow!("Text prompt should not be invoked"))
        }

        fn select<T>(&self, _: &str, options: Vec<SelectItem<T>>) -> anyhow::Result<SelectItem<T>> {
            match &self.select_item_name {
                Ok(name) => Ok(options
                    .into_iter()
                    .find(|i| i.name == name.clone())
                    .context("Failed to get item")?),
                Err(_) => Err(anyhow::anyhow!("Select prompt failed")),
            }
        }
    }
}
