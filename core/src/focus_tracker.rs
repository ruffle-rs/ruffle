use crate::avm1::Avm1;
use crate::avm1::Value;
use crate::context::{RenderContext, UpdateContext};
pub use crate::display_object::{
    DisplayObject, TDisplayObject, TDisplayObjectContainer, TextSelection,
};
use crate::display_object::{EditText, InteractiveObject, TInteractiveObject};
use crate::events::ClipEvent;
use crate::Player;
use either::Either;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use std::cell::RefCell;
use swf::{Color, Twips};

#[derive(Collect)]
#[collect(no_drop)]
pub struct FocusTrackerData<'gc> {
    focus: Lock<Option<InteractiveObject<'gc>>>,
    highlight: RefCell<Highlight>,
}

#[derive(Copy, Clone)]
pub enum Highlight {
    /// The focus is highlighted and the highlight is visible on the screen.
    ///
    /// This is the required state for keyboard navigation to work.
    ActiveVisible,

    /// The focus is highlighted, but the highlight is not visible on the screen.
    ///
    /// Some keyboard events (KeyUp, KeyDown) require this logic.
    ActiveHidden,

    /// The focus is not highlighted.
    Inactive,
}

impl Highlight {
    pub fn is_active(self) -> bool {
        matches!(self, Highlight::ActiveVisible | Highlight::ActiveHidden)
    }

    pub fn is_visible(self) -> bool {
        matches!(self, Highlight::ActiveVisible)
    }
}

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct FocusTracker<'gc>(Gc<'gc, FocusTrackerData<'gc>>);

impl<'gc> FocusTracker<'gc> {
    const HIGHLIGHT_THICKNESS: Twips = Twips::from_pixels_i32(3);
    const HIGHLIGHT_COLOR: Color = Color::YELLOW;

    pub fn new(mc: &Mutation<'gc>) -> Self {
        Self(Gc::new(
            mc,
            FocusTrackerData {
                focus: Lock::new(None),
                highlight: RefCell::new(Highlight::Inactive),
            },
        ))
    }

    pub fn highlight(&self) -> Highlight {
        *self.0.highlight.borrow()
    }

    pub fn reset_highlight(&self) {
        self.0.highlight.replace(Highlight::Inactive);
    }

    pub fn get(&self) -> Option<InteractiveObject<'gc>> {
        self.0.focus.get()
    }

    pub fn get_as_edit_text(&self) -> Option<EditText<'gc>> {
        self.get()
            .map(|o| o.as_displayobject())
            .and_then(|o| o.as_edit_text())
    }

    pub fn set(&self, new: Option<InteractiveObject<'gc>>, context: &mut UpdateContext<'_, 'gc>) {
        self.set_internal(new, context, false);
    }

    fn set_internal(
        &self,
        new: Option<InteractiveObject<'gc>>,
        context: &mut UpdateContext<'_, 'gc>,
        run_actions: bool,
    ) {
        Self::roll_over(context, new);

        if run_actions {
            // The order of events in avm1/tab_ordering_events suggests that
            // FP executes rollOut/rollOver events synchronously when tabbing,
            // but asynchronously when setting focus programmatically.
            Player::run_actions(context);
        }

        let old = self.0.focus.get();

        // Check if the focused element changed.
        if !InteractiveObject::option_ptr_eq(old, new) {
            let focus = unlock!(Gc::write(context.gc(), self.0), FocusTrackerData, focus);
            focus.set(new);

            // The highlight always follows the focus.
            self.update_highlight(context);

            if let Some(old) = old {
                old.set_has_focus(context.gc(), false);
                old.on_focus_changed(context, false, new);
                old.call_focus_handler(context, false, new);
            }
            if let Some(new) = new {
                new.set_has_focus(context.gc(), true);
                new.on_focus_changed(context, true, old);
                new.call_focus_handler(context, true, old);
            }

            tracing::info!("Focus is now on {:?}", new);

            if let Some(level0) = context.stage.root_clip() {
                Avm1::notify_system_listeners(
                    level0,
                    context,
                    "Selection".into(),
                    "onSetFocus".into(),
                    &[
                        old.map(|o| o.as_displayobject())
                            .map(|v| v.object())
                            .unwrap_or(Value::Null),
                        new.map(|o| o.as_displayobject())
                            .map(|v| v.object())
                            .unwrap_or(Value::Null),
                    ],
                );
            }
        }

        // This applies even if the focused element hasn't changed.
        if let Some(text_field) = self.get_as_edit_text() {
            if text_field.is_editable() {
                if !text_field.movie().is_action_script_3() {
                    let length = text_field.text_length();
                    text_field
                        .set_selection(Some(TextSelection::for_range(0, length)), context.gc());
                }
                context.ui.open_virtual_keyboard();
            }
        }
    }

    fn roll_over(context: &mut UpdateContext<'_, 'gc>, new: Option<InteractiveObject<'gc>>) {
        let old = context.mouse_data.hovered;

        // TODO It seems that AVM2 has a slightly different behavior here.
        //   It may be related to the fact that AVM2 handles key and mouse focus differently.
        //   AVM2 is being bypassed here conditionally until
        //   a proper support for AVM2 events is implemented.
        //   See https://github.com/ruffle-rs/ruffle/issues/16789
        if new.is_some_and(|int| int.as_displayobject().movie().is_action_script_3())
            || old.is_some_and(|int| int.as_displayobject().movie().is_action_script_3())
        {
            return;
        }

        context.mouse_data.hovered = new;
        if let Some(old) = old {
            old.handle_clip_event(context, ClipEvent::RollOut { to: new });
        }
        if let Some(new) = new {
            new.handle_clip_event(context, ClipEvent::RollOver { from: old });
        }
    }

    pub fn tab_order(&self, context: &mut UpdateContext<'_, 'gc>) -> Vec<InteractiveObject<'gc>> {
        let stage = context.stage;
        let mut tab_order = vec![];
        stage.fill_tab_order(&mut tab_order, context);

        let custom_ordering = tab_order.iter().any(|o| o.tab_index().is_some());
        if custom_ordering {
            Self::order_custom(&mut tab_order);
        } else {
            Self::order_automatic(&mut tab_order);
        };
        tab_order
    }

    pub fn cycle(&self, context: &mut UpdateContext<'_, 'gc>, reverse: bool) {
        // Ordering the whole array and finding the next object in it
        // is suboptimal, but it's a simple and infrequently performed operation.
        // Additionally, we want to display the whole list in the debug UI anyway,
        // so we do not want to complicate/duplicate logic here if it's unnecessary.
        let tab_order = self.tab_order(context);
        let mut tab_order = if reverse {
            Either::Left(tab_order.iter().rev())
        } else {
            Either::Right(tab_order.iter())
        }
        .peekable();
        let first = tab_order.peek().copied();

        let next = if let Some(current_focus) = self.get() {
            // Find the next object which should take the focus.
            tab_order
                .skip_while(|o| !InteractiveObject::ptr_eq(**o, current_focus))
                .nth(1)
                .or(first)
        } else {
            // If no focus is present, we start from the beginning.
            first
        };

        if next.is_some() {
            self.set_internal(next.copied(), context, true);
            self.update_highlight(context);
        }
    }

    pub fn update_highlight(&self, context: &mut UpdateContext<'_, 'gc>) {
        self.0.highlight.replace(self.calculate_highlight(context));
    }

    fn calculate_highlight(&self, context: &mut UpdateContext<'_, 'gc>) -> Highlight {
        let Some(focus) = self.get() else {
            return Highlight::Inactive;
        };

        if !focus.is_highlightable(context) {
            return Highlight::ActiveHidden;
        }

        Highlight::ActiveVisible
    }

    pub fn render_highlight(&self, context: &mut RenderContext<'_, 'gc>) {
        if !self.highlight().is_visible() {
            return;
        };

        let Some(focus) = self.get() else {
            return;
        };

        let bounds = focus.highlight_bounds();
        context.draw_rect_outline(Self::HIGHLIGHT_COLOR, bounds, Self::HIGHLIGHT_THICKNESS);
    }

    fn order_custom(tab_order: &mut Vec<InteractiveObject>) {
        // Custom ordering disables automatic ordering and
        // ignores all objects without tabIndex.
        tab_order.retain(|o| o.tab_index().is_some());

        // Then, items are sorted according to their tab indices.
        // When two objects have the same index, they are ordered by
        // their respective positions in hierarchy as returned by fill_tab_order().
        tab_order.sort_by_key(|o| o.tab_index());
    }

    /// The automatic ordering depends only on the position of
    /// the top-left highlight bound corner, referred to as `(x,y)`.
    /// It does not depend on object's size or other corners.
    ///
    /// The value of `6y+x` is used to order objects by it.
    /// This means that the next object to be tabbed is the next one
    /// that touches the line `y=-(x-p)/6` (with the smallest `p`).
    ///
    /// When two objects have the same value of `6y+x`
    /// (i.e. when the line touches two objects at the same time),
    /// only one of them is included.
    ///
    /// This behavior is similar to the naive approach of
    /// "left-to-right, top-to-bottom", but (besides being sometimes
    /// seen as random jumps) takes into account the fact that
    /// the next object to the right may be positioned slightly higher.
    /// This is especially true for objects placed by hand or objects with
    /// different heights (as FP uses the top left corner instead of the center).
    ///
    /// This behavior has been discovered experimentally by placing
    /// tabbable objects randomly and bisecting one of their
    /// coordinates to find a difference in behavior.
    ///
    /// See the test `avm2/tab_ordering_automatic_advanced`.
    ///
    /// *WARNING:* Be careful when testing automatic order in FP,
    /// as its behavior is slightly different with a zoom other than 100%.
    fn order_automatic(tab_order: &mut Vec<InteractiveObject>) {
        fn key_extractor(o: &InteractiveObject) -> i64 {
            let bounds = o.highlight_bounds();

            let x = bounds.x_min.get() as i64;
            let y = bounds.y_min.get() as i64;

            y * 6 + x
        }

        tab_order.sort_by_cached_key(key_extractor);

        // Objects with duplicate keys are removed, retaining only
        // the first instance with respect to the order of fill_tab_order().
        // This of course causes some objects to be skipped, even if far from one another,
        // but that's unfortunately how FP behaves.
        tab_order.dedup_by_key(|o| key_extractor(o));
    }
}
