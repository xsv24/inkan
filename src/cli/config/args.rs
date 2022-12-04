#[derive(Debug, Clone, clap::Subcommand)]
pub enum Arguments {
    /// Add / register a custom config file.
    Add {
        /// Name used to reference the config file.
        name: String,
        /// File path to the config file.
        path: String,
    },
    /// Switch to another config file.
    Set {
        /// Name used to reference the config file.
        name: Option<String>,
    },
    /// Display the current config in use.
    Show,
    /// Reset to the default config.
    Reset,
}
