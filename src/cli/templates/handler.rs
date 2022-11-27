use colored::Colorize;

use crate::app_config::AppConfig;

pub fn handler(config: &AppConfig) -> anyhow::Result<()> {
    log::info!("collect commit templates from config.");
    let mut pairs = config
        .commit
        .templates
        .clone()
        .into_iter()
        .collect::<Vec<_>>();

    pairs.sort_by_key(|(k, ..)| k.to_owned());

    for (key, value) in pairs {
        println!("- {} {}.", key.bold().green(), value.description.italic());
    }

    Ok(())
}
