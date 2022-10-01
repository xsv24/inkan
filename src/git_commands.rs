use anyhow::Context;
use std::process::Command;

use crate::try_convert::TryConvert;

#[derive(PartialEq, Eq)]
pub enum CheckoutStatus {
    New,
    Existing,
}

pub trait GitCommands {
    /// Get the current git repository name.
    fn get_repo_name(&self) -> anyhow::Result<String>;

    /// Get the current checked out branch name.
    fn get_branch_name(&self) -> anyhow::Result<String>;

    /// Checkout an existing branch of create a new branch if not.
    fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()>;

    /// Commit changes and open editor with the template.
    fn commit(&self, msg: &str) -> anyhow::Result<()>;
}

pub struct Git;

impl GitCommands for Git {
    fn get_repo_name(&self) -> anyhow::Result<String> {
        let repo_dir: String = git_command(&["rev-parse", "--show-toplevel"]).try_convert()?;

        let repo = repo_dir
            .split('/')
            .last()
            .context("Failed to get repository name")?;

        Ok(repo.trim().into())
    }

    fn get_branch_name(&self) -> anyhow::Result<String> {
        let branch: String = git_command(&["branch", "--show-current"]).try_convert()?;

        Ok(branch)
    }

    fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()> {
        let mut command = match status {
            CheckoutStatus::New => git_command(&["checkout", "-b", name]),
            CheckoutStatus::Existing => git_command(&["checkout", name]),
        };

        command.status()?;

        Ok(())
    }

    fn commit(&self, msg: &str) -> anyhow::Result<()> {
        git_command(&["commit", "-m", msg, "-e"]).status()?;

        Ok(())
    }
}

fn git_command(args: &[&str]) -> Command {
    let mut comm = Command::new("git");

    comm.args(args);

    comm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_repo_name_returns_this_repo_name() -> anyhow::Result<()> {
        let git = Git;

        // TODO: Find a more testable approach to check stdout maybe?
        assert_eq!(git.get_repo_name()?, "git-kit");

        Ok(())
    }
}
