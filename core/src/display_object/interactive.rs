//! Interactive object enumtrait

use crate::avm1::Avm1;
use crate::avm1::Value as Avm1Value;
use crate::avm2::activation::Activation as Avm2Activation;
use crate::avm2::{Avm2, EventObject as Avm2EventObject, EventObject, Value as Avm2Value};
use crate::backend::ui::MouseCursor;
use crate::context::UpdateContext;
use crate::display_object::avm1_button::Avm1Button;
use crate::display_object::avm2_button::Avm2Button;
use crate::display_object::container::DisplayObjectContainer;
use crate::display_object::edit_text::EditText;
use crate::display_object::loader_display::LoaderDisplay;
use crate::display_object::movie_clip::MovieClip;
use crate::display_object::stage::Stage;
use crate::display_object::{
    DisplayObject, DisplayObjectBase, TDisplayObject, TDisplayObjectContainer,
};
use crate::events::{ClipEvent, ClipEventResult, MouseButton};
use bitflags::bitflags;
use gc_arena::{Collect, Mutation};
use ruffle_macros::enum_trait_object;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use swf::{Point, Rectangle, Twips};

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
    #[derive(Clone, Copy)]
    struct InteractiveObjectFlags: u8 {
        /// Whether this `InteractiveObject` accepts mouse and other user
        /// events.
        const MOUSE_ENABLED = 1 << 0;

        /// Whether this `InteractiveObject` accepts double-clicks.
        const DOUBLE_CLICK_ENABLED = 1 << 1;

        /// Whether this `InteractiveObject` is currently focused.
        const HAS_FOCUS = 1 << 2;
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct InteractiveObjectBase<'gc> {
    pub base: DisplayObjectBase<'gc>,
    #[collect(require_static)]
    flags: InteractiveObjectFlags,
    context_menu: Avm2Value<'gc>,

    #[collect(require_static)]
    tab_enabled: Option<bool>,

    #[collect(require_static)]
    tab_index: Option<i32>,

    /// Specifies whether this object displays a yellow rectangle when focused.
    focus_rect: Option<bool>,
}

impl Default for InteractiveObjectBase<'_> {
    fn default() -> Self {
        Self {
            base: Default::default(),
            flags: InteractiveObjectFlags::MOUSE_ENABLED,
            context_menu: Avm2Value::Null,
            tab_enabled: None,
            tab_index: None,
            focus_rect: None,
        }
    }
}

#[enum_trait_object(
    #[derive(Clone, Collect, Copy, Debug)]
    #[collect(no_drop)]
    pub enum InteractiveObject<'gc> {
        Stage(Stage<'gc>),
        Avm1Button(Avm1Button<'gc>),
        Avm2Button(Avm2Button<'gc>),
        MovieClip(MovieClip<'gc>),
        EditText(EditText<'gc>),
        LoaderDisplay(LoaderDisplay<'gc>),
    }
)]
pub trait TInteractiveObject<'gc>:
    'gc + Clone + Copy + Collect + Debug + Into<InteractiveObject<'gc>>
{
    fn raw_interactive(&self) -> Ref<InteractiveObjectBase<'gc>>;

    fn raw_interactive_mut(&self, mc: &Mutation<'gc>) -> RefMut<InteractiveObjectBase<'gc>>;

    fn as_displayobject(self) -> DisplayObject<'gc>;

    /// Check if the interactive object accepts user input.
    fn mouse_enabled(self) -> bool {
        self.raw_interactive()
            .flags
            .contains(InteractiveObjectFlags::MOUSE_ENABLED)
    }

    /// Set if the interactive object accepts user input.
    fn set_mouse_enabled(self, mc: &Mutation<'gc>, value: bool) {
        self.raw_interactive_mut(mc)
            .flags
            .set(InteractiveObjectFlags::MOUSE_ENABLED, value)
    }

    /// Check if the interactive object accepts double-click events.
    fn double_click_enabled(self) -> bool {
        self.raw_interactive()
            .flags
            .contains(InteractiveObjectFlags::DOUBLE_CLICK_ENABLED)
    }

    // Set if the interactive object accepts double-click events.
    fn set_double_click_enabled(self, mc: &Mutation<'gc>, value: bool) {
        self.raw_interactive_mut(mc)
            .flags
            .set(InteractiveObjectFlags::DOUBLE_CLICK_ENABLED, value)
    }

    fn has_focus(self) -> bool {
        self.raw_interactive()
            .flags
            .contains(InteractiveObjectFlags::HAS_FOCUS)
    }

    fn set_has_focus(self, mc: &Mutation<'gc>, value: bool) {
        self.raw_interactive_mut(mc)
            .flags
            .set(InteractiveObjectFlags::HAS_FOCUS, value)
    }

    fn context_menu(self) -> Avm2Value<'gc> {
        self.raw_interactive().context_menu
    }

    fn set_context_menu(self, mc: &Mutation<'gc>, value: Avm2Value<'gc>) {
        self.raw_interactive_mut(mc).context_menu = value;
    }

    /// Get the boolean flag which determines whether objects display a glowing border
    /// when they have focus.
    fn focus_rect(self) -> Option<bool> {
        self.raw_interactive().focus_rect
    }

    /// Set the boolean flag which determines whether objects display a glowing border
    /// when they have focus.
    fn set_focus_rect(self, mc: &Mutation<'gc>, value: Option<bool>) {
        self.raw_interactive_mut(mc).focus_rect = value;
    }

    /// Filter the incoming clip event.
    ///
    /// If this returns `Handled`, then the rest of the event handling
    /// machinery should run. Otherwise, the event will not be handled, neither
    /// by this interactive object nor it's children. The event will be passed
    /// onto other siblings of the display object instead.
    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'gc>,
        event: ClipEvent,
    ) -> ClipEventResult;

    /// Propagate the event to children.
    ///
    /// If this function returns `Handled`, then further event processing will
    /// terminate, including the event default.
    fn propagate_to_children(
        self,
        context: &mut UpdateContext<'gc>,
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
        _context: &mut UpdateContext<'gc>,
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
        context: &mut UpdateContext<'gc>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        if !self.as_displayobject().movie().is_action_script_3() {
            return ClipEventResult::NotHandled;
        }

        // Flash appears to not fire events *at all* for a targeted EditText
        // that was originally created by the timeline. Normally, one of the ancestors
        // of the TextField would get targeted, but instead, the event isn't fired
        // (not even the Stage receives the event)
        if let Some(text) = self.as_displayobject().as_edit_text() {
            if text.is_selectable() && text.was_static() {
                return ClipEventResult::NotHandled;
            }
        }

        let target = if let Avm2Value::Object(target) = self.as_displayobject().object2() {
            target
        } else {
            return ClipEventResult::NotHandled;
        };

        let mut activation = Avm2Activation::from_nothing(context);

        match event {
            ClipEvent::Press { .. } | ClipEvent::RightPress | ClipEvent::MiddlePress => {
                let button = match event {
                    ClipEvent::Press { .. } => MouseButton::Left,
                    ClipEvent::RightPress => MouseButton::Right,
                    ClipEvent::MiddlePress => MouseButton::Middle,
                    _ => unreachable!(),
                };
                let avm2_event = Avm2EventObject::mouse_event_down(
                    &mut activation,
                    self.as_displayobject(),
                    button,
                );

                let handled = Avm2::dispatch_event(activation.context, avm2_event, target);
                if handled {
                    ClipEventResult::Handled
                } else {
                    // When there are any click handlers, the down event is considered handled.
                    let avm2_event = Avm2EventObject::mouse_event_click(
                        &mut activation,
                        self.as_displayobject(),
                        button,
                    );
                    Avm2::simulate_event_dispatch(activation.context, avm2_event, target).into()
                }
            }
            ClipEvent::MouseUpInside
            | ClipEvent::RightMouseUpInside
            | ClipEvent::MiddleMouseUpInside => {
                let avm2_event = Avm2EventObject::mouse_event_up(
                    &mut activation,
                    self.as_displayobject(),
                    match event {
                        ClipEvent::MouseUpInside => MouseButton::Left,
                        ClipEvent::RightMouseUpInside => MouseButton::Right,
                        ClipEvent::MiddleMouseUpInside => MouseButton::Middle,
                        _ => unreachable!(),
                    },
                );

                Avm2::dispatch_event(activation.context, avm2_event, target).into()
            }
            ClipEvent::Release { index } => {
                let is_double_click = index % 2 != 0;
                let double_click_enabled = self
                    .raw_interactive()
                    .flags
                    .contains(InteractiveObjectFlags::DOUBLE_CLICK_ENABLED);

                if is_double_click && double_click_enabled {
                    let avm2_event = Avm2EventObject::mouse_event(
                        &mut activation,
                        "doubleClick",
                        self.as_displayobject(),
                        None,
                        0,
                        true,
                        MouseButton::Left,
                    );

                    Avm2::dispatch_event(activation.context, avm2_event, target).into()
                } else {
                    let avm2_event = Avm2EventObject::mouse_event_click(
                        &mut activation,
                        self.as_displayobject(),
                        MouseButton::Left,
                    );

                    Avm2::dispatch_event(activation.context, avm2_event, target).into()
                }
            }
            ClipEvent::RightRelease | ClipEvent::MiddleRelease => {
                let avm2_event = Avm2EventObject::mouse_event_click(
                    &mut activation,
                    self.as_displayobject(),
                    match event {
                        ClipEvent::RightRelease => MouseButton::Right,
                        ClipEvent::MiddleRelease => MouseButton::Middle,
                        _ => unreachable!(),
                    },
                );

                Avm2::dispatch_event(activation.context, avm2_event, target).into()
            }
            ClipEvent::ReleaseOutside => {
                let avm2_event = Avm2EventObject::mouse_event(
                    &mut activation,
                    "releaseOutside",
                    self.as_displayobject(),
                    None,
                    0,
                    true,
                    MouseButton::Left,
                );

                Avm2::dispatch_event(activation.context, avm2_event, target).into()
            }
            ClipEvent::RollOut { to } | ClipEvent::DragOut { to } => {
                let avm2_event = Avm2EventObject::mouse_event(
                    &mut activation,
                    "mouseOut",
                    self.as_displayobject(),
                    to,
                    0,
                    true,
                    MouseButton::Left,
                );

                let mut handled = Avm2::dispatch_event(activation.context, avm2_event, target);

                let lca = lowest_common_ancestor(
                    self.as_displayobject(),
                    to.map(|t| t.as_displayobject())
                        .unwrap_or_else(|| activation.context.stage.into()),
                );

                let mut rollout_target = Some(self.as_displayobject());
                while let Some(tgt) = rollout_target {
                    if DisplayObject::option_ptr_eq(rollout_target, lca) {
                        break;
                    }

                    let avm2_event = Avm2EventObject::mouse_event(
                        &mut activation,
                        "rollOut",
                        tgt,
                        to,
                        0,
                        false,
                        MouseButton::Left,
                    );

                    if let Avm2Value::Object(avm2_target) = tgt.object2() {
                        handled = Avm2::dispatch_event(activation.context, avm2_event, avm2_target)
                            || handled;
                    }

                    rollout_target = tgt.parent();
                }

                handled.into()
            }
            ClipEvent::RollOver { from } | ClipEvent::DragOver { from } => {
                let lca = lowest_common_ancestor(
                    self.as_displayobject(),
                    from.map(|t| t.as_displayobject())
                        .unwrap_or_else(|| activation.context.stage.into()),
                );

                let mut handled = false;
                let mut rollover_target = Some(self.as_displayobject());
                while let Some(tgt) = rollover_target {
                    if DisplayObject::option_ptr_eq(rollover_target, lca) {
                        break;
                    }

                    let avm2_event = Avm2EventObject::mouse_event(
                        &mut activation,
                        "rollOver",
                        tgt,
                        from,
                        0,
                        false,
                        MouseButton::Left,
                    );

                    if let Avm2Value::Object(avm2_target) = tgt.object2() {
                        handled = Avm2::dispatch_event(activation.context, avm2_event, avm2_target)
                            || handled;
                    }

                    rollover_target = tgt.parent();
                }

                let avm2_event = Avm2EventObject::mouse_event(
                    &mut activation,
                    "mouseOver",
                    self.as_displayobject(),
                    from,
                    0,
                    true,
                    MouseButton::Left,
                );

                handled = Avm2::dispatch_event(activation.context, avm2_event, target) || handled;

                handled.into()
            }
            ClipEvent::MouseWheel { delta } => {
                let avm2_event = Avm2EventObject::mouse_event(
                    &mut activation,
                    "mouseWheel",
                    self.as_displayobject(),
                    None,
                    delta.lines() as i32,
                    true,
                    MouseButton::Left,
                );

                Avm2::dispatch_event(activation.context, avm2_event, target).into()
            }
            ClipEvent::MouseMoveInside => {
                let avm2_event = Avm2EventObject::mouse_event(
                    &mut activation,
                    "mouseMove",
                    self.as_displayobject(),
                    None,
                    0,
                    true,
                    MouseButton::Left,
                );

                Avm2::dispatch_event(activation.context, avm2_event, target).into()
            }
            _ => ClipEventResult::NotHandled,
        }
    }

    /// Executes and propagates the given clip event.
    /// Events execute inside-out; the deepest child will react first, followed
    /// by its parent, and so forth.
    fn handle_clip_event(
        self,
        context: &mut UpdateContext<'gc>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        if !self.mouse_enabled() {
            return ClipEventResult::NotHandled;
        }

        if self.filter_clip_event(context, event) == ClipEventResult::NotHandled {
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
    /// mouse-pickable, as doing so will make them eligible to receive targeted
    /// mouse events. As a result of this, the returned object will always be
    /// an `InteractiveObject`.
    fn mouse_pick_avm1(
        &self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        None
    }

    fn mouse_pick_avm2(
        &self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        Avm2MousePick::Miss
    }

    /// The cursor to use when this object is the hovered element under a mouse.
    fn mouse_cursor(self, _context: &mut UpdateContext<'gc>) -> MouseCursor {
        MouseCursor::Hand
    }

    /// Whether this object is focusable for keyboard input.
    fn is_focusable(&self, _context: &mut UpdateContext<'gc>) -> bool {
        // By default, all interactive objects are focusable.
        true
    }

    /// Whether this object is focusable using a pointer device,
    /// i.e. whether the focus should be updated when it's clicked.
    ///
    /// The default behavior is following:
    /// * in AVM1 objects cannot be focused by mouse,
    /// * in AVM2 objects can be focused by mouse when they are tab enabled.
    fn is_focusable_by_mouse(&self, context: &mut UpdateContext<'gc>) -> bool {
        let self_do = self.as_displayobject();
        self_do.movie().is_action_script_3() && self.tab_enabled(context)
    }

    /// Called whenever the focus tracker has deemed this display object worthy, or no longer worthy,
    /// of being the currently focused object.
    /// This should only be called by the focus manager. To change a focus, go through that.
    fn on_focus_changed(
        &self,
        _context: &mut UpdateContext<'gc>,
        _focused: bool,
        _other: Option<InteractiveObject<'gc>>,
    ) {
    }

    /// If this object has focus, this method drops it.
    fn drop_focus(&self, context: &mut UpdateContext<'gc>) {
        if self.has_focus() {
            let tracker = context.focus_tracker;
            tracker.set(None, context);
        }
    }

    fn call_focus_handler(
        &self,
        context: &mut UpdateContext<'gc>,
        focused: bool,
        other: Option<InteractiveObject<'gc>>,
    ) {
        let self_do = self.as_displayobject();
        if let Avm1Value::Object(object) = self_do.object() {
            let other = other
                .map(|d| d.as_displayobject().object())
                .unwrap_or(Avm1Value::Null);
            let method_name = if focused {
                "onSetFocus".into()
            } else {
                "onKillFocus".into()
            };
            Avm1::run_stack_frame_for_method(self_do, object, context, method_name, &[other]);
        } else if let Avm2Value::Object(object) = self_do.object2() {
            let mut activation = Avm2Activation::from_nothing(context);
            let event_name = if focused { "focusIn" } else { "focusOut" };
            let event = EventObject::focus_event(&mut activation, event_name, false, other, 0);
            Avm2::dispatch_event(activation.context, event, object);
        }
    }

    /// Whether this object may be highlighted when focused.
    fn is_highlightable(&self, context: &mut UpdateContext<'gc>) -> bool {
        self.is_highlight_enabled(context)
    }

    /// Whether highlight is enabled for this object.
    ///
    /// Note: This value does not mean that a highlight should actually be rendered,
    /// for that see [`Self::is_highlightable()`].
    fn is_highlight_enabled(&self, context: &mut UpdateContext<'gc>) -> bool {
        if self.as_displayobject().movie().version() >= 6 {
            self.focus_rect()
                .unwrap_or_else(|| context.stage.stage_focus_rect())
        } else {
            context.stage.stage_focus_rect()
        }
    }

    /// Get the bounds of the focus highlight.
    fn highlight_bounds(self) -> Rectangle<Twips> {
        self.as_displayobject().world_bounds()
    }

    /// Whether this object is included in tab ordering.
    fn is_tabbable(&self, context: &mut UpdateContext<'gc>) -> bool {
        self.tab_enabled(context)
    }

    /// Sets whether tab ordering is enabled for this object.
    ///
    /// Some objects may be excluded from tab ordering
    /// even if it's enabled, see [`Self::is_tabbable()`].
    fn tab_enabled(&self, context: &mut UpdateContext<'gc>) -> bool {
        if self.as_displayobject().movie().is_action_script_3() {
            self.raw_interactive()
                .tab_enabled
                .unwrap_or_else(|| self.tab_enabled_default(context))
        } else {
            self.as_displayobject()
                .get_avm1_boolean_property(context, "tabEnabled", |context| {
                    self.tab_enabled_default(context)
                })
        }
    }

    fn tab_enabled_default(&self, _context: &mut UpdateContext<'gc>) -> bool {
        false
    }

    fn set_tab_enabled(&self, context: &mut UpdateContext<'gc>, value: bool) {
        if self.as_displayobject().movie().is_action_script_3() {
            self.raw_interactive_mut(context.gc()).tab_enabled = Some(value)
        } else {
            self.as_displayobject()
                .set_avm1_property(context, "tabEnabled", value.into());
        }
    }

    /// Used to customize tab ordering.
    /// When not `None`, a custom ordering is used, and
    /// objects are ordered according to this value.
    fn tab_index(&self) -> Option<i32> {
        self.raw_interactive().tab_index
    }

    fn set_tab_index(&self, context: &mut UpdateContext<'gc>, value: Option<i32>) {
        // tabIndex = -1 is always equivalent to unset tabIndex
        let value = if matches!(value, Some(-1)) {
            None
        } else {
            value
        };
        self.raw_interactive_mut(context.gc()).tab_index = value
    }

    /// Whether event handlers (e.g. onKeyUp, onPress) should be fired for the given event.
    fn should_fire_event_handlers(
        &self,
        context: &mut UpdateContext<'gc>,
        event: ClipEvent,
    ) -> bool {
        // Event handlers are supported only by SWF6+.
        if self.as_displayobject().movie().version() < 6 {
            return false;
        }

        // Non-keyboard events are always handled.
        if !event.is_key_event() {
            return true;
        }

        // Keyboard events don't fire their methods unless the object has focus (#2120).
        // The focus highlight also has to be active (see test focus_keyboard_press).
        self.has_focus() && context.focus_tracker.highlight().is_active()
    }
}

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub enum Avm2MousePick<'gc> {
    Hit(InteractiveObject<'gc>),
    PropagateToParent,
    Miss,
}

impl Debug for Avm2MousePick<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Avm2MousePick::Hit(target) => write!(f, "Hit({:?})", target.as_displayobject().name()),
            Avm2MousePick::PropagateToParent => write!(f, "PropagateToParent"),
            Avm2MousePick::Miss => write!(f, "Miss"),
        }
    }
}

impl<'gc> Avm2MousePick<'gc> {
    /// Modifies this result to account for the parent's `mouseEnabled` and `mouseChildren` properties
    #[must_use]
    pub fn combine_with_parent(&self, parent: DisplayObjectContainer<'gc>) -> Avm2MousePick<'gc> {
        let parent_int = DisplayObject::from(parent).as_interactive().unwrap();
        let res = match self {
            Avm2MousePick::Hit(_) => {
                // If the parent has `mouseChildren=true` then propagate the existing
                // Avm2MousePick::Hit, leaving the target unchanged. This is unaffected
                // by the parent `mouseEnabled` property.
                if parent.raw_container().mouse_children() {
                    *self
                // If the parent has `mouseChildren=false`, then the eventual
                // MouseEvent (if it gets fired) will *not* have a `target`
                // set to the original child.
                } else {
                    // If the parent has `mouseChildren=false` and `mouseEnabled=true`,
                    // then the event from the child gets converted into an event
                    // targeting the parent - it 'absorbs' child events.
                    if parent_int.mouse_enabled() {
                        Avm2MousePick::Hit(parent_int)
                    // If the parent has `mouseChildren=false` and `mouseEnabled=true`,
                    // we have a weird case. The event can propagate through this 'fully disabled'
                    // parent - if it reaches an ancestor with `mouseEnabled=true`, it will get
                    // 'absorbed' by that ancestor. Otherwise, no event will be fired.
                    } else {
                        Avm2MousePick::PropagateToParent
                    }
                }
            }
            Avm2MousePick::PropagateToParent => {
                // If the parent has `mouseEnabled=true`, then 'absorb'
                // the event that was propagated up from some child. Note that
                // the `mouseChildren` setting plays no role here.
                if parent_int.mouse_enabled() {
                    Avm2MousePick::Hit(parent_int)
                // Otherwise, continue propagating the event up the tree.
                } else {
                    Avm2MousePick::PropagateToParent
                }
            }
            // A miss in a child always stays a miss, regardless of parent settings.
            Avm2MousePick::Miss => Avm2MousePick::Miss,
        };
        res
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

impl PartialEq for InteractiveObject<'_> {
    fn eq(&self, other: &Self) -> bool {
        InteractiveObject::ptr_eq(*self, *other)
    }
}

impl Eq for InteractiveObject<'_> {}
