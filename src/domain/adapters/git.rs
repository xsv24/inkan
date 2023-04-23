use std::path::{Path, PathBuf};

use crate::domain::{errors::GitError, models::path::AbsolutePath};

#[derive(Debug, PartialEq, Eq)]
pub enum CheckoutStatus {
    New,
    Existing,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommitMsgStatus {
    InComplete,
    Completed,
}

/// Used to abstract cli git commands for testing.
pub trait Git {
    /// Get the root directory of the current git repo.
    fn root_directory(&self) -> Result<AbsolutePath, GitError>;

    /// Get the current git repository name.
    fn repository_name(&self) -> Result<String, GitError>;

    /// Get the current checked out branch name.
    fn branch_name(&self) -> Result<String, GitError>;

    /// Checkout an existing branch of create a new branch if not.
    fn checkout(&self, name: &str, status: CheckoutStatus) -> Result<(), GitError>;

    /// Get the commit file path for the current repository.
    fn template_file_path(&self) -> Result<PathBuf, GitError>;

    /// Commit changes and open and editor with template file.
    fn commit_with_template(
        &self,
        template: &Path,
        completed: CommitMsgStatus,
    ) -> Result<(), GitError>;
}

pub trait GitResult {
    fn get_status(&mut self) -> anyhow::Result<()>;

    fn get_output(&mut self) -> anyhow::Result<String>;
}

/// Used to abstract system cli commands for testing.
pub trait GitSystem {
    type Result: GitResult;

    /// Used to abstract system cli commands for testing.
    fn command(&self, args: &[&str]) -> Self::Result;
}
