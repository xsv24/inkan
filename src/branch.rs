use anyhow::Context;
use chrono::{DateTime, Utc};
use rusqlite::{types::Type, Connection, Row};
use std::process::Command;

#[derive(Debug)]
pub struct Branch {
    pub name: String,
    pub ticket: String,
    pub data: Option<Vec<u8>>,
    pub created: DateTime<Utc>,
}

impl Branch {
    pub fn new(name: &str, ticket: Option<String>) -> anyhow::Result<Branch> {
        Ok(Branch {
            name: format!("{}-{}", get_repo_name()?, name),
            created: Utc::now(),
            ticket: ticket.unwrap_or(name.into()),
            data: None,
        })
    }

    pub fn insert_into_db(&self, conn: &Connection) -> anyhow::Result<()> {
        conn.execute(
            "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
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

    pub fn get(branch: &str, conn: &Connection) -> anyhow::Result<Branch> {
        let name = format!("{}-{}", get_repo_name()?, branch);

        let branch = conn.query_row(
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

pub fn get_repo_name() -> anyhow::Result<String> {
    let repo_dir: String = String::from_utf8_lossy(
        &Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()?
            .stdout,
    )
    .into();

    let repo = repo_dir
        .split("/")
        .last()
        .context("Failed to get repository name")?;

    Ok(repo.trim().into())
}

pub fn get_branch_name() -> anyhow::Result<String> {
    let branch: String = String::from_utf8_lossy(
        &Command::new("git")
            .args(["branch", "--show-current"])
            .output()?
            .stdout,
    )
    .into();

    Ok(branch.trim().into())
}
