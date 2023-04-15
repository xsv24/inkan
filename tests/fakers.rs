use chrono::Utc;
use std::path::{Path, PathBuf};

use fake::{Fake, Faker};
use git_kit::{
    adapters::sqlite::Sqlite,
    app_context::AppContext,
    domain::{
        adapters::{CheckoutStatus, CommitMsgStatus, Git},
        errors::GitError,
        models::{path::AbsolutePath, Branch, Config, ConfigStatus},
    },
    entry::Interactive,
    migrations::{db_migrations, MigrationContext},
};
use rusqlite::Connection;

lazy_static::lazy_static! {
    static ref VALID_FILE_PATH: AbsolutePath = valid_template_file_path();
}

pub fn valid_template_file_path() -> AbsolutePath {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let default_config = root.join("templates/default.yml");

    let path = std::env::temp_dir().join("default.yml");
    std::fs::copy(&default_config, &path).unwrap();

    path.try_into().unwrap()
}

pub fn fake_config() -> Config {
    Config {
        key: Faker.fake::<String>().as_str().into(),
        path: VALID_FILE_PATH.clone(),
        status: ConfigStatus::Active,
    }
}

pub fn fake_context<'a, C: Git>(git: C, config: Config) -> anyhow::Result<AppContext<C, Sqlite>> {
    let mut connection = Connection::open_in_memory()?;

    db_migrations(
        &mut connection,
        MigrationContext {
            default_configs: None,
            version: 4,
        },
    )?;

    let context = AppContext {
        store: Sqlite::new(connection),
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
    pub checkout_res: fn(&str, CheckoutStatus) -> Result<(), GitError>,
    pub commit_res: fn(&Path, CommitMsgStatus) -> Result<(), GitError>,
    pub template_file_path: fn() -> Result<AbsolutePath, GitError>,
}

impl GitCommandMock {
    pub fn fake() -> GitCommandMock {
        GitCommandMock {
            repo: Ok(Faker.fake()),
            branch_name: Ok(Faker.fake()),
            checkout_res: |_, _| Ok(()),
            commit_res: |_, _| Ok(()),
            template_file_path: || Ok(VALID_FILE_PATH.clone()),
        }
    }
}

impl Git for GitCommandMock {
    fn repository_name(&self) -> Result<String, GitError> {
        self.repo
            .as_ref()
            .map(|s| s.to_owned())
            .map_err(|e| GitError::Validation { message: e.into() })
    }

    fn branch_name(&self) -> Result<String, GitError> {
        self.branch_name
            .as_ref()
            .map(|s| s.to_owned())
            .map_err(|e| GitError::Validation { message: e.into() })
    }

    fn checkout(&self, name: &str, status: CheckoutStatus) -> Result<(), GitError> {
        (self.checkout_res)(name, status)
    }

    fn root_directory(&self) -> Result<AbsolutePath, GitError> {
        panic!("Did not expect Git 'root_directory' to be called.");
    }

    fn template_file_path(&self) -> Result<AbsolutePath, GitError> {
        (self.template_file_path)()
    }

    fn commit_with_template(
        &self,
        template: &Path,
        complete: CommitMsgStatus,
    ) -> Result<(), GitError> {
        (self.commit_res)(template, complete)
    }
}
