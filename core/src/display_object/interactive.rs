//! Interactive object enumtrait

use crate::context::UpdateContext;
use crate::display_object::avm1_button::Avm1Button;
use crate::display_object::avm2_button::Avm2Button;
use crate::display_object::edit_text::EditText;
use crate::display_object::movie_clip::MovieClip;
use crate::display_object::stage::Stage;
use crate::display_object::{
    DisplayObject, DisplayObjectBase, TDisplayObject, TDisplayObjectContainer,
};
use crate::events::{ClipEvent, ClipEventResult};
use bitflags::bitflags;
use gc_arena::{Collect, MutationContext};
use ruffle_macros::enum_trait_object;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

bitflags! {
    /// Boolean state flags used by `InteractiveObject`.
    #[derive(Collect)]
    #[collect(require_static)]
    struct InteractiveObjectFlags: u8 {
        /// Whether this `InteractiveObject` accepts mouse and other user
        /// events.
        const MOUSE_ENABLED = 1 << 0;

        /// Whether this `InteractiveObject` accepts double-clicks.
        const DOUBLE_CLICK_ENABLED = 1 << 1;
    }
}

#[derive(Collect, Clone, Debug)]
#[collect(no_drop)]
pub struct InteractiveObjectBase<'gc> {
    pub base: DisplayObjectBase<'gc>,
    flags: InteractiveObjectFlags,
}

impl<'gc> Default for InteractiveObjectBase<'gc> {
    fn default() -> Self {
        Self {
            base: Default::default(),
            flags: InteractiveObjectFlags::MOUSE_ENABLED,
        }
    }
}

#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum InteractiveObject<'gc> {
        Stage(Stage<'gc>),
        Avm1Button(Avm1Button<'gc>),
        Avm2Button(Avm2Button<'gc>),
        MovieClip(MovieClip<'gc>),
        EditText(EditText<'gc>),
    }
)]
pub trait TInteractiveObject<'gc>:
    'gc + Clone + Copy + Collect + Debug + Into<InteractiveObject<'gc>>
{
    fn ibase(&self) -> Ref<InteractiveObjectBase<'gc>>;

    fn ibase_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<InteractiveObjectBase<'gc>>;

    fn as_displayobject(self) -> DisplayObject<'gc>;

    /// Check if the interactive object accepts user input.
    fn mouse_enabled(self) -> bool {
        self.ibase()
            .flags
            .contains(InteractiveObjectFlags::MOUSE_ENABLED)
    }

    /// Set if the interactive object accepts user input.
    fn set_mouse_enabled(self, mc: MutationContext<'gc, '_>, value: bool) {
        self.ibase_mut(mc)
            .flags
            .set(InteractiveObjectFlags::MOUSE_ENABLED, value)
    }

    /// Check if the interactive object accepts double-click events.
    fn double_click_enabled(self) -> bool {
        self.ibase()
            .flags
            .contains(InteractiveObjectFlags::DOUBLE_CLICK_ENABLED)
    }

    // Set if the interactive object accepts double-click events.
    fn set_double_click_enabled(self, mc: MutationContext<'gc, '_>, value: bool) {
        self.ibase_mut(mc)
            .flags
            .set(InteractiveObjectFlags::DOUBLE_CLICK_ENABLED, value)
    }

    /// Filter the incoming clip event.
    ///
    /// If this returns `Handled`, then the rest of the event handling
    /// machinery should run. Otherwise, the event will not be handled, neither
    /// by this interactive object nor it's children. The event will be passed
    /// onto other siblings of the display object instead.
    fn filter_clip_event(self, event: ClipEvent) -> ClipEventResult;

    /// Propagate the event to children.
    ///
    /// If this function returns `Handled`, then further event processing will
    /// terminate, including the event default.
    fn propagate_to_children(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent,
    ) -> ClipEventResult {
        if event.propagates() {
            if let Some(container) = self.as_displayobject().as_container() {
                for child in container.iter_render_list() {
                    if let Some(interactive) = child.as_interactive() {
                        if interactive.handle_clip_event(context, event) == ClipEventResult::Handled
                        {
                            return ClipEventResult::Handled;
                        }
                    }
                }
            }
        }

        ClipEventResult::NotHandled
    }

    /// Dispatch the event to script event handlers.
    ///
    /// This function only runs if the clip event has not been filtered and
    /// none of the interactive object's children handled the event. It
    /// ultimately determines if this display object will handle the event, or
    /// if the event will be passed onto siblings and parents.
    fn event_dispatch(
        self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        _event: ClipEvent,
    ) -> ClipEventResult;

    /// Executes and propagates the given clip event.
    /// Events execute inside-out; the deepest child will react first, followed
    /// by its parent, and so forth.
    fn handle_clip_event(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent,
    ) -> ClipEventResult {
        if !self.mouse_enabled() {
            return ClipEventResult::NotHandled;
        }

        if self.filter_clip_event(event) == ClipEventResult::NotHandled {
            return ClipEventResult::NotHandled;
        }

        if self.propagate_to_children(context, event) == ClipEventResult::Handled {
            return ClipEventResult::Handled;
        }

        self.event_dispatch(context, event)
    }
}
