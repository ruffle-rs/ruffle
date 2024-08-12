use crate::avm1::Avm1;
use crate::avm1::Value;
use crate::avm2::{Activation, Avm2, EventObject, TObject};
use crate::context::{RenderContext, UpdateContext};
pub use crate::display_object::{
    DisplayObject, TDisplayObject, TDisplayObjectContainer, TextSelection,
};
use crate::display_object::{EditText, InteractiveObject, TInteractiveObject};
use crate::events::{ClipEvent, KeyCode};
use crate::prelude::Avm2Value;
use crate::Player;
use either::Either;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use std::cell::RefCell;
use std::slice::Iter;
use swf::{Color, Rectangle, Twips};

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

    /// Set the focus programmatically.
    pub fn set(&self, new: Option<InteractiveObject<'gc>>, context: &mut UpdateContext<'gc>) {
        self.set_internal(new, context, false);
    }

    /// Reset the focus programmatically.
    pub fn reset_focus(&self, context: &mut UpdateContext<'gc>) {
        self.set_internal(None, context, true);
    }

    /// Set the focus and acknowledge that this change was caused by a pointer device.
    pub fn set_by_mouse(
        &self,
        new: Option<InteractiveObject<'gc>>,
        context: &mut UpdateContext<'gc>,
    ) {
        let old = self.0.focus.get();

        // Mouse focus change events are not dispatched when the object is the same,
        // contrary to key focus change events.
        if InteractiveObject::option_ptr_eq(old, new) {
            // Re-open the keyboard when the user clicked an already focused text field.
            self.update_virtual_keyboard(context);
            return;
        }

        if Self::dispatch_focus_change_event(context, "mouseFocusChange", old, new, None) {
            return;
        }

        // When clicking an object that is not focusable by mouse,
        // the real object will be used to dispatch focus change events,
        // but `None` will be used when setting the focus.
        let new = new.filter(|new| new.is_focusable_by_mouse(context));

        self.set_internal(new, context, false);
    }

    /// Set the focus and acknowledge that this change was caused by a key.
    pub fn set_by_key(
        &self,
        new: Option<InteractiveObject<'gc>>,
        key_code: KeyCode,
        context: &mut UpdateContext<'gc>,
    ) {
        let old = self.0.focus.get();
        if Self::dispatch_focus_change_event(context, "keyFocusChange", old, new, Some(key_code)) {
            return;
        }

        self.set_internal(new, context, true);
    }

    fn set_internal(
        &self,
        new: Option<InteractiveObject<'gc>>,
        context: &mut UpdateContext<'gc>,
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
            if text_field.is_editable() && !text_field.movie().is_action_script_3() {
                // TODO This logic is inaccurate and addresses
                //   only setting the focus programmatically.
                let length = text_field.text_length();
                text_field.set_selection(Some(TextSelection::for_range(0, length)), context.gc());
            }
        }

        self.update_virtual_keyboard(context);
    }

    fn update_virtual_keyboard(&self, context: &mut UpdateContext<'gc>) {
        if let Some(text_field) = self.get_as_edit_text() {
            if text_field.is_editable() {
                context.ui.open_virtual_keyboard();
            } else {
                context.ui.close_virtual_keyboard();
            }
        } else {
            context.ui.close_virtual_keyboard();
        }
    }

    /// Dispatches the AVM2's focus change event.
    ///
    /// Returns `true` if the focus change operation should be canceled.
    fn dispatch_focus_change_event(
        context: &mut UpdateContext<'gc>,
        event_type: &'static str,
        target: Option<InteractiveObject<'gc>>,
        related_object: Option<InteractiveObject<'gc>>,
        key_code: Option<KeyCode>,
    ) -> bool {
        let target = target
            .map(|int| int.as_displayobject())
            .unwrap_or_else(|| context.stage.as_displayobject())
            .object2();
        let Avm2Value::Object(target) = target else {
            return false;
        };

        let mut activation = Activation::from_nothing(context);
        let key_code = key_code.map(|k| k as u8).unwrap_or_default();
        let event =
            EventObject::focus_event(&mut activation, event_type, true, related_object, key_code);
        Avm2::dispatch_event(activation.context, event, target);

        let canceled = event.as_event().unwrap().is_cancelled();
        canceled
    }

    fn roll_over(context: &mut UpdateContext<'gc>, new: Option<InteractiveObject<'gc>>) {
        let old = context.mouse_data.hovered;

        // AVM2 does not dispatch roll out/over events here and does not update hovered object.
        // TODO Analyze how this should behave in mixed AVM content.
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

    pub fn tab_order(&self, context: &mut UpdateContext<'gc>) -> TabOrder<'gc> {
        let mut tab_order = TabOrder::fill(context);
        tab_order.sort();
        tab_order
    }

    pub fn cycle(&self, context: &mut UpdateContext<'gc>, reverse: bool) {
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
            self.set_by_key(next.copied(), KeyCode::Tab, context);
            self.update_highlight(context);
        }
    }

    pub fn navigate(&self, context: &mut UpdateContext<'gc>, direction: NavigationDirection) {
        let Some(focus) = self.get() else {
            return;
        };

        let tab_order = TabOrder::fill(context);
        let ordering = NavigationOrdering::new(focus, direction);
        if let Some(next) = tab_order.first(ordering) {
            self.set_by_key(Some(next), direction.key(), context);
        }
    }

    pub fn update_highlight(&self, context: &mut UpdateContext<'gc>) {
        self.0.highlight.replace(self.calculate_highlight(context));
    }

    fn calculate_highlight(&self, context: &mut UpdateContext<'gc>) -> Highlight {
        let Some(focus) = self.get() else {
            return Highlight::Inactive;
        };

        if !focus.is_highlightable(context) {
            return Highlight::ActiveHidden;
        }

        // KJ: Flash Player has a fairly complicated, implementation-dependent behavior
        // related to degenerate bounds and bounds that change after being highlighted.
        // It seems that the highlight is hidden when there's nothing to be rendered:
        //   1. For a clip with untouched graphics, the highlight is hidden.
        //   2. For a clip with moveTo(0,0) lineTo(0,0), the highlight is not hidden.
        // Sometimes highlight is not updated after highlight bounds change,
        // but this behavior is rare and inconsistent, it even differs depending on
        // when the update happens.
        //
        // However, the most common case of depending on the behavior above is when
        // the SWF creates a movie clip, focuses it, and then adds some content to it.
        // In that case FP will not render the highlight (even if in theory it should).
        // The following condition covers this case, but:
        //   1. In some rare cases, it will hide the highlight when it shouldn't
        //      (e.g. non-empty graphics degenerated into a point).
        //   2. It does not take into account that sometimes FP does not update highlight bounds.
        // However, I've never seen these cases in real SWFs, only during testing.
        let bounds = focus.highlight_bounds();
        if !bounds.is_valid() || bounds.is_point() {
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
}

/// A list of interactive objects ordered
/// according to a specific tab order.
pub struct TabOrder<'gc> {
    objects: Vec<InteractiveObject<'gc>>,

    /// When any object has tab index set, objects without
    /// tab indices are filtered out and this value is `true`.
    is_custom: bool,
}

impl<'gc> TabOrder<'gc> {
    fn empty() -> Self {
        Self {
            objects: Vec::new(),
            is_custom: false,
        }
    }

    fn fill(context: &mut UpdateContext<'gc>) -> Self {
        let stage = context.stage;
        let mut tab_order = Self::empty();
        stage.fill_tab_order(&mut tab_order, context);
        tab_order
    }

    pub fn is_custom(&self) -> bool {
        self.is_custom
    }

    pub fn iter(&self) -> Iter<'_, InteractiveObject<'gc>> {
        self.objects.iter()
    }

    pub fn add_object(&mut self, object: InteractiveObject<'gc>) {
        let has_tab_index = object.tab_index().is_some();
        if has_tab_index && !self.is_custom {
            // If an object has tab index, we have to switch to a custom order,
            // and retain only objects with tab index, even for keyboard navigation.
            self.is_custom = true;
            self.objects.retain(|&o| o.tab_index().is_some());
        }

        if has_tab_index || !self.is_custom {
            self.objects.push(object);
        }
    }

    fn sort(&mut self) {
        if self.is_custom() {
            self.sort_with(CustomTabOrdering);
        } else {
            self.sort_with(AutomaticTabOrdering);
        }
    }

    fn sort_with(&mut self, ordering: impl TabOrdering) {
        self.objects.sort_by_cached_key(|&o| ordering.key(o));

        let to_skip = self
            .objects
            .iter()
            .take_while(|&&o| ordering.key(o).is_none())
            .count();
        self.objects.drain(..to_skip);

        if ordering.ignore_duplicates() {
            self.objects.dedup_by_key(|&mut o| ordering.key(o));
        }
    }

    fn first(&self, ordering: impl TabOrdering) -> Option<InteractiveObject<'gc>> {
        self.objects
            .iter()
            .filter(|&&object| ordering.key(object).is_some())
            .min_by_key(|&&object| ordering.key(object))
            .cloned()
    }
}

trait TabOrdering {
    fn key(&self, object: InteractiveObject) -> Option<impl Ord + Copy>;

    fn ignore_duplicates(&self) -> bool;
}

/// In custom ordering, items are sorted according to their tab indices.
/// When two objects have the same index, they are ordered by
/// their respective positions in hierarchy
struct CustomTabOrdering;

impl TabOrdering for CustomTabOrdering {
    fn key(&self, object: InteractiveObject) -> Option<impl Ord + Copy> {
        object.tab_index()
    }

    fn ignore_duplicates(&self) -> bool {
        false
    }
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
struct AutomaticTabOrdering;

impl TabOrdering for AutomaticTabOrdering {
    fn key(&self, object: InteractiveObject) -> Option<impl Ord + Copy> {
        let bounds = object.highlight_bounds();

        let x = bounds.x_min.get() as i64;
        let y = bounds.y_min.get() as i64;

        Some(y * 6 + x)
    }

    fn ignore_duplicates(&self) -> bool {
        // Objects with duplicate keys are removed, retaining only
        // the first instance with respect to the order of fill_tab_order().
        // This of course causes some objects to be skipped, even if far from one another,
        // but that's unfortunately how FP behaves.
        true
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NavigationDirection {
    Up,
    Right,
    Down,
    Left,
}

impl NavigationDirection {
    pub fn from_key_code(key_code: KeyCode) -> Option<Self> {
        Some(match key_code {
            KeyCode::Up => Self::Up,
            KeyCode::Right => Self::Right,
            KeyCode::Down => Self::Down,
            KeyCode::Left => Self::Left,
            _ => return None,
        })
    }

    fn key(self) -> KeyCode {
        match self {
            Self::Up => KeyCode::Up,
            Self::Right => KeyCode::Right,
            Self::Down => KeyCode::Down,
            Self::Left => KeyCode::Left,
        }
    }
}

/// Ordering used for keyboard navigation.
struct NavigationOrdering {
    /// Bounds of the object we are navigating from.
    origin_bounds: Rectangle<Twips>,

    /// The direction which we are navigating towards.
    direction: NavigationDirection,
}

/// When ordering objects for navigation, they are divided into two main categories:
///  1. objects directly behind the origin (taking into account the direction),
///  2. other objects.
///
/// Objects from category 1 always take precedence over objects from category 2.
///
/// Objects from category 1 are ordered according to their horizontal/vertical
/// distance to the origin, and when two objects have the same distance,
/// the default ordering based on the hierarchy is used.
///
/// Objects from category 2 are ordered according to their 2D distance to the origin.
/// The distance is calculated between the closest corners of highlight bounds, and when
/// two objects have the same distance, the default ordering based on the hierarchy is used.
///
/// In case of navigating down, there's an additional category (between 1 and 2)
/// which contains objects directly to the right/left of the origin.
/// In that category, objects are ordered according to their x-axis distance towards origin.
///
/// Note: the following implementation is still a little off. Its general idea is sound,
/// but there are situations where the exact ordering in category 2 is inaccurate.
/// It seems that FP's behavior in that case depends on things other than highlight bounds,
/// (e.g. past iteration order), however this inaccuracy is not very important as keyboard
/// navigation is performed by the user, and they may always navigate in some other way.
impl NavigationOrdering {
    fn new(origin: InteractiveObject, direction: NavigationDirection) -> Self {
        Self {
            origin_bounds: origin.highlight_bounds(),
            direction,
        }
    }
}

impl TabOrdering for NavigationOrdering {
    fn key(&self, other: InteractiveObject) -> Option<impl Ord + Copy> {
        let origin = &self.origin_bounds;
        let other = &other.highlight_bounds();

        /// Calculate x- or y-axis distance between two rectangles.
        fn calculate_distance(a: &Rectangle<Twips>, b: &Rectangle<Twips>, vertical: bool) -> i64 {
            if vertical {
                (a.y_max - b.y_min).max(b.y_max - a.y_min).get() as i64
            } else {
                (a.x_max - b.x_min).max(b.x_max - a.x_min).get() as i64
            }
        }

        // Note that these variants are very similar, but they do have differences!
        match self.direction {
            NavigationDirection::Down => {
                if other.y_max <= origin.y_max {
                    return None;
                }
                let is_behind = other.x_max >= origin.x_min && other.x_min <= origin.x_max;
                if is_behind {
                    return Some((0, (other.y_min - origin.y_min).get() as i64));
                }
                if other.y_min <= origin.y_max {
                    // Down is the only direction where this rule applies:
                    // if an object is to the right or to the left of origin,
                    // it has precedence here over objects which are not,
                    // but not objects behind the origin.
                    return Some((1, calculate_distance(origin, other, false)));
                }
            }
            NavigationDirection::Up => {
                if other.y_max >= origin.y_max {
                    return None;
                }
                let is_behind = other.x_max >= origin.x_min && other.x_min <= origin.x_max;
                if is_behind {
                    return Some((0, (origin.y_max - other.y_max).get() as i64));
                }
            }
            NavigationDirection::Right => {
                if other.x_max <= origin.x_max {
                    return None;
                }
                let is_behind = other.y_max >= origin.y_min && other.y_min <= origin.y_max;
                if is_behind {
                    return Some((0, (other.x_min - origin.x_min).get() as i64));
                }
            }
            NavigationDirection::Left => {
                if other.x_min >= origin.x_min {
                    return None;
                }
                let is_behind = other.y_max >= origin.y_min && other.y_min <= origin.y_max;
                if is_behind {
                    return Some((0, (origin.x_max - other.x_max).get() as i64));
                }
            }
        }

        let distance_x = calculate_distance(origin, other, false);
        let distance_y = calculate_distance(origin, other, true);

        Some((2, distance_x * distance_x + distance_y * distance_y))
    }

    fn ignore_duplicates(&self) -> bool {
        false
    }
}
