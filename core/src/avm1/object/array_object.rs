use crate::avm1::property::Attribute;
use crate::avm1::{Activation, Error, Object, ObjectPtr, ScriptObject, TObject, Value};
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::Cow;
use std::fmt;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ArrayObject<'gc>(GcCell<'gc, ArrayObjectData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
pub struct ArrayObjectData<'gc> {
    base: ScriptObject<'gc>,
    length: Option<i32>,
}

impl fmt::Debug for ArrayObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("ArrayObject")
            .field("length", &this.length)
            .finish()
    }
}

impl<'gc> ArrayObject<'gc> {
    pub fn empty(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        Self(GcCell::allocate(
            gc_context,
            ArrayObjectData {
                base: ScriptObject::object(gc_context, proto),
                length: Some(0),
            },
        ))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn as_array_length(length: f64) -> Option<i32> {
        if length.is_finite() && length >= i32::MIN.into() && length <= i32::MAX.into() {
            Some(length as i32)
        } else {
            None
        }
    }
}

impl<'gc> TObject<'gc> for ArrayObject<'gc> {
    fn get_data(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: &str,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if name == "length" {
            if let Some(length) = self.0.read().length {
                return Ok(length.into());
            }
        }

        self.0.read().base.get_data(activation, name)
    }

    fn call_getter(&self, name: &str, activation: &mut Activation<'_, 'gc, '_>) -> Value<'gc> {
        self.0.read().base.call_getter(name, activation)
    }

    fn set_data(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: &str,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        if name == "length" {
            let length = if let Value::Number(number) = value {
                Self::as_array_length(number)
            } else {
                None
            };
            // TODO: delete elements in new_length..old_length
            self.0.write(activation.context.gc_context).length = length;
            if length.is_none() {
                self.0.read().base.define_value(
                    activation.context.gc_context,
                    name,
                    value,
                    Attribute::DONT_ENUM | Attribute::DONT_DELETE,
                )
            }
            return Ok(());
        }

        if let Ok(index) = name.parse::<i32>() {
            let length = self.0.read().length;
            if let Some(length) = length {
                self.0.write(activation.context.gc_context).length = Some((index + 1).max(length));
            }
        }

        self.0.read().base.set_data(activation, name, value)
    }

    fn call_setter(&self, name: &str, value: Value<'gc>, activation: &mut Activation<'_, 'gc, '_>) {
        self.0.read().base.call_setter(name, value, activation)
    }

    fn call(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.0
            .read()
            .base
            .call(name, activation, this, base_proto, args)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(Self::empty(activation.context.gc_context, Some(this)).into())
    }

    fn delete(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().base.delete(activation, name)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.0
            .read()
            .base
            .add_property(gc_context, name, get, set, attributes)
    }

    fn add_property_with_case(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.0
            .read()
            .base
            .add_property_with_case(activation, name, get, set, attributes)
    }

    fn set_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: Cow<str>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.0
            .read()
            .base
            .set_watcher(activation, name, callback, user_data);
    }

    fn remove_watcher(&self, activation: &mut Activation<'_, 'gc, '_>, name: Cow<str>) -> bool {
        self.0.read().base.remove_watcher(activation, name)
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: Attribute,
    ) {
        self.0
            .read()
            .base
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: Attribute,
        clear_attributes: Attribute,
    ) {
        self.0
            .read()
            .base
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

    fn proto(&self) -> Value<'gc> {
        self.0.read().base.proto()
    }

    fn set_proto(&self, gc_context: MutationContext<'gc, '_>, prototype: Value<'gc>) {
        self.0.read().base.set_proto(gc_context, prototype);
    }

    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().base.has_property(activation, name)
    }

    fn has_own_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        name == "length" || self.0.read().base.has_own_property(activation, name)
    }

    fn has_own_virtual(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().base.has_own_virtual(activation, name)
    }

    fn is_property_enumerable(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.0.read().base.is_property_enumerable(activation, name)
    }

    fn get_keys(&self, activation: &mut Activation<'_, 'gc, '_>) -> Vec<String> {
        self.0.read().base.get_keys(activation)
    }

    fn type_of(&self) -> &'static str {
        self.0.read().base.type_of()
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.0.read().base.interfaces()
    }

    fn set_interfaces(&self, gc_context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.0.read().base.set_interfaces(gc_context, iface_list)
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.0.read().base)
    }

    fn as_array_object(&self) -> Option<ArrayObject<'gc>> {
        Some(*self)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.read().base.as_ptr() as *const ObjectPtr
    }
}
