#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigStatus {
    Active,
    Disabled,
}

impl From<ConfigStatus> for String {
    fn from(status: ConfigStatus) -> Self {
        match status {
            ConfigStatus::Active => "ACTIVE".into(),
            ConfigStatus::Disabled => "DISABLED".into(),
        }
    }
}

impl TryInto<ConfigStatus> for String {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ConfigStatus, Self::Error> {
        match self.as_str() {
            "ACTIVE" => Ok(ConfigStatus::Active),
            "DISABLED" => Ok(ConfigStatus::Disabled),
            value => anyhow::bail!("'{}' is not a valid ConfigStatus", value),
        }
    }
}
