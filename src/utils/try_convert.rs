use std::process::Command;

// Implement our own try_into just as a work around to implement
// TryInto for external types outside our crate.
pub trait TryConvert<T> {
    fn try_convert(self) -> anyhow::Result<T>;
}

impl TryConvert<String> for Command {
    fn try_convert(mut self) -> anyhow::Result<String> {
        let output = self.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(stdout.into())
    }
}
