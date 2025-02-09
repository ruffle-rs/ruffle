//! Object representation for regexp

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::regexp::RegExp;
use crate::avm2::Error;
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

#[derive(Clone, Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct RegExpObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    regexp: RefLock<RegExp<'gc>>,
}

const _: () = assert!(std::mem::offset_of!(RegExpObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<RegExpObjectData>() == std::mem::align_of::<ScriptObjectData>());

impl<'gc> RegExpObject<'gc> {
    pub fn from_regexp(
        activation: &mut Activation<'_, 'gc>,
        regexp: RegExp<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().regexp;
        let base = ScriptObjectData::new(class);

        let this: Object<'gc> = RegExpObject(Gc::new(
            activation.gc(),
            RegExpObjectData {
                base,
                regexp: RefLock::new(regexp),
            },
        ))
        .into();

        class.call_init(this.into(), &[], activation)?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for RegExpObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    /// Unwrap this object as a regexp.
    fn as_regexp_object(&self) -> Option<RegExpObject<'gc>> {
        Some(*self)
    }

    fn as_regexp(&self) -> Option<Ref<RegExp<'gc>>> {
        Some(self.0.regexp.borrow())
    }

    fn as_regexp_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<RegExp<'gc>>> {
        Some(unlock!(Gc::write(mc, self.0), RegExpObjectData, regexp).borrow_mut())
    }
}
