#![deny(clippy::unwrap_used)]

pub mod atf;
pub mod backend;
pub mod bitmap;
pub mod blend;
pub mod error;
pub mod filters;
pub mod lines;
pub mod matrix;
pub mod pixel_bender;
// The `renderdoc` crate doesn't compile on apple platforms
#[cfg(all(feature = "renderdoc", not(target_vendor = "apple")))]
pub mod renderdoc;
pub mod shader_source;
pub mod shape_utils;
pub mod transform;
pub mod utils;

pub mod commands;
pub mod quality;
#[cfg(feature = "tessellator")]
pub mod tessellator;
