use std::path::PathBuf;

use anyhow::Context;
use rusqlite::{Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};

use crate::domain::models::ConfigKey;

#[derive(Clone)]
pub struct DefaultConfig {
    pub default: PathBuf,
    pub conventional: PathBuf,
}

#[derive(Clone)]
pub struct MigrationContext {
    pub default_configs: Option<DefaultConfig>,
    pub version: usize,
}

pub fn db_migrations(
    connection: &mut Connection,
    context: MigrationContext,
) -> anyhow::Result<Migrations> {
    log::info!("Run migrations for version {}", context.version);

    // TODO: Move the migrations into a directory https://github.com/cljoly/rusqlite_migration/blob/08dc155cdedc83a2aef1017e95315fa6ca501daf/examples/from-directory/migrations/01-friend_car/up.sql#L1
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

    let current_version: usize = migrations
        .current_version(connection)
        .context("Failed to get current migration version.")?
        .into();

    migrations
        .to_version(connection, context.version)
        .with_context(|| format!("Failed to apply migration version '{}'", context.version))?;

    if current_version == context.version {
        log::info!("Using cached default templates due to no version change");
        return Ok(migrations);
    }

    if let Some(config_paths) = context.default_configs {
        migrate_default_configurations(config_paths, connection, context.version)?;
    }

    log::info!("Migrations complete for version '{}'.", context.version);

    Ok(migrations)
}

fn migrate_default_configurations(
    default_configs: DefaultConfig,
    connection: &mut Connection,
    version: usize,
) -> anyhow::Result<()> {
    if version < 2 {
        // 'config' table does not exist yet.
        return Ok(());
    }

    let active_key = connection
        .query_row(
            "SELECT key FROM config WHERE status == 'ACTIVE'",
            [],
            |row| Ok(ConfigKey::from(row.get::<_, String>(0)?.as_str())),
        )
        .optional()?;

    let active_key = active_key.unwrap_or(ConfigKey::Default);
    migrate_default_configuration(
        connection,
        &active_key,
        ConfigKey::Default,
        default_configs.default,
    )?;
    migrate_default_configuration(
        connection,
        &active_key,
        ConfigKey::Conventional,
        default_configs.conventional,
    )?;

    Ok(())
}

fn migrate_default_configuration(
    connection: &mut Connection,
    active_key: &ConfigKey,
    key: ConfigKey,
    path: PathBuf,
) -> anyhow::Result<()> {
    let path = path
        .to_str()
        .context("Expected valid default config path.")?;

    let status = if &key == active_key {
        "ACTIVE"
    } else {
        "DISABLED"
    };

    connection.execute(
        "REPLACE INTO config (key, path, status) VALUES (?1, ?2, ?3)",
        [&String::from(key), path, status],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use std::path::Path;

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
    fn templates_respect_currently_active_config() {
        // Arrange
        let context = MigrationContext {
            default_configs: Some(DefaultConfig {
                default: Path::new("default.yml").to_owned(),
                conventional: Path::new("conventional.yml").to_owned(),
            }),
            version: 2,
        };

        // Migrate to version 1 with default set to active
        let (connection, ..) = arrange(context.clone());

        // Assert test setup correctly
        let active_key: String = connection
            .query_row(
                "SELECT key FROM config WHERE status == 'ACTIVE'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(active_key, "default");

        // Update the 'active' configuration
        connection
            .execute("UPDATE config SET status = 'DISABLED'", [])
            .unwrap();

        connection
            .execute(
                "REPLACE INTO config (key, path, status) VALUES (?1, ?2, ?3)",
                [
                    "conventional",
                    Path::new("conventional.yml").to_str().unwrap(),
                    "ACTIVE",
                ],
            )
            .unwrap();

        // Act
        let mut connection = connection;
        db_migrations(
            &mut connection,
            MigrationContext {
                version: 3,
                ..context
            },
        )
        .unwrap();

        // Assert
        let default_configs = get_default_configs(&connection);
        let default_config = default_configs.get(1).unwrap();
        let conventional_config = default_configs.get(0).unwrap();

        assert_eq!(conventional_config.2, "ACTIVE");
        assert_eq!(default_config.2, "DISABLED");
    }

    #[test]
    fn verify_migration_1() {
        let (_, tables, _) = arrange(MigrationContext {
            default_configs: None,
            version: 1,
        });

        assert_eq!(tables.len(), 1);
        assert!(tables.contains(&"branch".to_string()));
    }

    #[test]
    fn verify_migration_2() {
        let context = MigrationContext {
            default_configs: Some(DefaultConfig {
                default: Path::new("default.yml").to_owned(),
                conventional: Path::new("conventional.yml").to_owned(),
            }),
            version: 2,
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
    fn verify_migration_3_and_4() {
        let context = MigrationContext {
            default_configs: Some(DefaultConfig {
                default: Path::new("default.yml").to_owned(),
                conventional: Path::new("conventional.yml").to_owned(),
            }),
            version: 4,
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
            version: 2,
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
            version: 4,
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
