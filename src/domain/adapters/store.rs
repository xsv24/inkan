use crate::domain::{
    errors::PersistError,
    models::{Branch, ConfigKey, Template},
};

pub trait Store {
    fn persist_branch(&self, branch: &Branch) -> Result<(), PersistError>;

    fn get_branch(&self, branch: &str, repo: &str) -> Result<Branch, PersistError>;

    fn persist_template(&self, config: &Template) -> Result<(), PersistError>;

    fn set_active_template(&mut self, key: &ConfigKey) -> Result<Template, PersistError>;

    fn get_templates(&self) -> Result<Vec<Template>, PersistError>;

    fn get_template(&self, key: Option<String>) -> Result<Template, PersistError>;

    fn close(self) -> anyhow::Result<()>;
}
