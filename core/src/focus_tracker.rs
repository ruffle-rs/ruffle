use crate::avm1::Avm1;
use crate::avm1::Value;
use crate::context::UpdateContext;
pub use crate::display_object::{
    DisplayObject, TDisplayObject, TDisplayObjectContainer, TextSelection,
};
use gc_arena::lock::GcLock;
use gc_arena::{Collect, Mutation};

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct FocusTracker<'gc>(GcLock<'gc, Option<DisplayObject<'gc>>>);

impl<'gc> FocusTracker<'gc> {
    pub fn new(mc: &Mutation<'gc>) -> Self {
        Self(GcLock::new(mc, None.into()))
    }

    pub fn get(&self) -> Option<DisplayObject<'gc>> {
        self.0.get()
    }

    pub fn set(
        &self,
        focused_element: Option<DisplayObject<'gc>>,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
        let old = self.0.get();

        // Check if the focused element changed.
        if old.map(|o| o.as_ptr()) != focused_element.map(|o| o.as_ptr()) {
            self.0.set(context.gc(), focused_element);

            if let Some(old) = old {
                old.on_focus_changed(context.gc(), false);
            }
            if let Some(new) = focused_element {
                new.on_focus_changed(context.gc(), true);
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
}
