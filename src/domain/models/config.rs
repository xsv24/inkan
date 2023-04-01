use std::fmt;

use crate::domain::models::ConfigStatus;

use super::{path::AbsolutePath, ConfigKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub key: ConfigKey,
    pub path: AbsolutePath,
    pub status: ConfigStatus,
}

impl Config {
    pub fn new(key: ConfigKey, path: AbsolutePath, status: ConfigStatus) -> anyhow::Result<Self> {
        Ok(Config { key, status, path })
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key: String = self.key.clone().into();

        write!(
            f,
            "Configuration is set to '{}' at path:\n {}",
            key,
            self.path.to_string()
        )
    }
}
