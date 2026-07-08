use image::{ImageBuffer, Rgba, imageops};
use plugin_interface::MirrorParams;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::{self, AssertUnwindSafe};
use std::slice;
use tracing::{error, info};

fn mirror(width: u32, height: u32, data: &mut [u8], params: &MirrorParams) -> Result<(), String> {
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or_else(|| "width/height do not match buffer length".to_string())?;

    if params.horizontal {
        imageops::flip_horizontal_in_place(&mut img);
    }
    if params.vertical {
        imageops::flip_vertical_in_place(&mut img);
    }

    data.copy_from_slice(img.as_raw());
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    plugin_interface::init_plugin_tracing();
    let _span = tracing::info_span!("process_image", width, height).entered();

    if rgba_data.is_null() || params.is_null() {
        error!("mirror: null pointer passed to process_image");
        return;
    }

    let params_str = match unsafe { CStr::from_ptr(params) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            error!("mirror: params must be valid UTF-8: {e}");
            return;
        }
    };

    let len = width as usize * height as usize * 4;
    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let params: MirrorParams =
            serde_json::from_str(params_str).map_err(|e| format!("invalid mirror params: {e}"))?;
        mirror(width, height, data, &params)
    }));

    match result {
        Ok(Ok(())) => info!("mirror applied to image"),
        Ok(Err(e)) => error!("mirror: {e}"),
        Err(_) => error!("mirror: panicked while processing image"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_2x2() -> Vec<u8> {
        vec![
            10, 10, 10, 255, 20, 20, 20, 255, // A B
            30, 30, 30, 255, 40, 40, 40, 255, // C D
        ]
    }

    #[test]
    fn horizontal_flip_swaps_columns() {
        let mut data = sample_2x2();
        mirror(
            2,
            2,
            &mut data,
            &MirrorParams {
                horizontal: true,
                vertical: false,
            },
        )
        .unwrap();
        assert_eq!(
            data,
            vec![
                20, 20, 20, 255, 10, 10, 10, 255, // B A
                40, 40, 40, 255, 30, 30, 30, 255, // D C
            ]
        );
    }

    #[test]
    fn vertical_flip_swaps_rows() {
        let mut data = sample_2x2();
        mirror(
            2,
            2,
            &mut data,
            &MirrorParams {
                horizontal: false,
                vertical: true,
            },
        )
        .unwrap();
        assert_eq!(
            data,
            vec![
                30, 30, 30, 255, 40, 40, 40, 255, // C D
                10, 10, 10, 255, 20, 20, 20, 255, // A B
            ]
        );
    }

    #[test]
    fn both_flips_rotate_180() {
        let mut data = sample_2x2();
        mirror(
            2,
            2,
            &mut data,
            &MirrorParams {
                horizontal: true,
                vertical: true,
            },
        )
        .unwrap();
        assert_eq!(
            data,
            vec![
                40, 40, 40, 255, 30, 30, 30, 255, // D C
                20, 20, 20, 255, 10, 10, 10, 255, // B A
            ]
        );
    }

    #[test]
    fn no_flip_is_identity() {
        let mut data = sample_2x2();
        let original = data.clone();
        mirror(
            2,
            2,
            &mut data,
            &MirrorParams {
                horizontal: false,
                vertical: false,
            },
        )
        .unwrap();
        assert_eq!(data, original);
    }
}
