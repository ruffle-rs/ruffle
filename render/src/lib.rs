#![deny(clippy::unwrap_used)]

pub mod backend;
pub mod bitmap;
pub mod error;
pub mod filters;
pub mod matrix;
pub mod pixel_bender;
pub mod shader_source;
pub mod shape_utils;
pub mod transform;
pub mod utils;

pub mod commands;
pub mod quality;
#[cfg(feature = "tessellator")]
pub mod tessellator;
