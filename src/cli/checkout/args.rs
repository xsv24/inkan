use clap::Args;

#[derive(Debug, Args, Clone)]
pub struct Arguments {
    /// Name of the branch to checkout or create.
    #[clap(value_parser)]
    pub name: String,

    /// Issue ticket number related to the branch.
    #[clap(short, long, value_parser)]
    pub ticket: Option<String>,
}
