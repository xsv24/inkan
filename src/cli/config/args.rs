use clap::Args;

use crate::{
    domain::{
        adapters::{
            prompt::{Prompter, SelectItem},
            Store,
        },
        errors::{Errors, UserInputError},
        models::{Config, ConfigKey},
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
    ) -> Result<ConfigKey, Errors> {
        Ok(match self.name {
            Some(name) => name.as_str().into(),
            None => prompt_configuration_select(
                store.get_configurations().map_err(Errors::PersistError)?,
                prompt,
                interactive.to_owned(),
            )
            .map_err(Errors::UserInput)?,
        })
    }
}

fn prompt_configuration_select<P: Prompter>(
    configurations: Vec<Config>,
    selector: P,
    interactive: Interactive,
) -> Result<ConfigKey, UserInputError> {
    if interactive == Interactive::Disable {
        return Err(UserInputError::Required {
            name: "name".into(),
        });
    }

    let configurations: Vec<SelectItem<ConfigKey>> = configurations
        .iter()
        .map(|config| SelectItem {
            name: config.key.clone().into(),
            value: config.key.clone(),
            description: None,
        })
        .collect();

    let selected = selector.select("Configuration", configurations)?;

    Ok(selected.value)
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use fake::{Fake, Faker};
    use std::path::PathBuf;

    use super::*;
    use crate::domain::{
        adapters::prompt::{Prompter, SelectItem},
        errors::UserInputError,
        models::{
            path::{AbsolutePath, PathType},
            ConfigStatus,
        },
    };

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
        assert_eq!(error.to_string(), "Missing required \"name\" input");
    }

    fn valid_dir_path() -> AbsolutePath {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .try_into()
            .unwrap()
    }

    fn valid_file_path() -> AbsolutePath {
        let path = valid_dir_path();
        path.join("templates/default.yml", PathType::File).unwrap()
    }

    fn fake_config() -> Config {
        Config {
            key: ConfigKey::User(Faker.fake()),
            path: valid_file_path(),
            status: ConfigStatus::Active,
        }
    }

    struct PromptTest {
        select_item_name: anyhow::Result<String>,
    }

    impl Prompter for PromptTest {
        fn text(&self, name: &str, _: Option<String>) -> Result<Option<String>, UserInputError> {
            Err(UserInputError::Validation {
                name: name.into(),
                message: "error occurred in mock".into(),
            })
        }

        fn select<T>(
            &self,
            name: &str,
            options: Vec<SelectItem<T>>,
        ) -> Result<SelectItem<T>, UserInputError> {
            match &self.select_item_name {
                Ok(name) => Ok(options
                    .into_iter()
                    .find(|i| i.name == name.clone())
                    .context("Failed to get item")
                    .map_err(|_| UserInputError::Validation {
                        name: name.into(),
                        message: "Failed to get item".into(),
                    })?),
                Err(_) => Err(UserInputError::Validation {
                    name: name.into(),
                    message: "error occured in mock".into(),
                }),
            }
        }
    }
}
