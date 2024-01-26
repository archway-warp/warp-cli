use std::{path::PathBuf, process::ExitStatus, string::FromUtf8Error};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WarpError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("TOML Serialiation Error {0:?}")]
    TomlSerializationError(#[from] toml::ser::Error),
    #[error("Toml Deserialization Error: {0:?}")]
    TomlDeserializationError(#[from] toml::de::Error),
    #[error("Project file can't be found. You have to navigate to a valid Warp project directory.")]
    ProjectFileNotFound,
    #[error("Another Warp project already exists at '{0}'.")]
    ProjectFileAlreadyExists(PathBuf),
    #[error("Starting the local node failed with exit code: {0}")]
    NodeStartupError(ExitStatus),
    #[error("Workspace initialization failed.")]
    InitFailed,
    #[error("Regex parser threw an error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Could not clone the contract template.")]
    ContractTemplateCloneFailed,
    #[error("Could not parse the UTF8 string: {0}")]
    FromUTF8Error(#[from] FromUtf8Error),
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Transaction reverted: {1} (Tx: {0})")]
    TxFailed(String, String),
    #[error("Wallet not specified")]
    UnspecifiedWallet,
    #[error("{0}")]
    UnderlyingCliError(String),
    #[error("Can't match the following ID: '{0}'")]
    ContractIdNotFound(String),
}
