//! Special object that implements `super`

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::object::script_object::TYPE_OF_OBJECT;
use crate::avm1::object::search_prototype;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::avm_warn;
use crate::display_object::DisplayObject;
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::Cow;

/// Implementation of the `super` object in AS2.
///
/// A `SuperObject` references all data from another object, but with one layer
/// of prototyping removed. It's as if the given object had been constructed
/// with it's parent class.
#[collect(no_drop)]
#[derive(Copy, Clone, Collect, Debug)]
pub struct SuperObject<'gc>(GcCell<'gc, SuperObjectData<'gc>>);

#[collect(no_drop)]
#[derive(Clone, Collect, Debug)]
pub struct SuperObjectData<'gc> {
    /// The object present as `this` throughout the superchain.
    child: Object<'gc>,

    /// The `proto` that the currently-executing method was pulled from.
    base_proto: Object<'gc>,
}

impl<'gc> SuperObject<'gc> {
    /// Construct a `super` for an incoming stack frame.
    ///
    /// `this` and `base_proto` must be the values provided to
    /// `Executable.exec`.
    ///
    /// NOTE: This function must not borrow any `GcCell` data as it is
    /// sometimes called while mutable borrows are held on cells. Specifically,
    /// `Object.call_setter` will panic if this function attempts to borrow
    /// *any* objects.
    pub fn from_this_and_base_proto(
        this: Object<'gc>,
        base_proto: Object<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error<'gc>> {
        Ok(Self(GcCell::allocate(
            activation.context.gc_context,
            SuperObjectData {
                child: this,
                base_proto,
            },
        )))
    }

    /// Retrieve the prototype that `super` should be pulling from.
    fn super_proto(self) -> Option<Object<'gc>> {
        self.0.read().base_proto.proto()
    }

    /// Retrieve the constructor associated with the super proto.
    fn super_constr(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Option<Object<'gc>>, Error<'gc>> {
        if let Some(super_proto) = self.super_proto() {
            Ok(Some(
                super_proto
                    .get("__constructor__", activation)?
                    .coerce_to_object(activation),
            ))
        } else {
            Ok(None)
        }
    }
}

impl<'gc> TObject<'gc> for SuperObject<'gc> {
    fn get_local(
        &self,
        _name: &str,
        _activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Undefined)
    }

    fn set(
        &self,
        _name: &str,
        _value: Value<'gc>,
        _activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        //TODO: What happens if you set `super.__proto__`?
        Ok(())
    }
    fn call(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
        _base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        if let Some(constr) = self.super_constr(activation)? {
            constr.call(
                name,
                activation,
                self.0.read().child,
                self.super_proto(),
                args,
            )
        } else {
            Ok(Value::Undefined)
        }
    }

    fn call_method(
        &self,
        name: &str,
        args: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let child = self.0.read().child;
        let super_proto = self.super_proto();
        let (method, base_proto) = search_prototype(super_proto, name, activation, child)?;
        let method = method;

        if let Value::Object(_) = method {
        } else {
            avm_warn!(activation, "Super method {} is not callable", name);
        }

        method.call(name, activation, child, base_proto, args)
    }

    fn call_setter(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0.read().child.call_setter(name, value, activation)
    }

    #[allow(clippy::new_ret_no_self)]
    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        if let Some(proto) = self.proto() {
            proto.create_bare_object(activation, this)
        } else {
            // TODO: What happens when you `new super` but there's no
            // super? Is this code even reachable?!
            self.0.read().child.create_bare_object(activation, this)
        }
    }

    fn delete(&self, _activation: &mut Activation<'_, 'gc, '_>, _name: &str) -> bool {
        //`super` cannot have properties deleted from it
        false
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.super_proto()
    }

    fn set_proto(&self, gc_context: MutationContext<'gc, '_>, prototype: Option<Object<'gc>>) {
        if let Some(prototype) = prototype {
            self.0.write(gc_context).base_proto = prototype;
        }
    }

    fn define_value(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: &str,
        _value: Value<'gc>,
        _attributes: EnumSet<Attribute>,
    ) {
        //`super` cannot have values defined on it
    }

    fn set_attributes(
        &mut self,
        _gc_context: MutationContext<'gc, '_>,
        _name: Option<&str>,
        _set_attributes: EnumSet<Attribute>,
        _clear_attributes: EnumSet<Attribute>,
    ) {
        //TODO: Does ASSetPropFlags work on `super`? What would it even work on?
    }

    fn add_property(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: &str,
        _get: Object<'gc>,
        _set: Option<Object<'gc>>,
        _attributes: EnumSet<Attribute>,
    ) {
        //`super` cannot have properties defined on it
    }

    fn add_property_with_case(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _gc_context: MutationContext<'gc, '_>,
        _name: &str,
        _get: Object<'gc>,
        _set: Option<Object<'gc>>,
        _attributes: EnumSet<Attribute>,
    ) {
        //`super` cannot have properties defined on it
    }

    fn set_watcher(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _gc_context: MutationContext<'gc, '_>,
        _name: Cow<str>,
        _callback: Object<'gc>,
        _user_data: Value<'gc>,
    ) {
        //`super` cannot have properties defined on it
    }

    fn remove_watcher(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _gc_context: MutationContext<'gc, '_>,
        _name: Cow<str>,
    ) -> bool {
        //`super` cannot have properties defined on it
        false
    }

    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().child.has_property(activation, name)
    }

    fn has_own_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().child.has_own_property(activation, name)
    }

    fn has_own_virtual(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().child.has_own_virtual(activation, name)
    }

    fn is_property_enumerable(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().child.is_property_enumerable(activation, name)
    }

    fn get_keys(&self, _activation: &mut Activation<'_, 'gc, '_>) -> Vec<String> {
        vec![]
    }

    fn as_string(&self) -> Cow<str> {
        Cow::Owned(self.0.read().child.as_string().into_owned())
    }

    fn type_of(&self) -> &'static str {
        TYPE_OF_OBJECT
    }

    fn length(&self) -> usize {
        0
    }

    fn set_length(&self, _gc_context: MutationContext<'gc, '_>, _new_length: usize) {}

    fn array(&self) -> Vec<Value<'gc>> {
        vec![]
    }

    fn array_element(&self, _index: usize) -> Value<'gc> {
        Value::Undefined
    }

    fn set_array_element(
        &self,
        _index: usize,
        _value: Value<'gc>,
        _gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        0
    }

    fn delete_array_element(&self, _index: usize, _gc_context: MutationContext<'gc, '_>) {}

    fn interfaces(&self) -> Vec<Object<'gc>> {
        //`super` does not implement interfaces
        vec![]
    }

    fn set_interfaces(
        &mut self,
        _gc_context: MutationContext<'gc, '_>,
        _iface_list: Vec<Object<'gc>>,
    ) {
        //`super` probably cannot have interfaces set on it
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        None
    }

    fn as_super_object(&self) -> Option<SuperObject<'gc>> {
        Some(*self)
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        //`super` actually can be used to invoke MovieClip methods
        self.0.read().child.as_display_object()
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        //well, `super` *can* be called...
        //...but `super_constr` needs an avm and context in order to get called.
        //ergo, we can't downcast.
        None
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }
}
