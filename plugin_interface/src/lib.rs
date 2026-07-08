use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fmt::Display;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The C ABI every plugin dylib must export.
pub type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char);

#[derive(Debug, Error)]
pub enum Error {
    #[error("unsupported OS: {0}")]
    UnsupportedOs(String),
}

/// The plugin to use. No prefixes, no extensions.
#[derive(Clone, Debug, clap::ValueEnum)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MirrorParams {
    pub horizontal: bool,
    pub vertical: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlurParams {
    pub radius: u32,
    pub iterations: u32,
}

/// loads `plugin`'s dylib from `dir` and calls its `process_image` export,
/// mutating `data` in place.
/// `params_json` is forwarded to the plugin as its JSON config.
pub fn call_dynamic(
    dir: &Path,
    plugin: Plugin,
    width: u32,
    height: u32,
    data: &mut [u8],
    params_json: &str,
) -> anyhow::Result<()> {
    let path: PathBuf = dir.join(plugin.as_lib_name()?);
    let lib = unsafe { Library::new(path)? };
    let func: Symbol<ProcessImageFn> = unsafe { lib.get(b"process_image\0")? };
    let params = CString::new(params_json)?;

    unsafe { func(width, height, data.as_mut_ptr(), params.as_ptr()) };

    Ok(())
}
