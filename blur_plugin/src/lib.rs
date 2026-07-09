use image::{ImageBuffer, Rgba, imageops};
use plugin_interface::BlurParams;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::{self, AssertUnwindSafe};
use std::slice;
use tracing::error;

/// Kept separate for testing purposes
fn blur(width: u32, height: u32, data: &mut [u8], params: &BlurParams) -> Result<(), String> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or_else(|| "width/height do not match buffer length".to_string())?;

    let img = imageops::blur(&img, params.sigma);

    data.copy_from_slice(img.as_raw());
    Ok(())
}

#[unsafe(no_mangle)]
/// Gaussian-blurs the given image using the image crate
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
        error!("blur: null pointer passed to process_image");
        return;
    }

    let params_str = match unsafe { CStr::from_ptr(params) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            error!("blur: params must be valid UTF-8: {e}");
            return;
        }
    };

    let len = width as usize * height as usize * 4;
    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let params: BlurParams =
            serde_json::from_str(params_str).map_err(|e| format!("invalid blur params: {e}"))?;

        blur(width, height, data, &params)
    }));

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => error!("blur: {e}"),
        Err(_) => error!("blur: panicked while processing image"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_buffer_is_unchanged_by_blur() {
        let mut data = vec![42u8; 4 * 4 * 4];
        let original = data.clone();

        blur(4, 4, &mut data, &BlurParams { sigma: 2.0 }).unwrap();

        assert_eq!(data, original);
    }

    #[test]
    fn bright_center_pixel_bleeds_into_neighbors() {
        let mut data = vec![0u8; 3 * 3 * 4];
        let center = 3 * 4;
        data[center..center + 4].copy_from_slice(&[255, 255, 255, 255]);

        blur(3, 3, &mut data, &BlurParams { sigma: 1.0 }).unwrap();

        let neighbors = [(0usize, 1usize), (1, 0), (1, 2), (2, 1)];
        for (row, col) in neighbors {
            let idx = (row * 3 + col) * 4;
            assert!(
                data[idx] > 0,
                "pixel at ({row},{col}) should have brightened, got {:?}",
                &data[idx..idx + 4]
            );
        }
    }
}
