use crate::Error;
use plugin_interface::Plugin;
use std::path::Path;

pub fn call_dynamic(
    dir: &Path,
    plugin: Plugin,
    width: u32,
    height: u32,
    data: &mut [u8],
    params_json: &str,
) -> Result<(), Error> {
    let lib_path = dir.join(plugin.as_lib_name()?);
    if !lib_path.is_file() {
        return Err(Error::PluginLibraryNotFound(lib_path));
    }

    plugin_interface::call_dynamic(dir, plugin, width, height, data, params_json)?;
    Ok(())
}
