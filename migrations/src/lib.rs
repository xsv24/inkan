use std::path::PathBuf;

use anyhow::Context;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

#[derive(Clone)]
pub struct MigrationContext {
    pub config_path: PathBuf,
    pub enable_side_effects: bool,
    pub version: Option<usize>,
}

pub fn db_migrations(
    connection: &mut Connection,
    context: MigrationContext,
) -> anyhow::Result<Migrations> {
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE IF NOT EXISTS branch (
                name TEXT NOT NULL PRIMARY KEY,
                ticket TEXT,
                data BLOB,
                created TEXT NOT NULL
            );",
        )
        .down("DROP TABLE branch;"),
        M::up(
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT NOT NULL PRIMARY KEY,
                path TEXT NOT NULL,
                status TEXT NOT NULL 
            );",
        )
        .down("DROP TABLE config;"),
    ]);

    if let Some(version) = context.version {
        migrations
            .to_version(connection, version)
            .with_context(|| format!("Failed to apply migration version '{}'", version))?;
    } else {
        migrations
            .to_latest(connection)
            .context("Failed to apply latest migration")?;
    };

    let version: usize = migrations
        .current_version(connection)
        .context("Failed to get current migration version.")?
        .into();

    // Had to do this dynamically since the default path will differ between operating systems.
    if context.enable_side_effects && version == 2 {
        let config_default = context
            .config_path
            .to_str()
            .context("Expected valid default config path.")?;

        connection.execute(
            "INSERT OR IGNORE INTO config (key, path, status) VALUES (?1, ?2, ?3);",
            ["default", config_default, "ACTIVE"],
        )?;
    }

    println!("git-kit migration version '{}'.", version);

    Ok(migrations)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use std::path::Path;

    use crate::{db_migrations, MigrationContext};

    fn arrange(version: usize) -> (Connection, Vec<String>, MigrationContext) {
        let mut connection = Connection::open_in_memory().unwrap();

        let defaults = MigrationContext {
            config_path: Path::new(".").to_owned(),
            enable_side_effects: true,
            version: Some(version),
        };

        let migrations = db_migrations(&mut connection, defaults.clone()).unwrap();
        // Validate migrations where applied.
        assert!(migrations.validate().is_ok());

        let tables = get_table_names(&mut connection);

        (connection, tables, defaults)
    }

    #[test]
    fn verify_migration_1() {
        let (_, tables, _) = arrange(1);

        assert_eq!(tables.len(), 1);
        assert!(tables.contains(&"branch".to_string()));
    }

    #[test]
    fn verify_migration_2() {
        let (connection, tables, _) = arrange(2);

        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"branch".to_string()));
        assert!(tables.contains(&"config".to_string()));

        let mut default_config = connection
            .prepare("SELECT * FROM config WHERE key='default'")
            .unwrap();

        let config = default_config
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<Vec<(String, String, String)>, _>>()
            .unwrap();

        assert_eq!(config.len(), 1);
        let (name, path, status) = &config[0];
        assert_eq!(name, "default");
        assert_eq!(path, ".");
        assert_eq!(status, "ACTIVE")
    }

    fn get_table_names(connection: &mut Connection) -> Vec<String> {
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_schema WHERE type='table'")
            .unwrap();

        let tables = statement
            .query_map([], |row| {
                let table: String = row.get(0)?;
                Ok(table)
            })
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap();

        tables
    }
}
