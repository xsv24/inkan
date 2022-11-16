use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use rusqlite::{types::Type, Connection, Row};

use crate::domain::{self, Branch};

pub struct Sqlite {
    connection: Connection,
}

impl Sqlite {
    pub fn new(connection: Connection) -> anyhow::Result<Sqlite> {
        connection.execute(
            "CREATE TABLE IF NOT EXISTS branch (
            name TEXT NOT NULL PRIMARY KEY,
            ticket TEXT,
            data BLOB,
            created TEXT NOT NULL
        )",
            (),
        )?;

        Ok(Sqlite { connection })
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.connection
            .close()
            .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

        Ok(())
    }
}

impl domain::Store for Sqlite {
    fn insert_or_update(&self, checkout: &domain::Branch) -> anyhow::Result<()> {
        self.connection
            .execute(
                "REPLACE INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &checkout.name,
                    &checkout.ticket,
                    &checkout.data,
                    &checkout.created.to_rfc3339(),
                ),
            )
            .with_context(|| format!("Failed to insert branch '{}'", &checkout.name))?;

        Ok(())
    }

    fn get(&self, branch: &str, repo: &str) -> anyhow::Result<Branch> {
        let name = format!("{}-{}", repo.trim(), branch.trim());

        let branch = self.connection.query_row(
            "SELECT name, ticket, data, created FROM branch where name = ?",
            [name],
            |row| Branch::try_from(row),
        )?;

        Ok(branch)
    }

    fn close(self) -> anyhow::Result<()> {
        self.connection
            .close()
            .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

        Ok(())
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
    use std::collections::HashMap;

    use crate::{
        adapters::Git,
        app_context::AppContext,
        config::{CommitConfig, Config},
        domain::Store as domain,
    };

    use super::*;
    use directories::ProjectDirs;
    use fake::{Fake, Faker};
    use uuid::Uuid;

    #[test]
    fn insert_or_update_creates_a_new_item_if_not_exists() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let connection = setup_db()?;
        let store = Sqlite::new(connection)?;

        // Act
        store.insert_or_update(&branch)?;

        // Assert
        assert_eq!(branch_count(&store.connection)?, 1);

        let (name, ticket, data, created) = select_branch_row(&store.connection)?;

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
        let connection = setup_db()?;
        let store = Sqlite { connection };

        store.connection.execute(
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
        store.insert_or_update(&updated_branch)?;

        // Assert
        assert_eq!(branch_count(&store.connection)?, 1);

        let (name, ticket, data, created) = select_branch_row(&store.connection)?;

        assert_eq!(updated_branch.name, name);
        assert_eq!(updated_branch.ticket, ticket);
        assert_eq!(updated_branch.data, data);
        assert_eq!(updated_branch.created.to_rfc3339(), created);

        Ok(())
    }

    #[test]
    fn get_retrieves_correct_branch() -> anyhow::Result<()> {
        // Arrange
        let connection = setup_db()?;
        let store = Sqlite { connection };

        let mut branches: HashMap<String, Branch> = HashMap::new();
        let repo = Faker.fake::<String>();

        // Insert random collection of branches.
        for _ in 0..(2..10).fake() {
            let name = Faker.fake::<String>();
            let branch = fake_branch(Some(name.clone()), Some(repo.clone()))?;
            branches.insert(name, branch.clone());

            store.connection.execute(
                "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    &branch.created.to_rfc3339(),
                ),
            )?;
        }

        let context = AppContext {
            store,
            project_dir: fake_project_dir()?,
            commands: Git,
            config: fake_config(),
        };

        let keys = branches.keys().cloned().collect::<Vec<String>>();

        let random_key = keys
            .get((0..keys.len() - 1).fake::<usize>())
            .with_context(|| "Expected to find a matching branch")?;

        let random_branch = branches
            .get(random_key)
            .with_context(|| "Expected to find a matching branch")?;

        // Act
        let branch = context.store.get(&random_key, &repo)?;

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
        let context = AppContext {
            store: Sqlite::new(setup_db()?)?,
            project_dir: fake_project_dir()?,
            commands: Git,
            config: fake_config(),
        };

        // Insert random collection of branches.
        let name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let expected = fake_branch(Some(name.clone()), Some(repo.clone()))?;

        context.store.connection.execute(
            "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
            (
                &expected.name,
                &expected.ticket,
                &expected.data,
                &expected.created.to_rfc3339(),
            ),
        )?;

        // Act
        let actual = context.store.get(&format!(" {}\n", name), &repo)?;

        context.close()?;
        // Assert
        assert_eq!(actual.name, expected.name);

        Ok(())
    }

    fn fake_config() -> Config {
        Config {
            commit: CommitConfig {
                templates: HashMap::new(),
            },
        }
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
