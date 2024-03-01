#![deny(clippy::unwrap_used)]

use slotmap::DefaultKey;

pub mod backend;
pub mod error;
pub mod frame;
pub mod null;

pub type VideoStreamHandle = DefaultKey;
