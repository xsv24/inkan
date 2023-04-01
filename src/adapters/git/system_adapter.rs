use crate::domain::adapters::{GitResult, GitSystem};
use std::process::Command;

pub struct GitCommand;

/// Implementation of GitResult wrapper for system command result
impl GitResult for Command {
    fn get_status(&mut self) -> anyhow::Result<()> {
        let status = self.status().map_err(|e| anyhow::anyhow!(e))?;

        if !status.success() {
            anyhow::bail!("Failed to process system command.")
        } else {
            Ok(())
        }
    }

    fn get_output(&mut self) -> anyhow::Result<String> {
        let stdout: String = self
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).into())
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(stdout)
    }
}

impl GitSystem for GitCommand {
    type Result = Command;

    fn command(&self, args: &[&str]) -> Self::Result {
        let mut comm = Command::new("git");
        comm.args(args);
        comm
    }
}
