use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::timer::Timer;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;

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
            timer: Lock::new(None),
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
            .field("id", &self.0.timer.get().and_then(|t| t.id()))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TimerObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The underlying Ruffle timer, present while the timer is running.
    timer: Lock<Option<Timer<'gc>>>,
}

impl<'gc> TimerObject<'gc> {
    pub fn timer(self) -> Option<Timer<'gc>> {
        self.0.timer.get()
    }

    pub fn set_timer(self, timer: Option<Timer<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TimerObjectData, timer).set(timer);
    }
}

impl<'gc> TObject<'gc> for TimerObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
