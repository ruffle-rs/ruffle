use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::value::Hint;
use crate::avm2::Error;
use crate::utils::HasPrefixField;
use chrono::{DateTime, Utc};
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
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
    ) -> Object<'gc> {
        let class = activation.avm2().classes().date;
        let base = ScriptObjectData::new(class);

        DateObject(Gc::new(
            activation.gc(),
            DateObjectData {
                base,
                date_time: Cell::new(Some(date_time)),
            },
        ))
        .into()
    }

    pub fn for_prototype(
        activation: &mut Activation<'_, 'gc>,
        date_class: ClassObject<'gc>,
    ) -> Object<'gc> {
        let object_class = activation.avm2().classes().object;
        let base = ScriptObjectData::custom_new(
            date_class.inner_class_definition(),
            Some(object_class.prototype()),
            date_class.instance_vtable(),
        );

        let instance: Object<'gc> = DateObject(Gc::new(
            activation.gc(),
            DateObjectData {
                base,
                date_time: Cell::new(None),
            },
        ))
        .into();

        instance
    }

    pub fn date_time(self) -> Option<DateTime<Utc>> {
        self.0.date_time.get()
    }

    pub fn set_date_time(self, date_time: Option<DateTime<Utc>>) {
        self.0.date_time.set(date_time);
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct DateObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    date_time: Cell<Option<DateTime<Utc>>>,
}

impl<'gc> TObject<'gc> for DateObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn default_hint(&self) -> Hint {
        Hint::String
    }
}
