use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::{Hint, Value};
use crate::avm2::Error;
use chrono::{DateTime, Utc};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Date objects.
pub fn date_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(DateObject(GcCell::allocate(
        activation.context.gc_context,
        DateObjectData {
            base,
            date_time: None,
        },
    ))
    .into())
}
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct DateObject<'gc>(GcCell<'gc, DateObjectData<'gc>>);

impl<'gc> DateObject<'gc> {
    pub fn date_time(self) -> Option<DateTime<Utc>> {
        self.0.read().date_time
    }

    pub fn set_date_time(
        self,
        gc_context: MutationContext<'gc, '_>,
        date_time: Option<DateTime<Utc>>,
    ) {
        self.0.write(gc_context).date_time = date_time;
    }
}

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct DateObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    #[collect(require_static)]
    date_time: Option<DateTime<Utc>>,
}

impl<'gc> TObject<'gc> for DateObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::DateObject(*self);
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(DateObject(GcCell::allocate(
            activation.context.gc_context,
            DateObjectData {
                base,
                date_time: None,
            },
        ))
        .into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
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
