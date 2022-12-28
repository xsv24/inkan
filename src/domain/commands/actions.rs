use crate::{
    app_context::AppContext,
    cli::{checkout, commit, context},
    domain::{
        adapters::{CheckoutStatus, Git, Store},
        models::Branch,
    },
};

use super::Actor;

pub struct Actions<'a, C: Git, S: Store> {
    context: &'a AppContext<C, S>,
}

impl<'a, C: Git, S: Store> Actions<'a, C, S> {
    pub fn new(context: &AppContext<C, S>) -> Actions<C, S> {
        Actions { context }
    }
}

impl<'a, C: Git, S: Store> Actor for Actions<'a, C, S> {
    fn current(&self, args: context::Arguments) -> anyhow::Result<Branch> {
        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = self.context.git.get_repo_name()?;
        let branch_name = self.context.git.get_branch_name()?;

        let branch = Branch::new(&branch_name, &repo_name, Some(args.ticket))?;
        self.context.store.persist_branch(&branch)?;

        Ok(branch)
    }

    fn checkout(&self, args: checkout::Arguments) -> anyhow::Result<Branch> {
        // Attempt to create branch
        let create = self.context.git.checkout(&args.name, CheckoutStatus::New);

        // If the branch already exists check it out
        if let Err(err) = create {
            log::error!("failed to create new branch: {}", err);

            self.context
                .git
                .checkout(&args.name, CheckoutStatus::Existing)?;
        }

        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = self.context.git.get_repo_name()?;
        let branch = Branch::new(&args.name, &repo_name, args.ticket.clone())?;
        self.context.store.persist_branch(&branch)?;

        Ok(branch)
    }

    fn commit(&self, args: commit::Arguments) -> anyhow::Result<String> {
        let config = self.context.config.get_template_config(&args.template)?;

        let contents = args.commit_message(config.content.clone(), self.context)?;

        self.context.git.commit(&contents)?;

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use anyhow::anyhow;
    use fake::{Fake, Faker};
    use rusqlite::Connection;

    use crate::adapters::sqlite::Sqlite;
    use crate::app_config::AppConfig;
    use crate::app_config::CommitConfig;
    use crate::app_config::TemplateConfig;
    use crate::app_context::AppContext;

    use crate::domain::adapters::CheckoutStatus;
    use crate::migrations::{db_migrations, MigrationContext};

    use super::*;

    #[test]
    fn checkout_success_with_ticket() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = checkout::Arguments {
            name: Faker.fake::<String>(),
            ticket: Some(Faker.fake()),
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = Actions::new(&context);

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = context.store.get_branch(&command.name, &repo)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        assert_eq!(&branch.name, &name);
        assert_eq!(branch.ticket, command.ticket.unwrap());

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_with_branch_already_exists_does_not_error() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = fake_checkout_args();

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            checkout_res: |_, status| {
                if status == CheckoutStatus::New {
                    Err(anyhow!("branch already exists!"))
                } else {
                    Ok(())
                }
            },
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = Actions::new(&context);

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = context.store.get_branch(&command.name, &repo)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        assert_eq!(&branch.name, &name);
        assert_eq!(branch.ticket, command.ticket.unwrap());

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_on_fail_to_checkout_branch_nothing_is_persisted() -> anyhow::Result<()> {
        // Arrange
        let command = fake_checkout_args();

        let repo = Faker.fake::<String>();
        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            checkout_res: |_, _| Err(anyhow!("failed to create or checkout existing branch!")),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = Actions::new(&context);

        // Act
        let result = actions.checkout(command.clone());

        // Assert
        assert!(result.is_err());

        let error = context
            .store
            .get_branch(&command.name, &repo)
            .expect_err("Expected error as there should be no stored branches.");

        assert_eq!(error.to_string(), "Query returned no rows");

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_success_without_ticket_uses_branch_name() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = checkout::Arguments {
            ticket: None,
            ..fake_checkout_args()
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = Actions::new(&context);

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = context.store.get_branch(&command.name, &repo)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        assert_eq!(&branch.name, &name);
        assert_eq!(&branch.ticket, &command.name);

        context.close()?;

        Ok(())
    }

    #[test]
    fn current_success() -> anyhow::Result<()> {
        // Arrange
        let branch_name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let command = context::Arguments {
            ticket: Faker.fake(),
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(branch_name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = Actions::new(&context);

        // Act
        actions.current(command.clone())?;

        // Assert
        let branch = context.store.get_branch(&branch_name, &repo)?;
        let name = format!(
            "{}-{}",
            &git_commands.repo.unwrap(),
            &git_commands.branch_name.unwrap()
        );

        assert_eq!(&branch.name, &name);
        assert_eq!(branch.ticket, command.ticket);

        context.close()?;

        Ok(())
    }

    #[test]
    fn commit_message_with_ticket_and_message_arg_are_formatted_correctly() -> anyhow::Result<()> {
        let args = commit::Arguments {
            ticket: Some(Faker.fake()),
            message: Some(Faker.fake()),
            ..fake_commit_args()
        };

        for (template_contents, args) in fake_commit_templates(Some(args)) {
            let context = fake_context(GitCommandMock::fake())?;
            let actions = Actions::new(&context);

            // Act
            let contents = actions
                .commit(args.clone())
                .expect("Error performing 'commit' action");

            // Assert
            let expected = format!(
                "[{}] {} {}",
                args.ticket.clone().unwrap_or("".into()),
                template_contents,
                args.message.clone().unwrap_or("".into())
            );
            assert_eq!(expected, contents);

            context.close()?;
        }

        Ok(())
    }

    #[test]
    fn commit_message_with_no_ticket_or_stored_branch_defaults_correctly() -> anyhow::Result<()> {
        let args = commit::Arguments {
            ticket: None,
            ..fake_commit_args()
        };

        for (template_contents, args) in fake_commit_templates(Some(args)) {
            let context = fake_context(GitCommandMock::fake())?;
            let actions = Actions::new(&context);

            // Act
            let contents = actions
                .commit(args.clone())
                .expect("Error performing 'commit' action");

            // Assert
            let expected = format!(
                "{} {}",
                template_contents,
                args.message.clone().unwrap_or("".into())
            );
            assert_eq!(expected.trim(), contents);

            context.close()?;
        }

        Ok(())
    }

    #[test]
    fn commit_message_with_no_ticket_uses_stored_branch() -> anyhow::Result<()> {
        let args = commit::Arguments {
            ticket: None,
            ..fake_commit_args()
        };

        for (template_contents, args) in fake_commit_templates(Some(args)) {
            let context = fake_context(GitCommandMock::fake())?;
            let actions = Actions::new(&context);

            let branch_name = context.git.get_branch_name()?;
            let repo_name = context.git.get_repo_name()?;
            setup_db(
                &context.store,
                Some(&fake_branch(Some(branch_name.clone()), Some(repo_name))?),
            )?;

            // Act
            let contents = actions
                .commit(args.clone())
                .expect("Error performing 'commit' action");

            // Assert
            let expected = format!(
                "[{}] {} {}",
                branch_name,
                template_contents,
                args.message.clone().unwrap_or("".into())
            );
            assert_eq!(expected.trim(), contents);

            context.close()?;
        }

        Ok(())
    }

    fn fake_config() -> AppConfig {
        AppConfig {
            commit: CommitConfig {
                templates: fake_template_config(),
            },
        }
    }

    fn fake_context<'a, C: Git>(git: C) -> anyhow::Result<AppContext<C, Sqlite>> {
        let mut connection = Connection::open_in_memory()?;
        db_migrations(
            &mut connection,
            MigrationContext {
                config_path: PathBuf::new(),
                enable_side_effects: false,
                version: None,
            },
        )?;

        let context = AppContext {
            store: Sqlite::new(connection)?,
            config: fake_config(),
            git,
        };

        Ok(context)
    }

    fn setup_db(store: &Sqlite, branch: Option<&Branch>) -> anyhow::Result<()> {
        if let Some(branch) = branch {
            store.persist_branch(branch.into())?;
        }

        Ok(())
    }

    #[derive(Clone)]
    struct GitCommandMock {
        repo: Result<String, String>,
        branch_name: Result<String, String>,
        checkout_res: fn(&str, CheckoutStatus) -> anyhow::Result<()>,
        commit_res: fn(&str) -> anyhow::Result<()>,
    }

    impl GitCommandMock {
        fn fake() -> GitCommandMock {
            GitCommandMock {
                repo: Ok(Faker.fake()),
                branch_name: Ok(Faker.fake()),
                checkout_res: |_, _| Ok(()),
                commit_res: |_| Ok(()),
            }
        }
    }

    impl Git for GitCommandMock {
        fn get_repo_name(&self) -> anyhow::Result<String> {
            self.repo
                .as_ref()
                .map(|s| s.to_owned())
                .map_err(|e| anyhow!(e.to_owned()))
        }

        fn get_branch_name(&self) -> anyhow::Result<String> {
            self.branch_name
                .as_ref()
                .map(|s| s.to_owned())
                .map_err(|e| anyhow!(e.to_owned()))
        }

        fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()> {
            (self.checkout_res)(name, status)
        }

        fn commit(&self, msg: &str) -> anyhow::Result<()> {
            (self.commit_res)(msg)
        }

        fn root_directory(&self) -> anyhow::Result<PathBuf> {
            todo!()
        }
    }

    fn fake_branch(name: Option<String>, repo: Option<String>) -> anyhow::Result<Branch> {
        let name = name.unwrap_or(Faker.fake());
        let repo = repo.unwrap_or(Faker.fake());

        Ok(Branch::new(&name, &repo, None)?)
    }

    fn fake_checkout_args() -> checkout::Arguments {
        checkout::Arguments {
            name: Faker.fake(),
            ticket: Some(Faker.fake()),
        }
    }

    fn fake_commit_args() -> commit::Arguments {
        commit::Arguments {
            template: Faker.fake(),
            ticket: Faker.fake(),
            message: Faker.fake(),
            scope: Faker.fake(),
        }
    }

    fn fake_commit_templates(
        args: Option<commit::Arguments>,
    ) -> Vec<(&'static str, commit::Arguments)> {
        let args = args.unwrap_or_else(fake_commit_args);

        vec![
            (
                "🐛",
                commit::Arguments {
                    template: "bug".into(),
                    ..args.clone()
                },
            ),
            (
                "✨",
                commit::Arguments {
                    template: "feature".into(),
                    ..args.clone()
                },
            ),
            (
                "🧹",
                commit::Arguments {
                    template: "refactor".into(),
                    ..args.clone()
                },
            ),
            (
                "⚠️",
                commit::Arguments {
                    template: "break".into(),
                    ..args.clone()
                },
            ),
            (
                "📦",
                commit::Arguments {
                    template: "deps".into(),
                    ..args.clone()
                },
            ),
            (
                "📖",
                commit::Arguments {
                    template: "docs".into(),
                    ..args.clone()
                },
            ),
            (
                "🧪",
                commit::Arguments {
                    template: "test".into(),
                    ..args.clone()
                },
            ),
        ]
    }

    fn fake_template_config() -> HashMap<String, TemplateConfig> {
        let mut map = HashMap::new();

        map.insert(
            "bug".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "[{ticket_num}] 🐛 {message}".into(),
            },
        );
        map.insert(
            "feature".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "[{ticket_num}] ✨ {message}".into(),
            },
        );
        map.insert(
            "refactor".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "[{ticket_num}] 🧹 {message}".into(),
            },
        );
        map.insert(
            "break".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "[{ticket_num}] ⚠️ {message}".into(),
            },
        );
        map.insert(
            "deps".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "[{ticket_num}] 📦 {message}".into(),
            },
        );
        map.insert(
            "docs".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "[{ticket_num}] 📖 {message}".into(),
            },
        );
        map.insert(
            "test".into(),
            TemplateConfig {
                description: Faker.fake(),
                content: "[{ticket_num}] 🧪 {message}".into(),
            },
        );

        map
    }
}
