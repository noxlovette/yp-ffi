use std::{ffi::NulError, path::PathBuf};
use thiserror::Error;

/// The central lib error
#[derive(Debug, Error)]
pub enum Error {
    /// Not windows, linux, or macos
    #[error("unsupported OS: {0}")]
    UnsupportedOs(String),
    /// library load fuck-up
    #[error("failed to load plugin library: {0}")]
    LibraryLoad(#[from] libloading::Error),
    /// json null byte issues
    #[error("params JSON contains an interior null byte: {0}")]
    InvalidParams(#[from] NulError),
    #[error("plugin library not found: {0}")]
    PluginLibraryNotFound(PathBuf),
}
