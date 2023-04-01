use chrono::{DateTime, Utc};
use rusqlite::{types::Type, Row};

use crate::domain::{
    errors::PersistError,
    models::{path::AbsolutePath, Branch, Config, ConfigKey},
};

impl PersistError {
    pub fn into_config_error<S>(message: S, error: rusqlite::Error) -> PersistError
    where
        S: Into<String>,
    {
        PersistError::into("config", message, error)
    }

    pub fn into_branch_error<S>(message: S, error: rusqlite::Error) -> PersistError
    where
        S: Into<String>,
    {
        PersistError::into("branch", message, error)
    }

    #[allow(clippy::wildcard_in_or_patterns)]
    fn into<S>(name: &str, message: S, error: rusqlite::Error) -> PersistError
    where
        S: Into<String>,
    {
        log::error!("{}\n{}", message.into(), &error);

        match error {
            rusqlite::Error::InvalidPath(_) => PersistError::Configuration,
            rusqlite::Error::ExecuteReturnedResults
            | rusqlite::Error::SqliteSingleThreadedMode
            | rusqlite::Error::StatementChangedRows(_)
            | rusqlite::Error::MultipleStatement
            | rusqlite::Error::IntegralValueOutOfRange(_, _)
            | rusqlite::Error::InvalidQuery
            | rusqlite::Error::NulError(_)
            | rusqlite::Error::Utf8Error(_)
            | rusqlite::Error::ToSqlConversionFailure(_) => PersistError::Validation {
                name: name.into(),
                source: error.into(),
            },
            rusqlite::Error::QueryReturnedNoRows => PersistError::NotFound { name: name.into() },
            rusqlite::Error::FromSqlConversionFailure(_, _, _)
            | rusqlite::Error::InvalidParameterCount(_, _)
            | rusqlite::Error::InvalidColumnIndex(_)
            | rusqlite::Error::InvalidParameterName(_)
            | rusqlite::Error::InvalidColumnName(_)
            | rusqlite::Error::InvalidColumnType(_, _, _)
            | rusqlite::Error::SqlInputError { .. } => PersistError::Corrupted {
                name: name.into(),
                source: Some(error.into()),
            },
            rusqlite::Error::SqliteFailure(_, _) | _ => PersistError::Unknown(error.into()),
        }
    }
}

impl<'a> TryFrom<&Row<'a>> for Config {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let status = value.get::<_, String>(2)?.try_into().map_err(|e| {
            log::error!(
                "Corrupted data failed to convert to valid config status, {}",
                e
            );
            rusqlite::Error::InvalidColumnType(
                2,
                "Failed to convert to valid config status".into(),
                Type::Text,
            )
        })?;

        let path: AbsolutePath = value.get::<_, String>(1)?.try_into().map_err(|e| {
            log::error!("Corrupted data failed to convert 'Config', {}", e);
            rusqlite::Error::InvalidColumnType(1, "Failed to convert path".into(), Type::Text)
        })?;

        Ok(Config {
            key: ConfigKey::from(value.get::<_, String>(0)?.as_str()),
            status,
            path,
        })
    }
}

impl<'a> TryFrom<&Row<'a>> for Branch {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let date = value.get::<usize, String>(3)?;
        let created = DateTime::parse_from_rfc3339(&date)
            .map_err(|e| {
                log::error!("Corrupted data failed to convert to datetime, {}", e);
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
            link: value.get(4)?,
            scope: value.get(5)?,
        };

        Ok(branch)
    }
}
