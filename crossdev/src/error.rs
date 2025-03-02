use crate::config::PackType;
use std::string::FromUtf8Error;
use thiserror::*;

#[derive(Error, Debug)]
pub enum CrossDevError {
    #[error("No toolchain found in the config file")]
    NoToolchainInConfig,

    #[error("Cross toolchain path not set")]
    CrossToolchainPathNotSet,

    #[error("Cross toolchain path already set")]
    CrossToolchainPathAlreadySet,

    #[error("Env vars already set")]
    EnvVarsAlreadySet,

    #[error("Root directory not set")]
    RootDirNotSet,

    #[error("Home directory not set")]
    HomeDirNotSet,

    #[error("Work directory not set")]
    WorkDirNotSet,

    #[error("Failed to find gcc from toolchain")]
    GccNotFound,

    #[error("Failed to get HOME directory")]
    HomeDirError(#[from] std::env::VarError),

    #[error("Failed to get ENV variables")]
    EnvVarsError,

    #[error("Failed to get the package {0}")]
    PackageError(PackType),

    #[error("Failed to convert to utf8")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Failed to convert to utf8")]
    Utf8Error(FromUtf8Error),

    #[error("Failed to get a new shell")]
    XshellError(#[from] xshell::Error),

    #[error("Failed to do a command")]
    StdIoError(#[from] std::io::Error),

    #[error("Failed to generate SSH key")]
    SshKeyError,

    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("Directory not created: {0}")]
    DirectoryNotCreated(String),
}

pub type Result<T> = std::result::Result<T, CrossDevError>;
