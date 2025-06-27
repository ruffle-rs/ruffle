//! Object representation for regexp

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::regexp::RegExp;
use crate::avm2::Error;
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::RefLock, Collect, Gc, GcWeak, Mutation};
use ruffle_macros::istr;
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates RegExp objects.
pub fn reg_exp_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(RegExpObject(Gc::new(
        activation.gc(),
        RegExpObjectData {
            base,
            regexp: RefLock::new(RegExp::new(istr!(""))),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct RegExpObject<'gc>(pub Gc<'gc, RegExpObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct RegExpObjectWeak<'gc>(pub GcWeak<'gc, RegExpObjectData<'gc>>);

impl fmt::Debug for RegExpObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegExpObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct RegExpObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    regexp: RefLock<RegExp<'gc>>,
}

impl<'gc> RegExpObject<'gc> {
    pub fn regexp(self) -> Ref<'gc, RegExp<'gc>> {
        Gc::as_ref(self.0).regexp.borrow()
    }

    pub fn regexp_mut(self, mc: &Mutation<'gc>) -> RefMut<'gc, RegExp<'gc>> {
        unlock!(Gc::write(mc, self.0), RegExpObjectData, regexp).borrow_mut()
    }
}

impl<'gc> TObject<'gc> for RegExpObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
