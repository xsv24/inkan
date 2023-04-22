use std::path::PathBuf;

use anyhow::Context;
use directories::ProjectDirs;
use rusqlite::Connection;

use crate::{
    adapters::{sqlite::Sqlite, Git},
    domain::{
        adapters::{Git as _, GitSystem, Store},
        errors::{Errors, UserInputError},
        models::{
            path::{AbsolutePath, PathType},
            Config, ConfigKey, ConfigStatus,
        },
    },
};

pub struct AppConfig {
    pub config: Config,
}

impl AppConfig {
    pub fn new<S: GitSystem>(
        once_off_config_path: Option<String>,
        git: &Git<S>,
        store: &Sqlite,
    ) -> Result<AppConfig, Errors> {
        let config = match once_off_config_path {
            Some(path) => Ok(Config {
                key: ConfigKey::Once,
                status: ConfigStatus::Active,
                path: TryInto::<AbsolutePath>::try_into(path).map_err(|e| {
                    Errors::UserInput(UserInputError::Validation {
                        name: "config".into(),
                        message: e.to_string(),
                    })
                })?,
            }),
            None => store
                .get_configuration(None)
                .map_err(|e| Errors::Configuration {
                    message: "Failed to get current 'active' config".into(),
                    source: e.into(),
                }),
        }?;

        let git_root_dir = git.root_directory().map_err(Errors::Git)?;
        let config = Self::map_config_overrides(config, git_root_dir)?;

        Ok(AppConfig { config })
    }

    pub fn db_connection() -> anyhow::Result<Connection> {
        let db_file = Self::template_config_dir()?.join("db");

        let connection = Connection::open(db_file).context("Failed to open sqlite connection")?;

        Ok(connection)
    }

    fn template_config_dir() -> anyhow::Result<PathBuf> {
        let project_dir = ProjectDirs::from("dev", "xsv24", "inkan")
            .context("Failed to retrieve 'inkan' config")?;

        Ok(project_dir.config_dir().to_owned())
    }

    pub fn join_config_filename(repo_root_dir: &AbsolutePath) -> Result<AbsolutePath, Errors> {
        repo_root_dir
            .join(".inkan.yml", PathType::File)
            .map_err(|e| Errors::Configuration {
                message: "Failed to load repositories local '.inkan.yml'".into(),
                source: e.into(),
            })
    }

    fn map_config_overrides(config: Config, repo_root_dir: AbsolutePath) -> Result<Config, Errors> {
        let local_config = AppConfig::join_config_filename(&repo_root_dir);

        match (config.key.clone(), &local_config) {
            // Once off override takes priority 1
            (ConfigKey::Once, Err(_)) => {
                log::info!("⏳ Loading once off config...");
                Ok(config)
            }
            // Repository has config file priority 2
            (ConfigKey::Local, _) | (_, Ok(_)) => {
                log::info!("⏳ Loading local repo config...");
                Ok(Config {
                    key: ConfigKey::Local,
                    path: local_config?,
                    status: config.status,
                })
            }
            // User has set custom config file and is active priority 3
            (ConfigKey::User(key), Err(_)) => {
                log::info!("⏳ Loading user '{:?}' config...", key);
                Ok(config)
            }
            // No set user config use provided defaults priority 4
            (ConfigKey::Default, Err(_)) => {
                log::info!("⏳ Loading 'default' config...");
                Ok(config)
            }
            (ConfigKey::Conventional, Err(_)) => {
                log::info!("⏳ Loading 'conventional' config...");
                Ok(config)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use crate::{
        adapters::GitCommand,
        domain::{adapters, models::ConfigStatus},
    };

    use super::*;

    #[test]
    fn once_off_config_has_priority_1() {
        let git: &dyn adapters::Git = &Git { git: GitCommand };

        let once_path = abs_repo_directory();
        let valid_repo_dir = git.root_directory().unwrap();

        let config = AppConfig::map_config_overrides(
            Config {
                key: ConfigKey::Once,
                path: once_path.clone(),
                status: ConfigStatus::Active,
            },
            valid_repo_dir,
        )
        .unwrap();

        assert_eq!(once_path, config.path);
        assert_eq!(ConfigKey::Once, config.key);
        assert_eq!(ConfigStatus::Active, config.status);
    }

    #[test]
    fn repo_dir_with_config_file_overrides_any_user_or_default_config_has_priority_2() {
        // Arrange
        // Mock repo 'local' level configuration
        let repo_root_with_config = std::env::temp_dir();
        let config_repo = repo_root_with_config.join(".inkan.yml");
        let path_buf: PathBuf = config_repo.clone().into();
        std::fs::File::create(&path_buf).unwrap();

        for key in [ConfigKey::Default, ConfigKey::User(Faker.fake())] {
            // Act
            let actual = AppConfig::map_config_overrides(
                Config {
                    key,
                    path: valid_file_path(),
                    status: ConfigStatus::Active,
                },
                repo_root_with_config.clone().try_into().unwrap(),
            )
            .unwrap();

            // Assert
            let path: PathBuf = actual.path.into();
            assert_eq!(config_repo, path);
            assert_eq!(ConfigKey::Local, actual.key);
            assert_eq!(ConfigStatus::Active, actual.status);
        }

        std::fs::remove_file(path_buf).unwrap();
    }

    #[test]
    fn user_sets_config_file_and_no_config_or_once_off_config_priority_3() {
        let user_path = valid_file_path();
        let repo_non_existing = abs_repo_directory();
        let key = ConfigKey::User(Faker.fake());

        let config = AppConfig::map_config_overrides(
            Config {
                key: key.clone(),
                path: user_path.clone(),
                status: ConfigStatus::Active,
            },
            repo_non_existing,
        )
        .unwrap();

        assert_eq!(key, config.key);
        assert_eq!(user_path, config.path);
        assert_eq!(ConfigStatus::Active, config.status);
    }

    #[test]
    fn no_user_path_or_valid_repo_dir_defaults_priority_4() {
        let default_path = valid_file_path();
        let repo_non_existing = abs_repo_directory();

        let config = AppConfig::map_config_overrides(
            Config {
                key: ConfigKey::Default,
                path: default_path.clone(),
                status: ConfigStatus::Active,
            },
            repo_non_existing,
        )
        .unwrap();

        assert_eq!(default_path, config.path);
        assert_eq!(ConfigKey::Default, config.key);
        assert_eq!(ConfigStatus::Active, config.status);
    }

    fn repo_directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn abs_repo_directory() -> AbsolutePath {
        repo_directory().try_into().unwrap()
    }

    fn valid_file_path() -> AbsolutePath {
        let path = abs_repo_directory();
        path.join("templates/default.yml", PathType::File).unwrap()
    }
}
