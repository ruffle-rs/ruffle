use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;

/// A class instance allocator that allocates Collator objects.
pub fn collator_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(CollatorObject(Gc::new(
        activation.gc(),
        CollatorObjectData {
            base,
            requested_locale_id_name: Lock::new(None),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct CollatorObject<'gc>(pub Gc<'gc, CollatorObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct CollatorObjectWeak<'gc>(pub GcWeak<'gc, CollatorObjectData<'gc>>);

impl fmt::Debug for CollatorObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollatorObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct CollatorObjectData<'gc> {
    /// Base script object.
    base: ScriptObjectData<'gc>,

    requested_locale_id_name: Lock<Option<AvmString<'gc>>>,
}

impl<'gc> CollatorObject<'gc> {
    pub fn requested_locale_id_name(self) -> Option<AvmString<'gc>> {
        self.0.requested_locale_id_name.get()
    }

    pub fn set_requested_locale_id_name(self, value: Option<AvmString<'gc>>, mc: &Mutation<'gc>) {
        unlock!(
            Gc::write(mc, self.0),
            CollatorObjectData,
            requested_locale_id_name
        )
        .set(value);
    }
}

impl<'gc> TObject<'gc> for CollatorObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
