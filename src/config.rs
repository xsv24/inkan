use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::utils::{expected_path, get_file_contents};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
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

impl Config {
    pub fn new(
        user_config_path: Option<String>,
        git_root_path: PathBuf,
        default_path: &Path,
    ) -> anyhow::Result<Self> {
        let config_path = Self::get_config_path(user_config_path, git_root_path, default_path)?;

        let config_contents = get_file_contents(&config_path)?;
        let config = serde_yaml::from_str::<Config>(&config_contents)
            .context("Failed to load 'config.yml' from please ensure yaml is valid.")?;

        Ok(config)
    }

    pub fn validate_template(&self, name: &str) -> clap::Result<()> {
        if self.commit.templates.contains_key(name) {
            Ok(())
        } else {
            // TODO: want a nice error message that shows the templates output
            Err(clap::Error::raw(
                clap::ErrorKind::InvalidSubcommand,
                format!("Found invalid subcommand '{}' given", name),
            ))?
        }
    }

    pub fn get_template_config(&self, name: &str) -> clap::Result<&TemplateConfig> {
        let template = self.commit.templates.get(name).ok_or_else(|| {
            clap::Error::raw(
                clap::ErrorKind::MissingSubcommand,
                format!("Found missing subcommand '{}'", name),
            )
        })?;

        Ok(template)
    }

    fn get_config_path(
        user_config: Option<String>,
        repo_config: PathBuf,
        default_path: &Path,
    ) -> anyhow::Result<PathBuf> {
        let filename = ".git-kit.yml";
        let repo_config = repo_config.join(filename);
        let default_path = default_path.join(filename);

        match (user_config, repo_config) {
            (Some(user), _) => expected_path(&user).map_err(|_| {
                anyhow::anyhow!(format!(
                    "Invalid config file path does not exist at '{}'",
                    &user
                ))
            }),
            (None, repo) if repo.exists() => Ok(repo),
            (_, _) => Ok(default_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use uuid::Uuid;

    use crate::{adapters::Git, domain::commands::GitCommands};

    use super::*;

    fn fake_project_dir() -> PathBuf {
        let dir = ProjectDirs::from("test", "xsv24", &format!("{}", Uuid::new_v4()))
            .expect("Failed to retrieve 'git-kit' config");

        dir.config_dir().to_owned()
    }

    #[test]
    fn no_user_path_or_valid_repo_dir_defaults() -> anyhow::Result<()> {
        let default_path = fake_project_dir();

        let repo_non_existing = Path::new(&Faker.fake::<String>()).to_owned();

        let config_dir = Config::get_config_path(None, repo_non_existing, &default_path)?;

        assert_eq!(config_dir, default_path.join(".git-kit.yml"));
        Ok(())
    }

    #[test]
    fn repo_dir_with_config_file_used_over_default() -> anyhow::Result<()> {
        let git: &dyn GitCommands = &Git;
        let repo_root_with_config = git.root_directory()?;

        let config_dir =
            Config::get_config_path(None, repo_root_with_config.clone(), &fake_project_dir())?;

        assert_eq!(config_dir, repo_root_with_config.join(".git-kit.yml"));
        Ok(())
    }

    #[test]
    fn user_config_file_used_over_repo_and_default() -> anyhow::Result<()> {
        let git: &dyn GitCommands = &Git;

        let user_config = Path::new(".").to_owned();

        let config_dir = Config::get_config_path(
            Some(user_config.to_str().unwrap().to_string()),
            git.root_directory()?,
            &fake_project_dir(),
        )?;

        assert_eq!(config_dir, user_config);

        Ok(())
    }

    #[test]
    fn invalid_path_for_user_config_file_errors() -> anyhow::Result<()> {
        let git: &dyn GitCommands = &Git;

        let user_config = Path::new(&Faker.fake::<String>()).to_owned();

        let error = Config::get_config_path(
            Some(user_config.to_str().unwrap().to_string()),
            git.root_directory()?,
            &fake_project_dir(),
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "Invalid config file path does not exist at '{}'",
                user_config.display()
            )
        );

        Ok(())
    }
}
