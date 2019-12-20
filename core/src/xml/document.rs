//! XML Document

use crate::xml::Error;
use crate::xml::XMLNode;
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::Event;
use quick_xml::Reader;

/// The entirety of an XML document.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct XMLDocument<'gc>(GcCell<'gc, XMLDocumentData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XMLDocumentData<'gc> {
    /// The root node(s) of the XML document.
    roots: Vec<XMLNode<'gc>>,
}

impl<'gc> XMLDocument<'gc> {
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(mc, XMLDocumentData { roots: Vec::new() }))
    }

    pub fn from_str(mc: MutationContext<'gc, '_>, data: &str) -> Result<Self, Error> {
        let mut parser = Reader::from_str(data);
        let mut buf = Vec::new();
        let mut roots = Vec::new();
        let mut open_tags: Vec<XMLNode<'gc>> = Vec::new();

        loop {
            match parser.read_event(&mut buf)? {
                Event::Start(bs) => {
                    let child = XMLNode::from_start_event(mc, bs)?;
                    if let Some(node) = open_tags.last_mut() {
                        node.append_child(mc, child)?;
                    } else {
                        roots.push(child);
                    }

                    open_tags.push(child);
                }
                Event::Empty(bs) => {
                    let child = XMLNode::from_start_event(mc, bs)?;
                    if let Some(node) = open_tags.last_mut() {
                        node.append_child(mc, child)?;
                    } else {
                        roots.push(child);
                    }
                }
                Event::End(_) => {
                    open_tags.pop();
                }
                Event::Text(bt) => {
                    let child = XMLNode::text_from_text_event(mc, bt)?;
                    if let Some(node) = open_tags.last_mut() {
                        node.append_child(mc, child)?;
                    } else {
                        roots.push(child);
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(Self(GcCell::allocate(mc, XMLDocumentData { roots })))
    }
}
