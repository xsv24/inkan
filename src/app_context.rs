use crate::domain::{
    adapters::{Git, Store},
    models::Config,
};

pub struct AppContext<G: Git, S: Store> {
    pub store: S,
    pub git: G,
    pub config: Config,
}

impl<G: Git, S: Store> AppContext<G, S> {
    pub fn new(git: G, store: S, config: Config) -> anyhow::Result<AppContext<G, S>> {
        Ok(AppContext { store, git, config })
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.store.close()
    }
}
