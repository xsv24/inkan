use std::path::{Path, PathBuf};

use anyhow::Context;
use directories::ProjectDirs;
use rusqlite::Connection;

use crate::{
    adapters::{sqlite::Sqlite, Git},
    domain::{
        adapters::{Git as _, Store},
        models::{Config, ConfigKey, ConfigStatus},
    },
};

pub struct AppConfig {
    pub config: Config,
}

impl AppConfig {
    pub fn new(
        once_off_config_path: Option<String>,
        git: &Git,
        store: &Sqlite,
    ) -> anyhow::Result<AppConfig> {
        let config = match once_off_config_path {
            Some(path) => Config::new(ConfigKey::Once, path, ConfigStatus::Active),
            None => store.get_configuration(None),
        }?;

        let git_root_dir = git.root_directory()?;
        let config = Self::map_config_overrides(config, git_root_dir)?;

        Ok(AppConfig { config })
    }

    pub fn db_connection() -> anyhow::Result<Connection> {
        let db_file = Self::template_config_dir()?.join("db");

        let connection = Connection::open(db_file).context("Failed to open sqlite connection")?;

        Ok(connection)
    }

    fn template_config_dir() -> anyhow::Result<PathBuf> {
        let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        Ok(project_dir.config_dir().to_owned())
    }

    pub fn join_config_filename(repo_config: &Path) -> PathBuf {
        let filename = ".git-kit.yml";
        repo_config.join(filename)
    }

    fn map_config_overrides(config: Config, repo_config: PathBuf) -> anyhow::Result<Config> {
        let repo_config = AppConfig::join_config_filename(&repo_config);

        match (config.key.clone(), repo_config.exists()) {
            // Once off override takes priority 1
            (ConfigKey::Once, _) => {
                log::info!("⏳ Loading once off config...");
                Ok(config)
            }
            // Repository has config file priority 2
            (ConfigKey::Local, _) | (_, true) => {
                log::info!("⏳ Loading local repo config...");
                Ok(Config {
                    key: ConfigKey::Local,
                    path: repo_config,
                    status: config.status,
                })
            }
            // User has set custom config file and is active priority 3
            (ConfigKey::User(key), _) => {
                log::info!("⏳ Loading user '{:?}' config...", key);
                Ok(config)
            }
            // No set user config use provided defaults priority 4
            (ConfigKey::Default, _) => {
                log::info!("⏳ Loading global config...");
                Ok(config)
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

        let config = AppConfig::map_config_overrides(
            Config {
                key: ConfigKey::Once,
                path: once_path.clone(),
                status: ConfigStatus::Active,
            },
            valid_repo_dir,
        )?;

        assert_eq!(once_path, config.path);
        assert_eq!(ConfigKey::Once, config.key);
        assert_eq!(ConfigStatus::Active, config.status);

        Ok(())
    }

    #[test]
    fn repo_dir_with_config_file_overrides_any_user_or_default_config_has_priority_2(
    ) -> anyhow::Result<()> {
        let git: &dyn adapters::Git = &Git;
        let repo_root_with_config = git.root_directory()?;
        let config_repo = repo_root_with_config.join(".git-kit.yml");
        std::fs::File::create(&config_repo)?;

        for key in [ConfigKey::Default, ConfigKey::User(Faker.fake())] {
            let actual = AppConfig::map_config_overrides(
                Config {
                    key,
                    path: fake_path_buf(),
                    status: ConfigStatus::Active,
                },
                repo_root_with_config.clone(),
            )?;

            assert_eq!(config_repo, actual.path);
            assert_eq!(ConfigKey::Local, actual.key);
            assert_eq!(ConfigStatus::Active, actual.status);
        }

        std::fs::remove_file(config_repo)?;

        Ok(())
    }

    #[test]
    fn user_sets_config_file_and_no_config_or_once_off_config_priority_3() -> anyhow::Result<()> {
        let user_path = fake_path_buf();
        let repo_non_existing = fake_path_buf();
        let key = ConfigKey::User(Faker.fake());

        let config = AppConfig::map_config_overrides(
            Config {
                key: key.clone(),
                path: user_path.clone(),
                status: ConfigStatus::Active,
            },
            repo_non_existing,
        )?;

        assert_eq!(key, config.key);
        assert_eq!(user_path, config.path);
        assert_eq!(ConfigStatus::Active, config.status);

        Ok(())
    }

    #[test]
    fn no_user_path_or_valid_repo_dir_defaults_priority_4() -> anyhow::Result<()> {
        let default_path = fake_path_buf();
        let repo_non_existing = fake_path_buf();

        let config = AppConfig::map_config_overrides(
            Config {
                key: ConfigKey::Default,
                path: default_path.clone(),
                status: ConfigStatus::Active,
            },
            repo_non_existing,
        )?;

        assert_eq!(default_path, config.path);
        assert_eq!(ConfigKey::Default, config.key);
        assert_eq!(ConfigStatus::Active, config.status);

        Ok(())
    }

    fn fake_path_buf() -> PathBuf {
        PathBuf::from(Faker.fake::<String>())
    }
}
