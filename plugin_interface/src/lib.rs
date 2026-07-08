//! Going slightly beyond the limitations imposed by the task,
//! This is more than just an interface. Rather, this is the central workspace lib
#![warn(missing_docs)]

use libloading::{Library, Symbol};
use std::ffi::CString;
use std::fmt::Display;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tracing::{info, instrument};
mod error;
mod params;
pub use error::*;
pub use params::*;

static PLUGIN_TRACING_INIT: Once = Once::new();

/// Installs a `tracing` subscriber local to the calling dylib
pub fn init_plugin_tracing() {
    PLUGIN_TRACING_INIT.call_once(|| {
        let filter =
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    });
}

/// The C ABI every plugin dylib must export.
pub type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char);

/// The plugin to use. No prefixes, no extensions.
#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Plugin {
    /// this guy mirrors your image
    Mirror,
    /// this guy blurs the image
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

/// loads `plugin`'s dylib from `dir` and calls its `process_image` export
#[instrument(skip(data))]
pub fn call_dynamic(
    dir: &Path,
    plugin: Plugin,
    width: u32,
    height: u32,
    data: &mut [u8],
    params_json: &str,
) -> Result<(), Error> {
    info!("calling dynamic plugin load");
    let path: PathBuf = dir.join(plugin.as_lib_name()?);
    if !path.is_file() {
        return Err(Error::PluginLibraryNotFound(path));
    }
    let lib = unsafe { Library::new(path)? };
    let func: Symbol<ProcessImageFn> = unsafe { lib.get(b"process_image\0")? };
    let params = CString::new(params_json)?;

    unsafe { func(width, height, data.as_mut_ptr(), params.as_ptr()) };
    info!("finished processing image");
    Ok(())
}
