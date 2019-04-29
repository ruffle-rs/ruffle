#[macro_use]
mod display_object;

pub mod backend;
mod character;
mod color_transform;
mod graphic;
mod library;
mod matrix;
mod movie_clip;
mod player;
mod prelude;
mod transform;

pub use player::Player;
use swf::Color;
