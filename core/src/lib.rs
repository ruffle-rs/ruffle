#![allow(clippy::unneeded_field_pattern)]

#[macro_use]
mod display_object;

#[macro_use]
extern crate smallvec;

#[macro_use]
extern crate downcast_rs;

mod avm1;
mod bounding_box;
mod character;
pub mod color_transform;
mod context;
pub mod events;
mod font;
mod library;
mod loader;
pub mod matrix;
mod player;
mod prelude;
mod property_map;
pub mod shape_utils;
pub mod string_utils;
pub mod tag_utils;
mod transform;
mod xml;

pub mod backend;

pub use events::PlayerEvent;
pub use player::Player;
pub use swf;
pub use swf::Color;
