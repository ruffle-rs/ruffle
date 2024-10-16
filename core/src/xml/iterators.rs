//! Iterator types for XML trees

use crate::xml::XmlNode;

/// Iterator that yields direct children of an XML node.
pub struct ChildIter<'gc> {
    base: XmlNode<'gc>,
    index: usize,
    back_index: usize,
}

impl<'gc> ChildIter<'gc> {
    /// Construct a new `ChildIter` that lists the children of an XML node.
    pub fn for_node(base: XmlNode<'gc>) -> Self {
        Self {
            base,
            index: 0,
            back_index: base.children_len(),
        }
    }
}

impl<'gc> Iterator for ChildIter<'gc> {
    type Item = XmlNode<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.back_index {
            let item = self.base.get_child_by_index(self.index);
            self.index += 1;

            return item;
        }

        None
    }
}

impl DoubleEndedIterator for ChildIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.back_index {
            self.back_index -= 1;
            let item = self.base.get_child_by_index(self.back_index);

            return item;
        }

        None
    }
}

/// Iterator that yields the ancestors of an XML node.
pub struct AnscIter<'gc> {
    next: Option<XmlNode<'gc>>,
}

impl<'gc> AnscIter<'gc> {
    /// Construct a new `AnscIter` that lists the parents of an XML node (including itself).
    pub fn for_node(node: XmlNode<'gc>) -> Self {
        Self { next: Some(node) }
    }
}

impl<'gc> Iterator for AnscIter<'gc> {
    type Item = XmlNode<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.next;

        if let Some(parent) = parent {
            self.next = parent.parent();
        }

        parent
    }
}
