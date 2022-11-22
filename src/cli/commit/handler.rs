use crate::{config::Config, domain::commands::Actor};

use super::Arguments;

pub fn handler(actions: &dyn Actor, config: &Config, args: Arguments) -> anyhow::Result<()> {
    config.validate_template(&args.template)?;
    actions.commit(args)?;

    Ok(())
}
