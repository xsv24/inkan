#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ConfigKey {
    User(String),
    Once,
    Local,
    Conventional,
    Default,
}

impl ConfigKey {
    pub fn is_overridable(&self) -> bool {
        match self {
            ConfigKey::User(_) => true,
            ConfigKey::Once | ConfigKey::Local | ConfigKey::Conventional | ConfigKey::Default => {
                false
            }
        }
    }
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
            ConfigKey::Conventional => "conventional".into(),
        }
    }
}

impl From<&str> for ConfigKey {
    fn from(value: &str) -> Self {
        match value {
            "default" => ConfigKey::Default,
            "once" => ConfigKey::Once,
            "local" => ConfigKey::Local,
            "conventional" => ConfigKey::Conventional,
            key => ConfigKey::User(key.into()),
        }
    }
}
