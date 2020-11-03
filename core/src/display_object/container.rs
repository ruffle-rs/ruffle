//! Container mix-in for display objects

use crate::context::UpdateContext;
use crate::display_object::{Depth, DisplayObject, TDisplayObject};
use crate::string_utils::swf_string_eq_ignore_case;
use gc_arena::{Collect, MutationContext};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// A structure that stores child display objects.
///
/// Child display objects are stored in two lists: a render list and a depth
/// list. The latter references display objects by their chosen depth; while
/// the render list represents the order in which those children should be
/// rendered. Not all children have a position on this depth.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct ChildContainer<'gc> {
    /// The list of all children in render order.
    ///
    /// This list is the actual list used to render children. All other list
    /// manipulations intended to change the order children are rendered in
    /// must adjust this list as well. All children must be present on the
    /// render list.
    ///
    /// In AVM1, the depth and render lists are identical; AS1/2 code interacts
    /// exclusively with the depth list. However, AS3 instead references clips
    /// by render list indexes and does not manipulate the depth list.
    render_list: Vec<DisplayObject<'gc>>,

    /// The mapping from timeline Depths to child display objects.
    ///
    /// This list is the list used to map depths to actual display objects.
    /// It does not affect render order. Unlike the render list, children may
    /// or may not live on the depth list.
    ///
    /// In AVM1, the depth and render lists are identical; AS1/2 code interacts
    /// exclusively with the depth list. However, AS3 instead references clips
    /// by render list indexes and does not manipulate the depth list.
    depth_list: BTreeMap<Depth, DisplayObject<'gc>>,

    /// The execution-order list for display objects' AVM1 scripts.
    ///
    /// This list is an intrusive linked list baked into all display objects.
    /// Thus, this merely references the first item in the list.
    exec_list: Option<DisplayObject<'gc>>,
}

impl<'gc> ChildContainer<'gc> {
    pub fn new() -> Self {
        ChildContainer {
            render_list: Vec::new(),
            depth_list: BTreeMap::new(),
            exec_list: None,
        }
    }

    /// Get the head of the execution list.
    pub fn first_child(&self) -> Option<DisplayObject<'gc>> {
        self.exec_list
    }

    /// Change the head of the execution list.
    pub fn set_first_child(
        &mut self,
        _context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    ) {
        self.exec_list = node;
    }

    /// Adds a child to the front of the execution list.
    ///
    /// This does not affect the render or depth lists.
    fn add_child_to_exec_list(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        child: DisplayObject<'gc>,
    ) {
        if let Some(head) = self.exec_list {
            head.set_prev_sibling(gc_context, Some(child));
            child.set_next_sibling(gc_context, Some(head));
        }

        self.exec_list = Some(child);
    }

    /// Removes a child from the execution list.
    ///
    /// This does not affect the render or depth lists.
    fn remove_child_from_exec_list(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        child: DisplayObject<'gc>,
    ) {
        // Remove from children linked list.
        let prev = child.prev_sibling();
        let next = child.next_sibling();

        if let Some(prev) = prev {
            prev.set_next_sibling(context.gc_context, next);
        }
        if let Some(next) = next {
            next.set_prev_sibling(context.gc_context, prev);
        }

        child.set_prev_sibling(context.gc_context, None);
        child.set_next_sibling(context.gc_context, None);

        if let Some(head) = self.exec_list {
            if DisplayObject::ptr_eq(head, child) {
                self.exec_list = next;
            }
        }
        // Flag child as removed.
        child.unload(context);
    }

    /// Remove a child from the display list.
    ///
    /// This returns `true` if the child was successfully removed, and `false`
    /// if no list alterations were made.
    fn remove_child_from_depth_list(&mut self, child: DisplayObject<'gc>) -> bool {
        if let Some(other_child) = self.depth_list.get(&child.depth()) {
            DisplayObject::ptr_eq(*other_child, child)
                && self.depth_list.remove(&child.depth()).is_some()
        } else {
            false
        }
    }

    /// Returns the highest depth in use by this container, or `None` if there
    /// are no children.
    pub fn highest_depth(&self) -> Option<Depth> {
        self.depth_list.keys().copied().rev().next()
    }

    /// Determine if the render list is empty.
    pub fn is_empty(&self) -> bool {
        self.render_list.is_empty()
    }

    /// Get a child at a given depth.
    pub fn get_depth(&self, depth: Depth) -> Option<DisplayObject<'gc>> {
        self.depth_list.get(&depth).copied()
    }

    /// Get a child by it's timeline name.
    ///
    /// The `case_sensitive` parameter determines if we should consider
    /// children with different capitalizations as being distinct names.
    ///
    /// If multiple children with the same name exist, the one with the lowest
    /// depth wins. Children not on the depth list will not be accessible via
    /// this mechanism.
    pub fn get_name(&self, name: &str, case_sensitive: bool) -> Option<DisplayObject<'gc>> {
        // TODO: Make a HashMap from name -> child?
        // But need to handle conflicting names (lowest in depth order takes priority).
        if case_sensitive {
            self.depth_list
                .values()
                .copied()
                .find(|child| &*child.name() == name)
        } else {
            self.depth_list
                .values()
                .copied()
                .find(|child| swf_string_eq_ignore_case(&*child.name(), name))
        }
    }

    /// Get a child by it's render list position (ID).
    pub fn get_id(&self, id: usize) -> Option<DisplayObject<'gc>> {
        self.render_list.get(id).copied()
    }

    /// Get the number of children on the render list.
    pub fn num_children(&self) -> usize {
        self.render_list.len()
    }

    /// Insert a child at a given render list position.
    ///
    /// If the child is already a child of another container, we will remove it
    /// from that container. This also applies to our own render list. Note
    /// that all children after the old position will be shifted back by one,
    /// which must be taken into account when calculating future insertion IDs.
    ///
    /// All children at or after the given ID will be shifted down in the
    /// render list. The child will *not* be put onto the depth list.
    pub fn insert_at_id(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        child: DisplayObject<'gc>,
        id: usize,
    ) {
        child.set_place_frame(context.gc_context, 0);

        if let Some(old_id) = self
            .render_list
            .iter()
            .position(|x| DisplayObject::ptr_eq(*x, child))
        {
            match old_id.cmp(&id) {
                Ordering::Less if id < self.render_list.len() => {
                    self.render_list[old_id..=id].rotate_left(1)
                }
                Ordering::Less => self.render_list[old_id..id].rotate_left(1),
                Ordering::Greater if old_id < self.render_list.len() => {
                    self.render_list[id..=old_id].rotate_right(1)
                }
                Ordering::Greater => self.render_list[id..old_id].rotate_right(1),
                Ordering::Equal => {}
            }
        } else {
            if let Some(old_parent) = child.parent() {
                //TODO: container access for non-clips
                if let Some(mut old_parent_mc) = old_parent.as_movie_clip() {
                    old_parent_mc.remove_child_from_avm(context, child);
                }
            }

            self.render_list.insert(id, child);
            self.add_child_to_exec_list(context.gc_context, child);
        }
    }

    /// Insert a child into the container at a given depth, replacing any child
    /// that already exists at the given depth.
    ///
    /// If a child was replaced, this function returns the child that was
    /// removed from the container. It should be removed from any other lists
    /// maintained by the owner of this container (e.g. the execution list).
    ///
    /// Children that have been placed by scripts will not be removed from the
    /// render list, only the depth list. This function will also not return
    /// such children.
    ///
    /// The `already_on_exec_list` flag indicates that the caller has already
    /// added the child to the execution list, and thus this function should
    /// not modify the execution list.
    pub fn replace_at_depth(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        child: DisplayObject<'gc>,
        depth: Depth,
        already_on_exec_list: bool,
    ) -> Option<DisplayObject<'gc>> {
        child.set_place_frame(context.gc_context, 0);
        child.set_depth(context.gc_context, depth);

        let prev_child = self.depth_list.insert(depth, child);
        let removed_child = if let Some(prev_child) = prev_child {
            let position = self
                .render_list
                .iter()
                .position(|x| DisplayObject::ptr_eq(*x, prev_child))
                .unwrap();

            if !prev_child.placed_by_script() {
                self.render_list[position] = child;

                Some(prev_child)
            } else {
                self.render_list.insert(position + 1, child);

                None
            }
        } else if let Some((_, below_child)) = self.depth_list.range(..depth).rev().next() {
            let position = self
                .render_list
                .iter()
                .position(|x| DisplayObject::ptr_eq(*x, *below_child))
                .unwrap();
            self.render_list.insert(position + 1, child);

            None
        } else {
            self.render_list.push(child);

            None
        };

        if !already_on_exec_list {
            if let Some(removed_child) = removed_child {
                self.remove_child_from_exec_list(context, removed_child);
            }

            self.add_child_to_exec_list(context.gc_context, child);
        }

        removed_child
    }

    /// Move an already-inserted child to a new location on the depth list.
    ///
    /// If another child already exists at the target depth, it will be moved
    /// to the current depth of the given child. Their relative positions in
    /// the render list will also be swapped. If the target depth is empty, the
    /// same steps occur, but the child will merely be removed and reinserted
    /// within the render list at a position after the closest previous child
    /// in the depth list.
    pub fn swap_at_depth(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        child: DisplayObject<'gc>,
        depth: Depth,
    ) {
        let prev_depth = child.depth();
        child.set_depth(gc_context, depth);

        if let Some(prev_child) = self.depth_list.insert(depth, child) {
            prev_child.set_depth(gc_context, prev_depth);
            prev_child.set_transformed_by_script(gc_context, true);
            self.depth_list.insert(prev_depth, prev_child);

            let prev_position = self
                .render_list
                .iter()
                .position(|x| DisplayObject::ptr_eq(*x, prev_child))
                .unwrap();
            let next_position = self
                .render_list
                .iter()
                .position(|x| DisplayObject::ptr_eq(*x, child))
                .unwrap();
            self.render_list.swap(prev_position, next_position);
        } else {
            self.depth_list.remove(&prev_depth);

            if let Some((_, below_child)) = self.depth_list.range(..depth).rev().next() {
                let old_position = self
                    .render_list
                    .iter()
                    .position(|x| DisplayObject::ptr_eq(*x, child))
                    .unwrap();
                self.render_list.remove(old_position);

                let new_position = self
                    .render_list
                    .iter()
                    .position(|x| DisplayObject::ptr_eq(*x, *below_child))
                    .unwrap();
                self.render_list.insert(new_position + 1, child);
            } else {
                self.render_list.insert(0, child);
            }
        }
    }

    /// Remove a child from the container.
    ///
    /// This function returns true if the child was present on either the
    /// render list or the depth list.
    pub fn remove_child(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        child: DisplayObject<'gc>,
    ) -> bool {
        let removed_from_depth_list = self.remove_child_from_depth_list(child);

        let render_list_position = self
            .render_list
            .iter()
            .position(|x| DisplayObject::ptr_eq(*x, child));
        let removed_from_render_list = render_list_position.is_some();
        if let Some(position) = render_list_position {
            self.render_list.remove(position);
        }

        self.remove_child_from_exec_list(context, child);

        removed_from_render_list || removed_from_depth_list
    }

    /// Remove all children from the container.
    pub fn clear(&mut self) {
        self.render_list.clear();
        self.depth_list.clear();
    }

    /// Yield children in the order that they are to be rendered,
    /// back-to-front.
    pub fn iter_render_children<'a>(
        &'a self,
    ) -> impl 'a + DoubleEndedIterator<Item = DisplayObject<'gc>> {
        self.render_list.iter().copied()
    }

    /// Yield children in the order expected of them by the timeline, alongside
    /// their corresponding depths.
    pub fn iter_children_by_depth<'a>(
        &'a self,
    ) -> impl 'a + Iterator<Item = (Depth, DisplayObject<'gc>)> {
        self.depth_list.iter().map(|(k, v)| (*k, *v))
    }
}
