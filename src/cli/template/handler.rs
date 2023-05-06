use crate::domain::adapters::prompt::Prompter;
use crate::domain::adapters::Store;
use crate::domain::errors::{Errors, UserInputError};
use crate::domain::models::path::{AbsolutePath, PathType};
use crate::domain::models::{ConfigKey, Template, TemplateStatus};
use crate::entry::Interactive;
use crate::template_config::TemplateConfig;

use super::commands::{TemplateAdd, TemplateSet};
use super::SubCommands;
use colored::Colorize;

pub fn handler<S: Store, P: Prompter>(
    store: &mut S,
    config: &Template,
    arguments: SubCommands,
    prompt: P,
    interactive: &Interactive,
) -> Result<(), Errors> {
    local_config_warning(&config.key);

    match arguments {
        SubCommands::Add(args) => add(args, store),
        SubCommands::Set(args) => set(args, store, prompt, interactive),
        SubCommands::Reset => reset(store),
        SubCommands::List => list(store),
        SubCommands::Active => active_list(config),
    }?;

    Ok(())
}

fn add<S: Store>(args: TemplateAdd, store: &mut S) -> Result<(), Errors> {
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

    let config = Template {
        key,
        path,
        status: TemplateStatus::Active,
    };

    store
        .persist_template(&config)
        .map_err(Errors::PersistError)?;

    store
        .set_active_template(&config.key)
        .map_err(Errors::PersistError)?;

    println!("🟢 {} (Active)", config.key.to_string().green());

    Ok(())
}

fn set<S: Store, P: Prompter>(
    args: TemplateSet,
    store: &mut S,
    prompt: P,
    interactive: &Interactive,
) -> Result<(), Errors> {
    let key = args.try_into_domain(store, prompt, interactive)?;

    store
        .set_active_template(&key)
        .map_err(Errors::PersistError)?;

    println!(
        "{} templates has been successfully added.",
        key.to_string().green()
    );
    Ok(())
}

fn reset<S: Store>(store: &mut S) -> Result<(), Errors> {
    let key = ConfigKey::Default;
    store
        .set_active_template(&key)
        .map_err(Errors::PersistError)?;

    println!("🟢 Config reset to {}", key.to_string().green());

    Ok(())
}

fn list<S: Store>(store: &S) -> Result<(), Errors> {
    let mut configurations = store.get_templates().map_err(Errors::PersistError)?;

    configurations.sort_by_key(|c| c.status.clone());

    for config in configurations {
        let key = config.key.to_string();
        let path: String = config
            .path
            .try_into()
            .ok()
            .unwrap_or_else(|| "Invalid configuration path please update".into());

        match config.status {
            TemplateStatus::Active => println!("🟢 {} (Active) ➜ '{}'", key.green(), path),
            TemplateStatus::Disabled => println!("🔴 {key} ➜ '{path}'"),
        }
    }

    Ok(())
}

pub fn active_list(config: &Template) -> Result<(), Errors> {
    log::info!("collect commit templates from config.");
    let templates = TemplateConfig::new(&config.path)?;

    let mut pairs = templates.commit.templates.into_iter().collect::<Vec<_>>();

    pairs.sort_by_key(|(k, ..)| k.to_owned());

    for (key, value) in pairs {
        println!("- {} {}.", key.bold().green(), value.description.italic());
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
