use anyhow::Context as anyhow_context;
use chrono::{DateTime, Utc};
use rusqlite::{types::Type, Connection, Row};

use crate::{context::Context, git_commands::GitCommands};

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub ticket: String,
    pub data: Option<Vec<u8>>,
    pub created: DateTime<Utc>,
}

impl Branch {
    pub fn new(name: &str, repo: &str, ticket: Option<String>) -> anyhow::Result<Branch> {
        Ok(Branch {
            name: format!("{}-{}", repo.trim(), name.trim()),
            created: Utc::now(),
            ticket: ticket.unwrap_or_else(|| name.into()),
            data: None,
        })
    }

    pub fn insert_or_update(&self, conn: &Connection) -> anyhow::Result<()> {
        conn.execute(
            "REPLACE INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
            (
                &self.name,
                &self.ticket,
                &self.data,
                &self.created.to_rfc3339(),
            ),
        )
        .with_context(|| format!("Failed to insert branch '{}'", &self.name))?;

        Ok(())
    }

    pub fn get<C: GitCommands>(
        branch: &str,
        repo: &str,
        connext: &Context<C>,
    ) -> anyhow::Result<Branch> {
        let name = format!("{}-{}", repo.trim(), branch.trim());

        let branch = connext.connection.query_row(
            "SELECT name, ticket, data, created FROM branch where name = ?",
            [name],
            |row| Branch::try_from(row),
        )?;

        Ok(branch)
    }
}

impl<'a> TryFrom<&Row<'a>> for Branch {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let date = value.get::<usize, String>(3)?;
        let created = DateTime::parse_from_rfc3339(&date)
            .map_err(|e| {
                dbg!("{}", e);
                rusqlite::Error::InvalidColumnType(
                    0,
                    "Failed to convert string to DateTime".into(),
                    Type::Text,
                )
            })?
            .with_timezone(&Utc);

        let branch = Branch {
            name: value.get(0)?,
            ticket: value.get(1)?,
            data: value.get(2)?,
            created,
        };

        Ok(branch)
    }
}

#[cfg(test)]
mod tests {
    use crate::git_commands::Git;

    use super::*;

    use anyhow::Context as anyhow_context;
    use chrono::Utc;
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use rusqlite::Connection;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn creating_branch_with_ticket_populates_correctly() -> anyhow::Result<()> {
        // Arrange
        let now = Utc::now();
        let repo = Faker.fake::<String>();
        let name = Faker.fake::<String>();
        let ticket = Faker.fake::<String>();

        // Act
        let branch = Branch::new(&name, &repo, Some(ticket.clone()))?;

        // Assert
        assert_eq!(branch.name, format!("{}-{}", &repo, &name));
        assert_eq!(branch.ticket, ticket);
        assert!(branch.created > now);
        assert_eq!(branch.data, None);

        Ok(())
    }

    #[test]
    fn creating_branch_without_ticket_defaults_to_name() -> anyhow::Result<()> {
        // Arrange
        let now = Utc::now();
        let name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();

        // Act
        let branch = Branch::new(&name, &repo, None)?;

        // Assert
        assert_eq!(branch.name, format!("{}-{}", &repo, &name));
        assert_eq!(branch.ticket, name);
        assert!(branch.created > now);
        assert_eq!(branch.data, None);

        Ok(())
    }

    #[test]
    fn branch_name_is_trimmed() -> anyhow::Result<()> {
        // Arrange
        let now = Utc::now();
        let name = format!("{}\n", Faker.fake::<String>());
        let ticket = Faker.fake::<String>();
        let repo = Faker.fake::<String>();

        // Act
        let branch = Branch::new(&name, &repo, Some(ticket.clone()))?;

        // Assert
        assert_eq!(branch.name, format!("{}-{}", &repo.trim(), &name.trim()));
        assert_eq!(branch.ticket, ticket);
        assert!(branch.created > now);
        assert_eq!(branch.data, None);

        Ok(())
    }

    #[test]
    fn insert_or_update_creates_a_new_item_if_not_exists() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let conn = setup_db()?;

        // Act
        branch.insert_or_update(&conn)?;

        // Assert
        assert_eq!(branch_count(&conn)?, 1);

        let (name, ticket, data, created) = select_branch_row(&conn)?;

        assert_eq!(branch.name, name);
        assert_eq!(branch.ticket, ticket);
        assert_eq!(branch.data, data);
        assert_eq!(branch.created.to_rfc3339(), created);

        Ok(())
    }

    #[test]
    fn insert_or_update_updates_an_existing_item() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let conn = setup_db()?;

        conn.execute(
            "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
            (
                &branch.name,
                &branch.ticket,
                &branch.data,
                &branch.created.to_rfc3339(),
            ),
        )?;

        let updated_branch = Branch {
            name: branch.name,
            ..fake_branch(None, None)?
        };

        // Act
        updated_branch.insert_or_update(&conn)?;

        // Assert
        assert_eq!(branch_count(&conn)?, 1);

        let (name, ticket, data, created) = select_branch_row(&conn)?;

        assert_eq!(updated_branch.name, name);
        assert_eq!(updated_branch.ticket, ticket);
        assert_eq!(updated_branch.data, data);
        assert_eq!(updated_branch.created.to_rfc3339(), created);

        Ok(())
    }

    #[test]
    fn get_retrieves_correct_branch() -> anyhow::Result<()> {
        // Arrange
        let context = Context {
            connection: setup_db()?,
            project_dir: fake_project_dir()?,
            commands: Git,
        };

        let mut branches: HashMap<String, Branch> = HashMap::new();
        let repo = Faker.fake::<String>();

        // Insert random collection of branches.
        for _ in 0..(2..10).fake() {
            let name = Faker.fake::<String>();
            let branch = fake_branch(Some(name.clone()), Some(repo.clone()))?;
            branches.insert(name, branch.clone());

            context.connection.execute(
                "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    &branch.created.to_rfc3339(),
                ),
            )?;
        }

        let keys = branches.keys().cloned().collect::<Vec<String>>();

        let random_key = keys
            .get((0..keys.len() - 1).fake::<usize>())
            .with_context(|| "Expected to find a matching branch")?;

        let random_branch = branches
            .get(random_key)
            .with_context(|| "Expected to find a matching branch")?;

        // Act
        let branch = Branch::get(&random_key, &repo, &context)?;

        context.close()?;
        // Assert
        assert_eq!(random_branch.name, branch.name);
        assert_eq!(random_branch.ticket, branch.ticket);
        assert_eq!(random_branch.data, branch.data);
        assert_eq!(
            random_branch.created.to_rfc3339(),
            branch.created.to_rfc3339()
        );

        Ok(())
    }

    #[test]
    fn get_trims_name_before_retrieving() -> anyhow::Result<()> {
        // Arrange
        let context = Context {
            connection: setup_db()?,
            project_dir: fake_project_dir()?,
            commands: Git,
        };

        // Insert random collection of branches.
        let name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let expected = fake_branch(Some(name.clone()), Some(repo.clone()))?;

        context.connection.execute(
            "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
            (
                &expected.name,
                &expected.ticket,
                &expected.data,
                &expected.created.to_rfc3339(),
            ),
        )?;

        // Act
        let actual = Branch::get(&format!(" {}\n", name), &repo, &context)?;

        context.close()?;
        // Assert
        assert_eq!(actual.name, expected.name);

        Ok(())
    }

    fn fake_branch(name: Option<String>, repo: Option<String>) -> anyhow::Result<Branch> {
        let name = name.unwrap_or(Faker.fake());
        let repo = repo.unwrap_or(Faker.fake());
        let ticket: Option<String> = Faker.fake();

        Ok(Branch::new(&name, &repo, ticket)?)
    }

    fn fake_project_dir() -> anyhow::Result<ProjectDirs> {
        let dirs = ProjectDirs::from(&format!("{}", Uuid::new_v4()), "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        Ok(dirs)
    }

    fn select_branch_row(
        conn: &Connection,
    ) -> anyhow::Result<(String, String, Option<Vec<u8>>, String)> {
        let (name, ticket, data, created) = conn.query_row("SELECT * FROM branch", [], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<Vec<u8>>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        Ok((name, ticket, data, created))
    }

    fn branch_count(conn: &Connection) -> anyhow::Result<i32> {
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM branch", [], |row| row.get(0))?;

        Ok(count)
    }

    fn setup_db() -> anyhow::Result<Connection> {
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

        Ok(conn)
    }
}
