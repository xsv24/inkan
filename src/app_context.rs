use directories::ProjectDirs;

use crate::{
    config::Config,
    domain::{commands::GitCommands, Store},
};

pub struct AppContext<C: GitCommands, S: Store> {
    pub project_dir: ProjectDirs,
    pub store: S,
    pub commands: C,
    pub config: Config,
}

impl<C: GitCommands, S: Store> AppContext<C, S> {
    pub fn new(
        commands: C,
        store: S,
        project_dir: ProjectDirs,
        config: Config,
    ) -> anyhow::Result<AppContext<C, S>> {
        Ok(AppContext {
            project_dir,
            store,
            commands,
            config,
        })
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.store.close()
    }
}
