use colored::Colorize;
use std::io;

use crate::domain::errors::{Errors, PersistError, UserInputError};

pub fn display_error(err: Errors) -> io::Result<()> {
    let message = err.to_string();
    match err {
        Errors::UserInput(inner) => print_user_error(inner),
        Errors::Git(inner) => print_error(inner.to_string(), None),
        Errors::PersistError(err) => print_persist_error(err),
        Errors::Configuration { source, .. } => print_error(message, Some(source)),
        Errors::ValidationError { .. } => print_error(message, None),
    }
}

fn print_user_error(err: UserInputError) -> io::Result<()> {
    // TODO: Need builder app to format error with clap
    // let input_error = match &err {
    //     UserInputError::Required { .. } => {
    //         clap::Error::raw(ErrorKind::MissingRequiredArgument, err)
    //     }
    //     UserInputError::InvalidCommand { .. } => {
    //         clap::Error::raw(ErrorKind::InvalidSubcommand, err)
    //     }
    //     UserInputError::Validation { .. } => clap::Error::raw(ErrorKind::ValueValidation, err),
    //     UserInputError::Cancelled => clap::Error::raw(ErrorKind::Io, "Operation cancelled"),
    // };
    // input_error.format(cmd)

    print_error(err.to_string(), None)
}

fn print_persist_error(err: PersistError) -> io::Result<()> {
    let message = err.to_string();

    match err {
        PersistError::Configuration => print_error(message, None),
        PersistError::NotFound { .. } => print_error(message, None),
        PersistError::Unknown(source) => print_error(message, Some(source)),
        PersistError::Validation { source, .. } => print_error(message, Some(source)),
        PersistError::Corrupted { source, .. } => print_error(message, source),
    }
}

fn print_error(message: String, source: Option<anyhow::Error>) -> io::Result<()> {
    log::error!("{} {:?}", message, source);
    println!("{}: {message}", "error".red());
    Ok(())
}
