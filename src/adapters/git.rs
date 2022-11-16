use anyhow::Context;
use std::process::Command;

use crate::{
    domain::commands::{CheckoutStatus, GitCommands},
    utils::TryConvert,
};

pub struct Git;

impl Git {
    fn command(args: &[&str]) -> Command {
        let mut comm = Command::new("git");

        comm.args(args);

        comm
    }
}

impl GitCommands for Git {
    fn get_repo_name(&self) -> anyhow::Result<String> {
        let repo_dir = self.root_directory()?;

        let repo = repo_dir
            .split('/')
            .last()
            .context("Failed to get repository name")?;

        Ok(repo.trim().into())
    }

    fn get_branch_name(&self) -> anyhow::Result<String> {
        let branch: String = Git::command(&["branch", "--show-current"]).try_convert()?;

        Ok(branch)
    }

    fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()> {
        let mut command = match status {
            CheckoutStatus::New => Git::command(&["checkout", "-b", name]),
            CheckoutStatus::Existing => Git::command(&["checkout", name]),
        };

        command.status()?;

        Ok(())
    }

    fn commit(&self, msg: &str) -> anyhow::Result<()> {
        Git::command(&["commit", "-m", msg, "-e"]).status()?;

        Ok(())
    }

    fn root_directory(&self) -> anyhow::Result<String> {
        let dir = Git::command(&["rev-parse", "--show-toplevel"]).try_convert()?;

        Ok(dir.trim().into())
    }
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
