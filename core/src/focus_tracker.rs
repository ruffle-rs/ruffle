use crate::avm1::Avm1;
use crate::avm1::Value;
use crate::context::{RenderContext, UpdateContext};
pub use crate::display_object::{
    DisplayObject, TDisplayObject, TDisplayObjectContainer, TextSelection,
};
use crate::drawing::Drawing;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_render::shape_utils::DrawCommand;
use std::cell::RefCell;
use swf::{Color, LineJoinStyle, Point, Twips};

#[derive(Collect)]
#[collect(no_drop)]
pub struct FocusTrackerData<'gc> {
    focus: Lock<Option<DisplayObject<'gc>>>,
    highlight: RefCell<Highlight>,
}

enum Highlight {
    Inactive,
    Active(Drawing),
}

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct FocusTracker<'gc>(Gc<'gc, FocusTrackerData<'gc>>);

impl<'gc> FocusTracker<'gc> {
    const HIGHLIGHT_WIDTH: Twips = Twips::from_pixels_i32(3);
    const HIGHLIGHT_COLOR: Color = Color::YELLOW;

    // Although at 3px width Round and Miter are similar
    // to each other, it seems that FP uses Round.
    const HIGHLIGHT_LINE_JOIN_STYLE: LineJoinStyle = LineJoinStyle::Round;

    pub fn new(mc: &Mutation<'gc>) -> Self {
        Self(Gc::new(
            mc,
            FocusTrackerData {
                focus: Lock::new(None),
                highlight: RefCell::new(Highlight::Inactive),
            },
        ))
    }

    pub fn reset_highlight(&self) {
        self.0.highlight.replace(Highlight::Inactive);
    }

    pub fn get(&self) -> Option<DisplayObject<'gc>> {
        self.0.focus.get()
    }

    pub fn set(
        &self,
        focused_element: Option<DisplayObject<'gc>>,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
        let old = self.0.focus.get();

        // Check if the focused element changed.
        if old.map(|o| o.as_ptr()) != focused_element.map(|o| o.as_ptr()) {
            let focus = unlock!(Gc::write(context.gc(), self.0), FocusTrackerData, focus);
            focus.set(focused_element);

            // The highlight always follows the focus.
            self.update_highlight();

            if let Some(old) = old {
                old.on_focus_changed(context, false, focused_element);
            }
            if let Some(new) = focused_element {
                new.on_focus_changed(context, true, old);
            }

            tracing::info!("Focus is now on {:?}", focused_element);

            if let Some(level0) = context.stage.root_clip() {
                Avm1::notify_system_listeners(
                    level0,
                    context,
                    "Selection".into(),
                    "onSetFocus".into(),
                    &[
                        old.map(|v| v.object()).unwrap_or(Value::Null),
                        focused_element.map(|v| v.object()).unwrap_or(Value::Null),
                    ],
                );
            }
        }

        // This applies even if the focused element hasn't changed.
        if let Some(text_field) = focused_element.and_then(|e| e.as_edit_text()) {
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

    pub fn cycle(&self, context: &mut UpdateContext<'_, 'gc>) {
        let stage = context.stage;
        let mut tab_order = vec![];
        stage.fill_tab_order(&mut tab_order, context);

        let custom_ordering = tab_order.iter().any(|o| o.tab_index().is_some());
        if custom_ordering {
            // Custom ordering disables automatic ordering and
            // ignores all objects without tabIndex.
            tab_order = tab_order
                .iter()
                .filter(|o| o.tab_index().is_some())
                .copied()
                .collect::<Vec<DisplayObject>>();

            // Then, items are sorted according to their tab indices.
            // TODO When two objects have the same index, the behavior is undefined.
            //      We should analyze and match FP's behavior here if possible.
            tab_order.sort_by_key(|o| o.tab_index());
        }

        let next = if let Some(current_focus) = self.get() {
            // Find the next object which should take the focus.
            tab_order
                .iter()
                .skip_while(|o| o.as_ptr() != current_focus.as_ptr())
                .nth(1)
                .or(tab_order.first())
        } else {
            // If no focus is present, we start from the beginning.
            tab_order.first()
        };

        if next.is_some() {
            self.set(next.copied(), context);
            self.update_highlight();
        }
    }

    fn update_highlight(&self) {
        self.0.highlight.replace(self.redraw_highlight());
    }

    fn redraw_highlight(&self) -> Highlight {
        let Some(focus) = self.get() else {
            return Highlight::Inactive;
        };

        if !focus.is_highlight_enabled() {
            return Highlight::Inactive;
        }

        let bounds = focus.world_bounds().grow(-Self::HIGHLIGHT_WIDTH / 2);
        let mut drawing = Drawing::new();
        drawing.set_line_style(Some(
            swf::LineStyle::new()
                .with_width(Self::HIGHLIGHT_WIDTH)
                .with_color(Self::HIGHLIGHT_COLOR)
                .with_join_style(Self::HIGHLIGHT_LINE_JOIN_STYLE),
        ));
        drawing.draw_command(DrawCommand::MoveTo(Point::new(bounds.x_min, bounds.y_min)));
        drawing.draw_command(DrawCommand::LineTo(Point::new(bounds.x_min, bounds.y_max)));
        drawing.draw_command(DrawCommand::LineTo(Point::new(bounds.x_max, bounds.y_max)));
        drawing.draw_command(DrawCommand::LineTo(Point::new(bounds.x_max, bounds.y_min)));
        drawing.draw_command(DrawCommand::LineTo(Point::new(bounds.x_min, bounds.y_min)));

        Highlight::Active(drawing)
    }

    pub fn render_highlight(&self, context: &mut RenderContext<'_, 'gc>) {
        if let Highlight::Active(ref highlight) = *self.0.highlight.borrow() {
            highlight.render(context);
        };
    }
}
