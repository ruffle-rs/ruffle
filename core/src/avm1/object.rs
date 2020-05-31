//! Object trait to expose objects to AVM

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::super_object::SuperObject;
use crate::avm1::value_object::ValueObject;
use crate::avm1::xml_attributes_object::XMLAttributesObject;
use crate::avm1::xml_idmap_object::XMLIDMapObject;
use crate::avm1::xml_object::XMLObject;
use crate::avm1::{Avm1, Error, ScriptObject, SoundObject, StageObject, UpdateContext, Value};
use crate::display_object::DisplayObject;
use crate::xml::XMLNode;
use enumset::EnumSet;
use gc_arena::{Collect, MutationContext};
use ruffle_macros::enum_trait_object;
use std::fmt::Debug;

/// Represents an object that can be directly interacted with by the AVM
/// runtime.
#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>),
        SoundObject(SoundObject<'gc>),
        StageObject(StageObject<'gc>),
        SuperObject(SuperObject<'gc>),
        XMLObject(XMLObject<'gc>),
        XMLAttributesObject(XMLAttributesObject<'gc>),
        XMLIDMapObject(XMLIDMapObject<'gc>),
        ValueObject(ValueObject<'gc>),
        FunctionObject(FunctionObject<'gc>),
    }
)]
pub trait TObject<'gc>: 'gc + Collect + Debug + Into<Object<'gc>> + Clone + Copy {
    /// Retrieve a named property from this object exclusively.
    ///
    /// This function takes a redundant `this` parameter which should be
    /// the object's own `GcCell`, so that it can pass it to user-defined
    /// overrides that may need to interact with the underlying object.
    ///
    /// This function should not inspect prototype chains. Instead, use `get`
    /// to do ordinary property look-up and resolution.
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Value<'gc>, Error>;

    /// Retrieve a named property from the object, or it's prototype.
    fn get(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        if self.has_own_property(avm, context, name) {
            self.get_local(name, avm, context, (*self).into())
        } else {
            search_prototype(self.proto(), name, avm, context, (*self).into())?
                .0
                .resolve(avm, context)
        }
    }

    /// Set a named property on this object, or it's prototype.
    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error>;

    /// Call the underlying object.
    ///
    /// This function takes a  `this` parameter which generally
    /// refers to the object which has this property, although
    /// it can be changed by `Function.apply`/`Function.call`.
    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Call a method on the object.
    ///
    /// It is highly recommended to use this convenience method to perform
    /// method calls. It is morally equivalent to an AVM1 `ActionCallMethod`
    /// opcode. It will take care of retrieving the method, calculating it's
    /// base prototype for `super` calls, and providing it with the correct
    /// `this` parameter.
    fn call_method(
        &self,
        name: &str,
        args: &[Value<'gc>],
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let (method, base_proto) =
            search_prototype(Some((*self).into()), name, avm, context, (*self).into())?;
        let method = method.resolve(avm, context)?;

        if let Value::Object(_) = method {
        } else {
            log::warn!("Object method {} is not callable", name);
        }

        method.call(avm, context, (*self).into(), base_proto, args)
    }

    /// Call a setter defined in this object.
    ///
    /// This function returns the `ReturnValue` of the called function; it
    /// should be resolved and discarded. Attempts to call a non-virtual setter
    /// or non-existent setter fail silently.
    ///
    /// The setter will be invoked with the provided `this`. It is assumed that
    /// this function is being called on the appropriate `base_proto` and
    /// `super` will be invoked following said guidance.
    fn call_setter(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Construct a host object of some kind and return it's cell.
    ///
    /// As the first step in object construction, the `new` method is called on
    /// the prototype to initialize an object. The prototype may construct any
    /// object implementation it wants, with itself as the new object's proto.
    /// Then, the constructor is `call`ed with the new object as `this` to
    /// initialize the object.
    ///
    /// The arguments passed to the constructor are provided here; however, all
    /// object construction should happen in `call`, not `new`. `new` exists
    /// purely so that host objects can be constructed by the VM.
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error>;

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&self, avm: &mut Avm1<'gc>, gc_context: MutationContext<'gc, '_>, name: &str)
        -> bool;

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    fn proto(&self) -> Option<Object<'gc>>;

    /// Sets the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` in
    /// `set`.
    fn set_proto(&self, gc_context: MutationContext<'gc, '_>, prototype: Option<Object<'gc>>);

    /// Define a value on an object.
    ///
    /// Unlike setting a value, this function is intended to replace any
    /// existing virtual or built-in properties already installed on a given
    /// object. As such, this should not run any setters; the resulting name
    /// slot should either be completely replaced with the value or completely
    /// untouched.
    ///
    /// It is not guaranteed that all objects accept value definitions,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    );

    /// Set the attributes of a given property.
    ///
    /// Leaving `name` unspecified allows setting all properties on a given
    /// object to the same set of properties.
    ///
    /// Attributes can be set, cleared, or left as-is using the pairs of `set_`
    /// and `clear_attributes` parameters.
    fn set_attributes(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: EnumSet<Attribute>,
        clear_attributes: EnumSet<Attribute>,
    );

    /// Define a virtual property onto a given object.
    ///
    /// A virtual property is a set of get/set functions that are called when a
    /// given named property is retrieved or stored on an object. These
    /// functions are then responsible for providing or accepting the value
    /// that is given to or taken from the AVM.
    ///
    /// It is not guaranteed that all objects accept virtual properties,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    );

    /// Define a virtual property onto a given object.
    ///
    /// A virtual property is a set of get/set functions that are called when a
    /// given named property is retrieved or stored on an object. These
    /// functions are then responsible for providing or accepting the value
    /// that is given to or taken from the AVM.
    ///
    /// It is not guaranteed that all objects accept virtual properties,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    fn add_property_with_case(
        &self,
        avm: &mut Avm1<'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    );

    /// Checks if the object has a given named property.
    fn has_property(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool;

    /// Checks if the object has a given named property on itself (and not,
    /// say, the object's prototype or superclass)
    fn has_own_property(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool;

    /// Checks if the object has a given named property on itself that is
    /// virtual.
    fn has_own_virtual(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool;

    /// Checks if a named property can be overwritten.
    fn is_property_overwritable(&self, avm: &mut Avm1<'gc>, name: &str) -> bool;

    /// Checks if a named property appears when enumerating the object.
    fn is_property_enumerable(&self, avm: &mut Avm1<'gc>, name: &str) -> bool;

    /// Enumerate the object.
    fn get_keys(&self, avm: &mut Avm1<'gc>) -> Vec<String>;

    /// Coerce the object into a string.
    fn as_string(&self) -> String;

    /// Get the object's type string.
    fn type_of(&self) -> &'static str;

    /// Enumerate all interfaces implemented by this object.
    fn interfaces(&self) -> Vec<Object<'gc>>;

    /// Set the interface list for this object. (Only useful for prototypes.)
    fn set_interfaces(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        iface_list: Vec<Object<'gc>>,
    );

    /// Determine if this object is an instance of a class.
    ///
    /// The class is provided in the form of it's constructor function and the
    /// explicit prototype of that constructor function. It is assumed that
    /// they are already linked.
    ///
    /// Because ActionScript 2.0 added interfaces, this function cannot simply
    /// check the prototype chain and call it a day. Each interface represents
    /// a new, parallel prototype chain which also needs to be checked. You
    /// can't implement interfaces within interfaces (fortunately), but if you
    /// somehow could this would support that, too.
    fn is_instance_of(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        constructor: Object<'gc>,
        prototype: Object<'gc>,
    ) -> Result<bool, Error> {
        let mut proto_stack = vec![];
        if let Some(p) = self.proto() {
            proto_stack.push(p);
        }

        while let Some(this_proto) = proto_stack.pop() {
            if Object::ptr_eq(this_proto, prototype) {
                return Ok(true);
            }

            if let Some(p) = this_proto.proto() {
                proto_stack.push(p);
            }

            if avm.current_swf_version() >= 7 {
                for interface in this_proto.interfaces() {
                    if Object::ptr_eq(interface, constructor) {
                        return Ok(true);
                    }

                    if let Value::Object(o) = interface.get("prototype", avm, context)? {
                        proto_stack.push(o);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get the underlying script object, if it exists.
    fn as_script_object(&self) -> Option<ScriptObject<'gc>>;

    /// Get the underlying sound object, if it exists.
    fn as_sound_object(&self) -> Option<SoundObject<'gc>> {
        None
    }

    /// Get the underlying super object, if it exists.
    fn as_super_object(&self) -> Option<SuperObject<'gc>> {
        None
    }

    /// Get the underlying display node for this object, if it exists.
    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        None
    }

    /// Get the underlying executable for this object, if it exists.
    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }

    /// Get the underlying XML node for this object, if it exists.
    fn as_xml_node(&self) -> Option<XMLNode<'gc>> {
        None
    }

    /// Get the underlying `ValueObject`, if it exists.
    fn as_value_object(&self) -> Option<ValueObject<'gc>> {
        None
    }

    fn as_ptr(&self) -> *const ObjectPtr;

    /// Check if this object is in the prototype chain of the specified test object.
    fn is_prototype_of(&self, other: Object<'gc>) -> bool {
        let mut proto = other.proto();

        while let Some(proto_ob) = proto {
            if self.as_ptr() == proto_ob.as_ptr() {
                return true;
            }

            proto = proto_ob.proto();
        }

        false
    }

    /// Get the length of this object, as if it were an array.
    fn length(&self) -> usize;

    /// Gets a copy of the array storage behind this object.
    fn array(&self) -> Vec<Value<'gc>>;

    /// Sets the length of this object, as if it were an array.
    ///
    /// Increasing this value will fill the gap with Value::Undefined.
    /// Decreasing this value will remove affected items from both the array and properties storage.
    fn set_length(&self, gc_context: MutationContext<'gc, '_>, length: usize);

    /// Gets a property of this object as if it were an array.
    ///
    /// Array element lookups do not respect the prototype chain, and will ignore virtual properties.
    fn array_element(&self, index: usize) -> Value<'gc>;

    /// Sets a property of this object as if it were an array.
    ///
    /// This will increase the "length" of this object to encompass the index, and return the new length.
    /// Any gap created by increasing the length will be filled with Value::Undefined, both in array
    /// and property storage.
    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize;

    /// Deletes a property of this object as if it were an array.
    ///
    /// This will not rearrange the array or adjust the length, nor will it affect the properties
    /// storage.
    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>);
}

pub enum ObjectPtr {}

impl<'gc> Object<'gc> {
    pub fn ptr_eq(a: Object<'gc>, b: Object<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
}

/// Perform a prototype lookup of a given object.
///
/// This function returns both the `ReturnValue` and the prototype that
/// generated the value. If the property did not resolve, then it returns
/// `undefined` and `None` for the prototype.
///
/// The second return value can and should be used to populate the `base_proto`
/// property necessary to make `super` work.
pub fn search_prototype<'gc>(
    mut proto: Option<Object<'gc>>,
    name: &str,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
) -> Result<(ReturnValue<'gc>, Option<Object<'gc>>), Error> {
    let mut depth = 0;

    while proto.is_some() {
        if depth == 255 {
            return Err("Encountered an excessively deep prototype chain.".into());
        }

        if proto.unwrap().has_own_property(avm, context, name) {
            return Ok((
                proto.unwrap().get_local(name, avm, context, this)?.into(),
                proto,
            ));
        }

        proto = proto.unwrap().proto();
        depth += 1;
    }

    Ok((Value::Undefined.into(), None))
}
