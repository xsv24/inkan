use anyhow::Context;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    domain::adapters::{self, CheckoutStatus, CommitMsgStatus},
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

impl adapters::Git for Git {
    fn repository_name(&self) -> anyhow::Result<String> {
        let repo_dir = self.root_directory()?.try_convert()?;

        let repo = repo_dir
            .split('/')
            .last()
            .context("Failed to get repository name")?;

        log::info!("git repository name '{}'", repo);

        Ok(repo.trim().into())
    }

    fn branch_name(&self) -> anyhow::Result<String> {
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

    fn root_directory(&self) -> anyhow::Result<PathBuf> {
        let dir = Git::command(&["rev-parse", "--show-toplevel"]).try_convert()?;
        log::info!("git root directory {}", dir);

        Ok(Path::new(dir.trim()).to_owned())
    }

    fn template_file_path(&self) -> anyhow::Result<PathBuf> {
        // Template file and stored in the .git directory to avoid users having to adding to their .gitignore
        // In future maybe we could make our own .git-kit dir to house config / templates along with this.
        let path = self
            .root_directory()?
            .join(".git")
            .join("GIT_KIT_COMMIT_TEMPLATE");

        Ok(path)
    }

    fn commit_with_template(
        &self,
        template: &Path,
        completed: CommitMsgStatus,
    ) -> anyhow::Result<()> {
        log::info!("commit template with CommitMsgStatus: '{:?}'", completed);

        let template = template
            .as_os_str()
            .to_str()
            .context("Failed to convert path to str.")?;

        let mut args = vec!["commit", "--template", template];

        // Pre-cautionary measure encase 'message' is provided but still matches template exactly.
        // Otherwise git will just abort the commit if theres no difference / change from the template.
        if completed == CommitMsgStatus::Completed {
            log::info!("allowing an empty message on commit");
            args.push("--allow-empty-message");
        }

        Git::command(&args).status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::adapters::Git;

    use super::*;

    #[test]
    fn get_repo_name_returns_this_repo_name() -> anyhow::Result<()> {
        let git = Git;

        // TODO: Find a more testable approach to check stdout maybe?
        assert_eq!(git.repository_name()?, "git-kit");

        Ok(())
    }
}
