use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::ffi::{CString, NulError};
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
    #[error("failed to load plugin library: {0}")]
    LibraryLoad(#[from] libloading::Error),
    #[error("params JSON contains an interior null byte: {0}")]
    InvalidParams(#[from] NulError),
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MirrorParams {
    #[serde(default)]
    pub horizontal: bool,
    #[serde(default)]
    pub vertical: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlurParams {
    // missing keys default to a no-op blur (radius 0, 0 iterations)
    #[serde(default)]
    pub radius: u32,
    #[serde(default)]
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
) -> Result<(), Error> {
    let path: PathBuf = dir.join(plugin.as_lib_name()?);
    let lib = unsafe { Library::new(path)? };
    let func: Symbol<ProcessImageFn> = unsafe { lib.get(b"process_image\0")? };
    let params = CString::new(params_json)?;

    unsafe { func(width, height, data.as_mut_ptr(), params.as_ptr()) };

    Ok(())
}

#[cfg(test)]
mod params_tests {
    use super::*;

    #[test]
    fn mirror_params_parses_valid_json() {
        let params: MirrorParams =
            serde_json::from_str(r#"{"horizontal":true,"vertical":false}"#).unwrap();
        assert_eq!(
            params,
            MirrorParams {
                horizontal: true,
                vertical: false
            }
        );
    }

    #[test]
    fn mirror_params_defaults_missing_keys_to_false() {
        let params: MirrorParams = serde_json::from_str("{}").unwrap();
        assert_eq!(
            params,
            MirrorParams {
                horizontal: false,
                vertical: false
            }
        );
    }

    #[test]
    fn mirror_params_rejects_empty_input() {
        assert!(serde_json::from_str::<MirrorParams>("").is_err());
    }

    #[test]
    fn mirror_params_rejects_unknown_keys() {
        assert!(
            serde_json::from_str::<MirrorParams>(r#"{"horizontal":true,"flip":true}"#).is_err()
        );
    }

    #[test]
    fn mirror_params_rejects_non_bool_values() {
        assert!(serde_json::from_str::<MirrorParams>(r#"{"horizontal":"yes"}"#).is_err());
    }

    #[test]
    fn blur_params_parses_valid_json() {
        let params: BlurParams = serde_json::from_str(r#"{"radius":3,"iterations":2}"#).unwrap();
        assert_eq!(
            params,
            BlurParams {
                radius: 3,
                iterations: 2
            }
        );
    }

    #[test]
    fn blur_params_defaults_missing_keys_to_zero() {
        let params: BlurParams = serde_json::from_str("{}").unwrap();
        assert_eq!(
            params,
            BlurParams {
                radius: 0,
                iterations: 0
            }
        );
    }

    #[test]
    fn blur_params_rejects_empty_input() {
        assert!(serde_json::from_str::<BlurParams>("").is_err());
    }

    #[test]
    fn blur_params_rejects_unknown_keys() {
        assert!(serde_json::from_str::<BlurParams>(r#"{"radius":3,"passes":2}"#).is_err());
    }

    #[test]
    fn blur_params_rejects_non_numeric_values() {
        assert!(
            serde_json::from_str::<BlurParams>(r#"{"radius":"three","iterations":2}"#).is_err()
        );
    }
}
