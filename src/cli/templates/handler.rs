use colored::Colorize;

use crate::{
    domain::{errors::Errors, models::Config},
    template_config::TemplateConfig,
};

pub fn handler(config: &Config) -> Result<(), Errors> {
    log::info!("collect commit templates from config.");
    let templates = TemplateConfig::new(&config.path)?;

    let mut pairs = templates.commit.templates.into_iter().collect::<Vec<_>>();

    pairs.sort_by_key(|(k, ..)| k.to_owned());

    for (key, value) in pairs {
        println!("- {} {}.", key.bold().green(), value.description.italic());
    }

    Ok(())
}
