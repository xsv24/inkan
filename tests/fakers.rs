use anyhow::anyhow;
use chrono::Utc;
use std::{
    env::temp_dir,
    path::{Path, PathBuf},
};

use fake::{Fake, Faker};
use git_kit::{
    adapters::sqlite::Sqlite,
    app_context::AppContext,
    domain::{
        adapters::{CheckoutStatus, CommitMsgStatus, Git},
        models::{Branch, Config, ConfigStatus},
    },
    entry::Interactive,
    migrations::{db_migrations, MigrationContext},
};
use rusqlite::Connection;
use uuid::Uuid;

pub fn fake_config() -> Config {
    Config {
        key: Faker.fake::<String>().into(),
        path: Faker.fake(),
        status: ConfigStatus::Active,
    }
}

pub fn fake_context<'a, C: Git>(git: C, config: Config) -> anyhow::Result<AppContext<C, Sqlite>> {
    let mut connection = Connection::open_in_memory()?;

    db_migrations(
        &mut connection,
        MigrationContext {
            default_configs: None,
            version: None,
        },
    )?;

    let context = AppContext {
        store: Sqlite::new(connection)?,
        config,
        git,
        interactive: Interactive::Enable,
    };

    Ok(context)
}

#[allow(dead_code)]
pub fn fake_branch() -> Branch {
    Branch {
        name: Faker.fake(),
        ticket: Faker.fake(),
        created: Utc::now(),
        data: Faker.fake(),
        link: Faker.fake(),
        scope: Faker.fake(),
    }
}

#[derive(Clone)]
pub struct GitCommandMock {
    pub repo: Result<String, String>,
    pub branch_name: Result<String, String>,
    pub checkout_res: fn(&str, CheckoutStatus) -> anyhow::Result<()>,
    pub commit_res: fn(&Path, CommitMsgStatus) -> anyhow::Result<()>,
    pub template_file_path: fn() -> anyhow::Result<PathBuf>,
}

impl GitCommandMock {
    pub fn fake() -> GitCommandMock {
        GitCommandMock {
            repo: Ok(Faker.fake()),
            branch_name: Ok(Faker.fake()),
            checkout_res: |_, _| Ok(()),
            commit_res: |_, _| Ok(()),
            template_file_path: || {
                let temp_file = temp_dir().join(Uuid::new_v4().to_string());
                Ok(temp_file)
            },
        }
    }
}

impl Git for GitCommandMock {
    fn repository_name(&self) -> anyhow::Result<String> {
        self.repo
            .as_ref()
            .map(|s| s.to_owned())
            .map_err(|e| anyhow!(e.to_owned()))
    }

    fn branch_name(&self) -> anyhow::Result<String> {
        self.branch_name
            .as_ref()
            .map(|s| s.to_owned())
            .map_err(|e| anyhow!(e.to_owned()))
    }

    fn checkout(&self, name: &str, status: CheckoutStatus) -> anyhow::Result<()> {
        (self.checkout_res)(name, status)
    }

    fn root_directory(&self) -> anyhow::Result<PathBuf> {
        panic!("Did not expect Git 'root_directory' to be called.");
    }

    fn template_file_path(&self) -> anyhow::Result<PathBuf> {
        (self.template_file_path)()
    }

    fn commit_with_template(
        &self,
        template: &Path,
        complete: CommitMsgStatus,
    ) -> anyhow::Result<()> {
        (self.commit_res)(template, complete)
    }
}
