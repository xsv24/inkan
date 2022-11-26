use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use directories::ProjectDirs;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{
    domain::models::{Config, ConfigKey},
    utils::get_file_contents,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub commit: CommitConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommitConfig {
    pub templates: HashMap<String, TemplateConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub description: String,
    pub content: String,
}

impl AppConfig {
    pub fn new(user_config_path: Config, git_root_path: PathBuf) -> anyhow::Result<Self> {
        let config_path = Self::get_config_path(user_config_path, git_root_path)?;

        let config_contents = get_file_contents(&config_path)?;
        let config = serde_yaml::from_str::<AppConfig>(&config_contents)
            .context("Failed to load 'config.yml' from please ensure yaml is valid.")?;

        Ok(config)
    }

    fn config_dir() -> anyhow::Result<PathBuf> {
        let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        Ok(project_dir.config_dir().to_owned())
    }

    pub fn db_connection() -> anyhow::Result<Connection> {
        let db_file = Self::config_dir()?.join("db");

        let connection = Connection::open(db_file).context("Failed to open sqlite connection")?;

        Ok(connection)
    }

    pub fn validate_template(&self, name: &str) -> clap::error::Result<()> {
        log::info!("validating template {}", name);

        if self.commit.templates.contains_key(name) {
            log::info!("template {} 👌", name);
            Ok(())
        } else {
            // TODO: want a nice error message that shows the templates output
            Err(clap::Error::raw(
                clap::error::ErrorKind::InvalidSubcommand,
                format!("Found invalid subcommand '{}' given", name),
            ))?
        }
    }

    pub fn get_template_config(&self, name: &str) -> clap::error::Result<&TemplateConfig> {
        log::info!("fetching template {}", name);
        let template = self.commit.templates.get(name).ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::MissingSubcommand,
                format!("Found missing subcommand '{}'", name),
            )
        })?;

        Ok(template)
    }

    fn get_config_path(config: Config, repo_config: PathBuf) -> anyhow::Result<PathBuf> {
        let filename = ".git-kit.yml";
        let repo_config = repo_config.join(filename);

        match (config.key, repo_config.exists()) {
            // Once off override takes priority 1
            (ConfigKey::Once, _) => {
                log::info!("⏳ Loading once off config...");
                Ok(config.path)
            }
            // Repository has config file priority 2
            (_, true) => {
                log::info!("⏳ Loading local repo config...");
                Ok(repo_config)
            }
            // User has set custom config file and is active priority 3
            (ConfigKey::User(key), _) => {
                log::info!("⏳ Loading user '{:?}' config...", key);
                Ok(config.path)
            }
            // No set user config use provided defaults priority 4
            (ConfigKey::Default, _) => {
                log::info!("⏳ Loading global config...");
                Ok(config.path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use crate::{
        adapters::Git,
        domain::{adapters, models::ConfigStatus},
    };

    use super::*;

    #[test]
    fn once_off_config_has_priority_1() -> anyhow::Result<()> {
        let git: &dyn adapters::Git = &Git;

        let once_path = fake_path_buf();
        let valid_repo_dir = git.root_directory()?;

        let config_path = AppConfig::get_config_path(
            Config {
                key: ConfigKey::Once,
                path: once_path.clone(),
                status: ConfigStatus::ACTIVE,
            },
            valid_repo_dir,
        )?;

        assert_eq!(once_path, config_path);

        Ok(())
    }

    #[test]
    fn repo_dir_with_config_file_used_over_user_and_default_has_priority_2() -> anyhow::Result<()> {
        let git: &dyn adapters::Git = &Git;
        let repo_root_with_config = git.root_directory()?;

        for key in [ConfigKey::Default, ConfigKey::User(Faker.fake())] {
            let config_dir = AppConfig::get_config_path(
                Config {
                    key,
                    path: fake_path_buf(),
                    status: ConfigStatus::ACTIVE,
                },
                repo_root_with_config.clone(),
            )?;

            assert_eq!(config_dir, repo_root_with_config.join(".git-kit.yml"));
        }
        Ok(())
    }

    #[test]
    fn user_sets_config_file_and_no_config_or_once_off_config_priority_3() -> anyhow::Result<()> {
        let user_path = fake_path_buf();
        let repo_non_existing = fake_path_buf();

        let config_dir = AppConfig::get_config_path(
            Config {
                key: ConfigKey::User(Faker.fake()),
                path: user_path.clone(),
                status: ConfigStatus::ACTIVE,
            },
            repo_non_existing,
        )?;

        assert_eq!(user_path, config_dir);

        Ok(())
    }

    #[test]
    fn no_user_path_or_valid_repo_dir_defaults_priority_4() -> anyhow::Result<()> {
        let default_path = fake_path_buf();
        let repo_non_existing = fake_path_buf();

        let config_dir = AppConfig::get_config_path(
            Config {
                key: ConfigKey::Default,
                path: default_path.clone(),
                status: ConfigStatus::ACTIVE,
            },
            repo_non_existing,
        )?;

        assert_eq!(default_path, config_dir);
        Ok(())
    }

    fn fake_path_buf() -> PathBuf {
        PathBuf::from(Faker.fake::<String>())
    }
}
