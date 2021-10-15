//! Special object that implements `super`

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::script_object::TYPE_OF_OBJECT;
use crate::avm1::object::search_prototype;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::avm_warn;
use crate::display_object::DisplayObject;
use gc_arena::{Collect, GcCell, MutationContext};

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

    /// The prototype depth of the currently-executing method.
    depth: u8,
}

impl<'gc> SuperObject<'gc> {
    /// Construct a `super` for an incoming stack frame.
    ///
    /// `this` and `base_proto` must be the values provided to `Executable::exec`.
    pub fn new(
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Object<'gc>,
    ) -> Self {
        // This is a temporary hack to calculate `depth` from `this` and `base_proto`.
        // TODO: Pass `depth` alone (preferably in `Activation`),
        // and remove all `base_proto` parameters.
        let mut object = this;
        let mut depth = 0;
        while !Object::ptr_eq(object, base_proto) {
            object = object.proto(activation).coerce_to_object(activation);
            depth += 1;
        }

        Self(GcCell::allocate(
            activation.context.gc_context,
            SuperObjectData { this, depth },
        ))
    }

    fn base_proto(&self, activation: &mut Activation<'_, 'gc, '_>) -> Object<'gc> {
        let read = self.0.read();
        let depth = read.depth;
        let mut proto = read.this;
        for _ in 0..depth {
            proto = proto.proto(activation).coerce_to_object(activation);
        }
        proto
    }
}

impl<'gc> TObject<'gc> for SuperObject<'gc> {
    fn get_local_stored(
        &self,
        _name: impl Into<AvmString<'gc>>,
        _activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Value<'gc>> {
        None
    }

    fn set_local(
        &self,
        _name: AvmString<'gc>,
        _value: Value<'gc>,
        _activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
        _base_proto: Option<Object<'gc>>,
    ) -> Result<(), Error<'gc>> {
        //TODO: What happens if you set `super.__proto__`?
        Ok(())
    }

    fn call(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
        _base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        let constructor = self
            .base_proto(activation)
            .get("__constructor__", activation)?
            .coerce_to_object(activation);
        let this = self.0.read().this;
        let base_proto = self.proto(activation).coerce_to_object(activation);
        constructor.call(name, activation, this, Some(base_proto), args)
    }

    fn call_method(
        &self,
        name: AvmString<'gc>,
        args: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = self.0.read().this;
        let (method, base_proto) =
            search_prototype(self.proto(activation), name, activation, this)?;

        if method.is_primitive() {
            avm_warn!(activation, "Super method {} is not callable", name);
        }

        method.call(name, activation, this, base_proto, args)
    }

    fn getter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0.read().this.getter(name, activation)
    }

    fn setter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0.read().this.setter(name, activation)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        if let Value::Object(proto) = self.proto(activation) {
            proto.create_bare_object(activation, this)
        } else {
            // TODO: What happens when you `new super` but there's no
            // super? Is this code even reachable?!
            self.0.read().this.create_bare_object(activation, this)
        }
    }

    fn delete(&self, _activation: &mut Activation<'_, 'gc, '_>, _name: AvmString<'gc>) -> bool {
        //`super` cannot have properties deleted from it
        false
    }

    fn proto(&self, activation: &mut Activation<'_, 'gc, '_>) -> Value<'gc> {
        self.base_proto(activation).proto(activation)
    }

    fn define_value(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: impl Into<AvmString<'gc>>,
        _value: Value<'gc>,
        _attributes: Attribute,
    ) {
        //`super` cannot have values defined on it
    }

    fn set_attributes(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: Option<AvmString<'gc>>,
        _set_attributes: Attribute,
        _clear_attributes: Attribute,
    ) {
        //TODO: Does ASSetPropFlags work on `super`? What would it even work on?
    }

    fn add_property(
        &self,
        _gc_context: MutationContext<'gc, '_>,
        _name: AvmString<'gc>,
        _get: Object<'gc>,
        _set: Option<Object<'gc>>,
        _attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
    }

    fn add_property_with_case(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _name: AvmString<'gc>,
        _get: Object<'gc>,
        _set: Option<Object<'gc>>,
        _attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
    }

    fn call_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        value: &mut Value<'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.0
            .read()
            .this
            .call_watcher(activation, name, value, this)
    }

    fn watch(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _name: AvmString<'gc>,
        _callback: Object<'gc>,
        _user_data: Value<'gc>,
    ) {
        //`super` cannot have properties defined on it
    }

    fn unwatch(&self, _activation: &mut Activation<'_, 'gc, '_>, _name: AvmString<'gc>) -> bool {
        //`super` cannot have properties defined on it
        false
    }

    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        self.0.read().this.has_property(activation, name)
    }

    fn has_own_property(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.0.read().this.has_own_property(activation, name)
    }

    fn has_own_virtual(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.0.read().this.has_own_virtual(activation, name)
    }

    fn is_property_enumerable(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.0.read().this.is_property_enumerable(activation, name)
    }

    fn get_keys(&self, _activation: &mut Activation<'_, 'gc, '_>) -> Vec<AvmString<'gc>> {
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
