use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        errors::{Errors, UserInputError},
        models::path::AbsolutePath,
    },
    utils::get_file_contents,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub commit: CommitConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommitConfig {
    pub templates: HashMap<String, Template>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Template {
    pub description: String,
    pub content: String,
}

impl TemplateConfig {
    pub fn new(config_path: &AbsolutePath) -> Result<Self, Errors> {
        let config_contents =
            get_file_contents(config_path).map_err(|e| Errors::Configuration {
                message: format!(
                    "Failed to read configuration at path '{}'",
                    config_path.to_string()
                ),
                source: e,
            })?;

        let config = serde_yaml::from_str::<TemplateConfig>(&config_contents).map_err(|e| {
            Errors::Configuration {
                message: format!(
                    "Failed to parse configuration from please ensure yaml is valid.\n{}",
                    config_path.to_string()
                ),
                source: e.into(),
            }
        })?;

        Ok(config)
    }

    pub fn get_template_config(&self, name: &str) -> Result<&Template, UserInputError> {
        log::info!("fetching template {}", name);
        let template = self
            .commit
            .templates
            .get(name)
            .ok_or_else(|| UserInputError::InvalidCommand { name: name.into() })?;

        Ok(template)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::errors::UserInputError,
        template_config::{CommitConfig, Template, TemplateConfig},
    };
    use fake::{Fake, Faker};
    use std::collections::HashMap;

    #[test]
    fn get_template_config_by_name_key() {
        let key: String = Faker.fake();

        let config = TemplateConfig {
            commit: CommitConfig {
                templates: HashMap::from([(
                    key.clone(),
                    Template {
                        description: key.clone(),
                        content: key.clone(),
                    },
                )]),
            },
        };

        let template_config = config.get_template_config(&key).unwrap();

        assert_eq!(key, template_config.description);
    }

    #[test]
    fn get_template_config_errors_on_non_existent_key() {
        let key: String = Faker.fake();

        let config = TemplateConfig {
            commit: CommitConfig {
                templates: HashMap::from([]),
            },
        };

        let result = config.get_template_config(&key).unwrap_err();
        assert!(matches!(
            result,
            UserInputError::InvalidCommand { name } if name == key,
        ));
    }
}
