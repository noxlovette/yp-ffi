use std::{fmt::Display, os::raw::c_char};
use thiserror::Error;

/// Common interface for plugins in this workspace
///
/// Don't forget to add unsafe no mangle
pub trait PluginInt {
    extern "C" fn process_image(width: u32, height: u32, rgba_data: *mut u8, params: *const c_char);
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported OS: {0}")]
    UnsupportedOs(String),
}

/// The plugin to use. No prefixes, no extensions
#[derive(Clone, Debug)]
pub enum Plugin {
    Mirror,
    Blur,
}

impl Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Plugin {
    fn as_str(&self) -> &'static str {
        match self {
            Plugin::Blur => "blur",
            Plugin::Mirror => "mirror",
        }
    }

    /// A fallible method that returns the name of the associated library
    ///
    /// Fails if an OS is not supported
    pub fn as_lib_name(&self) -> Result<String, Error> {
        Ok(match std::env::consts::OS {
            "macos" => format!("lib{}.dylib", self.as_str()),
            "windows" => format!("{}.dll", self.as_str()),
            "linux" => format!("lib{}.so", self.as_str()),
            other => return Err(Error::UnsupportedOs(other.into())),
        })
    }
}
