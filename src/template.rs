use anyhow::Context;
use clap::Subcommand;
use directories::ProjectDirs;
use std::fs;

use crate::args::Arguments;

#[derive(Debug, Subcommand)]
pub enum Template {
    Break(Arguments),
    Bug(Arguments),
    Deps(Arguments),
    Docs(Arguments),
    Feature(Arguments),
    Refactor(Arguments),
    Test(Arguments),
}

impl Template {
    pub fn file_name(&self) -> &str {
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

    pub fn args(&self) -> &Arguments {
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

    pub fn read_file(&self, project_dir: &ProjectDirs) -> anyhow::Result<String> {
        let file_name = self.file_name();
        let sub_dir = format!("templates/commit/{}", file_name);
        let template = project_dir.config_dir().join(sub_dir);

        let contents: String = fs::read_to_string(&template)
            .with_context(|| format!("Failed to read template '{}'", file_name))?
            .parse()?;

        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use std::{
        env,
        path::{Path, PathBuf},
    };

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
        let dirs = ProjectDirs::from("test", "xsv24", "git-kit")
            .context("Failed to retrieve 'git-kit' config")?;

        // https://doc.rust-lang.org/cargo/reference/environment-variables.html
        let project_root = &env::var("CARGO_MANIFEST_DIR").unwrap();
        let templates_path = &dirs.config_dir().join("templates/");

        copy_or_replace(&Path::new(project_root).join("templates/"), templates_path)?;

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
}
