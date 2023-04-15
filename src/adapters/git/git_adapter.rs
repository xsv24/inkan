use std::path::{Path, PathBuf};

use crate::domain::{
    adapters::{self, CheckoutStatus, CommitMsgStatus, GitResult, GitSystem},
    errors::GitError,
    models::path::{AbsolutePath, PathType},
};

pub struct Git<S: GitSystem> {
    pub git: S,
}

impl<S: GitSystem> adapters::Git for Git<S> {
    fn root_directory(&self) -> Result<AbsolutePath, GitError> {
        let dir: String = self
            .git
            .command(&["rev-parse", "--show-toplevel"])
            .get_output()
            .map_err(|e| {
                log::error!("Failed to get root directory: {}", e);
                GitError::RootDirectory
            })?;

        log::info!("git root directory {}", dir);
        let path = AbsolutePath::try_from(dir, PathType::Directory).map_err(|e| {
            log::error!("Expected git root directory: {}", e);
            GitError::RootDirectory
        })?;

        Ok(path)
    }

    fn repository_name(&self) -> Result<String, GitError> {
        let repo_dir: String = self.root_directory()?.try_into().map_err(|e| {
            log::error!("Failed to get repository name: {}", e);
            GitError::RootDirectory
        })?;

        let repo = repo_dir.split('/').last().ok_or_else(|| {
            log::error!("Failed to get repository name");
            GitError::RootDirectory
        })?;

        log::info!("git repository name '{}'", repo);

        Ok(repo.trim().into())
    }

    fn branch_name(&self) -> Result<String, GitError> {
        let branch = self
            .git
            .command(&["branch", "--show-current"])
            .get_output()
            .map_err(|e| {
                log::error!("Failed to get current branch name: {}", e);
                GitError::BranchName
            })?;

        log::info!("current git branch name '{}'", branch);

        Ok(branch)
    }

    fn checkout(&self, name: &str, status: CheckoutStatus) -> Result<(), GitError> {
        log::info!("checkout '{:?}' branch", status);

        let mut command = match status {
            CheckoutStatus::New => self.git.command(&["checkout", "-b", name]),
            CheckoutStatus::Existing => self.git.command(&["checkout", name]),
        };

        command.get_status().map_err(|e| {
            log::error!("Failed to checkout branch: {}", e);
            GitError::Checkout { name: name.into() }
        })?;

        Ok(())
    }

    fn template_file_path(&self) -> Result<AbsolutePath, GitError> {
        // Template file and stored in the .git directory to avoid users having to adding to their .gitignore
        // In future maybe we could make our own .git-kit dir to house config / templates along with this.
        let path: PathBuf = self.root_directory()?.into();

        let path: AbsolutePath = path
            .join(".git")
            .join("GIT_KIT_COMMIT_TEMPLATE")
            .try_into()
            .map_err(|e| {
                log::error!("{}", e);
                GitError::Validation {
                    message: "Failed to build template file path".into(),
                }
            })?;

        Ok(path)
    }

    fn commit_with_template(
        &self,
        template: &Path,
        completed: CommitMsgStatus,
    ) -> Result<(), GitError> {
        log::info!("commit template with CommitMsgStatus: '{:?}'", completed);

        if !template.is_file() {
            return Err(GitError::Validation {
                message: "Invalid template provided".into(),
            });
        }

        let template = template
            .as_os_str()
            .to_str()
            .ok_or_else(|| GitError::Validation {
                message: "Failed to convert path to str".into(),
            })?;

        let mut args = vec!["commit", "--template", template];

        // Pre-cautionary measure encase 'message' is provided but still matches template exactly.
        // Otherwise git will just abort the commit if theres no difference / change from the template.
        if completed == CommitMsgStatus::Completed {
            log::info!("allowing an empty message on commit");
            args.push("--allow-empty-message");
        }

        self.git.command(&args).get_status().map_err(|e| {
            log::error!("Failed to commit template: {}", e);
            GitError::Commit
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;

    use crate::{adapters::git::GitCommand, domain::adapters::Git as _};

    use super::*;

    #[test]
    fn root_repository_adds_expected_git_options() {
        let git = Git {
            git: GitSystemMock {
                result: |args| {
                    assert_eq!(args, ["rev-parse", "--show-toplevel"]);

                    GitResultMock {
                        get_status_result: || panic!("Should not be called!"),
                        get_output_result: || Ok(valid_dir_path().display().to_string()),
                    }
                },
            },
        };

        let result = git.root_directory().unwrap();
        assert_eq!(result, valid_dir_path().try_into().unwrap());
    }

    #[test]
    fn root_repository_errors_on_system_command_failure() {
        let git = Git {
            git: GitSystemMock {
                result: |_| GitResultMock {
                    get_status_result: || panic!("Should not be called!"),
                    get_output_result: || anyhow::bail!("Whoops"),
                },
            },
        };

        let err = git.root_directory().unwrap_err();
        assert!(matches!(err, GitError::RootDirectory));
    }

    #[test]
    #[ignore] // ignoring for CI
    fn repository_name_returns_this_repo_name() {
        let git = Git { git: GitCommand };
        let repo = git.repository_name().unwrap();
        assert_eq!(repo, "git-kit");
    }

    #[test]
    fn repository_name_errors_on_get_output_system_command() {
        let git = Git {
            git: GitSystemMock {
                result: |_| GitResultMock {
                    get_status_result: || panic!("Should not be called!"),
                    get_output_result: || anyhow::bail!("Whoops"),
                },
            },
        };

        let err = git.repository_name().unwrap_err();
        assert!(matches!(err, GitError::RootDirectory));
    }

    #[test]
    fn branch_name_add_expected_git_options() {
        let git = Git {
            git: GitSystemMock {
                result: |args| {
                    assert_eq!(args, ["branch", "--show-current"]);

                    GitResultMock {
                        get_status_result: || panic!("Should not be called!"),
                        get_output_result: || Ok("my_branch".into()),
                    }
                },
            },
        };

        let branch_name = git.branch_name().unwrap();
        assert_eq!(branch_name, "my_branch");
    }

    #[test]
    fn branch_name_error_on_get_output_system_command() {
        let git = Git {
            git: GitSystemMock {
                result: |_| GitResultMock {
                    get_status_result: || panic!("Should not be called!"),
                    get_output_result: || anyhow::bail!("Whoops"),
                },
            },
        };

        let err = git.branch_name().unwrap_err();
        assert!(matches!(err, GitError::BranchName));
    }

    #[test]
    fn commit_with_template_with_incomplete_status_adds_expected_options() {
        let path = valid_file_path();

        let git = Git {
            git: GitSystemMock {
                result: |args| {
                    assert_eq!(
                        args,
                        [
                            "commit",
                            "--template",
                            &valid_file_path().display().to_string()
                        ]
                    );

                    GitResultMock {
                        get_status_result: || Ok(()),
                        get_output_result: || panic!("Should not be called!"),
                    }
                },
            },
        };

        git.commit_with_template(&path, CommitMsgStatus::InComplete)
            .unwrap();
    }

    #[test]
    fn commit_with_template_with_completed_status_adds_expected_options() {
        let path = valid_file_path();

        let git = Git {
            git: GitSystemMock {
                result: |args| {
                    assert_eq!(
                        args,
                        [
                            "commit",
                            "--template",
                            &valid_file_path().display().to_string(),
                            "--allow-empty-message"
                        ]
                    );

                    GitResultMock {
                        get_status_result: || Ok(()),
                        get_output_result: || panic!("Should not be called!"),
                    }
                },
            },
        };

        git.commit_with_template(&path, CommitMsgStatus::Completed)
            .unwrap();
    }

    #[test]
    fn commit_with_template_errors_on_invalid_path() {
        let git = Git {
            git: GitSystemMock {
                result: |_| GitResultMock {
                    get_status_result: || Ok(()),
                    get_output_result: || panic!("Should not be called!"),
                },
            },
        };

        let err = git
            .commit_with_template(&PathBuf::new(), CommitMsgStatus::Completed)
            .unwrap_err();
        assert!(
            matches!(err, GitError::Validation { message } if message == "Invalid template provided")
        );
    }

    #[test]
    fn commit_with_template_errors_on_failed_get_output_system_command() {
        let git = Git {
            git: GitSystemMock {
                result: |_| GitResultMock {
                    get_status_result: || anyhow::bail!("Whoops"),
                    get_output_result: || panic!("Should not be called!"),
                },
            },
        };

        let err = git
            .commit_with_template(&valid_file_path(), CommitMsgStatus::Completed)
            .unwrap_err();
        assert!(matches!(err, GitError::Commit));
    }

    #[test]
    fn checkout_new_branch_adds_expected_git_options() {
        let git = Git {
            git: GitSystemMock {
                result: |args| {
                    assert_eq!(args, ["checkout", "-b", "my-branch"]);
                    GitResultMock {
                        get_status_result: || Ok(()),
                        get_output_result: || panic!("Should not be called!"),
                    }
                },
            },
        };

        git.checkout("my-branch", CheckoutStatus::New).unwrap()
    }

    #[test]
    fn checkout_existing_branch_adds_expected_git_options() {
        let git = Git {
            git: GitSystemMock {
                result: |args| {
                    assert_eq!(args, ["checkout", "my-branch"]);
                    GitResultMock {
                        get_status_result: || Ok(()),
                        get_output_result: || panic!("Should not be called!"),
                    }
                },
            },
        };

        git.checkout("my-branch", CheckoutStatus::Existing).unwrap()
    }

    #[test]
    fn checkout_errors_on_get_status_system_command_fail() {
        let git = Git {
            git: GitSystemMock {
                result: |_| GitResultMock {
                    get_status_result: || anyhow::bail!("Whoops"),
                    get_output_result: || panic!("Should not be called!"),
                },
            },
        };

        let err = git.checkout("name", CheckoutStatus::New).unwrap_err();
        assert!(matches!(err, GitError::Checkout { name } if name == "name"));
    }

    #[derive(Debug, Clone)]
    pub struct GitResultMock {
        pub get_status_result: fn() -> anyhow::Result<()>,
        pub get_output_result: fn() -> anyhow::Result<String>,
    }

    impl GitResult for GitResultMock {
        fn get_status(&mut self) -> anyhow::Result<()> {
            (self.get_status_result)()
        }

        fn get_output(&mut self) -> anyhow::Result<String> {
            (self.get_output_result)()
        }
    }

    pub struct GitSystemMock {
        result: fn(args: &[&str]) -> GitResultMock,
    }

    impl GitSystem for GitSystemMock {
        type Result = GitResultMock;

        fn command(&self, args: &[&str]) -> Self::Result {
            (self.result)(args)
        }
    }

    fn valid_dir_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn valid_file_path() -> PathBuf {
        let mut path = valid_dir_path();
        path.push("./README.md");

        path
    }
}
