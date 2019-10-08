#![allow(clippy::unneeded_field_pattern)]

#[macro_use]
mod display_object;

mod avm1;
mod bounding_box;
mod button;
mod character;
mod color_transform;
mod edit_text;
mod events;
mod font;
mod graphic;
mod library;
pub mod matrix;
mod morph_shape;
mod movie_clip;
mod player;
mod prelude;
pub mod shape_utils;
pub mod tag_utils;
mod text;
mod transform;

pub mod backend;

pub use events::PlayerEvent;
pub use player::Player;
pub use swf;
pub use swf::Color;
