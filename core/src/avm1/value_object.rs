//! Object impl for boxed values

use crate::avm1::function::Executable;
use crate::avm1::globals::SystemPrototypes;
use crate::avm1::object::{ObjectPtr, TObject};
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, UpdateContext, Value};
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashSet;
use std::fmt;

/// An Object that serves as a box for a primitive value.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ValueObject<'gc>(GcCell<'gc, ValueObjectData<'gc>>);

/// The internal data for a boxed value.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ValueObjectData<'gc> {
    /// Base implementation of ScriptObject.
    base: ScriptObject<'gc>,

    /// The value being boxed.
    ///
    /// It is a logic error for this to be another object. All extant
    /// constructors for `ValueObject` guard against this by returning the
    /// original object if an attempt is made to box objects.
    value: Value<'gc>,
}

impl<'gc> ValueObject<'gc> {
    /// Box a value into a `ValueObject`.
    ///
    /// If this function is given an object to box, then this function returns
    /// the already-defined object.
    ///
    /// If a class exists for a given value type, this function automatically
    /// selects the correct prototype for it from the system prototypes list.
    pub fn boxed(
        gc_context: MutationContext<'gc, '_>,
        value: Value<'gc>,
        system_prototypes: &SystemPrototypes<'gc>,
    ) -> Object<'gc> {
        if let Value::Object(ob) = value {
            ob
        } else {
            let proto = match value {
                Value::String(_) => Some(system_prototypes.string),
                _ => None,
            };

            ValueObject(GcCell::allocate(
                gc_context,
                ValueObjectData {
                    base: ScriptObject::object(gc_context, proto),
                    value,
                },
            ))
            .into()
        }
    }

    /// Construct an empty box to be filled by a constructor.
    pub fn empty_box(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        ValueObject(GcCell::allocate(
            gc_context,
            ValueObjectData {
                base: ScriptObject::object(gc_context, proto),
                value: Value::Undefined,
            },
        ))
        .into()
    }

    /// Retrieve the boxed value.
    pub fn unbox(self) -> Value<'gc> {
        self.0.read().value.clone()
    }

    /// Change the value in the box.
    pub fn replace_value(&mut self, gc_context: MutationContext<'gc, '_>, value: Value<'gc>) {
        self.0.write(gc_context).value = value;
    }
}

impl fmt::Debug for ValueObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("ValueObject")
            .field("base", &this.base)
            .field("value", &this.value)
            .finish()
    }
}

impl<'gc> TObject<'gc> for ValueObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().base.get_local(name, avm, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.0.read().base.set(name, value, avm, context)
    }

    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().base.call(avm, context, this, args)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        _avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        Ok(ValueObject::empty_box(context.gc_context, Some(this)))
    }

    fn delete(&self, gc_context: MutationContext<'gc, '_>, name: &str) -> bool {
        self.0.read().base.delete(gc_context, name)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0
            .read()
            .base
            .add_property(gc_context, name, get, set, attributes)
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0
            .read()
            .base
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: EnumSet<Attribute>,
        clear_attributes: EnumSet<Attribute>,
    ) {
        self.0.write(gc_context).base.set_attributes(
            gc_context,
            name,
            set_attributes,
            clear_attributes,
        )
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().base.proto()
    }

    fn has_property(&self, name: &str) -> bool {
        self.0.read().base.has_property(name)
    }

    fn has_own_property(&self, name: &str) -> bool {
        self.0.read().base.has_own_property(name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.0.read().base.is_property_overwritable(name)
    }

    fn is_property_enumerable(&self, name: &str) -> bool {
        self.0.read().base.is_property_enumerable(name)
    }

    fn get_keys(&self) -> HashSet<String> {
        self.0.read().base.get_keys()
    }

    fn as_string(&self) -> String {
        self.0.read().base.as_string()
    }

    fn type_of(&self) -> &'static str {
        self.0.read().base.type_of()
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.0.read().base.interfaces()
    }

    fn set_interfaces(&mut self, context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.0
            .write(context)
            .base
            .set_interfaces(context, iface_list)
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.0.read().base)
    }

    fn as_value_object(&self) -> Option<ValueObject<'gc>> {
        Some(*self)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.read().base.as_ptr() as *const ObjectPtr
    }

    fn length(&self) -> usize {
        self.0.read().base.length()
    }

    fn array(&self) -> Vec<Value<'gc>> {
        self.0.read().base.array()
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, length: usize) {
        self.0.read().base.set_length(gc_context, length)
    }

    fn array_element(&self, index: usize) -> Value<'gc> {
        self.0.read().base.array_element(index)
    }

    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        self.0
            .read()
            .base
            .set_array_element(index, value, gc_context)
    }

    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>) {
        self.0.read().base.delete_array_element(index, gc_context)
    }
}
