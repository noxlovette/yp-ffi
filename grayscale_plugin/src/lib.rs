use image::{ImageBuffer, Rgba, imageops};
use plugin_interface::GrayscaleParams;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::{self, AssertUnwindSafe};
use std::slice;
use tracing::error;

fn grayscale(
    width: u32,
    height: u32,
    data: &mut [u8],
    _params: &GrayscaleParams,
) -> Result<(), String> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or_else(|| "width/height do not match buffer length".to_string())?;

    // drops alpha
    let luma = imageops::colorops::grayscale(&img);

    let mut out: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    for (x, y, px) in luma.enumerate_pixels() {
        let l = px.0[0];
        let a = img.get_pixel(x, y).0[3];
        out.put_pixel(x, y, Rgba([l, l, l, a]));
    }

    data.copy_from_slice(out.as_raw());
    Ok(())
}

#[unsafe(no_mangle)]
/// Converts the given image to grayscale using the image crate
///
/// # Safety
///
/// will not panic, no UB
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    plugin_interface::init_plugin_tracing();
    let _span = tracing::info_span!("process_image", width, height).entered();

    if rgba_data.is_null() || params.is_null() {
        error!("grayscale: null pointer passed to process_image");
        return;
    }

    let params_str = match unsafe { CStr::from_ptr(params) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            error!("grayscale: params must be valid UTF-8: {e}");
            return;
        }
    };

    let len = width as usize * height as usize * 4;
    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let params: GrayscaleParams = serde_json::from_str(params_str)
            .map_err(|e| format!("invalid grayscale params: {e}"))?;
        grayscale(width, height, data, &params)
    }));

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => error!("grayscale: {e}"),
        Err(_) => error!("grayscale: panicked while processing image"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desaturates_a_pure_red_pixel() {
        let mut data = vec![255, 0, 0, 255];
        grayscale(1, 1, &mut data, &GrayscaleParams {}).unwrap();
        assert_eq!(data[0], data[1]);
        assert_eq!(data[1], data[2]);
        assert_eq!(data[3], 255);
    }

    #[test]
    fn preserves_alpha() {
        let mut data = vec![10, 20, 30, 128];
        grayscale(1, 1, &mut data, &GrayscaleParams {}).unwrap();
        assert_eq!(data[3], 128);
    }

    #[test]
    fn grays_are_unchanged() {
        let mut data = vec![100, 100, 100, 255, 200, 200, 200, 255];
        let original = data.clone();
        grayscale(2, 1, &mut data, &GrayscaleParams {}).unwrap();
        assert_eq!(data, original);
    }
}
