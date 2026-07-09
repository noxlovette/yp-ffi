use miette::Diagnostic;
use std::{ffi::NulError, path::PathBuf};
use thiserror::Error;

/// The central lib error
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// Not windows, linux, or macos
    #[error("unsupported OS: {0}")]
    #[diagnostic(
        code(plugin_interface::unsupported_os),
        help("this plugin loader only knows how to name dylibs for macos, linux, and windows")
    )]
    UnsupportedOs(String),
    /// library load fuck-up
    #[error("failed to load plugin library: {0}")]
    #[diagnostic(
        code(plugin_interface::library_load),
        help("is the plugin built for this platform, and does it export `process_image`?")
    )]
    LibraryLoad(#[from] libloading::Error),
    /// json null byte issues
    #[error("params JSON contains an interior null byte: {0}")]
    #[diagnostic(
        code(plugin_interface::invalid_params),
        help("strip any embedded NUL bytes from the params JSON before passing it in")
    )]
    InvalidParams(#[from] NulError),
    /// plugin dylib missing from the plugin path
    #[error("plugin library not found: {0}")]
    #[diagnostic(
        code(plugin_interface::plugin_not_found),
        help("check --plugin-path, or build the plugin crate first")
    )]
    PluginLibraryNotFound(PathBuf),
}
