use crate::{branch::Branch, git_commands::get_branch_name, try_convert::TryConvert};
use clap::Args;
use rusqlite::Connection;

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
    pub fn commit_message(&self, template: String, conn: &Connection) -> anyhow::Result<String> {
        let ticket_num = match &self.ticket {
            Some(num) => num.into(),
            None => Branch::get(&get_branch_name().try_convert()?, &conn)?.ticket,
        };

        let contents = template.replace("{ticket_num}", &format!("[{}]", ticket_num));

        let contents = match &self.message {
            Some(message) => contents.replace("{message}", &message),
            None => contents.replace("{message}", ""),
        };

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

    #[test]
    fn commit_message_with_both_args_are_populated() -> anyhow::Result<()> {
        let conn = setup_db(None)?;

        let args = Arguments {
            ticket: Some(Faker.fake()),
            message: Some(Faker.fake()),
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &conn)?;
        let expected = format!("[{}] {}", args.ticket.unwrap(), args.message.unwrap());

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_message_is_replaced_with_empty_str() -> anyhow::Result<()> {
        let conn = setup_db(None)?;

        let args = Arguments {
            ticket: Some(Faker.fake()),
            message: None,
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &conn)?;
        let expected = format!("[{}] ", args.ticket.unwrap());

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn commit_template_ticket_num_is_replaced_with_branch_name() -> anyhow::Result<()> {
        let name = get_branch_name().try_convert()?;
        let branch = Branch::new(&name, None)?;

        let conn = setup_db(Some(&branch))?;

        let args = Arguments {
            ticket: None,
            message: Some(Faker.fake()),
        };

        let actual = args.commit_message("{ticket_num} {message}".into(), &conn)?;
        let expected = format!("[{}] {}", name, args.message.unwrap());

        assert_eq!(actual, expected);

        Ok(())
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
