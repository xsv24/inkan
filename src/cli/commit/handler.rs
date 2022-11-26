use crate::{config::AppConfig, domain::commands::Actor};

use super::Arguments;

pub fn handler(actions: &dyn Actor, config: &AppConfig, args: Arguments) -> anyhow::Result<()> {
    config.validate_template(&args.template)?;
    actions.commit(args)?;

    Ok(())
}
