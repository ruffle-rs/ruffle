//! Object representation for regexp

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::regexp::RegExp;
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance deriver that constructs RegExp objects.
pub fn regexp_deriver<'gc>(
    mut constr: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    let base_proto = constr
        .get_property(
            constr,
            &QName::new(Namespace::public(), "prototype"),
            activation,
        )?
        .coerce_to_object(activation)?;

    Ok(RegExpObject::derive(
        base_proto,
        activation.context.gc_context,
        class,
        scope,
    ))
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
        mc: MutationContext<'gc, '_>,
        base_proto: Option<Object<'gc>>,
        regexp: RegExp<'gc>,
    ) -> Object<'gc> {
        let base = ScriptObjectData::base_new(base_proto, ScriptObjectClass::NoClass);

        RegExpObject(GcCell::allocate(mc, RegExpObjectData { base, regexp })).into()
    }

    /// Instantiate a regexp subclass.
    pub fn derive(
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Object<'gc> {
        let base = ScriptObjectData::base_new(
            Some(base_proto),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        RegExpObject(GcCell::allocate(
            mc,
            RegExpObjectData {
                base,
                regexp: RegExp::new(""),
            },
        ))
        .into()
    }
}

impl<'gc> TObject<'gc> for RegExpObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::RegExpObject(*self);
        Ok(RegExpObject::from_regexp(
            activation.context.gc_context,
            Some(this),
            RegExp::new(""),
        ))
    }

    fn derive(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::RegExpObject(*self);
        Ok(Self::derive(
            this,
            activation.context.gc_context,
            class,
            scope,
        ))
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
