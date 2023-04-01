use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error(transparent)]
    Git(GitError),

    #[error(transparent)]
    UserInput(UserInputError),

    #[error("Invalid configuration {}", .message.to_lowercase())]
    Configuration {
        message: String,
        source: anyhow::Error,
    },

    #[error(transparent)]
    PersistError(PersistError),

    #[error("Validation error occurred {}", .message.to_lowercase())]
    ValidationError { message: String },
}

#[derive(Error, Debug)]
pub enum UserInputError {
    #[error("Missing required {name:?} input")]
    Required { name: String },

    #[error("Invalid command {name:?} found")]
    InvalidCommand { name: String },

    #[error("Input prompt cancelled by user")]
    Cancelled,

    #[error("Invalid input {name:?} found {}", .message.to_lowercase())]
    Validation { name: String, message: String },
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed retrieve the current git branch name")]
    BranchName,

    #[error("Failed retrieve the current git root directory")]
    RootDirectory,

    #[error("Failed to checkout branch {name:?}")]
    Checkout { name: String },

    #[error("Failed to apply commit")]
    Commit,

    #[error("Validation error occurred {message}")]
    Validation { message: String },
}

#[derive(Error, Debug)]
pub enum PersistError {
    #[error("Invalid store configuration")]
    Configuration,

    #[error("Persisted {name:?} has been corrupted or is out of date")]
    Corrupted {
        name: String,
        source: Option<anyhow::Error>,
    },

    #[error("Requested {name:?} not found in persisted store")]
    NotFound { name: String },

    #[error("Failed to persist or retrieve {name:?}")]
    Validation { name: String, source: anyhow::Error },

    #[error("Unknown error occurred while connecting persisted store")]
    Unknown(anyhow::Error),
}
