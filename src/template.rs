use clap::{Args, Subcommand};

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

#[derive(Debug, Args)]
pub struct Arguments {
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,

    #[clap(short, long, value_parser)]
    pub message: Option<String>,
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
}
