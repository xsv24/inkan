#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ConfigKey {
    User(String),
    Once,
    Local,
    Default,
}

impl ToString for ConfigKey {
    fn to_string(&self) -> String {
        self.to_owned().into()
    }
}

impl From<ConfigKey> for String {
    fn from(config: ConfigKey) -> Self {
        match config {
            ConfigKey::User(key) => key,
            ConfigKey::Once => "once".into(),
            ConfigKey::Default => "default".into(),
            ConfigKey::Local => "local".into(),
        }
    }
}

impl From<String> for ConfigKey {
    fn from(value: String) -> Self {
        match value.as_str() {
            "default" => ConfigKey::Default,
            "once" => ConfigKey::Once,
            "local" => ConfigKey::Local,
            key => ConfigKey::User(key.into()),
        }
    }
}
