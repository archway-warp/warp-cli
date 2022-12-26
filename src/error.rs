use std::{
    path::PathBuf,
    process::{ExitCode, ExitStatus},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WarpError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("TOML Serialiation Error {0:?}")]
    TomlSerializationError(#[from] toml::ser::Error),
    #[error("Toml Deserialization Error: {0:?}")]
    TomlDeserializationError(#[from] toml::de::Error),
    #[error("Error: Project file can't be found. You have to navigate to a valid Warp project directory.")]
    ProjectFileNotFound,
    #[error("Error: Another Warp project already exists at '{0}'.")]
    ProjectFileAlreadyExists(PathBuf),
    #[error("Error: Starting the local node failed with exit code: {0}")]
    NodeStartupError(ExitStatus),
    #[error("Error: Workspace initialization failed.")]
    InitFailed,
    #[error("Error: Regex parser threw an error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Error: Could not clone the contract template.")]
    ContractTemplateCloneFailed,
}
