#[macro_use]
mod display_object;
pub use display_object::StageDisplayState;

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
mod character;
pub mod context;
pub mod context_menu;
mod drawing;
mod ecma_conversions;
pub(crate) mod either;
pub mod events;
pub mod focus_tracker;
mod font;
mod frame_lifecycle;
mod html;
mod library;
pub mod loader;
mod locale;
mod player;
mod prelude;
pub mod string;
pub mod tag_utils;
pub mod timer;
mod types;
mod vminterface;
mod xml;

pub mod backend;
pub mod config;
pub mod external;

pub use context_menu::ContextMenuItem;
pub use events::PlayerEvent;
pub use indexmap;
pub use player::{Player, PlayerBuilder, StaticCallstack};
pub use ruffle_render::backend::ViewportDimensions;
pub use swf;
pub use swf::Color;
