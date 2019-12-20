//! Garbage-collectable XML DOM impl

mod document;
mod tree;

type Error = Box<dyn std::error::Error>;

pub use document::XMLDocument;
pub use tree::XMLNode;
