use crate::domain::{
    adapters::{Git, Store},
    models::{Config, ConfigKey, ConfigStatus},
};
use crate::AppConfig;

use super::Arguments;
use colored::Colorize;
use inquire::{
    ui::{Color, RenderConfig, Styled},
    Select,
};

pub fn handler<S: Store, G: Git>(
    store: &mut S,
    git: &G,
    arguments: Arguments,
) -> anyhow::Result<()> {
    inquire::set_global_render_config(get_render_config());

    match arguments {
        Arguments::Add { name, path } => {
            let config = add(store, name, path)?;
            local_config_warning(git)?;
            println!("🟢 {}", config.key.to_string().green());
        }
        Arguments::Set { name } => {
            let config = set(store, name)?;
            local_config_warning(git)?;
            println!("🟢 {} (Active) ", config.key.to_string().green());
        }
        Arguments::Reset => {
            let config = reset(store)?;
            local_config_warning(git)?;
            println!("🟢 Config reset to {}", config.key.to_string().green());
        }
        Arguments::Show => {
            let configurations = list(store)?;
            local_config_warning(git)?;

            for config in configurations {
                let key = config.key.to_string();
                let path = config.path.display();

                match config.status {
                    ConfigStatus::Active => println!("🟢 {} (Active) ➜ '{}'", key.green(), path),
                    ConfigStatus::Disabled => println!("🔴 {} ➜ '{}'", key, path),
                }
            }
        }
    };

    Ok(())
}

fn add<S: Store>(store: &mut S, name: String, path: String) -> anyhow::Result<Config> {
    let config = Config::new(name.into(), path, ConfigStatus::Active)?;

    store.persist_config(&config)?;

    store.set_active_config(config.key)
}

fn set<S: Store>(store: &mut S, name: Option<String>) -> anyhow::Result<Config> {
    let name = if let Some(name) = name {
        name
    } else {
        let configurations: Vec<String> = store
            .get_configurations()?
            .iter()
            .map(|config| config.key.clone().into())
            .collect();

        Select::new("Configuration:", configurations).prompt()?
    };

    store.set_active_config(ConfigKey::from(name))
}

fn reset<S: Store>(store: &mut S) -> anyhow::Result<Config> {
    store.set_active_config(ConfigKey::Default)
}

fn list<S: Store>(store: &mut S) -> anyhow::Result<Vec<Config>> {
    let mut configurations = store.get_configurations()?;
    configurations.sort_by_key(|c| c.status.clone());

    Ok(configurations)
}

fn local_config_warning<G: Git>(git: &G) -> anyhow::Result<()> {
    let local_config_path = AppConfig::join_config_filename(&git.root_directory()?);

    if local_config_path.exists() {
        println!("{}: 'Active' configurations are currently overridden due to a local repo configuration being used.\n", "⚠️ Warning".yellow());
    }

    Ok(())
}

fn get_render_config() -> RenderConfig {
    RenderConfig {
        highlighted_option_prefix: Styled::new("➜").with_fg(Color::LightBlue),
        selected_checkbox: Styled::new("✅").with_fg(Color::LightGreen),
        unselected_checkbox: Styled::new("🔳"),
        ..RenderConfig::default()
    }
}
