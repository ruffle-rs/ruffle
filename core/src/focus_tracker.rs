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
        // TODO When two objects have the same index, the behavior is undefined.
        //      We should analyze and match FP's behavior here if possible.
        tab_order.sort_by_key(|o| o.tab_index());
    }

    // TODO This ordering is yet far from being perfect.
    //      FP actually has some weird ordering logic, which
    //      sometimes jumps up, sometimes even ignores some objects.
    fn order_automatic(tab_order: &mut Vec<InteractiveObject>) {
        fn key_extractor(o: &InteractiveObject) -> (Twips, Twips) {
            let bounds = o.highlight_bounds();
            (bounds.y_min, bounds.x_min)
        }

        // The automatic order is mainly dependent on
        // the position of the top-left bound corner.
        tab_order.sort_by_cached_key(key_extractor);

        // Duplicated positions are removed,
        // only the first element is retained.
        tab_order.dedup_by_key(|o| key_extractor(o));
    }
}
