//! Special object that implements `super`

use core::fmt;

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::ExecutionReason;
use crate::avm1::object::{search_prototype, ExecutionName};
use crate::avm1::property::Attribute;
use crate::avm1::{NativeObject, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::display_object::DisplayObject;
use crate::string::AvmString;
use gc_arena::{Collect, Gc, Mutation};

/// Implementation of the `super` object in AS2.
///
/// A `SuperObject` references all data from another object, but with one layer
/// of prototyping removed. It's as if the given object had been constructed
/// with its parent class.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct SuperObject<'gc>(Gc<'gc, SuperObjectData<'gc>>);

impl fmt::Debug for SuperObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SuperObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SuperObjectData<'gc> {
    /// The object present as `this` throughout the superchain.
    this: Object<'gc>,

    /// The prototype depth of the currently-executing method.
    depth: u8,
}

impl<'gc> SuperObject<'gc> {
    /// Construct a `super` for an incoming stack frame.
    pub fn new(activation: &mut Activation<'_, 'gc>, this: Object<'gc>, depth: u8) -> Self {
        Self(Gc::new(activation.gc(), SuperObjectData { this, depth }))
    }

    pub fn this(&self) -> Object<'gc> {
        self.0.this
    }

    fn base_proto(&self, activation: &mut Activation<'_, 'gc>) -> Object<'gc> {
        let depth = self.0.depth;
        let mut proto = self.0.this;
        for _ in 0..depth {
            proto = proto.proto(activation).coerce_to_object(activation);
        }
        proto
    }
}

impl<'gc> TObject<'gc> for SuperObject<'gc> {
    fn raw_script_object(&self) -> ScriptObject<'gc> {
        self.0.this.raw_script_object()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn get_local_stored(
        &self,
        _name: impl Into<AvmString<'gc>>,
        _activation: &mut Activation<'_, 'gc>,
        _is_slash_path: bool,
    ) -> Option<Value<'gc>> {
        None
    }

    fn set_local(
        &self,
        _name: AvmString<'gc>,
        _value: Value<'gc>,
        _activation: &mut Activation<'_, 'gc>,
        _this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        //TODO: What happens if you set `super.__proto__`?
        Ok(())
    }

    fn call(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
        _this: Value<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        let constructor = self
            .base_proto(activation)
            .get("__constructor__", activation)?
            .coerce_to_object(activation);
        match constructor.as_executable() {
            Some(exec) => exec.exec(
                ExecutionName::Dynamic(name),
                activation,
                self.0.this.into(),
                self.0.depth + 1,
                args,
                ExecutionReason::FunctionCall,
                constructor,
            ),
            None => Ok(Value::Undefined),
        }
    }

    fn call_method(
        &self,
        name: AvmString<'gc>,
        args: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
        reason: ExecutionReason,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = self.0.this;
        let (method, depth) =
            match search_prototype(self.proto(activation), name, activation, this, false)? {
                Some((Value::Object(method), depth)) => (method, depth),
                _ => return Ok(Value::Undefined),
            };

        match method.as_executable() {
            Some(exec) => exec.exec(
                ExecutionName::Dynamic(name),
                activation,
                this.into(),
                self.0.depth + depth + 1,
                args,
                reason,
                method,
            ),
            None => method.call(name, activation, this.into(), args),
        }
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        if let Value::Object(proto) = self.proto(activation) {
            proto.create_bare_object(activation, this)
        } else {
            // TODO: What happens when you `new super` but there's no
            // super? Is this code even reachable?!
            self.0.this.create_bare_object(activation, this)
        }
    }

    fn delete(&self, _activation: &mut Activation<'_, 'gc>, _name: AvmString<'gc>) -> bool {
        //`super` cannot have properties deleted from it
        false
    }

    fn proto(&self, activation: &mut Activation<'_, 'gc>) -> Value<'gc> {
        self.base_proto(activation).proto(activation)
    }

    fn define_value(
        &self,
        _gc_context: &Mutation<'gc>,
        _name: impl Into<AvmString<'gc>>,
        _value: Value<'gc>,
        _attributes: Attribute,
    ) {
        //`super` cannot have values defined on it
    }

    fn set_attributes(
        &self,
        _gc_context: &Mutation<'gc>,
        _name: Option<AvmString<'gc>>,
        _set_attributes: Attribute,
        _clear_attributes: Attribute,
    ) {
        //TODO: Does ASSetPropFlags work on `super`? What would it even work on?
    }

    fn add_property(
        &self,
        _gc_context: &Mutation<'gc>,
        _name: AvmString<'gc>,
        _get: Object<'gc>,
        _set: Option<Object<'gc>>,
        _attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
    }

    fn add_property_with_case(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        _name: AvmString<'gc>,
        _get: Object<'gc>,
        _set: Option<Object<'gc>>,
        _attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
    }

    fn watch(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        _name: AvmString<'gc>,
        _callback: Object<'gc>,
        _user_data: Value<'gc>,
    ) {
        //`super` cannot have properties defined on it
    }

    fn unwatch(&self, _activation: &mut Activation<'_, 'gc>, _name: AvmString<'gc>) -> bool {
        //`super` cannot have properties defined on it
        false
    }

    fn get_keys(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        _include_hidden: bool,
    ) -> Vec<AvmString<'gc>> {
        vec![]
    }

    fn length(&self, _activation: &mut Activation<'_, 'gc>) -> Result<i32, Error<'gc>> {
        Ok(0)
    }

    fn set_length(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        _length: i32,
    ) -> Result<(), Error<'gc>> {
        Ok(())
    }

    fn has_element(&self, _activation: &mut Activation<'_, 'gc>, _index: i32) -> bool {
        false
    }

    fn get_element(&self, _activation: &mut Activation<'_, 'gc>, _index: i32) -> Value<'gc> {
        Value::Undefined
    }

    fn set_element(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        _index: i32,
        _value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        Ok(())
    }

    fn delete_element(&self, _activation: &mut Activation<'_, 'gc>, _index: i32) -> bool {
        false
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        //`super` does not implement interfaces
        vec![]
    }

    fn set_interfaces(&self, _gc_context: &Mutation<'gc>, _iface_list: Vec<Object<'gc>>) {
        //`super` probably cannot have interfaces set on it
    }

    fn as_super_object(&self) -> Option<SuperObject<'gc>> {
        Some(*self)
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        //`super` actually can be used to invoke MovieClip methods
        self.0.this.as_display_object()
    }

    fn native(&self) -> NativeObject<'gc> {
        self.0.this.native()
    }

    fn set_native(&self, gc_context: &Mutation<'gc>, native: NativeObject<'gc>) {
        self.0.this.set_native(gc_context, native);
    }
}
