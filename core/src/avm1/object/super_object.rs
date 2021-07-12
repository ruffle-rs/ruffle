//! Special object that implements `super`

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::script_object::TYPE_OF_OBJECT;
use crate::avm1::object::search_prototype;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::avm_warn;
use crate::display_object::DisplayObject;
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::Cow;

/// Implementation of the `super` object in AS2.
///
/// A `SuperObject` references all data from another object, but with one layer
/// of prototyping removed. It's as if the given object had been constructed
/// with its parent class.
#[derive(Copy, Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct SuperObject<'gc>(GcCell<'gc, SuperObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct SuperObjectData<'gc> {
    /// The object present as `this` throughout the superchain.
    this: Object<'gc>,

    depth: u8,
}

impl<'gc> SuperObject<'gc> {
    /// Construct a `super` for an incoming stack frame.
    ///
    /// `this` and `depth` must be the values provided to `Executable.exec`.
    ///
    /// NOTE: This function must not borrow any `GcCell` data as it is
    /// sometimes called while mutable borrows are held on cells. Specifically,
    /// `Object.call_setter` will panic if this function attempts to borrow
    /// *any* objects.
    pub fn new(activation: &mut Activation<'_, 'gc, '_>, this: Object<'gc>, depth: u8) -> Self {
        Self(GcCell::allocate(
            activation.context.gc_context,
            SuperObjectData { this, depth },
        ))
    }

    fn super_proto(&self, depth: u8) -> Value<'gc> {
        let mut proto = self.0.read().this.into();
        for _ in 0..depth {
            if let Value::Object(p) = proto {
                proto = p.proto();
            } else {
                return Value::Undefined;
            }
        }
        proto
    }
}

impl<'gc> TObject<'gc> for SuperObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
        _depth: u8,
    ) -> Option<Result<Value<'gc>, Error<'gc>>> {
        let depth = self.0.read().depth + 1;
        if let Value::Object(proto) = self.super_proto(depth) {
            proto.get_local(name, activation, this, depth)
        } else {
            Some(Ok(Value::Undefined))
        }
    }

    fn set_local(
        &self,
        _name: &str,
        _value: Value<'gc>,
        _activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
        _depth: u8,
    ) -> Result<(), Error<'gc>> {
        // `super` ignores all sets, including `super.__proto__`.
        Ok(())
    }

    fn call(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
        _depth: u8,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = self.0.read().this;
        let depth = self.0.read().depth;
        let (constructor, d) =
            search_prototype(self.super_proto(depth), "__constructor__", activation, this)?;

        if constructor.is_primitive() {
            avm_warn!(activation, "Super constructor is not callable");
        }

        let depth = depth + d + 1;
        constructor.call(name, activation, this, depth, args)
    }

    fn call_method(
        &self,
        name: &str,
        _depth: u8,
        args: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = self.0.read().this;
        let depth = self.0.read().depth + 1;
        let (method, d) = search_prototype(self.super_proto(depth), name, activation, this)?;

        if method.is_primitive() {
            avm_warn!(activation, "Super method {} is not callable", name);
        }

        let depth = depth + d;
        method.call(name, activation, this, depth, args)
    }

    fn call_setter(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0.read().this.call_setter(name, value, activation)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        // TODO: validate.
        let depth = self.0.read().depth;
        if let Value::Object(proto) = self.super_proto(depth) {
            proto.create_bare_object(activation, this)
        } else {
            // TODO: What happens when you `new super` but there's no
            // super? Is this code even reachable?!
            self.0.read().this.create_bare_object(activation, this)
        }
    }

    fn delete(&self, _activation: &mut Activation<'_, 'gc, '_>, _name: &str) -> bool {
        //`super` cannot have properties deleted from it
        false
    }

    fn proto(&self) -> Value<'gc> {
        let depth = self.0.read().depth;
        self.super_proto(depth + 2)
    }

    fn set_proto(&self, _gc_context: MutationContext<'gc, '_>, _prototype: Value<'gc>) {}

    fn define_value(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: &str,
        _value: Value<'gc>,
        _attributes: Attribute,
    ) {
        //`super` cannot have values defined on it
    }

    fn set_attributes(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: Option<&str>,
        _set_attributes: Attribute,
        _clear_attributes: Attribute,
    ) {
        //TODO: Does ASSetPropFlags work on `super`? What would it even work on?
    }

    fn add_property(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: &str,
        _get: Object<'gc>,
        _set: Option<Object<'gc>>,
        _attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
    }

    fn add_property_with_case(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _name: &str,
        _get: Object<'gc>,
        _set: Option<Object<'gc>>,
        _attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
    }

    fn set_watcher(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _name: Cow<str>,
        _callback: Object<'gc>,
        _user_data: Value<'gc>,
    ) {
        //`super` cannot have properties defined on it
    }

    fn remove_watcher(&self, _activation: &mut Activation<'_, 'gc, '_>, _name: Cow<str>) -> bool {
        //`super` cannot have properties defined on it
        false
    }

    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().this.has_property(activation, name)
    }

    fn has_own_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().this.has_own_property(activation, name)
    }

    fn has_own_virtual(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().this.has_own_virtual(activation, name)
    }

    fn is_property_enumerable(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().this.is_property_enumerable(activation, name)
    }

    fn get_keys(&self, _activation: &mut Activation<'_, 'gc, '_>) -> Vec<String> {
        vec![]
    }

    fn type_of(&self) -> &'static str {
        TYPE_OF_OBJECT
    }

    fn length(&self, _activation: &mut Activation<'_, 'gc, '_>) -> Result<i32, Error<'gc>> {
        Ok(0)
    }

    fn set_length(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _length: i32,
    ) -> Result<(), Error<'gc>> {
        Ok(())
    }

    fn has_element(&self, _activation: &mut Activation<'_, 'gc, '_>, _index: i32) -> bool {
        false
    }

    fn get_element(&self, _activation: &mut Activation<'_, 'gc, '_>, _index: i32) -> Value<'gc> {
        Value::Undefined
    }

    fn set_element(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _index: i32,
        _value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        Ok(())
    }

    fn delete_element(&self, _activation: &mut Activation<'_, 'gc, '_>, _index: i32) -> bool {
        false
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        //`super` does not implement interfaces
        vec![]
    }

    fn set_interfaces(&self, _gc_context: MutationContext<'gc, '_>, _iface_list: Vec<Object<'gc>>) {
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
        self.0.read().this.as_display_object()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }
}
