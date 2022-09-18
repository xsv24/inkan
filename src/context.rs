use anyhow::{anyhow, Context as anyhow_context};
use directories::ProjectDirs;
use rusqlite::Connection;

use crate::git_commands::GitCommands;

pub struct Context<C: GitCommands> {
    pub project_dir: ProjectDirs,
    pub connection: Connection,
    pub commands: C,
}

impl<C: GitCommands> Context<C> {
    pub fn new(commands: C) -> anyhow::Result<Context<C>> {
        let project_dir = ProjectDirs::from("dev", "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        let connection = Connection::open(project_dir.config_dir().join("db"))?;

        Ok(Context {
            project_dir,
            connection,
            commands,
        })
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.connection
            .close()
            .map_err(|_| anyhow!("Failed to close 'git-kit' connection"))?;

        Ok(())
    }
}
