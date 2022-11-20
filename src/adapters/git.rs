use anyhow::Context;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
            .to_str()
            .context("Failed to get repository name")?
            .split('/')
            .last()
            .context("Failed to get repository name")?;

        log::info!("git repository name '{}'", repo);

        Ok(repo.trim().into())
    }

    fn get_branch_name(&self) -> anyhow::Result<String> {
        let branch: String = Git::command(&["branch", "--show-current"]).try_convert()?;
        log::info!("current git branch name '{}'", branch);

        Ok(branch)
    }

    fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()> {
        log::info!("checkout '{:?}' branch", status);

        let mut command = match status {
            CheckoutStatus::New => Git::command(&["checkout", "-b", name]),
            CheckoutStatus::Existing => Git::command(&["checkout", name]),
        };

        command.status()?;

        Ok(())
    }

    fn commit(&self, msg: &str) -> anyhow::Result<()> {
        log::info!("git commit with message '{}'", msg);
        Git::command(&["commit", "-m", msg, "-e"]).status()?;

        Ok(())
    }

    fn root_directory(&self) -> anyhow::Result<PathBuf> {
        let dir = Git::command(&["rev-parse", "--show-toplevel"]).try_convert()?;
        log::info!("git root directory {}", dir);

        Ok(Path::new(dir.trim()).to_owned())
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
