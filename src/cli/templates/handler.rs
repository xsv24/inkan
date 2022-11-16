use colored::Colorize;

use crate::config::Config;

pub fn handler(config: &Config) -> anyhow::Result<()> {
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
