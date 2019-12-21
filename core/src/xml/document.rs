//! XML Document

use crate::xml::Error;
use crate::xml::XMLNode;
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fmt;

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
    /// Construct a new, empty XML document.
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(mc, XMLDocumentData { roots: Vec::new() }))
    }

    /// Ensure that a newly-encountered node is added to an ongoing parsing
    /// stack, or to the document itself if the parsing stack is empty.
    fn add_child_to_tree(
        &mut self,
        mc: MutationContext<'gc, '_>,
        open_tags: &mut Vec<XMLNode<'gc>>,
        child: XMLNode<'gc>,
    ) -> Result<(), Error> {
        if let Some(node) = open_tags.last_mut() {
            node.append_child(mc, child)?;
        } else {
            self.0.write(mc).roots.push(child);
        }

        Ok(())
    }

    pub fn from_str(mc: MutationContext<'gc, '_>, data: &str) -> Result<Self, Error> {
        let mut parser = Reader::from_str(data);
        let mut buf = Vec::new();
        let mut document = Self::new(mc);
        let mut open_tags: Vec<XMLNode<'gc>> = Vec::new();

        loop {
            match parser.read_event(&mut buf)? {
                Event::Start(bs) => {
                    let child = XMLNode::from_start_event(mc, bs)?;
                    document.add_child_to_tree(mc, &mut open_tags, child)?;
                    open_tags.push(child);
                }
                Event::Empty(bs) => {
                    let child = XMLNode::from_start_event(mc, bs)?;
                    document.add_child_to_tree(mc, &mut open_tags, child)?;
                }
                Event::End(_) => {
                    open_tags.pop();
                }
                Event::Text(bt) => {
                    let child = XMLNode::text_from_text_event(mc, bt)?;
                    if child.node_value().as_deref() != Some("") {
                        document.add_child_to_tree(mc, &mut open_tags, child)?;
                    }
                }
                Event::Comment(bt) => {
                    let child = XMLNode::comment_from_text_event(mc, bt)?;
                    if child.node_value().as_deref() != Some("") {
                        document.add_child_to_tree(mc, &mut open_tags, child)?;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(document)
    }

    /// Returns an iterator that yields the document's root nodes.
    pub fn roots(&self) -> impl Iterator<Item = XMLNode<'gc>> {
        struct RootsIter<'gc> {
            base: XMLDocument<'gc>,
            index: usize,
        };

        impl<'gc> RootsIter<'gc> {
            fn for_document(base: XMLDocument<'gc>) -> Self {
                Self { base, index: 0 }
            }
        }

        impl<'gc> Iterator for RootsIter<'gc> {
            type Item = XMLNode<'gc>;

            fn next(&mut self) -> Option<Self::Item> {
                let (len, item) = {
                    let r = self.base.0.read();
                    (r.roots.len(), r.roots.get(self.index).cloned())
                };

                if self.index < len {
                    self.index += 1;
                }

                item
            }
        }

        RootsIter::for_document(*self)
    }
}

impl<'gc> fmt::Debug for XMLDocument<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("XMLDocument")
            .field("roots", &self.0.read().roots)
            .finish()
    }
}
