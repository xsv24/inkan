use crate::domain::{
    adapters::Store,
    models::{Config, ConfigKey, ConfigStatus},
};

use super::Arguments;

pub fn handler<S: Store>(store: &mut S, arguments: Arguments) -> anyhow::Result<()> {
    let config = match arguments {
        Arguments::Add { name, path } => {
            let config = Config::new(name.into(), path, ConfigStatus::ACTIVE)?;
            store.persist_config(&config)?;
            store.set_active_config(config.key)?
        }
        Arguments::Set { name } => store.set_active_config(ConfigKey::from(name))?,
        Arguments::Reset => store.set_active_config(ConfigKey::Default)?,
        Arguments::Show => store.get_config(None)?,
    };

    println!("{}", config);

    Ok(())
}
