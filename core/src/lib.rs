#[macro_use]
mod display_object;

mod audio;
mod avm1;
mod button;
mod character;
mod color_transform;
mod font;
mod graphic;
mod library;
pub mod matrix;
mod movie_clip;
mod player;
mod prelude;
mod text;
mod transform;

pub mod backend;

pub use player::Player;
pub use swf::Color;

#[macro_use]
extern crate gc_derive;
