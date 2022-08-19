use crate::avm1::Avm1;
use crate::avm1::Value;
use crate::context::UpdateContext;
pub use crate::display_object::{DisplayObject, TDisplayObject, TDisplayObjectContainer};
use gc_arena::{Collect, GcCell, MutationContext};

#[derive(Clone, Copy, Collect, Debug)]
#[collect(no_drop)]
pub struct FocusTracker<'gc>(GcCell<'gc, Option<DisplayObject<'gc>>>);

impl<'gc> FocusTracker<'gc> {
    pub fn new(gc_context: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(gc_context, None))
    }

    pub fn get(&self) -> Option<DisplayObject<'gc>> {
        *self.0.read()
    }

    pub fn set(
        &self,
        focused_element: Option<DisplayObject<'gc>>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        let old = std::mem::replace(&mut *self.0.write(context.gc_context), focused_element);

        if old.is_none() && focused_element.is_none() {
            // We didn't have anything, we still don't, no change.
            return;
        }
        if old.is_some() == focused_element.is_some()
            && old.unwrap().as_ptr() == focused_element.unwrap().as_ptr()
        {
            // We're setting it to the same object as before, no change.
            return;
        }

        if let Some(old) = old {
            old.on_focus_changed(context.gc_context, false);
        }
        if let Some(new) = focused_element {
            new.on_focus_changed(context.gc_context, true);
        }

        log::info!("Focus is now on {:?}", focused_element);

        let level0 = context.stage.root_clip();
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
