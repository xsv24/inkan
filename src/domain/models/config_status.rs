#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigStatus {
    ACTIVE,
    INACTIVE,
}

impl From<ConfigStatus> for String {
    fn from(status: ConfigStatus) -> Self {
        match status {
            ConfigStatus::ACTIVE => "ACTIVE".into(),
            ConfigStatus::INACTIVE => "INACTIVE".into(),
        }
    }
}

impl TryInto<ConfigStatus> for String {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ConfigStatus, Self::Error> {
        match self.as_str() {
            "ACTIVE" => Ok(ConfigStatus::ACTIVE),
            "INACTIVE" => Ok(ConfigStatus::INACTIVE),
            value => anyhow::bail!("'{}' is not a valid ConfigStatus", value),
        }
    }
}
