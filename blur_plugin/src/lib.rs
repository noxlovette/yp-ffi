use image::{ImageBuffer, Rgba, imageops};
use plugin_interface::BlurParams;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;

/// Applies the blur transform to a raw RGBA buffer in place. Kept separate
/// from `process_image` so it can be unit-tested without going through FFI.
fn blur(width: u32, height: u32, data: &mut [u8], params: &BlurParams) {
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data.to_vec())
            .expect("width/height do not match buffer length");

    for _ in 0..params.iterations {
        img = imageops::fast_blur(&img, params.radius as f32);
    }

    data.copy_from_slice(img.as_raw());
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
    let params: BlurParams = serde_json::from_str(params_str).expect("invalid blur params");

    let len = width as usize * height as usize * 4;
    let data = unsafe { slice::from_raw_parts_mut(rgba_data, len) };

    blur(width, height, data, &params);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_buffer_is_unchanged_by_blur() {
        // Every pixel identical: the mean of any neighborhood is the same
        // color, so a box blur must be a no-op regardless of radius.
        let mut data = vec![42u8; 4 * 4 * 4]; // 4x4 RGBA, all pixels (42,42,42,42)
        let original = data.clone();

        blur(
            4,
            4,
            &mut data,
            &BlurParams {
                radius: 1,
                iterations: 2,
            },
        );

        assert_eq!(data, original);
    }

    #[test]
    fn bright_center_pixel_bleeds_into_neighbors() {
        // 3x3 buffer, black everywhere except a white center pixel.
        let mut data = vec![0u8; 3 * 3 * 4];
        let center = (1 * 3 + 1) * 4; // row 1, col 1
        data[center..center + 4].copy_from_slice(&[255, 255, 255, 255]);

        blur(
            3,
            3,
            &mut data,
            &BlurParams {
                radius: 1,
                iterations: 1,
            },
        );

        // Every neighbor of the center (the 4 edge-adjacent pixels) should
        // now have picked up some brightness from the blur.
        let neighbors = [
            (0usize, 1usize),
            (1, 0),
            (1, 2),
            (2, 1),
        ];
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
