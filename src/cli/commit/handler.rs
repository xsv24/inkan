use crate::{app_config::AppConfig, domain::commands::Actor};

use super::Arguments;

pub fn handler(actions: &dyn Actor, config: &AppConfig, args: Arguments) -> anyhow::Result<()> {
    config.validate_template(&args.template)?;
    // TODO: Could we do a prompt if no ticket / args found ?
    actions.commit(args)?;

    Ok(())
}
