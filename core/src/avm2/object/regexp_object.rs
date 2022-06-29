//! Object representation for regexp

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::regexp::{RegExp, RegExpFlags};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::{AvmString, WString};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates RegExp objects.
pub fn regexp_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::new(class);

    Ok(RegExpObject(GcCell::allocate(
        activation.context.gc_context,
        RegExpObjectData {
            base,
            regexp: RegExp::new(""),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct RegExpObject<'gc>(GcCell<'gc, RegExpObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct RegExpObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    regexp: RegExp<'gc>,
}

impl<'gc> RegExpObject<'gc> {
    pub fn from_regexp(
        activation: &mut Activation<'_, 'gc, '_>,
        regexp: RegExp<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().regexp;
        let base = ScriptObjectData::new(class);

        let mut this: Object<'gc> = RegExpObject(GcCell::allocate(
            activation.context.gc_context,
            RegExpObjectData { base, regexp },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for RegExpObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn value_of(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
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

    fn as_regexp_mut(&self, mc: MutationContext<'gc, '_>) -> Option<RefMut<RegExp<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.regexp))
    }
}
