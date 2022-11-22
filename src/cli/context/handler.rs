use crate::domain::commands::Actor;

use super::Arguments;

pub fn handler(actions: &dyn Actor, args: Arguments) -> anyhow::Result<()> {
    actions.current(args)?;
    Ok(())
}
