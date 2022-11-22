use crate::domain::commands::Actor;

use super::Arguments;

pub fn handler(actions: &dyn Actor, args: Arguments) -> anyhow::Result<()> {
    actions.checkout(args)?;
    Ok(())
}
