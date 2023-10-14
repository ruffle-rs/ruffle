//! Iterator types for E4XNodes

use crate::avm2::e4x::E4XNode;

/// Iterator that yields the ancestors of an E4XNode.
pub struct AnscIter<'gc> {
    next: Option<E4XNode<'gc>>,
}

impl<'gc> AnscIter<'gc> {
    /// Construct a new `AnscIter` that lists the parents of an E4X node (including itself).
    pub fn for_node(node: E4XNode<'gc>) -> Self {
        Self { next: Some(node) }
    }
}

impl<'gc> Iterator for AnscIter<'gc> {
    type Item = E4XNode<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.next;

        if let Some(parent) = parent {
            self.next = parent.parent();
        }

        parent
    }
}
