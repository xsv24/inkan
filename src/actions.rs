use crate::{
    branch::Branch,
    cli::{Checkout, Current},
    context::Context,
    git_commands::{CheckoutStatus, GitCommands},
    template::Template,
};

pub trait Actions<C: GitCommands> {
    /// Actions on a context update on the current branch.
    fn current(&self, current: Current) -> anyhow::Result<()>;
    /// Actions on a checkout of a new or existing branch.
    fn checkout(&self, checkout: Checkout) -> anyhow::Result<()>;
    /// Actions on a commit.
    fn commit(&self, template: Template) -> anyhow::Result<()>;
}

pub struct CommandActions<'a, C: GitCommands> {
    context: &'a Context<C>,
}

impl<'a, C: GitCommands> CommandActions<'a, C> {
    pub fn new(context: &Context<C>) -> anyhow::Result<CommandActions<C>> {
        // TODO: Move into build script ?
        context.connection.execute(
            "CREATE TABLE IF NOT EXISTS branch (
            name TEXT NOT NULL PRIMARY KEY,
            ticket TEXT,
            data BLOB,
            created TEXT NOT NULL
        )",
            (),
        )?;

        Ok(CommandActions { context })
    }
}

impl<'a, C: GitCommands> Actions<C> for CommandActions<'a, C> {
    fn current(&self, current: Current) -> anyhow::Result<()> {
        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = self.context.commands.get_repo_name()?;
        let branch_name = self.context.commands.get_branch_name()?;

        let branch = Branch::new(&branch_name, &repo_name, Some(current.ticket.clone()))?;
        branch.insert_or_update(&self.context.connection)?;

        Ok(())
    }

    fn checkout(&self, checkout: Checkout) -> anyhow::Result<()> {
        // Attempt to create branch
        let create = self
            .context
            .commands
            .checkout(&checkout.name, CheckoutStatus::New);

        // If the branch already exists check it out
        if create.is_err() {
            self.context
                .commands
                .checkout(&checkout.name, CheckoutStatus::Existing)?;
        }

        // We want to store the branch name against and ticket number
        // So whenever we commit we get the ticket number from the branch
        let repo_name = self.context.commands.get_repo_name()?;
        let branch = Branch::new(&checkout.name, &repo_name, checkout.ticket.clone())?;
        branch.insert_or_update(&self.context.connection)?;

        Ok(())
    }

    fn commit(&self, template: Template) -> anyhow::Result<()> {
        let contents = template.commit(&self.context)?;
        self.context.commands.commit(&contents)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use anyhow::Context as anyhow_context;
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use rusqlite::Connection;
    use uuid::Uuid;

    use crate::branch::Branch;
    use crate::cli::Checkout;
    use crate::cli::Current;
    use crate::git_commands::CheckoutStatus;
    use crate::{context::Context, git_commands::GitCommands};

    use super::*;

    #[test]
    fn checkout_success_with_ticket() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = Checkout {
            name: Faker.fake::<String>(),
            ticket: Some(Faker.fake()),
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = Branch::get(&command.name, &repo, &context)?;
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

        let command = Checkout {
            name: Faker.fake::<String>(),
            ticket: Some(Faker.fake()),
        };

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
        let actions = CommandActions::new(&context)?;

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = Branch::get(&command.name, &repo, &context)?;
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
        let command = Checkout {
            name: Faker.fake::<String>(),
            ticket: Some(Faker.fake()),
        };

        let repo = Faker.fake::<String>();
        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            checkout_res: |_, _| Err(anyhow!("failed to create or checkout existing branch!")),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        let result = actions.checkout(command.clone());

        // Assert
        assert!(result.is_err());

        let error = Branch::get(&command.name, &repo, &context)
            .expect_err("Expected error as there should be no stored branches.");

        assert_eq!(error.to_string(), "Query returned no rows");

        context.close()?;

        Ok(())
    }

    #[test]
    fn checkout_success_without_ticket_uses_branch_name() -> anyhow::Result<()> {
        // Arrange
        let repo = Faker.fake::<String>();

        let command = Checkout {
            name: Faker.fake::<String>(),
            ticket: None,
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(command.name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        actions.checkout(command.clone())?;

        // Assert
        let branch = Branch::get(&command.name, &repo, &context)?;
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
        let command = Current {
            ticket: Faker.fake(),
        };

        let git_commands = GitCommandMock {
            repo: Ok(repo.clone()),
            branch_name: Ok(branch_name.clone()),
            ..GitCommandMock::fake()
        };

        let context = fake_context(git_commands.clone())?;
        let actions = CommandActions::new(&context)?;

        // Act
        actions.current(command.clone())?;

        // Assert
        let branch = Branch::get(&branch_name, &repo, &context)?;
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

    fn fake_project_dir() -> anyhow::Result<ProjectDirs> {
        let dirs = ProjectDirs::from(&format!("{}", Uuid::new_v4()), "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        Ok(dirs)
    }

    fn fake_context<C: GitCommands>(commands: C) -> anyhow::Result<Context<C>> {
        let conn = Connection::open_in_memory()?;

        let context = Context {
            connection: conn,
            project_dir: fake_project_dir()?,
            commands,
        };

        Ok(context)
    }

    #[derive(Clone)]
    struct GitCommandMock {
        repo: Result<String, String>,
        branch_name: Result<String, String>,
        checkout_res: fn(&str, CheckoutStatus) -> anyhow::Result<()>,
        commit_res: Result<(), String>,
    }

    impl GitCommandMock {
        fn fake() -> GitCommandMock {
            GitCommandMock {
                repo: Ok(Faker.fake()),
                branch_name: Ok(Faker.fake()),
                checkout_res: |_, _| Ok(()),
                commit_res: Ok(()),
            }
        }
    }

    impl GitCommands for GitCommandMock {
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

        fn checkout(
            &self,
            name: &str,
            status: crate::git_commands::CheckoutStatus,
        ) -> anyhow::Result<()> {
            (self.checkout_res)(name, status)
        }

        fn commit(&self, _msg: &str) -> anyhow::Result<()> {
            self.commit_res
                .as_ref()
                .map(|s| s.to_owned())
                .map_err(|e| anyhow!(e.to_owned()))
        }
    }
}
