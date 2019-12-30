//! Garbage-collectable XML DOM impl

mod document;
mod namespace;
mod tree;

#[cfg(test)]
mod tests;

type Error = Box<dyn std::error::Error>;

pub use document::XMLDocument;
pub use namespace::XMLName;
pub use tree::XMLNode;

pub const ELEMENT_NODE: u8 = 1;
pub const TEXT_NODE: u8 = 3;
pub const COMMENT_NODE: u8 = 8;
pub const DOCUMENT_NODE: u8 = 9;
