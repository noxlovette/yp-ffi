use image::{ImageBuffer, Rgba, imageops};
use plugin_interface::BlurParams;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;

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

    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, data.to_vec())
            .expect("width/height do not match buffer length");

    for _ in 0..params.iterations {
        img = imageops::fast_blur(&img, params.radius as f32);
    }

    data.copy_from_slice(img.as_raw());
}
