use std::path::PathBuf;

use anyhow::Context;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

#[derive(Clone)]
pub struct DefaultConfig {
    pub default: PathBuf,
    pub conventional: PathBuf,
}

#[derive(Clone)]
pub struct MigrationContext {
    pub default_configs: Option<DefaultConfig>,
    pub version: Option<usize>,
}

#[allow(dead_code)]
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
        M::up("ALTER TABLE branch ADD COLUMN link TEXT;")
            .down("ALTER TABLE branch DROP COLUMN link;"),
        M::up("ALTER TABLE branch ADD COLUMN scope TEXT;")
            .down("ALTER TABLE branch DROP COLUMN scope;"),
    ]);

    if let Some(version) = context.version {
        migrations
            .to_version(connection, version)
            .with_context(|| format!("Failed to apply migration version '{version}'"))?;
    } else {
        migrations
            .to_latest(connection)
            .context("Failed to apply latest migration")?;
    };

    let version: usize = migrations
        .current_version(connection)
        .context("Failed to get current migration version.")?
        .into();

    if let Some(config_paths) = context.default_configs {
        migrate_default_configuration(config_paths, connection, version)?;
    }

    println!("git-kit migration version '{version}'.");

    Ok(migrations)
}

fn migrate_default_configuration(
    default_configs: DefaultConfig,
    connection: &mut Connection,
    version: usize,
) -> anyhow::Result<()> {
    if version >= 2 {
        let config_default = default_configs
            .default
            .to_str()
            .context("Expected valid default config path.")?;

        connection.execute(
            "INSERT OR IGNORE INTO config (key, path, status) VALUES (?1, ?2, ?3);",
            ["default", config_default, "ACTIVE"],
        )?;
    }

    if version >= 3 {
        // Update to latest changed path for the default configuration. '.git-kit.yml' -> 'templates/default.yml'
        let default_path = default_configs
            .default
            .to_str()
            .context("Expected valid default config path.")?;
        connection.execute(
            "UPDATE config SET path = ?1 WHERE key='default'",
            [default_path],
        )?;

        // Insert the new 'conventional' default configuration.
        // Note there is a possibility for conflicts with user custom configs and the name 'conventional' which be replaced.
        // User custom configs using the name 'conventional' will have to re-added under a new name via the 'config add' command.
        let conventional_path = default_configs
            .conventional
            .to_str()
            .context("Expected valid conventional config path.")?;
        connection.execute(
            "REPLACE INTO config (key, path, status) VALUES ('conventional', ?1, 'DISABLED');",
            [conventional_path],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use std::path::{Path, PathBuf};

    use crate::migrations::{db_migrations, DefaultConfig, MigrationContext};

    fn arrange(context: MigrationContext) -> (Connection, Vec<String>, MigrationContext) {
        let mut connection = Connection::open_in_memory().unwrap();

        let migrations = db_migrations(&mut connection, context.clone()).unwrap();
        // Validate migrations where applied.
        assert!(migrations.validate().is_ok());

        let tables = get_table_names(&mut connection);

        (connection, tables, context)
    }

    #[test]
    fn verify_migration_1() {
        let (_, tables, _) = arrange(MigrationContext {
            default_configs: None,
            version: Some(1),
        });

        assert_eq!(tables.len(), 1);
        assert!(tables.contains(&"branch".to_string()));
    }

    #[test]
    fn verify_migration_2() {
        let context = MigrationContext {
            default_configs: Some(DefaultConfig {
                default: Path::new("default.yml").to_owned(),
                conventional: PathBuf::new(),
            }),
            version: Some(2),
        };
        let (connection, tables, _) = arrange(context);

        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"branch".to_string()));
        assert!(tables.contains(&"config".to_string()));

        let default_configs = get_default_configs(&connection);

        assert_eq!(default_configs.len(), 1);
        let (name, path, status) = default_configs.get(0).unwrap();

        assert_eq!(name, "default");
        assert_eq!(path, "default.yml");
        assert_eq!(status, "ACTIVE");
    }

    #[test]
    fn verify_migration_3_and_4() {
        let context = MigrationContext {
            default_configs: Some(DefaultConfig {
                default: Path::new("default.yml").to_owned(),
                conventional: Path::new("conventional.yml").to_owned(),
            }),
            version: Some(4),
        };
        let (connection, tables, _) = arrange(context);

        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"branch".to_string()));
        assert!(tables.contains(&"config".to_string()));

        let default_configs = get_default_configs(&connection);

        assert_eq!(default_configs.len(), 2);
        let default_config = default_configs.get(1).unwrap();
        let conventional_config = default_configs.get(0).unwrap();

        assert_eq!(default_config.0, "default");
        assert_eq!(default_config.1, "default.yml");
        assert_eq!(default_config.2, "ACTIVE");

        assert_eq!(conventional_config.0, "conventional");
        assert_eq!(conventional_config.1, "conventional.yml");
        assert_eq!(conventional_config.2, "DISABLED");
    }

    #[test]
    fn verify_any_custom_user_conventional_is_replaced_via_the_new_default_at_migration_3_and_4() {
        // Arrange
        // Apply a previous version first.
        let context = MigrationContext {
            default_configs: None,
            version: Some(2),
        };
        let (connection, ..) = arrange(context);

        // Insert custom user 'conventional' config and assure dummy data is there.
        connection.execute("INSERT INTO config (key, path, status) VALUES ('conventional', 'custom_path.yml', 'ACTIVE');", []).unwrap();
        let default_configs = get_default_configs(&connection);
        let conventional_config = default_configs.get(0).unwrap();

        assert_eq!(conventional_config.0, "conventional");
        assert_eq!(conventional_config.1, "custom_path.yml");
        assert_eq!(conventional_config.2, "ACTIVE");

        connection.close().unwrap();

        // Act
        let context = MigrationContext {
            default_configs: Some(DefaultConfig {
                default: Path::new("default.yml").to_owned(),
                conventional: Path::new("conventional.yml").to_owned(),
            }),
            version: Some(4),
        };

        let (connection, ..) = arrange(context);

        // Assert
        let default_configs = get_default_configs(&connection);
        let conventional_config = default_configs.get(0).unwrap();

        assert_eq!(conventional_config.0, "conventional");
        assert_eq!(conventional_config.1, "conventional.yml");
        assert_eq!(conventional_config.2, "DISABLED");
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

    fn get_default_configs(connection: &Connection) -> Vec<(String, String, String)> {
        let mut default_configs = connection
            .prepare("SELECT * FROM config WHERE key='default' OR key='conventional'")
            .unwrap();

        default_configs
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<Vec<(String, String, String)>, _>>()
            .unwrap()
    }
}
