use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Issue ticket number related to the current branch.
    #[clap(value_parser)]
    pub ticket: String,
}
