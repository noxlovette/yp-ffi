use libloading::{Library, Symbol};
use plugin_interface::Plugin;
use std::{
    os::raw::c_char,
    path::{Path, PathBuf},
};

type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char);

pub fn call_dynamic(path: PathBuf, plugin: Plugin) -> anyhow::Result<()> {
    let lib = unsafe { Library::new(path.join(Path::new(&plugin.as_lib_name()?)))? };
    let func: Symbol<ProcessImageFn> = unsafe { lib.get(b"process_image\0")? };

    // now pass this to the plugin
    Ok(func())
}
