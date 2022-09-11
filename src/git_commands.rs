use anyhow::Context;
use std::process::Command;

use crate::try_convert::TryConvert;

fn git_command(args: &[&str]) -> Command {
    let mut comm = Command::new("git");

    comm.args(args);

    comm
}

pub fn get_repo_name() -> anyhow::Result<String> {
    let repo_dir: String = git_command(&["rev-parse", "--show-toplevel"]).try_convert()?;

    let repo = repo_dir
        .split("/")
        .last()
        .context("Failed to get repository name")?;

    Ok(repo.trim().into())
}

pub fn get_repo_name_command() -> Command {
    git_command(&["rev-parse", "--show-toplevel"])
}

pub fn get_branch_name() -> Command {
    git_command(&["branch", "--show-current"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_repo_name_returns_this_repo_name() -> anyhow::Result<()> {
        // TODO: Find a more testable approach to check stdout maybe?
        assert_eq!(get_repo_name()?, "git-kit");

        Ok(())
    }
}
