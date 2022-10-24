use crate::{branch::Branch, context::Context, git_commands::GitCommands};
use clap::Args;

#[derive(Debug, Args, PartialEq, Eq, Clone)]
pub struct Arguments {
    /// Issue ticket number related to the commit.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    /// Message for the commit.
    #[clap(short, long, value_parser)]
    pub message: Option<String>,
}

impl Arguments {
    pub fn commit_message<C: GitCommands>(
        &self,
        template: String,
        context: &Context<C>,
    ) -> anyhow::Result<String> {
        let ticket = self.ticket.as_ref().map(|num| num.trim());

        let ticket_num = match ticket {
            Some(num) => match (num, num.len()) {
                (_, 0) => None,
                (value, _) => Some(value.into()),
            },
            None => Branch::get(
                &context.commands.get_branch_name()?,
                &context.commands.get_repo_name()?,
                context,
            )
            .map_or(None, |branch| Some(branch.ticket)),
        };

        let contents = if let Some(ticket) = ticket_num {
            template.replace("{ticket_num}", &format!("[{}]", ticket))
        } else {
            template.replace("{ticket_num}", "").trim().into()
        };

        let contents = match &self.message {
            Some(message) => contents.replace("{message}", message),
            None => contents.replace("{message}", ""),
        };

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use rusqlite::Connection;
    use uuid::Uuid;

    use super::*;

    #[derive(Clone)]
    struct TestCommand {
        repo: String,
        branch_name: String,
    }

    impl TestCommand {
        fn fake() -> TestCommand {
            TestCommand {
                repo: Faker.fake(),
                branch_name: Faker.fake(),
            }
        }
    }

    impl GitCommands for TestCommand {
        fn get_repo_name(&self) -> anyhow::Result<String> {
            Ok(self.repo.to_owned())
        }

        fn get_branch_name(&self) -> anyhow::Result<String> {
            Ok(self.branch_name.to_owned())
        }

        fn checkout(
            &self,
            _name: &str,
            _status: crate::git_commands::CheckoutStatus,
        ) -> anyhow::Result<()> {
            todo!()
        }

        fn commit(&self, _msg: &str) -> anyhow::Result<()> {
            todo!()
        }
    }

    #[test]
    fn empty_ticket_num_removes_square_brackets() -> anyhow::Result<()> {
        let context = Context {
            connection: setup_db(None)?,
            project_dir: fake_project_dir()?,
            commands: TestCommand::fake(),
        };

        let args = Arguments {
            ticket: Some("".into()),
            message: Some(Faker.fake()),
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("{}", args.message.unwrap());

        context.close()?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn when_ticket_num_is_empty_square_brackets_are_removed() -> anyhow::Result<()> {
        for ticket in [Some("".into()), Some("   ".into()), None] {
            let context = Context {
                connection: setup_db(None)?,
                project_dir: fake_project_dir()?,
                commands: TestCommand::fake(),
            };

            let args = Arguments {
                ticket,
                message: Some(Faker.fake()),
            };

            let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
            let expected = format!("{}", args.message.unwrap());

            context.close()?;
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn commit_message_with_both_args_are_populated() -> anyhow::Result<()> {
        let context = Context {
            connection: setup_db(None)?,
            project_dir: fake_project_dir()?,
            commands: TestCommand::fake(),
        };

        let args = Arguments {
            ticket: Some(Faker.fake()),
            message: Some(Faker.fake()),
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("[{}] {}", args.ticket.unwrap(), args.message.unwrap());

        context.close()?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_message_is_replaced_with_empty_str() -> anyhow::Result<()> {
        let context = Context {
            connection: setup_db(None)?,
            project_dir: fake_project_dir()?,
            commands: TestCommand::fake(),
        };

        let args = Arguments {
            ticket: Some(Faker.fake()),
            message: None,
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("[{}] ", args.ticket.unwrap());

        context.close()?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_ticket_num_is_replaced_with_branch_name() -> anyhow::Result<()> {
        let commands = TestCommand::fake();

        let branch = Branch::new(&commands.branch_name, &commands.repo, None)?;

        let context = Context {
            connection: setup_db(Some(&branch))?,
            commands: commands.clone(),
            project_dir: fake_project_dir()?,
        };

        let args = Arguments {
            ticket: None,
            message: Some(Faker.fake()),
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &context)?;
        let expected = format!("[{}] {}", &commands.branch_name, args.message.unwrap());

        context.close()?;

        assert_eq!(actual, expected);

        Ok(())
    }

    fn fake_project_dir() -> anyhow::Result<ProjectDirs> {
        let dirs = ProjectDirs::from(&format!("{}", Uuid::new_v4()), "xsv24", "git-kit")
            .expect("Failed to retrieve 'git-kit' config");

        Ok(dirs)
    }

    fn setup_db(branch: Option<&Branch>) -> anyhow::Result<Connection> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE branch (
                name TEXT NOT NULL PRIMARY KEY,
                ticket TEXT,
                data BLOB,
                created TEXT NOT NULL
            )",
            (),
        )?;

        if let Some(branch) = branch {
            conn.execute(
                "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    branch.created.to_rfc3339(),
                ),
            )?;
        }

        Ok(conn)
    }
}
