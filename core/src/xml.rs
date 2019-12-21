//! Garbage-collectable XML DOM impl

mod document;
mod tree;

#[cfg(test)]
mod tests;

type Error = Box<dyn std::error::Error>;

pub use document::XMLDocument;
pub use tree::XMLName;
pub use tree::XMLNode;

pub const ELEMENT_NODE: u8 = 1;
pub const TEXT_NODE: u8 = 3;
pub const COMMENT_NODE: u8 = 8;
