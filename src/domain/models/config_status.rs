#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TemplateStatus {
    Active,
    Disabled,
}

impl From<TemplateStatus> for String {
    fn from(status: TemplateStatus) -> Self {
        match status {
            TemplateStatus::Active => "ACTIVE".into(),
            TemplateStatus::Disabled => "DISABLED".into(),
        }
    }
}

impl TryInto<TemplateStatus> for String {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<TemplateStatus, Self::Error> {
        match self.as_str() {
            "ACTIVE" => Ok(TemplateStatus::Active),
            "DISABLED" => Ok(TemplateStatus::Disabled),
            value => anyhow::bail!("'{}' is not a valid ConfigStatus", value),
        }
    }
}
