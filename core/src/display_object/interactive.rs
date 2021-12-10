//! Interactive object enumtrait

use crate::avm2::{Avm2, Event as Avm2Event, EventData as Avm2EventData, Value as Avm2Value};
use crate::backend::ui::MouseCursor;
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
use swf::Twips;

/// Find the lowest common ancestor between the display objects in `from` and
/// `to`.
///
/// If no such common ancestor exists, this returns `None`.
fn lowest_common_ancestor<'gc>(
    from: DisplayObject<'gc>,
    to: DisplayObject<'gc>,
) -> Option<DisplayObject<'gc>> {
    let mut from_parents = vec![];
    let mut us = Some(from);
    while let Some(parent) = us {
        from_parents.push(parent);
        us = parent.parent();
    }

    let mut to_parents = vec![];
    let mut them = Some(to);
    while let Some(parent) = them {
        to_parents.push(parent);
        them = parent.parent();
    }

    let mut hca = None;
    for (us_parent, them_parent) in from_parents
        .into_iter()
        .rev()
        .zip(to_parents.into_iter().rev())
    {
        if DisplayObject::ptr_eq(us_parent, them_parent) {
            hca = Some(us_parent);
        } else {
            break;
        }
    }

    hca
}

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
    context_menu: Avm2Value<'gc>,
}

impl<'gc> Default for InteractiveObjectBase<'gc> {
    fn default() -> Self {
        Self {
            base: Default::default(),
            flags: InteractiveObjectFlags::MOUSE_ENABLED,
            context_menu: Avm2Value::Null,
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

    fn context_menu(self) -> Avm2Value<'gc> {
        self.ibase().context_menu
    }

    fn set_context_menu(self, mc: MutationContext<'gc, '_>, value: Avm2Value<'gc>) {
        self.ibase_mut(mc).context_menu = value;
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
        event: ClipEvent<'gc>,
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
        _event: ClipEvent<'gc>,
    ) -> ClipEventResult;

    /// Convert the clip event into an AVM2 event and dispatch it into the
    /// AVM2 side of this object.
    ///
    /// This is only intended to be called for events defined by
    /// `InteractiveObject` itself. Display object impls that have their own
    /// event types should dispatch them in `event_dispatch`.
    fn event_dispatch_to_avm2(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        let target = if let Avm2Value::Object(target) = self.as_displayobject().object2() {
            target
        } else {
            return ClipEventResult::NotHandled;
        };

        match event {
            ClipEvent::Press => {
                let mut avm2_event = Avm2Event::new(
                    "mouseDown",
                    Avm2EventData::mouse_event(context, self.as_displayobject(), None, 0),
                );

                avm2_event.set_bubbles(true);

                if let Err(e) = Avm2::dispatch_event(context, avm2_event, target) {
                    log::error!("Got error when dispatching {:?} to AVM2: {}", event, e);
                }

                ClipEventResult::Handled
            }
            ClipEvent::MouseUpInside => {
                let mut avm2_event = Avm2Event::new(
                    "mouseUp",
                    Avm2EventData::mouse_event(context, self.as_displayobject(), None, 0),
                );

                avm2_event.set_bubbles(true);

                if let Err(e) = Avm2::dispatch_event(context, avm2_event, target) {
                    log::error!("Got error when dispatching {:?} to AVM2: {}", event, e);
                }

                ClipEventResult::Handled
            }
            ClipEvent::Release => {
                let mut avm2_event = Avm2Event::new(
                    "click",
                    Avm2EventData::mouse_event(context, self.as_displayobject(), None, 0),
                );

                avm2_event.set_bubbles(true);

                if let Err(e) = Avm2::dispatch_event(context, avm2_event, target) {
                    log::error!("Got error when dispatching {:?} to AVM2: {}", event, e);
                }

                ClipEventResult::Handled
            }
            ClipEvent::ReleaseOutside => {
                let mut avm2_event = Avm2Event::new(
                    "releaseOutside",
                    Avm2EventData::mouse_event(context, self.as_displayobject(), None, 0),
                );

                avm2_event.set_bubbles(true);

                if let Err(e) = Avm2::dispatch_event(context, avm2_event, target) {
                    log::error!("Got error when dispatching {:?} to AVM2: {}", event, e);
                }

                ClipEventResult::Handled
            }
            ClipEvent::RollOut { to } | ClipEvent::DragOut { to } => {
                let mut avm2_event = Avm2Event::new(
                    "mouseOut",
                    Avm2EventData::mouse_event(context, self.as_displayobject(), to, 0),
                );

                avm2_event.set_bubbles(true);

                if let Err(e) = Avm2::dispatch_event(context, avm2_event, target) {
                    log::error!("Got error when dispatching {:?} to AVM2: {}", event, e);
                }

                let lca = lowest_common_ancestor(
                    self.as_displayobject(),
                    to.map(|t| t.as_displayobject())
                        .unwrap_or_else(|| context.stage.into()),
                );

                let mut rollover_target = Some(self.as_displayobject());
                while let Some(tgt) = rollover_target {
                    if DisplayObject::option_ptr_eq(rollover_target, lca) {
                        break;
                    }

                    let avm2_event =
                        Avm2Event::new("rollOut", Avm2EventData::mouse_event(context, tgt, to, 0));

                    if let Avm2Value::Object(avm2_target) = tgt.object2() {
                        if let Err(e) = Avm2::dispatch_event(context, avm2_event, avm2_target) {
                            log::error!("Got error when dispatching {:?} to AVM2: {}", event, e);
                        }
                    }

                    rollover_target = tgt.parent();
                }

                ClipEventResult::Handled
            }
            ClipEvent::RollOver { from } | ClipEvent::DragOver { from } => {
                let lca = lowest_common_ancestor(
                    self.as_displayobject(),
                    from.map(|t| t.as_displayobject())
                        .unwrap_or_else(|| context.stage.into()),
                );

                let mut rollover_target = Some(self.as_displayobject());
                while let Some(tgt) = rollover_target {
                    if DisplayObject::option_ptr_eq(rollover_target, lca) {
                        break;
                    }

                    let avm2_event = Avm2Event::new(
                        "rollOver",
                        Avm2EventData::mouse_event(context, tgt, from, 0),
                    );

                    if let Avm2Value::Object(avm2_target) = tgt.object2() {
                        if let Err(e) = Avm2::dispatch_event(context, avm2_event, avm2_target) {
                            log::error!("Got error when dispatching {:?} to AVM2: {}", event, e);
                        }
                    }

                    rollover_target = tgt.parent();
                }

                let mut avm2_event = Avm2Event::new(
                    "mouseOver",
                    Avm2EventData::mouse_event(context, self.as_displayobject(), from, 0),
                );

                avm2_event.set_bubbles(true);

                if let Err(e) = Avm2::dispatch_event(context, avm2_event, target) {
                    log::error!("Got error when dispatching {:?} to AVM2: {}", event, e);
                }

                ClipEventResult::Handled
            }
            _ => ClipEventResult::NotHandled,
        }
    }

    /// Executes and propagates the given clip event.
    /// Events execute inside-out; the deepest child will react first, followed
    /// by its parent, and so forth.
    fn handle_clip_event(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent<'gc>,
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

    /// Determine the bottom-most interactive display object under the given
    /// mouse cursor.
    ///
    /// Only objects capable of handling mouse input should flag themselves as
    /// mouse-pickable, as doing so will make them eligible to recieve targeted
    /// mouse events. As a result of this, the returned object will always be
    /// an `InteractiveObject`.
    fn mouse_pick(
        &self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        _pos: (Twips, Twips),
        _require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        None
    }

    /// The cursor to use when this object is the hovered element under a mouse.
    fn mouse_cursor(self, _context: &mut UpdateContext<'_, 'gc, '_>) -> MouseCursor {
        MouseCursor::Hand
    }
}

impl<'gc> InteractiveObject<'gc> {
    pub fn ptr_eq<T: TInteractiveObject<'gc>>(a: T, b: T) -> bool {
        a.as_displayobject().as_ptr() == b.as_displayobject().as_ptr()
    }

    pub fn option_ptr_eq(
        a: Option<InteractiveObject<'gc>>,
        b: Option<InteractiveObject<'gc>>,
    ) -> bool {
        a.map(|o| o.as_displayobject().as_ptr()) == b.map(|o| o.as_displayobject().as_ptr())
    }
}

impl<'gc> PartialEq for InteractiveObject<'gc> {
    fn eq(&self, other: &Self) -> bool {
        InteractiveObject::ptr_eq(*self, *other)
    }
}

impl<'gc> Eq for InteractiveObject<'gc> {}
