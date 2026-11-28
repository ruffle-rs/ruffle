// cargo install bindgen-cli
// bindgen ../openh264/codec/api/wels/codec_api.h --no-prepend-enum-name \
//         --no-layout-tests --dynamic-loading OpenH264 -o openh264_sys.rs
#[cfg(feature = "openh264")]
#[allow(dead_code)]
#[allow(nonstandard_style)]
#[allow(unsafe_op_in_unsafe_fn)]
#[allow(unused_qualifications)]
mod openh264_sys;

#[cfg(feature = "openh264")]
pub mod openh264;

#[cfg(feature = "webcodecs")]
pub mod webcodecs;

pub use ruffle_video_software::decoder::VideoDecoder;
