use crate::domain::adapters::prompt::Prompter;
use crate::domain::adapters::Store;
use crate::domain::models::{ConfigKey, ConfigStatus};
use crate::entry::Interactive;

use super::Arguments;
use colored::Colorize;

pub fn handler<S: Store, P: Prompter>(
    store: &mut S,
    config_key: &ConfigKey,
    arguments: Arguments,
    prompt: P,
    interactive: &Interactive,
) -> anyhow::Result<()> {
    local_config_warning(config_key)?;

    match arguments {
        Arguments::Add(args) => {
            let config = args.try_into_domain()?;
            store.persist_config(&config)?;
            println!("🟢 {} (Active)", config.key.to_string().green());
        }
        Arguments::Set(args) => {
            let key = args.try_into_domain(store, prompt, interactive)?;
            store.set_active_config(&key)?;
            println!("🟢 {} (Active)", key.to_string().green());
        }
        Arguments::Reset => {
            let key = ConfigKey::Default;
            store.set_active_config(&key)?;
            println!("🟢 Config reset to {}", key.to_string().green());
        }
        Arguments::Show => {
            let mut configurations = store.get_configurations()?;
            configurations.sort_by_key(|c| c.status.clone());

            for config in configurations {
                let key = config.key.to_string();
                let path = config.path.display();

                match config.status {
                    ConfigStatus::Active => println!("🟢 {} (Active) ➜ '{}'", key.green(), path),
                    ConfigStatus::Disabled => println!("🔴 {key} ➜ '{path}'"),
                }
            }
        }
    };

    Ok(())
}

fn local_config_warning(config_key: &ConfigKey) -> anyhow::Result<()> {
    let warn_message = match config_key {
        ConfigKey::Once => Some("'once off' --config"),
        ConfigKey::Local => Some("'local' repository"),
        ConfigKey::User(_) | ConfigKey::Default => None,
    };

    if let Some(msg) = warn_message {
        println!(
            "{}: (Active) configurations are currently overridden due to a {} configuration being used.\n",
            "⚠️ Warning".yellow(),
            msg
        );
    }
    Ok(())
}
