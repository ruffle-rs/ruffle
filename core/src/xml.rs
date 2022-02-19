//! Garbage-collectable XML DOM impl

mod iterators;
mod tree;

pub use tree::XmlNode;

pub const ELEMENT_NODE: u8 = 1;
pub const TEXT_NODE: u8 = 3;
pub const COMMENT_NODE: u8 = 8;
pub const DOCUMENT_NODE: u8 = 9;
pub const DOCUMENT_TYPE_NODE: u8 = 10;
