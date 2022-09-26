use anyhow::Context as anyhow_context;
use clap::Subcommand;
use directories::ProjectDirs;
use std::fs;

use crate::{args::Arguments, context::Context, git_commands::GitCommands};

#[derive(Debug, Subcommand)]
pub enum Template {
    /// Breaking change that could break a consuming application.
    Break(Arguments),
    /// Fix that resolves an unintended issue.
    Bug(Arguments),
    /// Dependency update or migration to a new dependency.
    Deps(Arguments),
    /// Documentation change.
    Docs(Arguments),
    /// Adds new functionality.
    Feature(Arguments),
    /// Improvement of code / structure without adding new functionality.
    Refactor(Arguments),
    /// Adds or improves the existing tests related to the code base.
    Test(Arguments),
}

impl Template {
    fn file_name(&self) -> &str {
        match self {
            Template::Bug(_) => "bug.md",
            Template::Feature(_) => "feature.md",
            Template::Refactor(_) => "refactor.md",
            Template::Break(_) => "break.md",
            Template::Deps(_) => "deps.md",
            Template::Docs(_) => "docs.md",
            Template::Test(_) => "test.md",
        }
    }

    fn args(&self) -> &Arguments {
        match &self {
            Template::Bug(args) => args,
            Template::Feature(args) => args,
            Template::Refactor(args) => args,
            Template::Break(args) => args,
            Template::Deps(args) => args,
            Template::Docs(args) => args,
            Template::Test(args) => args,
        }
    }

    fn read_file(&self, project_dir: &ProjectDirs) -> anyhow::Result<String> {
        let file_name = self.file_name();
        let sub_dir = format!("templates/commit/{}", file_name);
        let template = project_dir.config_dir().join(sub_dir);

        let contents: String = fs::read_to_string(&template)
            .with_context(|| format!("Failed to read template '{}'", file_name))?
            .parse()?;

        Ok(contents)
    }

    pub fn commit<C: GitCommands>(&self, context: &Context<C>) -> anyhow::Result<String> {
        let args = self.args();
        let template = self.read_file(&context.project_dir)?;
        let contents = args.commit_message(template, &context)?;

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rusqlite::Connection;
    use std::{
        env,
        path::{Path, PathBuf},
    };
    use uuid::Uuid;

    use crate::{branch::Branch, git_commands::Git};

    use super::*;

    fn fake_args() -> Arguments {
        Arguments {
            ticket: Faker.fake(),
            message: Faker.fake(),
        }
    }

    #[test]
    fn template_args() {
        let args = fake_args();
        let templates = vec![
            Template::Bug(args.clone()),
            Template::Feature(args.clone()),
            Template::Refactor(args.clone()),
            Template::Break(args.clone()),
            Template::Deps(args.clone()),
            Template::Docs(args.clone()),
            Template::Test(args.clone()),
        ];

        templates.iter().for_each(|template| {
            assert_eq!(template.args(), &args);
        });
    }

    #[test]
    fn template_filename() {
        let templates = vec![
            (Template::Bug(fake_args()), "bug.md"),
            (Template::Feature(fake_args()), "feature.md"),
            (Template::Refactor(fake_args()), "refactor.md"),
            (Template::Break(fake_args()), "break.md"),
            (Template::Deps(fake_args()), "deps.md"),
            (Template::Docs(fake_args()), "docs.md"),
            (Template::Test(fake_args()), "test.md"),
        ];

        templates.iter().for_each(|(template, expected)| {
            assert_eq!(template.file_name(), expected.to_string());
        });
    }

    #[test]
    fn read_file() -> anyhow::Result<()> {
        let (dirs, templates_path) = fake_project_dir()?;

        let expected_templates = [
            Template::Bug(fake_args()).read_file(&dirs)?.contains("🐛"),
            Template::Feature(fake_args())
                .read_file(&dirs)?
                .contains("✨"),
            Template::Refactor(fake_args())
                .read_file(&dirs)?
                .contains("🧹"),
            Template::Break(fake_args()).read_file(&dirs)?.contains("⚠️"),
            Template::Deps(fake_args()).read_file(&dirs)?.contains("📦"),
            Template::Docs(fake_args()).read_file(&dirs)?.contains("📖"),
            Template::Test(fake_args()).read_file(&dirs)?.contains("🧪"),
        ];

        assert!(expected_templates
            .iter()
            .all(|contains| contains.to_owned()));

        fs::remove_dir_all(templates_path)?;

        Ok(())
    }

    #[test]
    fn commit_msg_without_ticket_override_using_branch_name() -> anyhow::Result<()> {
        let (dirs, templates_path) = fake_project_dir()?;

        let context = Context {
            project_dir: dirs,
            connection: Connection::open_in_memory()?,
            commands: Git,
        };

        let args = Arguments {
            message: Faker.fake(),
            ticket: None,
        };

        let branch_name = context.commands.get_branch_name()?;
        let repo_name = context.commands.get_repo_name()?;
        setup_db(
            &context.connection,
            Some(&fake_branch(Some(branch_name.clone()), Some(repo_name))?),
        )?;

        let expected_templates = [
            ("🐛", Template::Bug(args.clone()).commit(&context)?),
            ("✨", Template::Feature(args.clone()).commit(&context)?),
            ("🧹", Template::Refactor(args.clone()).commit(&context)?),
            ("⚠️", Template::Break(args.clone()).commit(&context)?),
            ("📦", Template::Deps(args.clone()).commit(&context)?),
            ("📖", Template::Docs(args.clone()).commit(&context)?),
            ("🧪", Template::Test(args.clone()).commit(&context)?),
        ];

        for (template, msg) in expected_templates {
            let expected = format!(
                "[{}] {} {}",
                branch_name,
                template,
                args.message.clone().unwrap_or("".into())
            );
            assert_eq!(msg, expected);
        }

        fs::remove_dir_all(templates_path)?;
        context.close()?;

        Ok(())
    }

    #[test]
    fn commit_msg_with_ticket_override() -> anyhow::Result<()> {
        let (dirs, templates_path) = fake_project_dir()?;

        let context = Context {
            project_dir: dirs,
            connection: Connection::open_in_memory()?,
            commands: Git,
        };

        let args = Arguments {
            message: Faker.fake(),
            ticket: Some(Faker.fake()),
        };

        let expected_templates = [
            ("🐛", Template::Bug(args.clone()).commit(&context)?),
            ("✨", Template::Feature(args.clone()).commit(&context)?),
            ("🧹", Template::Refactor(args.clone()).commit(&context)?),
            ("⚠️", Template::Break(args.clone()).commit(&context)?),
            ("📦", Template::Deps(args.clone()).commit(&context)?),
            ("📖", Template::Docs(args.clone()).commit(&context)?),
            ("🧪", Template::Test(args.clone()).commit(&context)?),
        ];

        for (template, msg) in expected_templates {
            let expected = format!(
                "[{}] {} {}",
                args.ticket.clone().unwrap_or("".into()),
                template,
                args.message.clone().unwrap_or("".into())
            );
            assert_eq!(msg, expected);
        }

        fs::remove_dir_all(templates_path)?;
        context.close()?;

        Ok(())
    }

    fn copy_or_replace(source_path: &PathBuf, target_path: &PathBuf) -> std::io::Result<()> {
        match fs::read_dir(source_path) {
            Ok(entry_iter) => {
                fs::create_dir_all(target_path)?;
                for dir in entry_iter {
                    let entry = dir?;
                    copy_or_replace(&entry.path(), &target_path.join(entry.file_name()))?;
                }
            }
            Err(_) => {
                fs::copy(&source_path, &target_path)?;
            }
        }

        Ok(())
    }

    fn fake_project_dir() -> anyhow::Result<(ProjectDirs, PathBuf)> {
        let dirs = ProjectDirs::from(&format!("{}", Uuid::new_v4()), "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        // https://doc.rust-lang.org/cargo/reference/environment-variables.html
        let project_root = &env::var("CARGO_MANIFEST_DIR")?;
        let templates_path = &dirs.config_dir().join("templates/");

        copy_or_replace(&Path::new(project_root).join("templates/"), templates_path)?;

        Ok((dirs, templates_path.to_owned()))
    }

    fn fake_branch(name: Option<String>, repo: Option<String>) -> anyhow::Result<Branch> {
        let name = name.unwrap_or(Faker.fake());
        let repo = repo.unwrap_or(Faker.fake());

        Ok(Branch::new(&name, &repo, None)?)
    }

    fn setup_db(conn: &Connection, branch: Option<&Branch>) -> anyhow::Result<()> {
        conn.execute(
            "CREATE TABLE branch (
                name TEXT NOT NULL PRIMARY KEY,
                ticket TEXT,
                data BLOB,
                created TEXT NOT NULL
            )",
            (),
        )?;

        if let Some(branch) = branch {
            conn.execute(
                "INSERT INTO branch (name, ticket, data, created) VALUES (?1, ?2, ?3, ?4)",
                (
                    &branch.name,
                    &branch.ticket,
                    &branch.data,
                    branch.created.to_rfc3339(),
                ),
            )?;
        }

        Ok(())
    }
}
