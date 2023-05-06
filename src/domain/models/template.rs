use std::fmt;

use crate::domain::models::TemplateStatus;

use super::{path::AbsolutePath, ConfigKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub key: ConfigKey,
    pub path: AbsolutePath,
    pub status: TemplateStatus,
}

impl Template {
    pub fn new(key: ConfigKey, path: AbsolutePath, status: TemplateStatus) -> anyhow::Result<Self> {
        Ok(Template { key, status, path })
    }
}

impl fmt::Display for Template {
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
