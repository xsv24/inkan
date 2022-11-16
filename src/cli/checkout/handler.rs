use crate::domain::commands::{Commands, GitCommands};

use super::Arguments;

pub fn handler<C: GitCommands>(actions: &dyn Commands<C>, args: Arguments) -> anyhow::Result<()> {
    actions.checkout(args)?;
    Ok(())
}
