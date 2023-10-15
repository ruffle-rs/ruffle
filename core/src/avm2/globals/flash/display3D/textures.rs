//! `flash.display3D.textures` namespace

#[cfg(feature = "jpegxr")]
mod atf_jpegxr;
#[cfg(not(feature = "jpegxr"))]
#[path = "textures/atf_jpegxr_stub.rs"]
mod atf_jpegxr;

pub mod cube_texture;
pub mod rectangle_texture;
pub mod texture;
