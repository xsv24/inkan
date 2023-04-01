use clap::Parser;
use git_kit::{adapters::prompt::Prompt, cli::error::display_error, entry::Cli};

fn main() -> anyhow::Result<()> {
    // TODO: Dynamic builder + derive https://docs.rs/clap/latest/clap/_derive/index.html#mixing-builder-and-derive-apis
    let cli = Cli::parse();

    let mut context = cli.init()?;
    let result = cli.commands.execute(&mut context, Prompt);

    // close the connection no matter if we error or not.
    context.close()?;

    if let Err(error) = result {
        display_error(error)?;
    }

    Ok(())
}
