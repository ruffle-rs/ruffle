//! Object representation for regexp

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::regexp::RegExp;
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates RegExp objects.
pub fn regexp_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(class));

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
        let proto = activation.avm2().prototypes().regexp;
        let base = ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(class));

        let mut this: Object<'gc> = RegExpObject(GcCell::allocate(
            activation.context.gc_context,
            RegExpObjectData { base, regexp },
        ))
        .into();
        this.install_instance_traits(activation, class)?;

        class.call_native_init(Some(this), &[], activation, Some(class))?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for RegExpObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some((*self).into()), ScriptObjectClass::NoClass);

        Ok(RegExpObject(GcCell::allocate(
            activation.context.gc_context,
            RegExpObjectData {
                base,
                regexp: RegExp::new(""),
            },
        ))
        .into())
    }

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn value_of(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        let read = self.0.read();
        let mut s = format!("/{}/", read.regexp.source());

        if read.regexp.global() {
            s.push('g');
        }
        if read.regexp.ignore_case() {
            s.push('i');
        }
        if read.regexp.multiline() {
            s.push('m');
        }
        if read.regexp.dotall() {
            s.push('s');
        }
        if read.regexp.extended() {
            s.push('x');
        }

        Ok(AvmString::new(mc, s).into())
    }

    fn as_regexp(&self) -> Option<Ref<RegExp<'gc>>> {
        Some(Ref::map(self.0.read(), |d| &d.regexp))
    }

    fn as_regexp_mut(&self, mc: MutationContext<'gc, '_>) -> Option<RefMut<RegExp<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.regexp))
    }
}
