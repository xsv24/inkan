use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Issue ticket number related to the current branch.
    #[clap(value_parser)]
    pub ticket: Option<String>,

    /// Short describing a section of the codebase the changes relate to.
    #[clap(short, long, value_parser)]
    pub scope: Option<String>,

    /// Issue ticket number link.
    #[clap(short, long, value_parser)]
    pub link: Option<String>,
}
