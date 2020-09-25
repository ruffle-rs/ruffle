#![allow(
    clippy::unneeded_field_pattern,
    clippy::same_item_push,
    clippy::unknown_clippy_lints
)]

#[macro_use]
mod display_object;

#[macro_use]
extern crate smallvec;

#[macro_use]
extern crate downcast_rs;

#[macro_use]
mod avm1;
mod avm2;
mod bounding_box;
mod character;
mod collect;
pub mod color_transform;
pub mod context;
mod drawing;
mod ecma_conversions;
pub mod events;
mod font;
mod html;
mod library;
pub mod loader;
mod percentage;
mod player;
mod prelude;
mod property_map;
pub mod shape_utils;
pub mod string_utils;
pub mod tag_utils;
mod transform;
mod xml;

pub mod backend;
pub mod external;

pub use chrono;
pub use events::PlayerEvent;
pub use indexmap;
pub use player::Player;
pub use swf;
pub use swf::Color;
