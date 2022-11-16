use crate::{
    config::Config,
    domain::commands::{Commands, GitCommands},
};

use super::Arguments;

pub fn handler<C: GitCommands>(
    actions: &dyn Commands<C>,
    config: &Config,
    args: Arguments,
) -> anyhow::Result<()> {
    config.validate_template(&args.template)?;
    actions.commit(args)?;

    Ok(())
}
