//! Object representation for regexp

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::regexp::{RegExp, RegExpFlags};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::{AvmString, WString};
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates RegExp objects.
pub fn reg_exp_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(RegExpObject(GcCell::new(
        activation.context.gc_context,
        RegExpObjectData {
            base,
            regexp: RegExp::new(""),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct RegExpObject<'gc>(pub GcCell<'gc, RegExpObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct RegExpObjectWeak<'gc>(pub GcWeakCell<'gc, RegExpObjectData<'gc>>);

impl fmt::Debug for RegExpObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegExpObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct RegExpObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    regexp: RegExp<'gc>,
}

impl<'gc> RegExpObject<'gc> {
    pub fn from_regexp(
        activation: &mut Activation<'_, 'gc>,
        regexp: RegExp<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().regexp;
        let base = ScriptObjectData::new(class);

        let this: Object<'gc> = RegExpObject(GcCell::new(
            activation.context.gc_context,
            RegExpObjectData { base, regexp },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        class.call_native_init(this.into(), &[], activation)?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for RegExpObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        let read = self.0.read();
        let mut s = WString::new();
        s.push_byte(b'/');
        s.push_str(&read.regexp.source());
        s.push_byte(b'/');

        let flags = read.regexp.flags();

        if flags.contains(RegExpFlags::GLOBAL) {
            s.push_byte(b'g');
        }
        if flags.contains(RegExpFlags::IGNORE_CASE) {
            s.push_byte(b'i');
        }
        if flags.contains(RegExpFlags::MULTILINE) {
            s.push_byte(b'm');
        }
        if flags.contains(RegExpFlags::DOTALL) {
            s.push_byte(b's');
        }
        if flags.contains(RegExpFlags::EXTENDED) {
            s.push_byte(b'x');
        }

        Ok(AvmString::new(mc, s).into())
    }

    /// Unwrap this object as a regexp.
    fn as_regexp_object(&self) -> Option<RegExpObject<'gc>> {
        Some(*self)
    }

    fn as_regexp(&self) -> Option<Ref<RegExp<'gc>>> {
        Some(Ref::map(self.0.read(), |d| &d.regexp))
    }

    fn as_regexp_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<RegExp<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.regexp))
    }
}
