#![allow(
    renamed_and_removed_lints,
    unknown_lints,
    clippy::unknown_clippy_lints,
    clippy::inconsistent_struct_constructor,
    clippy::manual_map,
    clippy::manual_range_contains,
    clippy::same_item_push,
    clippy::unnecessary_wraps
)]

#[macro_use]
mod display_object;

#[macro_use]
extern crate smallvec;

#[macro_use]
extern crate downcast_rs;

#[macro_use]
extern crate num_derive;

#[macro_use]
mod avm1;
mod avm2;
mod binary_data;
pub mod bitmap;
mod bounding_box;
mod character;
pub mod color_transform;
pub mod context;
pub mod context_menu;
mod drawing;
mod ecma_conversions;
pub mod events;
pub mod focus_tracker;
mod font;
mod html;
mod library;
pub mod loader;
pub mod matrix;
mod player;
mod prelude;
pub mod shape_utils;
pub mod string_utils;
pub mod tag_utils;
mod transform;
mod types;
mod vminterface;
mod xml;

pub mod backend;
pub mod config;
pub mod external;

pub use chrono;
pub use context_menu::ContextMenuItem;
pub use events::PlayerEvent;
pub use indexmap;
pub use player::Player;
pub use swf;
pub use swf::Color;
