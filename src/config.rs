use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{domain::commands::GitCommands, utils::get_file_contents};

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
    pub fn new<C: GitCommands>(config_path: &Path, git: &dyn GitCommands) -> anyhow::Result<Self> {
        let current_repo_root = format!("{}/.git-kit.yml", git.root_directory()?);
        let config_path = Self::get_config_path(&current_repo_root, config_path)?;

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

    fn get_config_path<'a>(
        current_repo_root: &'a str,
        config_path: &'a Path,
    ) -> anyhow::Result<PathBuf> {
        let repo_config = Path::new(current_repo_root);

        let config_path = if repo_config.exists() {
            // TODO: logging -> println!("⏳ Loading from current local repo...");
            repo_config.to_owned()
        } else {
            // TODO: logging -> println!("⏳ Loading from global config...");
            config_path.join(".git-kit.yml")
        };

        Ok(config_path)
    }
}
