use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
use std::cell::Cell;

/// A class instance allocator that allocates Timer objects.
pub fn timer_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(TimerObject(Gc::new(
        activation.gc(),
        TimerObjectData {
            base,
            timer_id: Cell::new(None),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TimerObject<'gc>(pub Gc<'gc, TimerObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct TimerObjectWeak<'gc>(pub GcWeak<'gc, TimerObjectData<'gc>>);

impl fmt::Debug for TimerObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimerObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .field("id", &self.0.timer_id.get())
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TimerObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The ID of the underlying Ruffle timer, present while the timer is running.
    timer_id: Cell<Option<i32>>,
}

impl TimerObject<'_> {
    pub fn timer_id(self) -> Option<i32> {
        self.0.timer_id.get()
    }

    pub fn set_timer_id(self, timer_id: Option<i32>) {
        self.0.timer_id.set(timer_id);
    }
}

impl<'gc> TObject<'gc> for TimerObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
