use anyhow::Context;
use clap::Subcommand;
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

    pub fn read_file(&self) -> anyhow::Result<String> {
        let template = format!("templates/commit/{}", self.file_name());

        let contents: String = fs::read_to_string(&template)
            .with_context(|| format!("Failed to read template '{}'", template))?
            .parse()?;

        Ok(contents)
    }
}
