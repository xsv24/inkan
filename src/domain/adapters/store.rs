use crate::domain::{
    errors::PersistError,
    models::{Branch, Config, ConfigKey},
};

pub trait Store {
    fn persist_branch(&self, branch: &Branch) -> Result<(), PersistError>;

    fn get_branch(&self, branch: &str, repo: &str) -> Result<Branch, PersistError>;

    fn persist_config(&self, config: &Config) -> Result<(), PersistError>;

    fn set_active_config(&mut self, key: &ConfigKey) -> Result<Config, PersistError>;

    fn get_configurations(&self) -> Result<Vec<Config>, PersistError>;

    fn get_configuration(&self, key: Option<String>) -> Result<Config, PersistError>;

    fn close(self) -> anyhow::Result<()>;
}
