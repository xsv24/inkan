use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;

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

impl TryConvert<String> for PathBuf {
    fn try_convert(self) -> anyhow::Result<String> {
        let path = self
            .to_str()
            .context("Failed to convert path into string")?;
        Ok(path.into())
    }
}

impl TryConvert<String> for &PathBuf {
    fn try_convert(self) -> anyhow::Result<String> {
        self.to_owned().try_convert()
    }
}

impl TryConvert<PathBuf> for String {
    fn try_convert(self) -> anyhow::Result<PathBuf> {
        let path = Path::new(&self);

        if path.exists() {
            Ok(path.to_owned())
        } else {
            Err(anyhow::anyhow!("Expected file not found."))
        }
    }
}

impl TryConvert<PathBuf> for &String {
    fn try_convert(self) -> anyhow::Result<PathBuf> {
        self.to_owned().try_convert()
    }
}
