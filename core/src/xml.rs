//! Garbage-collectable XML DOM impl

mod document;
mod tree;

type Error = Box<dyn std::error::Error>;

pub use tree::XMLNode;
