#[macro_use]
mod display_object;

mod avm1;
mod button;
mod character;
mod color_transform;
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

pub use player::Player;
pub use swf::Color;
pub use swf;