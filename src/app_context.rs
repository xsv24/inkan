use crate::{
    domain::{
        adapters::{Git, Store},
        models::Config,
    },
    entry::Interactive,
};

pub struct AppContext<G: Git, S: Store> {
    pub store: S,
    pub git: G,
    pub config: Config,
    pub interactive: Interactive,
}

impl<G: Git, S: Store> AppContext<G, S> {
    pub fn new(
        git: G,
        store: S,
        config: Config,
        interactive: Interactive,
    ) -> anyhow::Result<AppContext<G, S>> {
        Ok(AppContext {
            store,
            git,
            config,
            interactive,
        })
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.store.close()
    }
}
