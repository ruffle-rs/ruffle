#![allow(clippy::unneeded_field_pattern)]
#![allow(clippy::trivially_copy_pass_by_ref)]

#[macro_use]
mod display_object;

#[macro_use]
extern crate smallvec;

mod avm1;
mod bounding_box;
mod character;
mod color_transform;
mod context;
mod events;
mod font;
mod library;
pub mod matrix;
mod player;
mod prelude;
pub mod shape_utils;
pub mod tag_utils;
mod transform;

pub mod backend;

pub use events::PlayerEvent;
pub use player::Player;
pub use swf;
pub use swf::Color;
