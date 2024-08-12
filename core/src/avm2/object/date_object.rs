use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::{Hint, Value};
use crate::avm2::Error;
use chrono::{DateTime, Utc};
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::cell::Cell;

/// A class instance allocator that allocates Date objects.
pub fn date_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(DateObject(Gc::new(
        activation.gc(),
        DateObjectData {
            base: ScriptObjectData::new(class),
            date_time: Cell::new(None),
        },
    ))
    .into())
}
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct DateObject<'gc>(pub Gc<'gc, DateObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct DateObjectWeak<'gc>(pub GcWeak<'gc, DateObjectData<'gc>>);

impl fmt::Debug for DateObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DateObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

impl<'gc> DateObject<'gc> {
    pub fn from_date_time(
        activation: &mut Activation<'_, 'gc>,
        date_time: DateTime<Utc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().date;
        let base = ScriptObjectData::new(class);

        let instance: Object<'gc> = DateObject(Gc::new(
            activation.context.gc_context,
            DateObjectData {
                base,
                date_time: Cell::new(Some(date_time)),
            },
        ))
        .into();
        instance.install_instance_slots(activation.context.gc_context);

        class.call_native_init(instance.into(), &[], activation)?;

        Ok(instance)
    }

    pub fn date_time(self) -> Option<DateTime<Utc>> {
        self.0.date_time.get()
    }

    pub fn set_date_time(self, date_time: Option<DateTime<Utc>>) {
        self.0.date_time.set(date_time);
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct DateObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    date_time: Cell<Option<DateTime<Utc>>>,
}

const _: () = assert!(std::mem::offset_of!(DateObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<DateObjectData>() == std::mem::align_of::<ScriptObjectData>());

impl<'gc> TObject<'gc> for DateObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        if let Some(date) = self.date_time() {
            Ok((date.timestamp_millis() as f64).into())
        } else {
            Ok(f64::NAN.into())
        }
    }

    fn default_hint(&self) -> Hint {
        Hint::String
    }

    fn as_date_object(&self) -> Option<DateObject<'gc>> {
        Some(*self)
    }
}
