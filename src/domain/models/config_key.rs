#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigKey {
    User(String),
    Once,
    Default,
}

impl From<ConfigKey> for String {
    fn from(config: ConfigKey) -> Self {
        match config {
            ConfigKey::User(key) => key,
            ConfigKey::Once => "once".into(),
            ConfigKey::Default => "default".into(),
        }
    }
}

impl From<String> for ConfigKey {
    fn from(value: String) -> Self {
        match value.as_str() {
            "default" => ConfigKey::Default,
            "once" => ConfigKey::Once,
            key => ConfigKey::User(key.into()),
        }
    }
}
