use crate::{
    config::Config,
    domain::adapters::{Git, Store},
};

pub struct AppContext<C: Git, S: Store> {
    pub store: S,
    pub git: C,
    pub config: Config,
}

impl<C: Git, S: Store> AppContext<C, S> {
    pub fn new(git: C, store: S, config: Config) -> anyhow::Result<AppContext<C, S>> {
        Ok(AppContext { store, git, config })
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.store.close()
    }
}
