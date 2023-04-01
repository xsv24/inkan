use crate::domain::adapters::prompt::Prompter;
use crate::domain::adapters::Store;
use crate::domain::errors::{Errors, UserInputError};
use crate::domain::models::path::{AbsolutePath, PathType};
use crate::domain::models::{Config, ConfigKey, ConfigStatus};
use crate::entry::Interactive;

use super::args::{ConfigAdd, ConfigSet};
use super::Arguments;
use colored::Colorize;

pub fn handler<S: Store, P: Prompter>(
    store: &mut S,
    config_key: &ConfigKey,
    arguments: Arguments,
    prompt: P,
    interactive: &Interactive,
) -> Result<(), Errors> {
    local_config_warning(config_key);

    match arguments {
        Arguments::Add(args) => add(args, store),
        Arguments::Set(args) => set(args, store, prompt, interactive),
        Arguments::Reset => reset(store),
        Arguments::Show => list(store),
    }?;

    Ok(())
}

fn add<S: Store>(args: ConfigAdd, store: &mut S) -> Result<(), Errors> {
    let key = ConfigKey::from(args.name.as_str());

    if !key.is_overridable() {
        return Err(Errors::UserInput(UserInputError::Validation {
            name: "name".into(),
            message: format!(
                "Configuration '{}' cannot be overridden please choose another name",
                args.name
            ),
        }));
    }

    let path = AbsolutePath::try_from(args.path, PathType::File)
        .map_err(|e| UserInputError::Validation {
            name: "path".into(),
            message: e.to_string(),
        })
        .map_err(Errors::UserInput)?;

    let config = Config {
        key,
        path,
        status: ConfigStatus::Active,
    };

    store
        .persist_config(&config)
        .map_err(Errors::PersistError)?;

    store
        .set_active_config(&config.key)
        .map_err(Errors::PersistError)?;

    println!("🟢 {} (Active)", config.key.to_string().green());

    Ok(())
}

fn set<S: Store, P: Prompter>(
    args: ConfigSet,
    store: &mut S,
    prompt: P,
    interactive: &Interactive,
) -> Result<(), Errors> {
    let key = args.try_into_domain(store, prompt, interactive)?;

    store
        .set_active_config(&key)
        .map_err(Errors::PersistError)?;

    println!("🟢 {} (Active)", key.to_string().green());

    Ok(())
}

fn reset<S: Store>(store: &mut S) -> Result<(), Errors> {
    let key = ConfigKey::Default;
    store
        .set_active_config(&key)
        .map_err(Errors::PersistError)?;

    println!("🟢 Config reset to {}", key.to_string().green());

    Ok(())
}

fn list<S: Store>(store: &S) -> Result<(), Errors> {
    let mut configurations = store.get_configurations().map_err(Errors::PersistError)?;

    configurations.sort_by_key(|c| c.status.clone());

    for config in configurations {
        let key = config.key.to_string();
        let path: String = config
            .path
            .try_into()
            .ok()
            .unwrap_or_else(|| "Invalid configuration path please update".into());

        match config.status {
            ConfigStatus::Active => println!("🟢 {} (Active) ➜ '{}'", key.green(), path),
            ConfigStatus::Disabled => println!("🔴 {key} ➜ '{path}'"),
        }
    }

    Ok(())
}

fn local_config_warning(config_key: &ConfigKey) {
    let warn_message = match config_key {
        ConfigKey::Once => Some("'once off' --config"),
        ConfigKey::Local => Some("'local' repository"),
        ConfigKey::User(_) | ConfigKey::Default | ConfigKey::Conventional => None,
    };

    if let Some(msg) = warn_message {
        println!(
            "{}: (Active) configurations are currently overridden due to a {} configuration being used.\n",
            "⚠️ Warning".yellow(),
            msg
        );
    }
}
