use anyhow::anyhow;
use rusqlite::{Connection, Transaction};

use crate::domain::{
    self,
    errors::PersistError,
    models::{Branch, Config, ConfigKey, ConfigStatus},
};

pub struct Sqlite {
    connection: Connection,
}

impl Sqlite {
    pub fn new(connection: Connection) -> Sqlite {
        Sqlite { connection }
    }

    pub fn transaction(&mut self) -> Result<Transaction, PersistError> {
        let transaction = self.connection.transaction().map_err(|e| {
            log::error!("Failed to open transaction for sqlite db: {}", &e);
            PersistError::Unknown(e.into())
        })?;

        Ok(transaction)
    }
}

impl domain::adapters::Store for Sqlite {
    fn persist_branch(&self, branch: &Branch) -> Result<(), PersistError> {
        log::info!(
            "insert or update for '{}' branch with ticket '{}'",
            branch.name,
            branch.ticket
        );

        self.connection
            .execute(
                "REPLACE INTO branch (name, ticket, data, created, link, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    &branch.created.to_rfc3339(),
                    &branch.link,
                    &branch.scope
                ),
            )
            .map_err(|e| PersistError::into_branch_error(format!("Failed to update branch '{}'", branch.name), e))?;

        Ok(())
    }

    fn get_branch(&self, branch: &str, repo: &str) -> Result<Branch, PersistError> {
        let name = format!("{}-{}", repo.trim(), branch.trim());

        log::info!(
            "retrieve branch with ticket for branch '{}' and repo '{}'",
            name,
            repo
        );

        let branch = self
            .connection
            .query_row(
                "SELECT name, ticket, data, created, link, scope FROM branch where name = ?",
                [name],
                |row| Branch::try_from(row),
            )
            .map_err(|e| {
                PersistError::into_branch_error("Failed to retrieve branch '{name}'", e)
            })?;

        Ok(branch)
    }

    fn persist_config(&self, config: &Config) -> Result<(), PersistError> {
        let key: String = config.key.clone().into();
        let path: String = config.path.to_string();

        log::info!("insert or update user config '{}' path '{}'", &key, &path);

        self.connection
            .execute(
                "REPLACE INTO config (key, path, status) VALUES (?1, ?2, ?3)",
                (key, path, String::from(ConfigStatus::Disabled)),
            )
            .map_err(|e| PersistError::into_config_error("Failed to update config.", e))?;

        Ok(())
    }

    fn set_active_config(&mut self, key: &ConfigKey) -> Result<Config, PersistError> {
        let transaction = self.transaction()?;
        let key: String = key.to_owned().into();

        let (active, disabled) = (
            String::from(ConfigStatus::Active),
            String::from(ConfigStatus::Disabled),
        );

        // Check the record actually exists before changing statuses.
        transaction
            .query_row("SELECT * FROM config where key = ?1;", [&key], |_| Ok(()))
            .map_err(|e| {
                PersistError::into_config_error(format!("Configuration '{key}' does not exist."), e)
            })?;

        // Update any 'ACTIVE' config to 'DISABLED'
        transaction
            .execute(
                "UPDATE config SET status = ?1 WHERE status = ?2;",
                (&disabled, &active),
            )
            .map_err(|e| {
                PersistError::into_config_error(
                    format!("Failed to set any '{disabled}' config to '{active}'."),
                    e,
                )
            })?;

        // Update the desired config to 'ACTIVE'
        transaction
            .execute(
                "UPDATE config SET status = ?1 WHERE key = ?2;",
                (&active, &key),
            )
            .map_err(|e| {
                PersistError::into_config_error(
                    format!("Failed to update config status to '{active}'."),
                    e,
                )
            })?;

        transaction.commit().map_err(|e| {
            PersistError::into_config_error("Failed to commit transaction to update config", e)
        })?;

        self.get_configuration(Some(key))
            .map_err(|e| PersistError::Validation {
                name: "config".into(),
                source: e.into(),
            })
    }

    // TODO: split this out into separate functions get_config_by_id & get_config_by_status
    fn get_configuration(&self, key: Option<String>) -> Result<Config, PersistError> {
        match key {
            Some(key) => self
                .connection
                .query_row("SELECT * FROM config WHERE key = ?1", [key], |row| {
                    Config::try_from(row)
                })
                .map_err(|e| {
                    PersistError::into_config_error("Failed to retrieve config '{key}'.", e)
                }),
            None => self
                .connection
                .query_row(
                    "SELECT * FROM config WHERE status = ?1",
                    [String::from(ConfigStatus::Active)],
                    |row| Config::try_from(row),
                )
                .map_err(|e| {
                    PersistError::into_config_error("Failed to retrieve 'active' config.", e)
                }),
        }
    }

    fn get_configurations(&self) -> Result<Vec<Config>, PersistError> {
        let mut statement = self
            .connection
            .prepare("SELECT * FROM config")
            .map_err(|e| PersistError::into_config_error("Failed to retrieve configs", e))?;

        let configs: Vec<_> = statement
            .query_map([], |row| Config::try_from(row))
            .map_err(|e| PersistError::into_config_error("Failed to retrieve configs", e))?
            .collect::<Result<_, _>>()
            .map_err(|e| PersistError::into_config_error("Failed to retrieve configs", e))?;

        Ok(configs)
    }

    fn close(self) -> anyhow::Result<()> {
        log::info!("closing sqlite connection");

        self.connection
            .close()
            .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path};

    use crate::adapters::git::{Git, GitCommand};
    use crate::domain::models::path::AbsolutePath;
    use crate::entry::Interactive;
    use crate::{app_context::AppContext, domain::adapters::Store};

    use crate::migrations::{db_migrations, MigrationContext};

    use super::*;
    use anyhow::Context;
    use chrono::{DateTime, Utc};
    use fake::{Fake, Faker};

    #[test]
    fn persist_branch_creates_a_new_item_if_not_exists() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let connection = setup_db()?;
        let store = Sqlite::new(connection);

        // Act
        store.persist_branch(&branch)?;

        // Assert
        assert_eq!(branch_count(&store.connection)?, 1);

        let actual_branch = select_branch_row(&store.connection)?;

        assert_eq!(branch, actual_branch);

        Ok(())
    }

    #[test]
    fn persist_branch_updates_an_existing_item() -> anyhow::Result<()> {
        // Arrange
        let branch = fake_branch(None, None)?;
        let connection = setup_db()?;
        let store = Sqlite { connection };

        insert_branch(&store.connection, &branch);

        let updated_branch = Branch {
            name: branch.name,
            ..fake_branch(None, None)?
        };

        // Act
        store.persist_branch(&updated_branch)?;

        // Assert
        assert_eq!(branch_count(&store.connection)?, 1);

        let actual_branch = select_branch_row(&store.connection)?;

        assert_eq!(updated_branch, actual_branch);

        Ok(())
    }

    #[test]
    fn get_branch_retrieves_correct_branch() -> anyhow::Result<()> {
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

            insert_branch(&store.connection, &branch);
        }

        let context = AppContext {
            store,
            git: Git { git: GitCommand },
            config: fake_config(),
            interactive: Interactive::Enable,
        };

        let keys = branches.keys().cloned().collect::<Vec<String>>();

        let random_key = keys
            .get((0..keys.len() - 1).fake::<usize>())
            .with_context(|| "Expected to find a matching branch")?;

        let random_branch = branches
            .get(random_key)
            .with_context(|| "Expected to find a matching branch")?;

        // Act
        let branch = context.store.get_branch(&random_key, &repo)?;

        context.close()?;

        // Assert
        assert_eq!(random_branch.to_owned(), branch);

        Ok(())
    }

    #[test]
    fn attempt_to_get_non_existent_branch_throws_not_found() {
        // Arrange
        let connection = setup_db().unwrap();
        let store = Sqlite { connection };

        let random_key = Faker.fake::<String>();
        let repo = Faker.fake::<String>();

        let context = AppContext {
            store,
            git: Git { git: GitCommand },
            config: fake_config(),
            interactive: Interactive::Enable,
        };

        // Act
        let error = context.store.get_branch(&random_key, &repo).unwrap_err();
        context.close().unwrap();

        // Assert
        assert!(matches!(error, PersistError::NotFound { name } if name == "branch" ));
    }

    #[test]
    fn invalid_stored_created_date_returns_corrupted_error() {
        // Arrange
        let connection = setup_db().unwrap();
        let store = Sqlite { connection };
        let name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let key = format!("{}-{}", repo.trim(), name.trim());

        store.connection.execute(
            "INSERT INTO branch (name, ticket, data, created, link, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &key,
                &Faker.fake::<String>(),
                None::<Vec<u8>>,
                "invalid_date",
                &Faker.fake::<String>(),
                &Faker.fake::<String>()
            )
        ).unwrap();

        let context = AppContext {
            store,
            git: Git { git: GitCommand },
            config: fake_config(),
            interactive: Interactive::Enable,
        };
        // Act
        let error = context.store.get_branch(&name, &repo).unwrap_err();
        context.close().unwrap();

        // Assert
        assert!(matches!(error, PersistError::Corrupted { name, .. } if name == "branch" ));
    }

    #[test]
    fn get_branch_trims_branch_name_before_retrieving() -> anyhow::Result<()> {
        // Arrange
        let context = AppContext {
            store: Sqlite::new(setup_db()?),
            git: Git { git: GitCommand },
            config: fake_config(),
            interactive: Interactive::Enable,
        };

        // Insert random collection of branches.
        let name = Faker.fake::<String>();
        let repo = Faker.fake::<String>();
        let expected = fake_branch(Some(name.clone()), Some(repo.clone()))?;

        insert_branch(&context.store.connection, &expected);

        // Act
        let actual = context.store.get_branch(&format!(" {}\n", name), &repo)?;

        context.close()?;
        // Assert
        assert_eq!(actual.name, expected.name);

        Ok(())
    }

    #[test]
    fn persist_config_creates_a_new_item_if_not_exists() -> anyhow::Result<()> {
        // Arrange
        let config = Config {
            status: ConfigStatus::Disabled,
            ..fake_config()
        };

        let connection = setup_db()?;
        let store = Sqlite::new(connection);

        // Act
        store.persist_config(&config)?;

        // Assert
        assert_eq!(config_count(&store.connection)?, 1);

        let expected = select_config_row(&store.connection, config.key.clone().into())?;

        assert_eq!(expected, config);

        Ok(())
    }

    #[test]
    fn persist_config_updates_an_existing_item() -> anyhow::Result<()> {
        // Arrange
        let config = fake_config();
        let connection = setup_db()?;
        insert_config(&connection, &config)?;

        let store = Sqlite::new(connection);

        // Act
        let config = Config {
            key: config.key,
            ..fake_config()
        };
        store.persist_config(&config)?;

        // Assert
        assert_eq!(config_count(&store.connection)?, 1);

        let actual = select_config_row(&store.connection, config.key.clone().into())?;
        assert_eq!(actual, actual);

        Ok(())
    }

    #[test]
    fn get_list_of_registered_configs() -> anyhow::Result<()> {
        // Arrange
        let connection = setup_db()?;
        let expected = vec![fake_config(), fake_config(), fake_config()];

        for config in &expected {
            insert_config(&connection, config)?;
        }

        let store = Sqlite::new(connection);

        // Act
        let configs = store.get_configurations()?;

        assert_eq!(expected, configs);

        Ok(())
    }

    #[test]
    fn with_no_stored_configs_an_empty_list_is_returned() {
        // Arrange
        let connection = setup_db().unwrap();
        let store = Sqlite::new(connection);

        // Act
        let configs = store.get_configurations().unwrap();

        assert!(configs.is_empty());
    }

    #[test]
    fn get_config_by_key_success() -> anyhow::Result<()> {
        // Arrange
        let expected = fake_config();
        let connection = setup_db()?;
        insert_config(&connection, &expected)?;

        let store = Sqlite::new(connection);

        // Act
        let config = store
            .get_configuration(Some(expected.key.clone().into()))
            .unwrap();

        // Assert
        assert_eq!(1, config_count(&store.connection)?);
        assert_eq!(expected, config);
        Ok(())
    }

    #[test]
    fn get_config_by_key_that_does_not_exist() {
        // Arrange
        let expected = fake_config();
        let connection = setup_db().unwrap();
        let store = Sqlite::new(connection);

        // Act
        let error = store
            .get_configuration(Some(expected.key.clone().into()))
            .unwrap_err();

        // Assert
        assert!(matches!(error, PersistError::NotFound { name } if name == "config"));
    }

    #[test]
    fn get_config_by_active() -> anyhow::Result<()> {
        // Arrange
        let expected = Config {
            status: ConfigStatus::Active,
            ..fake_config()
        };
        let connection = setup_db()?;
        insert_config(&connection, &expected)?;
        let store = Sqlite::new(connection);

        // Act
        let config = store.get_configuration(None).unwrap();

        // Assert
        assert_eq!(1, config_count(&store.connection)?);
        assert_eq!(expected, config);
        Ok(())
    }

    #[test]
    fn on_no_active_config_not_found_thrown() {
        // Arrange
        let expected = Config {
            status: ConfigStatus::Disabled,
            ..fake_config()
        };
        let connection = setup_db().unwrap();
        insert_config(&connection, &expected).unwrap();
        let store = Sqlite::new(connection);

        // Act
        let error = store.get_configuration(None).unwrap_err();

        // Assert
        assert!(matches!(error, PersistError::NotFound { name } if name == "config"));
    }

    #[test]
    fn corrupted_path_returns_corrupted_error_on_get() {
        // Arrange
        let connection = setup_db().unwrap();
        let key = Faker.fake::<String>();
        connection
            .execute(
                "INSERT INTO config (key, path, status) VALUES (?1, ?2, ?3)",
                (&key, "invalid_path", String::from(ConfigStatus::Active)),
            )
            .unwrap();

        let store = Sqlite::new(connection);

        // Act
        let error = store.get_configuration(Some(key)).unwrap_err();

        // Assert
        assert!(matches!(error, PersistError::Corrupted { name, .. } if name == "config"));
    }

    #[test]
    fn corrupted_config_status_returns_corrupted_error_on_get() {
        // Arrange
        let connection = setup_db().unwrap();
        let key = Faker.fake::<String>();

        insert_raw_config(&connection, &key, &valid_path_str(), "invalid_status");

        let store = Sqlite::new(connection);

        // Act
        let error = store.get_configuration(Some(key)).unwrap_err();
        // Assert
        assert!(matches!(error, PersistError::Corrupted { name, .. } if name == "config"));
    }

    #[test]
    fn set_active_config_success() -> anyhow::Result<()> {
        let mut original = Config {
            status: ConfigStatus::Disabled,
            ..fake_config()
        };
        let connection = setup_db()?;
        insert_config(&connection, &original)?;

        let mut store = Sqlite::new(connection);
        let actual = store.set_active_config(&original.key.clone())?;

        original.status = ConfigStatus::Active;
        assert_eq!(original, actual);

        Ok(())
    }

    #[test]
    fn set_active_checks_row_exists_before_clearing_status() {
        let connection = setup_db().unwrap();
        let mut store = Sqlite::new(connection);

        let active_config = Config {
            status: ConfigStatus::Active,
            ..fake_config()
        };

        insert_config(&store.connection, &active_config).unwrap();

        let result = store
            .set_active_config(&ConfigKey::User(Faker.fake()))
            .unwrap_err();
        assert!(matches!(result, PersistError::NotFound { name } if name == "config"));

        let default = store
            .get_configuration(Some(active_config.key.clone().into()))
            .unwrap();
        assert_eq!(active_config.key, default.key);
    }

    #[test]
    fn set_active_config_sets_any_active_configs_to_disabled() -> anyhow::Result<()> {
        let connection = setup_db()?;

        for _ in 0..(2..10).fake() {
            let config = Config {
                status: ConfigStatus::Active,
                ..fake_config()
            };

            insert_config(&connection, &config)?;
        }

        let original = Config {
            status: ConfigStatus::Disabled,
            ..fake_config()
        };

        insert_config(&connection, &original)?;

        let mut store = Sqlite::new(connection);
        // Act
        store.set_active_config(&original.key)?;

        let configs = select_all_config(&store.connection)?;

        let active: Vec<Config> = configs
            .into_iter()
            .filter(|c| c.status == ConfigStatus::Active)
            .collect();

        assert_eq!(active.len(), 1);
        let only_active = active.first().unwrap();
        let expected = Config {
            status: ConfigStatus::Active,
            ..original
        };
        assert_eq!(&expected, only_active);

        Ok(())
    }

    fn valid_path() -> AbsolutePath {
        let path = Path::new(".").to_owned();
        dunce::canonicalize(path)
            .expect("Valid conversion to absolute path")
            .try_into()
            .unwrap()
    }

    fn valid_path_str() -> String {
        valid_path().to_string()
    }

    fn fake_config() -> Config {
        let absolute_path = valid_path();

        Config {
            key: ConfigKey::User(Faker.fake()),
            path: absolute_path,
            status: ConfigStatus::Active,
        }
    }

    fn insert_branch(connection: &Connection, branch: &Branch) {
        connection.execute(
            "INSERT INTO branch (name, ticket, data, created, link, scope) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &branch.name,
                &branch.ticket,
                &branch.data,
                &branch.created.to_rfc3339(),
                &branch.link,
                &branch.scope
            ),
        ).unwrap();
    }

    fn insert_config(connection: &Connection, config: &Config) -> anyhow::Result<()> {
        let key: String = config.key.clone().into();
        insert_raw_config(
            connection,
            &key,
            &config.path.to_string(),
            &String::from(config.status.clone()),
        );

        Ok(())
    }

    fn insert_raw_config(connection: &Connection, key: &str, path: &str, status: &str) {
        connection
            .execute(
                "INSERT INTO config (key, path, status) VALUES (?1, ?2, ?3)",
                (&key, path, status),
            )
            .unwrap();
    }

    fn fake_branch(name: Option<String>, repo: Option<String>) -> anyhow::Result<Branch> {
        let name = name.unwrap_or(Faker.fake());
        let repo = repo.unwrap_or(Faker.fake());

        Ok(Branch::new(
            &name,
            &repo,
            Faker.fake(),
            Faker.fake(),
            Faker.fake(),
        ))
    }

    fn select_branch_row(conn: &Connection) -> anyhow::Result<Branch> {
        let (name, ticket, data, created, link, scope) =
            conn.query_row("SELECT * FROM branch", [], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<Vec<u8>>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })?;
        let created = DateTime::parse_from_rfc3339(&created)?.with_timezone(&Utc);

        Ok(Branch {
            name,
            ticket,
            data,
            created,
            link,
            scope,
        })
    }

    fn select_config_row(conn: &Connection, key: String) -> anyhow::Result<Config> {
        let path = conn.query_row("SELECT * FROM config where key = ?1;", [key], |row| {
            Config::try_from(row)
        })?;

        Ok(path)
    }

    fn select_all_config(conn: &Connection) -> anyhow::Result<Vec<Config>> {
        let mut statement = conn.prepare("SELECT * FROM config")?;
        let configs = statement
            .query_map([], |row| Config::try_from(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(configs)
    }

    fn branch_count(conn: &Connection) -> anyhow::Result<i32> {
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM branch", [], |row| row.get(0))?;

        Ok(count)
    }

    fn config_count(conn: &Connection) -> anyhow::Result<i32> {
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM config", [], |row| row.get(0))?;

        Ok(count)
    }

    fn setup_db() -> anyhow::Result<Connection> {
        let mut conn = Connection::open_in_memory()?;
        db_migrations(
            &mut conn,
            MigrationContext {
                default_configs: None,
                version: 4,
            },
        )?;
        Ok(conn)
    }
}
