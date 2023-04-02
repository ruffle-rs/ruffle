//! Garbage-collectable XML DOM impl

mod iterators;
mod tree;

pub use tree::{custom_unescape, XmlNode, ELEMENT_NODE, TEXT_NODE};
