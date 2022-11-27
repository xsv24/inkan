use std::{fmt, path::PathBuf};

use anyhow::Context;

use crate::{domain::models::ConfigStatus, utils::TryConvert};

use super::ConfigKey;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub key: ConfigKey,
    pub path: PathBuf,
    pub status: ConfigStatus,
}

impl Config {
    pub fn new(key: ConfigKey, path: String, status: ConfigStatus) -> anyhow::Result<Self> {
        Ok(Config {
            key,
            status,
            path: path
                .try_convert()
                .context("Invalid config file path does not exist.")?,
        })
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key: String = self.key.clone().into();

        write!(
            f,
            "Configuration is set to '{}' at path:\n {}",
            key,
            self.path.display()
        )
    }
}

#[cfg(test)]
mod test {
    use fake::{Fake, Faker};

    use super::*;

    #[test]
    fn invalid_path_on_config_new_throws_an_error() -> anyhow::Result<()> {
        let user_config = PathBuf::new();

        let error = Config::new(
            ConfigKey::User(Faker.fake()),
            user_config.to_str().unwrap().to_string(),
            ConfigStatus::Active,
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Invalid config file path does not exist.",
        );

        Ok(())
    }
}
