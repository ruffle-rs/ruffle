//! Iterator types for XML trees

use crate::xml::XMLNode;

/// Iterator that yields direct children of an XML node.
pub struct ChildIter<'gc> {
    base: XMLNode<'gc>,
    index: usize,
    back_index: usize,
}

impl<'gc> ChildIter<'gc> {
    /// Construct a new `ChildIter` that lists the children of an XML node.
    pub fn for_node(base: XMLNode<'gc>) -> Self {
        Self {
            base,
            index: 0,
            back_index: base.children_len(),
        }
    }

    /// Yield the base element whose children are being read out of.
    pub fn base(&self) -> XMLNode<'gc> {
        self.base
    }
}

impl<'gc> Iterator for ChildIter<'gc> {
    type Item = XMLNode<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.back_index {
            let item = self.base.get_child_by_index(self.index);
            self.index += 1;

            return item;
        }

        None
    }
}

impl<'gc> DoubleEndedIterator for ChildIter<'gc> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.back_index {
            self.back_index -= 1;
            let item = self.base.get_child_by_index(self.back_index);

            return item;
        }

        None
    }
}

/// Indicates the current action being taken by `WalkIter` as it walks
/// throughout the tree.
#[derive(Copy, Clone)]
pub enum Step<'gc> {
    /// `WalkIter` has discovered a new element and will begin to yield it's
    /// children's steps.
    In(XMLNode<'gc>),

    /// `WalkIter` has discovered a non-element node that cannot have children.
    ///
    /// Note that elements will never be stepped around, even if they are
    /// empty. They will be stepped in and out.
    Around(XMLNode<'gc>),

    /// `WalkIter` has exhausted the children of an element, stepping out of
    /// it.
    Out(XMLNode<'gc>),
}

impl<'gc> Step<'gc> {
    /// Discard the information regarding how we approached a given node, and
    /// just return the underlying `XMLNode`.
    pub fn unwrap(self) -> XMLNode<'gc> {
        match self {
            Self::In(node) | Self::Around(node) | Self::Out(node) => node,
        }
    }

    /// Yields true if this step entered an element.
    pub fn stepped_in(self) -> bool {
        match self {
            Self::In(_) => true,
            Self::Around(_) | Self::Out(_) => false,
        }
    }

    /// Yields true if this step encountered a non-element node.
    pub fn stepped_around(self) -> bool {
        match self {
            Self::Around(_) => true,
            Self::In(_) | Self::Out(_) => false,
        }
    }

    /// Yields true if this step exited an element.
    pub fn stepped_out(self) -> bool {
        match self {
            Self::Out(_) => true,
            Self::Around(_) | Self::In(_) => false,
        }
    }
}

/// Iterator that yields each step needed to visit all indirect descendents of
/// an XML node.
pub struct WalkIter<'gc> {
    stack: Vec<ChildIter<'gc>>,
}

impl<'gc> WalkIter<'gc> {
    /// Construct a new `WalkIter` that lists a tree out in `Step`s.
    pub fn for_node(base: XMLNode<'gc>) -> Self {
        Self {
            stack: vec![ChildIter::for_node(base)],
        }
    }
}

impl<'gc> Iterator for WalkIter<'gc> {
    type Item = Step<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        let last_stack_next = self.stack.last_mut().and_then(|i| i.next());

        if last_stack_next.is_none() && self.stack.len() > 1 {
            let last = self.stack.pop().unwrap();
            return Some(Step::Out(last.base()));
        }

        let next_node = last_stack_next?;
        if next_node.has_children() {
            self.stack.push(ChildIter::for_node(next_node));
            Some(Step::In(next_node))
        } else {
            Some(Step::Around(next_node))
        }
    }
}
