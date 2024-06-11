//! Container mix-in for display objects

use crate::avm1::{Activation, ActivationIdentifier, TObject};
use crate::avm2::{
    Activation as Avm2Activation, Avm2, EventObject as Avm2EventObject, Multiname as Avm2Multiname,
    TObject as _, Value as Avm2Value,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::avm1_button::Avm1Button;
use crate::display_object::loader_display::LoaderDisplay;
use crate::display_object::movie_clip::MovieClip;
use crate::display_object::stage::Stage;
use crate::display_object::{
    Depth, DisplayObject, InteractiveObject, TDisplayObject, TInteractiveObject,
};
use crate::string::WStr;
use crate::tag_utils::SwfMovie;
use gc_arena::{Collect, Mutation};
use ruffle_macros::enum_trait_object;
use ruffle_render::commands::CommandHandler;
use std::cell::{Ref, RefMut};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};
use std::rc::Rc;
use std::sync::Arc;

/// Dispatch the `removedFromStage` event on a child and all of it's
/// grandchildren, recursively.
pub fn dispatch_removed_from_stage_event<'gc>(
    child: DisplayObject<'gc>,
    context: &mut UpdateContext<'_, 'gc>,
) {
    if let Avm2Value::Object(object) = child.object2() {
        let removed_evt = Avm2EventObject::bare_default_event(context, "removedFromStage");
        Avm2::dispatch_event(context, removed_evt, object);
    }

    if let Some(child_container) = child.as_container() {
        for grandchild in child_container.iter_render_list() {
            dispatch_removed_from_stage_event(grandchild, context)
        }
    }
}

/// Dispatch the `removed` event on a child and log any errors encountered
/// whilst doing so.
pub fn dispatch_removed_event<'gc>(
    child: DisplayObject<'gc>,
    context: &mut UpdateContext<'_, 'gc>,
) {
    if let Avm2Value::Object(object) = child.object2() {
        let removed_evt = Avm2EventObject::bare_event(context, "removed", true, false);
        Avm2::dispatch_event(context, removed_evt, object);

        if child.is_on_stage(context) {
            dispatch_removed_from_stage_event(child, context)
        }
    }
}

/// Dispatch the `addedToStage` event on a child, ignoring it's grandchildren.
pub fn dispatch_added_to_stage_event_only<'gc>(
    child: DisplayObject<'gc>,
    context: &mut UpdateContext<'_, 'gc>,
) {
    if let Avm2Value::Object(object) = child.object2() {
        let added_evt = Avm2EventObject::bare_default_event(context, "addedToStage");
        Avm2::dispatch_event(context, added_evt, object);
    }
}

/// Dispatch the `addedToStage` event on a child and all of it's grandchildren,
/// recursively.
pub fn dispatch_added_to_stage_event<'gc>(
    child: DisplayObject<'gc>,
    context: &mut UpdateContext<'_, 'gc>,
) {
    dispatch_added_to_stage_event_only(child, context);

    if let Some(child_container) = child.as_container() {
        for grandchild in child_container.iter_render_list() {
            dispatch_added_to_stage_event(grandchild, context)
        }
    }
    if let Some(button) = child.as_avm2_button() {
        if let Some(child) = button.get_state_child(button.state().into()) {
            dispatch_added_to_stage_event(child, context);
        }
    }
}

/// Dispatch an `added` event to one object, and log any errors encountered
/// whilst doing so.
pub fn dispatch_added_event_only<'gc>(
    child: DisplayObject<'gc>,
    context: &mut UpdateContext<'_, 'gc>,
) {
    if let Avm2Value::Object(object) = child.object2() {
        let added_evt = Avm2EventObject::bare_event(context, "added", true, false);
        Avm2::dispatch_event(context, added_evt, object);
    }
}

/// Dispatch the `added` event on a child and log any errors encountered
/// whilst doing so.
///
/// `child_was_on_stage` should be the result of calling `child.is_on_stage()`
/// before any container manipulation has been made. The `added` event is
/// generally fired after the container manipulation has been made.
pub fn dispatch_added_event<'gc>(
    parent: DisplayObject<'gc>,
    child: DisplayObject<'gc>,
    child_was_on_stage: bool,
    context: &mut UpdateContext<'_, 'gc>,
) {
    dispatch_added_event_only(child, context);

    if parent.is_on_stage(context) && !child_was_on_stage {
        dispatch_added_to_stage_event(child, context);
    }
}

#[enum_trait_object(
    #[derive(Clone, Collect, Copy, Debug)]
    #[collect(no_drop)]
    pub enum DisplayObjectContainer<'gc> {
        Stage(Stage<'gc>),
        Avm1Button(Avm1Button<'gc>),
        MovieClip(MovieClip<'gc>),
        LoaderDisplay(LoaderDisplay<'gc>),
    }
)]
pub trait TDisplayObjectContainer<'gc>:
    'gc + Clone + Copy + Collect + Debug + Into<DisplayObjectContainer<'gc>> + Into<DisplayObject<'gc>>
{
    /// Get read-only access to the raw container.
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>>;

    /// Get mutable access to the raw container.
    fn raw_container_mut(&self, gc_context: &Mutation<'gc>) -> RefMut<'_, ChildContainer<'gc>>;

    /// Get a child display object by its position in the render list.
    ///
    /// The `index` provided here should not be confused with the `Depth`s used
    /// to index the depth list.
    fn child_by_index(self, index: usize) -> Option<DisplayObject<'gc>> {
        self.raw_container().get_id(index)
    }

    /// Get a child display object by its position in the depth list.
    ///
    /// The `Depth` provided here should not be confused with the `index`s used
    /// to index the render list.
    fn child_by_depth(self, depth: Depth) -> Option<DisplayObject<'gc>> {
        self.raw_container().get_depth(depth)
    }

    /// Get a child display object by its instance/timeline name.
    ///
    /// The `case_sensitive` parameter determines if we should consider
    /// children with different capitalizations as being distinct names.
    ///
    /// If multiple children with the same name exist, the one with the lowest
    /// depth wins. Children not on the depth list will not be accessible via
    /// this mechanism.
    fn child_by_name(self, name: &WStr, case_sensitive: bool) -> Option<DisplayObject<'gc>> {
        self.raw_container().get_name(name, case_sensitive)
    }

    /// Returns the number of children on the render list.
    fn num_children(self) -> usize {
        self.raw_container().num_children()
    }

    /// Returns the highest depth among children.
    fn highest_depth(self) -> Depth {
        self.raw_container().highest_depth()
    }

    /// Insert a child display object into the container at a specific position
    /// in the depth list, removing any child already at that position.
    ///
    /// After inserting the child into the depth list, we will attempt to
    /// assign it a render list position one after the previous item in the
    /// depth list. The position that children are placed into the render list
    /// matches Flash Player behavior.
    ///
    /// Any child removed from the depth list will also be removed from the
    /// render list if and only if the child was not flagged as
    /// being placed by script. If such a child was removed from said lists, it
    /// will be returned here. Otherwise, this method returns `None`.
    ///
    /// Note: this method specifically does *not* dispatch events on any
    /// children it modifies. You must do this yourself.
    fn replace_at_depth(
        self,
        context: &mut UpdateContext<'_, 'gc>,
        child: DisplayObject<'gc>,
        depth: Depth,
    ) -> Option<DisplayObject<'gc>> {
        let removed_child = self
            .raw_container_mut(context.gc_context)
            .replace_at_depth(child, depth);

        child.set_parent(context, Some(self.into()));
        child.set_place_frame(context.gc_context, 0);
        child.set_depth(context.gc_context, depth);

        if let Some(removed_child) = removed_child {
            if !self.raw_container().movie().is_action_script_3() {
                removed_child.avm1_unload(context);
            }
            removed_child.set_parent(context, None);
        }

        let this: DisplayObject<'_> = self.into();
        this.invalidate_cached_bitmap(context.gc_context);

        removed_child
    }

    /// Move a child display object around in the container's depth list.
    ///
    /// Any child already at the desired position will move back to the new
    /// child's former position. The render list positions of each child will
    /// also be swapped. If no child has been displaced by the swap operation,
    /// then the render list position of the child will be determined in the same
    /// way as `replace_at_depth`.
    fn swap_at_depth(
        &mut self,
        context: &mut UpdateContext<'_, 'gc>,
        child: DisplayObject<'gc>,
        depth: Depth,
    ) {
        let this: DisplayObject<'_> = (*self).into();
        // Verify this is actually our child.
        // TODO: This seems unnecessary (especially since AS3 MovieClips
        // are allowed to be used in ways that would trip this assert).
        debug_assert!(DisplayObject::ptr_eq(child.parent().unwrap(), this));

        self.raw_container_mut(context.gc_context)
            .swap_at_depth(context, this, child, depth);

        this.invalidate_cached_bitmap(context.gc_context);
    }

    /// Insert a child display object into the container at a specific position
    /// in the render list.
    ///
    /// Callers of this method should be aware that reordering items onto or off of the
    /// render list can make further depth list manipulations (e.g. from the
    /// timeline) produce unusual results.
    fn insert_at_index(
        &mut self,
        context: &mut UpdateContext<'_, 'gc>,
        child: DisplayObject<'gc>,
        index: usize,
    ) {
        let this: DisplayObject<'_> = (*self).into();
        let parent_changed = if let Some(old_parent) = child.parent() {
            if !DisplayObject::ptr_eq(old_parent, this) {
                if let Some(mut old_parent) = old_parent.as_container() {
                    old_parent.remove_child(context, child);
                }

                true
            } else {
                false
            }
        } else {
            true
        };

        let child_was_on_stage = child.is_on_stage(context);

        child.set_place_frame(context.gc_context, 0);
        child.set_parent(context, Some(this));
        if !self.raw_container().movie().is_action_script_3() {
            child.set_avm1_removed(context.gc_context, false);
        }

        self.raw_container_mut(context.gc_context)
            .insert_at_id(child, index);

        if parent_changed {
            dispatch_added_event(this, child, child_was_on_stage, context);
        }

        this.invalidate_cached_bitmap(context.gc_context);
    }

    /// Swap two children in the render list.
    ///
    /// No changes to the depth or render lists are made by this function.
    fn swap_at_index(
        &mut self,
        context: &mut UpdateContext<'_, 'gc>,
        index1: usize,
        index2: usize,
    ) {
        self.raw_container_mut(context.gc_context)
            .swap_at_id(index1, index2);
        let this: DisplayObject<'_> = (*self).into();
        this.invalidate_cached_bitmap(context.gc_context);
    }

    /// Remove (and unloads) a child display object from this container's render and depth lists.
    ///
    /// Will also handle AVM1 delayed clip removal, when a unload listener is present
    fn remove_child(&mut self, context: &mut UpdateContext<'_, 'gc>, child: DisplayObject<'gc>) {
        let this: DisplayObject<'_> = (*self).into();

        // We should always be the parent of this child
        debug_assert!(DisplayObject::ptr_eq(
            child.parent().unwrap(),
            (*self).into()
        ));

        // Check if this child should have delayed removal (AVM1 only)
        if !self.raw_container().movie().is_action_script_3() {
            let should_delay_removal = {
                let mut activation = Activation::from_nothing(
                    context.reborrow(),
                    ActivationIdentifier::root("[Unload Handler Check]"),
                    this.avm1_root(),
                );

                ChildContainer::should_delay_removal(&mut activation, child)
            };

            if should_delay_removal {
                let mut raw_container = self.raw_container_mut(context.gc_context);

                // Remove the child from the depth list, before moving it to a negative depth
                raw_container.remove_child_from_depth_list(child);

                // Enqueue for removal
                ChildContainer::queue_removal(child, context);

                // Mark that we have a pending removal
                raw_container.set_pending_removals(true);

                // Re-Insert the child at the new depth
                raw_container.insert_child_into_depth_list(child.depth(), child);

                drop(raw_container);
                this.invalidate_cached_bitmap(context.gc_context);

                return;
            }
        }

        self.remove_child_directly(context, child);
    }

    /// Remove (and unloads) a child display object from this container's render and depth lists.
    fn remove_child_directly(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        child: DisplayObject<'gc>,
    ) {
        dispatch_removed_event(child, context);
        let this: DisplayObjectContainer<'gc> = (*self).into();
        let mut write = self.raw_container_mut(context.gc_context);
        write.remove_child_from_depth_list(child);
        drop(write);

        let removed_from_render_list =
            ChildContainer::remove_child_from_render_list(this, child, context);

        if removed_from_render_list {
            if !self.raw_container().movie.is_action_script_3() {
                child.avm1_unload(context);
            } else if !matches!(child.object2(), Avm2Value::Null) {
                //TODO: This is an awful, *awful* hack to deal with the fact
                //that unloaded AVM1 clips see their parents, while AVM2 clips
                //don't.
                child.set_parent(context, None);
            }

            let this: DisplayObject<'_> = (*self).into();
            this.invalidate_cached_bitmap(context.gc_context);
        }
    }

    /// Insert a child directly into this container's depth list.
    fn insert_child_into_depth_list(
        &mut self,
        context: &mut UpdateContext<'_, 'gc>,
        depth: Depth,
        child: DisplayObject<'gc>,
    ) {
        let this: DisplayObject<'_> = (*self).into();

        child.set_depth(context.gc_context, depth);
        child.set_parent(context, Some(this));
        self.raw_container_mut(context.gc_context)
            .insert_child_into_depth_list(depth, child);

        this.invalidate_cached_bitmap(context.gc_context);
    }

    /// Removes (without unloading) a child display object from this container's depth list.
    fn remove_child_from_depth_list(
        &mut self,
        context: &mut UpdateContext<'_, 'gc>,
        child: DisplayObject<'gc>,
    ) {
        debug_assert!(DisplayObject::ptr_eq(
            child.parent().unwrap(),
            (*self).into()
        ));

        self.raw_container_mut(context.gc_context)
            .remove_child_from_depth_list(child);

        let this: DisplayObject<'_> = (*self).into();
        this.invalidate_cached_bitmap(context.gc_context);
    }

    /// Remove a set of children identified by their render list indices from
    /// this container's render and depth lists.
    fn remove_range<R>(&mut self, context: &mut UpdateContext<'_, 'gc>, range: R)
    where
        R: RangeBounds<usize>,
    {
        let removed_list: Vec<DisplayObject<'gc>> = self
            .raw_container()
            .iter_render_list()
            .enumerate()
            .filter(|(i, _)| range.contains(i))
            .map(|(_, child)| child)
            .collect();

        for removed in removed_list.iter() {
            dispatch_removed_event(*removed, context);
        }

        let mut write = self.raw_container_mut(context.gc_context);

        for removed in removed_list {
            // The `remove_range` method is only ever called as a result of an ActionScript
            // call
            removed.set_placed_by_script(context.gc_context, true);
            write.remove_child_from_depth_list(removed);
            drop(write);

            let this: DisplayObjectContainer<'gc> = (*self).into();
            ChildContainer::remove_child_from_render_list(this, removed, context);

            if !self.raw_container().movie.is_action_script_3() {
                removed.avm1_unload(context);
            } else if !matches!(removed.object2(), Avm2Value::Null) {
                removed.set_parent(context, None);
            }

            write = self.raw_container_mut(context.gc_context);
        }

        drop(write);
        let this: DisplayObject<'_> = (*self).into();
        this.invalidate_cached_bitmap(context.gc_context);
    }

    /// Determine if the container is empty.
    fn is_empty(self) -> bool {
        self.raw_container().is_empty()
    }

    /// Iterates over the children of this display object in render order.
    ///
    /// This yields an iterator that does *not* lock the parent and can be
    /// safely held in situations where display objects need to be unlocked.
    /// This will iterate over a snapshot of the render list as it was at
    /// the time `iter_render_list()` was called.
    ///
    /// The iterator's concrete type is stated here due to Rust language
    /// limitations.
    fn iter_render_list(self) -> RenderIter<'gc> {
        RenderIter::from_container(self.into())
    }

    fn is_tab_children_avm1(&self, _context: &mut UpdateContext<'_, 'gc>) -> bool {
        true
    }

    /// The property `tabChildren` allows changing the behavior of
    /// tab ordering hierarchically.
    /// When set to `false`, it excludes the whole subtree represented by
    /// the container from tab ordering.
    ///
    /// _NOTE:_
    /// According to the AS2 documentation, it should affect only automatic tab ordering.
    /// However, that does not seem to be the case, as it also affects custom ordering.
    fn is_tab_children(&self, context: &mut UpdateContext<'_, 'gc>) -> bool {
        let this: DisplayObject<'_> = (*self).into();
        if this.movie().is_action_script_3() {
            self.raw_container().tab_children
        } else {
            self.is_tab_children_avm1(context)
        }
    }

    fn set_tab_children(&self, context: &mut UpdateContext<'_, 'gc>, value: bool) {
        let this: DisplayObject<'_> = (*self).into();
        if this.movie().is_action_script_3() {
            self.raw_container_mut(context.gc()).tab_children = value;
        } else {
            this.set_avm1_property(context, "tabChildren", value.into());
        }
    }

    fn fill_tab_order(
        &self,
        tab_order: &mut Vec<InteractiveObject<'gc>>,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
        if !self.is_tab_children(context) {
            // AS3 docs say that objects with custom ordering (tabIndex set)
            // are included even when tabChildren is false.
            // Do not be fooled for that is untrue!
            return;
        }

        for child in self.iter_render_list() {
            if !child.visible() {
                // Non-visible objects and their children are excluded from tab ordering.
                continue;
            }
            if let Some(child) = child.as_interactive() {
                if child.is_tabbable(context) {
                    tab_order.push(child);
                }
            }
            if let Some(container) = child.as_container() {
                container.fill_tab_order(tab_order, context);
            }
        }
    }

    /// Renders the children of this container in render list order.
    fn render_children(self, context: &mut RenderContext<'_, 'gc>) {
        let mut clip_depth = 0;
        let mut clip_depth_stack: Vec<(Depth, DisplayObject<'_>)> = vec![];
        for child in self.iter_render_list() {
            let depth = child.depth();

            child.pre_render(context);

            // Check if we need to pop off a mask.
            // This must be a while loop because multiple masks can be popped
            // at the same depth.
            while clip_depth > 0 && depth > clip_depth {
                // Clear the mask stencil and pop the mask.
                let (prev_clip_depth, clip_child) = clip_depth_stack.pop().unwrap();
                clip_depth = prev_clip_depth;
                context.commands.deactivate_mask();
                clip_child.render(context);
                context.commands.pop_mask();
            }
            if child.clip_depth() > 0 && child.allow_as_mask() {
                // Push and render the mask.
                clip_depth_stack.push((clip_depth, child));
                clip_depth = child.clip_depth();
                context.commands.push_mask();
                child.render(context);
                context.commands.activate_mask();
            } else if child.visible() || context.commands.drawing_mask() {
                // Either a normal visible child, or a descendant of a mask object
                // that we're drawing. The 'visible' flag is ignored for all descendants
                // of a mask.
                child.render(context);
            }
        }

        // Pop any remaining masks.
        for (_, clip_child) in clip_depth_stack.into_iter().rev() {
            context.commands.deactivate_mask();
            clip_child.render(context);
            context.commands.pop_mask();
        }
    }

    #[cfg(not(feature = "avm_debug"))]
    fn recurse_render_tree(&self, _depth: usize) {}

    #[cfg(feature = "avm_debug")]
    fn recurse_render_tree(&self, depth: usize) {
        for child in self.iter_render_list() {
            child.display_render_tree(depth);
        }
    }
}

impl<'gc> From<DisplayObjectContainer<'gc>> for DisplayObject<'gc> {
    #[inline(always)]
    fn from(obj: DisplayObjectContainer<'gc>) -> Self {
        match obj {
            DisplayObjectContainer::Stage(o) => DisplayObject::Stage(o),
            DisplayObjectContainer::Avm1Button(o) => DisplayObject::Avm1Button(o),
            DisplayObjectContainer::MovieClip(o) => DisplayObject::MovieClip(o),
            DisplayObjectContainer::LoaderDisplay(o) => DisplayObject::LoaderDisplay(o),
        }
    }
}

/// A structure that stores child display objects.
///
/// Child display objects are stored in two lists: a render list and a depth
/// list. The latter references display objects by their chosen depth; while
/// the render list represents the order in which those children should be
/// rendered. Not all children have a position on this depth.
#[derive(Clone, Collect)]
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
    ///
    /// We store an `Rc` to support iterating over the render list
    /// when modifications may occur during iteration.
    /// Whenever we modify the render list, we call `Rc::make_mut`.
    /// Normally, there will only be one strong reference to this `Rc`,
    /// so we will end up modifying the list in place. However, if
    /// any `RenderIter`s exist (which hold a strong reference to the `Rc`),
    /// modifying the `ChildContainer` will cause `Rc::make_mut` to clone
    /// the underlying data, and store a new `Rc` allocation. This will
    /// cause the `RenderIter` to continue iterating over the old list,
    /// while consumers of the `ChildContainer` will immediately see the
    /// updated list.
    render_list: Rc<Vec<DisplayObject<'gc>>>,

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

    /// Does this container have any AVM1 objects that are pending removal
    ///
    /// Objects that are pending removal are placed at a negative depth in the depth list,
    /// because accessing children exclusively interacts with the render list, which cannot handle
    /// negative render depths, we need to check both lists when we have something pending removal.
    ///
    /// This should be more efficient than switching the render list to a `BTreeMap`,
    /// as it will usually be false
    has_pending_removals: bool,

    mouse_children: bool,

    /// The movie this ChildContainer belongs to.
    movie: Arc<SwfMovie>,

    /// Specifies whether children are present in the tab ordering.
    tab_children: bool,
}

impl<'gc> ChildContainer<'gc> {
    pub fn new(movie: Arc<SwfMovie>) -> Self {
        Self {
            render_list: Rc::new(Vec::new()),
            depth_list: BTreeMap::new(),
            has_pending_removals: false,
            mouse_children: true,
            movie,
            tab_children: true,
        }
    }

    /// Add a child to the depth list.
    ///
    /// This returns the child that was previously at that particular depth, if
    /// such a child exists. If so, that constitutes removing the child from
    /// the depth list.
    fn insert_child_into_depth_list(
        &mut self,
        depth: Depth,
        child: DisplayObject<'gc>,
    ) -> Option<DisplayObject<'gc>> {
        self.depth_list.insert(depth, child)
    }

    /// Remove a child from the depth list.
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

    /// Remove a child from the render list.
    ///
    /// This returns `true` if the child was successfully removed, and `false`
    /// if no list alterations were made.
    ///
    /// This must be called *before* setting the child's parent field to `None`.
    /// Note: This cannot be a normal method that takes &self, since we need to call
    /// `parent.object2()` on our own `DisplayObject` (which is borrowed mutably
    /// by `raw_container_mut`)
    fn remove_child_from_render_list(
        container: DisplayObjectContainer<'gc>,
        child: DisplayObject<'gc>,
        context: &mut UpdateContext<'_, 'gc>,
    ) -> bool {
        let mut this = container.raw_container_mut(context.gc_context);

        let render_list_position = this
            .render_list
            .iter()
            .position(|x| DisplayObject::ptr_eq(*x, child));
        if let Some(position) = render_list_position {
            this.render_list_mut().remove(position);
            drop(this);

            // Only set the parent's field to 'null' if the child was not placed/modified
            // on the render list by AVM2 code.
            if !child.placed_by_script() {
                let parent = child.parent().expect(
                    "Parent must be removed *after* calling `remove_child_from_render_list`",
                );
                if child.has_explicit_name() {
                    if let Avm2Value::Object(parent_obj) = parent.object2() {
                        let mut activation = Avm2Activation::from_nothing(context.reborrow());
                        let name = Avm2Multiname::new(
                            activation.avm2().find_public_namespace(),
                            child.name(),
                        );
                        let current_val = parent_obj.get_property(&name, &mut activation);
                        match current_val {
                            Ok(Avm2Value::Null) | Ok(Avm2Value::Undefined) => {}
                            Ok(_other) => {
                                let res = parent_obj.set_property(
                                    &name,
                                    Avm2Value::Null,
                                    &mut activation,
                                );
                                if let Err(e) = res {
                                    tracing::error!("Failed to set child {} ({:?}) to null on parent obj {:?}: {:?}", child.name(), child, parent_obj, e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to get current value of child {} ({:?}) on parent obj {:?}: {:?}", child.name(), child, parent_obj, e);
                            }
                        }
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Insert a child display object into the container at a specific position
    /// in the depth list, removing any child already at that position.
    ///
    /// After inserting the child into the depth list, we will attempt to
    /// assign it a render list position one after the previous item in the
    /// depth list. The position that children are placed into the render list
    /// matches Flash Player behavior.
    ///
    /// Any child removed from the depth list will also be removed from the
    /// render list if and only if the child was not flagged as
    /// being placed by script. If such a child was removed from said lists, it
    /// will be returned here. Otherwise, this method returns `None`.
    fn replace_at_depth(
        &mut self,
        child: DisplayObject<'gc>,
        depth: Depth,
    ) -> Option<DisplayObject<'gc>> {
        let prev_child = self.insert_child_into_depth_list(depth, child);
        if let Some(prev_child) = prev_child {
            if let Some(position) = self
                .render_list
                .iter()
                .position(|x| DisplayObject::ptr_eq(*x, prev_child))
            {
                if !prev_child.placed_by_script() {
                    self.replace_id(position, child);
                    Some(prev_child)
                } else {
                    self.insert_id(position + 1, child);
                    None
                }
            } else {
                tracing::error!(
                    "ChildContainer::replace_at_depth: Previous child is not in render list"
                );
                self.push_id(child);
                None
            }
        } else {
            let above = self
                .depth_list
                .range((Bound::Excluded(depth), Bound::Unbounded))
                .map(|(_, v)| *v)
                .next();

            if let Some(above_child) = above {
                if let Some(position) = self
                    .render_list
                    .iter()
                    .position(|x| DisplayObject::ptr_eq(*x, above_child))
                {
                    self.insert_id(position, child);
                    None
                } else {
                    self.push_id(child);
                    None
                }
            } else {
                self.push_id(child);
                None
            }
        }
    }

    /// Returns the highest depth among children.
    fn highest_depth(&self) -> Depth {
        self.depth_list.keys().next_back().copied().unwrap_or(0)
    }

    /// Determine if the render list is empty.
    fn is_empty(&self) -> bool {
        self.render_list.is_empty()
    }

    /// Get a child at a given depth.
    fn get_depth(&self, depth: Depth) -> Option<DisplayObject<'gc>> {
        self.depth_list.get(&depth).copied()
    }

    /// Get a child by it's instance/timeline name.
    ///
    /// The `case_sensitive` parameter determines if we should consider
    /// children with different capitalizations as being distinct names.
    ///
    /// If multiple children with the same name exist, the one that occurs
    /// first in the render list wins.
    fn get_name(&self, name: &WStr, case_sensitive: bool) -> Option<DisplayObject<'gc>> {
        if self.has_pending_removals {
            // Find matching children by searching the depth list
            let mut matching_render_children = if case_sensitive {
                self.depth_list
                    .iter()
                    .filter(|(_, child)| child.name_optional().map_or(false, |n| n == name))
                    .collect::<Vec<_>>()
            } else {
                self.depth_list
                    .iter()
                    .filter(|(_, child)| {
                        child
                            .name_optional()
                            .map_or(false, |n| n.eq_ignore_case(name))
                    })
                    .collect::<Vec<_>>()
            };

            // Sort so we can get the lowest depth child
            matching_render_children.sort_by_key(|&(depth, _child)| *depth);

            // First child will have the lowest depth
            return matching_render_children
                .first()
                .map(|&(_depth, child)| child)
                .copied();
        } else {
            // TODO: Make a HashMap from name -> child?
            // But need to handle conflicting names (lowest in depth order takes priority).
            if case_sensitive {
                self.render_list
                    .iter()
                    .copied()
                    .find(|child| child.name_optional().map_or(false, |n| n == name))
            } else {
                self.render_list.iter().copied().find(|child| {
                    child
                        .name_optional()
                        .map_or(false, |n| n.eq_ignore_case(name))
                })
            }
        }
    }

    /// Get a child by it's render list position (ID).
    fn get_id(&self, id: usize) -> Option<DisplayObject<'gc>> {
        self.render_list.get(id).copied()
    }

    /// Replace a child in the render list with another child in the same
    /// position.
    fn replace_id(&mut self, id: usize, child: DisplayObject<'gc>) {
        self.render_list_mut()[id] = child;
    }

    /// Insert a child into the render list at a particular position.
    fn insert_id(&mut self, id: usize, child: DisplayObject<'gc>) {
        self.render_list_mut().insert(id, child);
    }

    /// Push a child onto the end of the render list.
    fn push_id(&mut self, child: DisplayObject<'gc>) {
        self.render_list_mut().push(child);
    }

    /// Get the number of children on the render list.
    fn num_children(&self) -> usize {
        self.render_list.len()
    }

    pub fn mouse_children(&self) -> bool {
        self.mouse_children
    }

    pub fn set_mouse_children(&mut self, mouse_children: bool) {
        self.mouse_children = mouse_children;
    }

    pub fn movie(&self) -> Arc<SwfMovie> {
        self.movie.clone()
    }

    pub fn set_movie(&mut self, movie: Arc<SwfMovie>) {
        self.movie = movie;
    }

    /// Insert a child at a given render list position.
    ///
    /// If the child is already a child of another container, you must remove
    /// it from that container before calling this method. If it's already a
    /// member of this container, do not remove it, as we will need to take
    /// care of that. In that case, note that all children after the old
    /// position will be shifted back by one, which must be taken into account
    /// when calculating future insertion IDs.
    ///
    /// `parent` should be the display object that owns this container.
    ///
    /// All children at or after the given ID will be shifted down in the
    /// render list. The child will *not* be put onto the depth list.
    fn insert_at_id(&mut self, child: DisplayObject<'gc>, id: usize) {
        if let Some(old_id) = self
            .render_list
            .iter()
            .position(|x| DisplayObject::ptr_eq(*x, child))
        {
            match old_id.cmp(&id) {
                Ordering::Less if id < self.render_list.len() => {
                    self.render_list_mut()[old_id..=id].rotate_left(1)
                }
                Ordering::Less => self.render_list_mut()[old_id..id].rotate_left(1),
                Ordering::Greater if old_id < self.render_list.len() => {
                    self.render_list_mut()[id..=old_id].rotate_right(1)
                }
                Ordering::Greater => self.render_list_mut()[id..old_id].rotate_right(1),
                Ordering::Equal => {}
            }
        } else {
            self.render_list_mut().insert(id, child);
        }
    }

    /// Swap two children in the render list.
    ///
    /// No changes to the depth or render lists are made by this function.
    fn swap_at_id(&mut self, id1: usize, id2: usize) {
        self.render_list_mut().swap(id1, id2);
    }

    /// Move an already-inserted child to a new location on the depth list.
    ///
    /// If another child already exists at the target depth, it will be moved
    /// to the current depth of the given child. Their relative positions in
    /// the render list will also be swapped. If the target depth is empty, the
    /// same steps occur, but the child will merely be removed and reinserted
    /// within the render list at a position after the closest previous child
    /// in the depth list.
    ///
    /// `parent` should be the display object that owns this container.
    fn swap_at_depth(
        &mut self,
        context: &mut UpdateContext<'_, 'gc>,
        parent: DisplayObject<'gc>,
        child: DisplayObject<'gc>,
        depth: Depth,
    ) {
        let prev_depth = child.depth();
        child.set_depth(context.gc_context, depth);
        child.set_parent(context, Some(parent));

        if let Some(prev_child) = self.depth_list.insert(depth, child) {
            child.set_clip_depth(context.gc_context, 0);
            prev_child.set_depth(context.gc_context, prev_depth);
            prev_child.set_clip_depth(context.gc_context, 0);
            prev_child.set_transformed_by_script(context.gc_context, true);
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
            self.render_list_mut().swap(prev_position, next_position);
        } else {
            self.depth_list.remove(&prev_depth);

            let old_position = self
                .render_list
                .iter()
                .position(|x| DisplayObject::ptr_eq(*x, child))
                .unwrap();
            self.render_list_mut().remove(old_position);

            if let Some((_, below_child)) = self.depth_list.range(..depth).next_back() {
                let new_position = self
                    .render_list
                    .iter()
                    .position(|x| DisplayObject::ptr_eq(*x, *below_child))
                    .unwrap();
                self.render_list_mut().insert(new_position + 1, child);
            } else {
                self.render_list_mut().insert(0, child);
            }
        }
    }

    /// Yield children in the order they are rendered.
    pub fn iter_render_list<'a>(&'a self) -> impl 'a + Iterator<Item = DisplayObject<'gc>> {
        self.render_list.iter().copied()
    }

    /// Check for pending removals and update the pending removals flag
    pub fn update_pending_removals(&mut self) {
        self.has_pending_removals = self.depth_list.values().any(|c| c.avm1_pending_removal());
    }

    /// Set the pending_removals flag
    pub fn set_pending_removals(&mut self, pending: bool) {
        self.has_pending_removals = pending;
    }

    /// Should the removal of this clip be delayed to the start of the next frame
    ///
    /// Checks recursively for unload handlers
    pub fn should_delay_removal(
        activation: &mut Activation<'_, 'gc>,
        child: DisplayObject<'gc>,
    ) -> bool {
        // Do we have an unload event handler
        if let Some(mc) = child.as_movie_clip() {
            // If we have an unload handler, we need the delay
            if mc.has_unload_handler() {
                return true;
            // otherwise, check for a dynamic unload handler
            } else {
                let obj = child.object().coerce_to_object(activation);
                if obj.has_property(activation, "onUnload".into()) {
                    return true;
                }
            }
        }

        // Otherwise, check children if we have them
        if let Some(c) = child.as_container() {
            for child in c.iter_render_list() {
                if Self::should_delay_removal(activation, child) {
                    return true;
                }
            }
        }

        false
    }

    /// Enqueue the given child and all sub-children for delayed removal at the start of the next frame
    ///
    /// This just moves the children to a negative depth
    /// Will also fire unload events, as they should occur when the removal is queued, not when it actually occurs
    fn queue_removal(child: DisplayObject<'gc>, context: &mut UpdateContext<'_, 'gc>) {
        if let Some(c) = child.as_container() {
            for child in c.iter_render_list() {
                Self::queue_removal(child, context);
            }
        }

        let cur_depth = child.depth();
        // Note that the depth returned by AS will be offset by the `AVM_DEPTH_BIAS`, so this is really `-(cur_depth+1+AVM_DEPTH_BIAS)`
        child.set_depth(context.gc_context, -cur_depth - 1);
        child.set_avm1_pending_removal(context.gc_context, true);

        if let Some(mc) = child.as_movie_clip() {
            // Clip events should still fire
            mc.event_dispatch(context, crate::events::ClipEvent::Unload);
        }
    }

    fn render_list_mut(&mut self) -> &mut Vec<DisplayObject<'gc>> {
        Rc::make_mut(&mut self.render_list)
    }
}

pub struct RenderIter<'gc> {
    // We store an `Rc` cloned from the original `DisplayObjectContainer`.
    // Any modifications to the render list will call `Rc::make_mut` on
    // the `Rc` field stored by `ChildContainer`, which will leave this
    // `Rc` unaffected.
    src: Rc<Vec<DisplayObject<'gc>>>,
    i: usize,
    neg_i: usize,
}

impl<'gc> RenderIter<'gc> {
    fn from_container(src: DisplayObjectContainer<'gc>) -> Self {
        Self {
            src: src.raw_container().render_list.clone(),
            i: 0,
            neg_i: src.num_children(),
        }
    }
}

impl<'gc> Iterator for RenderIter<'gc> {
    type Item = DisplayObject<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.neg_i {
            return None;
        }

        let this = self.src.get(self.i).cloned();

        self.i += 1;

        this
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.neg_i - self.i;
        (len, Some(len))
    }
}

impl<'gc> DoubleEndedIterator for RenderIter<'gc> {
    fn next_back(&mut self) -> Option<DisplayObject<'gc>> {
        if self.i == self.neg_i {
            return None;
        }

        let this = self.src.get(self.neg_i - 1).cloned();

        self.neg_i -= 1;

        this
    }
}

impl<'gc> ExactSizeIterator for RenderIter<'gc> {}
