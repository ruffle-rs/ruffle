#[macro_use]
mod display_object;
pub use display_object::StageDisplayState;
pub use ruffle_types::string;

#[macro_use]
extern crate smallvec;

#[macro_use]
extern crate downcast_rs;

#[macro_use]
extern crate num_derive;

#[macro_use]
mod avm1;
mod avm2;
pub mod bitmap;
mod bounding_box;
mod character;
pub mod color_transform;
pub mod context;
pub mod context_menu;
mod drawing;
mod ecma_conversions;
pub(crate) mod either;
pub mod events;
pub mod focus_tracker;
mod font;
mod html;
mod library;
pub mod loader;
mod locale;
mod player;
mod prelude;
pub mod shape_utils;
pub mod timer;
mod transform;
pub mod vminterface;
mod xml;

pub mod backend;
pub mod config;
pub mod external;

pub use context_menu::ContextMenuItem;
pub use indexmap;
pub use player::{Player, PlayerBuilder};
pub use ruffle_types::events::PlayerEvent;
pub use swf;
pub use swf::Color;
