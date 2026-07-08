use image::{ImageBuffer, Rgba, imageops};
use plugin_interface::MirrorParams;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;

/// Applies the mirror transform to a raw RGBA buffer in place. Kept separate
/// from `process_image` so it can be unit-tested without going through FFI.
fn mirror(width: u32, height: u32, data: &mut [u8], params: &MirrorParams) {
    let mut img: ImageBuffer<Rgba<u8>, &mut [u8]> = ImageBuffer::from_raw(width, height, data)
        .expect("width/height do not match buffer length");

    if params.horizontal {
        imageops::flip_horizontal_in_place(&mut img);
    }
    if params.vertical {
        imageops::flip_vertical_in_place(&mut img);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    let params_str = unsafe { CStr::from_ptr(params) }
        .to_str()
        .expect("params must be valid UTF-8");
    let params: MirrorParams = serde_json::from_str(params_str).expect("invalid mirror params");

    let len = width as usize * height as usize * 4;
    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };

    mirror(width, height, data, &params);
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2x2 buffer, row-major, one distinct color per pixel:
    // A B
    // C D
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
        );
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
        );
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
        );
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
        );
        assert_eq!(data, original);
    }
}
