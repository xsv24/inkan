use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::utils::get_file_contents;

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
    pub fn new(config_path: &PathBuf) -> anyhow::Result<Self> {
        let config_contents = get_file_contents(config_path)?;
        let config = serde_yaml::from_str::<TemplateConfig>(&config_contents)
            .context("Failed to load 'config.yml' from please ensure yaml is valid.")?;

        Ok(config)
    }

    pub fn get_template_config(&self, name: &str) -> clap::error::Result<&Template> {
        log::info!("fetching template {}", name);
        let template = self.commit.templates.get(name).ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::MissingSubcommand,
                format!("Found invalid subcommand '{name}' given"),
            )
        })?;

        Ok(template)
    }
}

#[cfg(test)]
mod tests {
    use crate::template_config::{CommitConfig, Template, TemplateConfig};
    use fake::{Fake, Faker};
    use std::collections::HashMap;

    #[test]
    fn get_template_config_by_name_key() -> anyhow::Result<()> {
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

        let template_config = config.get_template_config(&key)?;
        assert_eq!(key, template_config.description);

        Ok(())
    }
}
