#[macro_use]
mod display_object;

mod audio;
mod avm1;
mod character;
mod color_transform;
mod graphic;
mod library;
mod matrix;
mod movie_clip;
mod player;
mod prelude;
mod transform;

pub mod backend;

pub use player::Player;
use swf::Color;
