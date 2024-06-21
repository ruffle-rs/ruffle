//! Object trait to expose objects to AVM

use crate::avm1::function::{Executable, ExecutionName, ExecutionReason, FunctionObject};
use crate::avm1::globals::bevel_filter::BevelFilter;
use crate::avm1::globals::blur_filter::BlurFilter;
use crate::avm1::globals::color_matrix_filter::ColorMatrixFilter;
use crate::avm1::globals::color_transform::ColorTransformObject;
use crate::avm1::globals::convolution_filter::ConvolutionFilter;
use crate::avm1::globals::date::Date;
use crate::avm1::globals::displacement_map_filter::DisplacementMapFilter;
use crate::avm1::globals::drop_shadow_filter::DropShadowFilter;
use crate::avm1::globals::file_reference::FileReferenceObject;
use crate::avm1::globals::glow_filter::GlowFilter;
use crate::avm1::globals::gradient_filter::GradientFilter;
use crate::avm1::globals::local_connection::LocalConnection;
use crate::avm1::globals::netconnection::NetConnection;
use crate::avm1::globals::shared_object::SharedObject;
use crate::avm1::globals::transform::TransformObject;
use crate::avm1::globals::xml::Xml;
use crate::avm1::globals::xml_socket::XmlSocket;
use crate::avm1::object::array_object::ArrayObject;
use crate::avm1::object::super_object::SuperObject;
use crate::avm1::object::value_object::ValueObject;
use crate::avm1::{Activation, Attribute, Error, ScriptObject, SoundObject, StageObject, Value};
use crate::bitmap::bitmap_data::BitmapDataWrapper;
use crate::display_object::DisplayObject;
use crate::display_object::TDisplayObject;
use crate::html::TextFormat;
use crate::streams::NetStream;
use crate::string::AvmString;
use crate::xml::XmlNode;
use gc_arena::{Collect, Gc, GcCell, Mutation};
use ruffle_macros::enum_trait_object;
use std::cell::{Cell, RefCell};
use std::fmt::Debug;

pub mod array_object;
mod custom_object;
pub mod script_object;
pub mod sound_object;
pub mod stage_object;
pub mod super_object;
pub mod value_object;

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum NativeObject<'gc> {
    None,
    Date(Gc<'gc, Cell<Date>>),
    BlurFilter(BlurFilter<'gc>),
    BevelFilter(BevelFilter<'gc>),
    GlowFilter(GlowFilter<'gc>),
    DropShadowFilter(DropShadowFilter<'gc>),
    ColorMatrixFilter(ColorMatrixFilter<'gc>),
    DisplacementMapFilter(DisplacementMapFilter<'gc>),
    ConvolutionFilter(ConvolutionFilter<'gc>),
    GradientBevelFilter(GradientFilter<'gc>),
    GradientGlowFilter(GradientFilter<'gc>),
    ColorTransform(GcCell<'gc, ColorTransformObject>),
    Transform(TransformObject<'gc>),
    TextFormat(Gc<'gc, RefCell<TextFormat>>),
    NetStream(NetStream<'gc>),
    BitmapData(BitmapDataWrapper<'gc>),
    Xml(Xml<'gc>),
    XmlNode(XmlNode<'gc>),
    SharedObject(GcCell<'gc, SharedObject>),
    XmlSocket(XmlSocket<'gc>),
    FileReference(FileReferenceObject<'gc>),
    NetConnection(NetConnection<'gc>),
    LocalConnection(LocalConnection<'gc>),
}

/// Represents an object that can be directly interacted with by the AVM
/// runtime.
#[enum_trait_object(
    #[allow(clippy::enum_variant_names)]
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>),
        ArrayObject(ArrayObject<'gc>),
        SoundObject(SoundObject<'gc>),
        StageObject(StageObject<'gc>),
        SuperObject(SuperObject<'gc>),
        ValueObject(ValueObject<'gc>),
        FunctionObject(FunctionObject<'gc>),
    }
)]
pub trait TObject<'gc>: 'gc + Collect + Into<Object<'gc>> + Clone + Copy {
    /// Get the underlying raw script object.
    fn raw_script_object(&self) -> ScriptObject<'gc>;

    /// Retrieve a named, non-virtual property from this object exclusively.
    ///
    /// This function should not inspect prototype chains. Instead, use
    /// `get_stored` to do ordinary property look-up and resolution.
    fn get_local_stored(
        &self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc>,
        is_slash_path: bool,
    ) -> Option<Value<'gc>> {
        self.raw_script_object()
            .get_local_stored(name, activation, is_slash_path)
    }

    /// Retrieve a named property from the object, or its prototype.
    fn get_non_slash_path(
        &self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // TODO: Extract logic to a `lookup` function.
        let (this, proto) = if let Some(super_object) = self.as_super_object() {
            (super_object.this(), super_object.proto(activation))
        } else {
            ((*self).into(), Value::Object((*self).into()))
        };
        match search_prototype(proto, name.into(), activation, this, false)? {
            Some((value, _depth)) => Ok(value),
            None => Ok(Value::Undefined),
        }
    }

    /// Retrieve a named property from the object, or its prototype.
    fn get(
        &self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // TODO: Extract logic to a `lookup` function.
        let (this, proto) = if let Some(super_object) = self.as_super_object() {
            (super_object.this(), super_object.proto(activation))
        } else {
            ((*self).into(), Value::Object((*self).into()))
        };
        match search_prototype(proto, name.into(), activation, this, true)? {
            Some((value, _depth)) => Ok(value),
            None => Ok(Value::Undefined),
        }
    }

    /// Retrieve a non-virtual property from the object, or its prototype.
    fn get_stored(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = (*self).into();

        let mut depth = 0;
        let mut proto = Value::Object(this);

        while let Value::Object(p) = proto {
            if depth == 255 {
                return Err(Error::PrototypeRecursionLimit);
            }

            if let Some(value) = p.get_local_stored(name, activation, true) {
                return Ok(value);
            }

            proto = p.proto(activation);
            depth += 1;
        }

        Ok(Value::Undefined)
    }

    fn set_local(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.raw_script_object()
            .set_local(name, value, activation, this)
    }

    /// Set a named property on this object, or its prototype.
    fn set(
        &self,
        name: impl Into<AvmString<'gc>>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let name = name.into();
        if name.is_empty() {
            return Ok(());
        }

        let mut value = value;
        let (this, mut proto) = if let Some(super_object) = self.as_super_object() {
            (super_object.this(), super_object.proto(activation))
        } else {
            ((*self).into(), Value::Object((*self).into()))
        };
        let watcher_result = self.call_watcher(activation, name, &mut value, this);

        if !self.has_own_property(activation, name) {
            // Before actually inserting a new property, we need to crawl the
            // prototype chain for virtual setters.
            while let Value::Object(this_proto) = proto {
                if this_proto.has_own_virtual(activation, name) {
                    if let Some(setter) = this_proto.setter(name, activation) {
                        if let Some(exec) = setter.as_executable() {
                            let _ = exec.exec(
                                ExecutionName::Static("[Setter]"),
                                activation,
                                this.into(),
                                1,
                                &[value],
                                ExecutionReason::Special,
                                setter,
                            );
                        }
                    }
                    return Ok(());
                }

                proto = this_proto.proto(activation);
            }
        }

        let result = self.set_local(name, value, activation, this);
        watcher_result.and(result)
    }

    /// Call the underlying object.
    ///
    /// This function takes a  `this` parameter which generally
    /// refers to the object which has this property, although
    /// it can be changed by `Function.apply`/`Function.call`.
    fn call(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
        this: Value<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.raw_script_object().call(name, activation, this, args)
    }

    /// Construct the underlying object, if this is a valid constructor, and returns the result.
    /// Calling this on something other than a constructor will return a new Undefined object.
    fn construct(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Undefined)
    }

    /// Takes an already existing object and performs this constructor (if valid) on it.
    fn construct_on_existing(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        mut _this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<(), Error<'gc>> {
        Ok(())
    }

    /// Call a method on the object.
    ///
    /// It is highly recommended to use this convenience method to perform
    /// method calls. It is morally equivalent to an AVM1 `ActionCallMethod`
    /// opcode. It will take care of retrieving the method, calculating its
    /// base prototype for `super` calls, and providing it with the correct
    /// `this` parameter.
    fn call_method(
        &self,
        name: AvmString<'gc>,
        args: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
        reason: ExecutionReason,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = (*self).into();

        if let Some(s) = this.as_stage_object() {
            let d_o = s.as_display_object().unwrap();

            if d_o.avm1_removed() {
                return Ok(Value::Undefined);
            }
        }

        let (method, depth) =
            match search_prototype(Value::Object(this), name, activation, this, false)? {
                Some((Value::Object(method), depth)) => (method, depth),
                _ => return Ok(Value::Undefined),
            };

        // If the method was found on the object itself, change `depth` as-if
        // the method was found on the object's prototype.
        let depth = depth.max(1);

        match method.as_executable() {
            Some(exec) => exec.exec(
                ExecutionName::Dynamic(name),
                activation,
                this.into(),
                depth,
                args,
                reason,
                method,
            ),
            None => method.call(name, activation, this.into(), args),
        }
    }

    /// Retrieve a getter defined on this object.
    fn getter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Object<'gc>> {
        self.raw_script_object().getter(name, activation)
    }

    /// Retrieve a setter defined on this object.
    fn setter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Object<'gc>> {
        self.raw_script_object().setter(name, activation)
    }

    /// Construct a host object of some kind and return its cell.
    ///
    /// As the first step in object construction, the `new` method is called on
    /// the prototype to initialize an object. The prototype may construct any
    /// object implementation it wants, with itself as the new object's proto.
    /// Then, the constructor is `call`ed with the new object as `this` to
    /// initialize the object.
    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>>;

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        self.raw_script_object().delete(activation, name)
    }

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    fn proto(&self, activation: &mut Activation<'_, 'gc>) -> Value<'gc> {
        self.raw_script_object().proto(activation)
    }

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
        gc_context: &Mutation<'gc>,
        name: impl Into<AvmString<'gc>>,
        value: Value<'gc>,
        attributes: Attribute,
    ) {
        self.raw_script_object()
            .define_value(gc_context, name, value, attributes)
    }

    /// Set the attributes of a given property.
    ///
    /// Leaving `name` unspecified allows setting all properties on a given
    /// object to the same set of properties.
    ///
    /// Attributes can be set, cleared, or left as-is using the pairs of `set_`
    /// and `clear_attributes` parameters.
    fn set_attributes(
        &self,
        gc_context: &Mutation<'gc>,
        name: Option<AvmString<'gc>>,
        set_attributes: Attribute,
        clear_attributes: Attribute,
    ) {
        self.raw_script_object()
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

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
        gc_context: &Mutation<'gc>,
        name: AvmString<'gc>,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.raw_script_object()
            .add_property(gc_context, name, get, set, attributes)
    }

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
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.raw_script_object()
            .add_property_with_case(activation, name, get, set, attributes)
    }

    /// Calls the 'watcher' of a given property, if it exists.
    fn call_watcher(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
        value: &mut Value<'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.raw_script_object()
            .call_watcher(activation, name, value, this)
    }

    /// Set the 'watcher' of a given property.
    ///
    /// The property does not need to exist at the time of this being called.
    fn watch(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.raw_script_object()
            .watch(activation, name, callback, user_data)
    }

    /// Removed any assigned 'watcher' from the given property.
    ///
    /// The return value will indicate if there was a watcher present before this method was
    /// called.
    fn unwatch(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        self.raw_script_object().unwatch(activation, name)
    }

    /// Checks if the object has a given named property.
    fn has_property(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        self.raw_script_object().has_property(activation, name)
    }

    /// Checks if the object has a given named property on itself (and not,
    /// say, the object's prototype or superclass)
    fn has_own_property(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        self.raw_script_object().has_own_property(activation, name)
    }

    /// Checks if the object has a given named property on itself that is
    /// virtual.
    fn has_own_virtual(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        self.raw_script_object().has_own_virtual(activation, name)
    }

    /// Checks if a named property appears when enumerating the object.
    fn is_property_enumerable(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
    ) -> bool {
        self.raw_script_object()
            .is_property_enumerable(activation, name)
    }

    /// Enumerate the object.
    fn get_keys(
        &self,
        activation: &mut Activation<'_, 'gc>,
        include_hidden: bool,
    ) -> Vec<AvmString<'gc>> {
        self.raw_script_object()
            .get_keys(activation, include_hidden)
    }

    /// Enumerate all interfaces implemented by this object.
    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.raw_script_object().interfaces()
    }

    /// Set the interface list for this object. (Only useful for prototypes.)
    fn set_interfaces(&self, gc_context: &Mutation<'gc>, iface_list: Vec<Object<'gc>>) {
        self.raw_script_object()
            .set_interfaces(gc_context, iface_list)
    }

    /// Determine if this object is an instance of a class.
    ///
    /// The class is provided in the form of its constructor function and the
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
        activation: &mut Activation<'_, 'gc>,
        constructor: Object<'gc>,
        prototype: Object<'gc>,
    ) -> Result<bool, Error<'gc>> {
        let mut proto_stack = vec![];
        if let Value::Object(p) = self.proto(activation) {
            proto_stack.push(p);
        }

        while let Some(this_proto) = proto_stack.pop() {
            if Object::ptr_eq(this_proto, prototype) {
                return Ok(true);
            }

            if let Value::Object(p) = this_proto.proto(activation) {
                proto_stack.push(p);
            }

            if activation.swf_version() >= 7 {
                for interface in this_proto.interfaces() {
                    if Object::ptr_eq(interface, constructor) {
                        return Ok(true);
                    }

                    if let Value::Object(o) = interface.get("prototype", activation)? {
                        proto_stack.push(o);
                    }
                }
            }
        }

        Ok(false)
    }

    fn native(&self) -> NativeObject<'gc> {
        NativeObject::None
    }

    fn set_native(&self, _gc_context: &Mutation<'gc>, _native: NativeObject<'gc>) {}

    /// Get the underlying array object, if it exists.
    fn as_array_object(&self) -> Option<ArrayObject<'gc>> {
        None
    }

    /// Get the underlying sound object, if it exists.
    fn as_sound_object(&self) -> Option<SoundObject<'gc>> {
        None
    }

    /// Get the underlying stage object, if it exists.
    fn as_stage_object(&self) -> Option<StageObject<'gc>> {
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
    fn as_xml_node(&self) -> Option<XmlNode<'gc>> {
        match self.native() {
            NativeObject::Xml(xml) => Some(xml.root()),
            NativeObject::XmlNode(xml_node) => Some(xml_node),
            _ => None,
        }
    }

    /// Get the underlying `ValueObject`, if it exists.
    fn as_value_object(&self) -> Option<ValueObject<'gc>> {
        None
    }

    fn as_ptr(&self) -> *const ObjectPtr;

    /// Check if this object is in the prototype chain of the specified test object.
    fn is_prototype_of(&self, activation: &mut Activation<'_, 'gc>, other: Object<'gc>) -> bool {
        let mut proto = other.proto(activation);

        while let Value::Object(proto_ob) = proto {
            if self.as_ptr() == proto_ob.as_ptr() {
                return true;
            }

            proto = proto_ob.proto(activation);
        }

        false
    }

    /// Gets the length of this object, as if it were an array.
    fn length(&self, activation: &mut Activation<'_, 'gc>) -> Result<i32, Error<'gc>> {
        self.raw_script_object().length(activation)
    }

    /// Sets the length of this object, as if it were an array.
    fn set_length(
        &self,
        activation: &mut Activation<'_, 'gc>,
        length: i32,
    ) -> Result<(), Error<'gc>> {
        self.raw_script_object().set_length(activation, length)
    }

    /// Checks if this object has an element.
    fn has_element(&self, activation: &mut Activation<'_, 'gc>, index: i32) -> bool {
        self.raw_script_object().has_element(activation, index)
    }

    /// Gets a property of this object, as if it were an array.
    fn get_element(&self, activation: &mut Activation<'_, 'gc>, index: i32) -> Value<'gc> {
        self.raw_script_object().get_element(activation, index)
    }

    /// Sets a property of this object, as if it were an array.
    fn set_element(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: i32,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.raw_script_object()
            .set_element(activation, index, value)
    }

    /// Deletes a property of this object as if it were an array.
    fn delete_element(&self, activation: &mut Activation<'_, 'gc>, index: i32) -> bool {
        self.raw_script_object().delete_element(activation, index)
    }
}

pub enum ObjectPtr {}

impl<'gc> Object<'gc> {
    pub fn ptr_eq(a: Object<'gc>, b: Object<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
}

/// Perform a prototype lookup of a given object.
///
/// This function returns both the `Value` and the prototype depth from which
/// it was grabbed from. If the property did not resolve, then it returns
/// `Ok(None)`.
///
/// The prototype depth can and should be used to populate the `depth`
/// parameter necessary to make `super` work.
pub fn search_prototype<'gc>(
    mut proto: Value<'gc>,
    name: AvmString<'gc>,
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    is_slash_path: bool,
) -> Result<Option<(Value<'gc>, u8)>, Error<'gc>> {
    let mut depth = 0;
    let orig_proto = proto;

    while let Value::Object(p) = proto {
        if depth == 255 {
            return Err(Error::PrototypeRecursionLimit);
        }

        if let Some(getter) = p.getter(name, activation) {
            if let Some(exec) = getter.as_executable() {
                let result = exec.exec(
                    ExecutionName::Static("[Getter]"),
                    activation,
                    this.into(),
                    1,
                    &[],
                    ExecutionReason::Special,
                    getter,
                );
                let value = match result {
                    Ok(v) => v,
                    Err(Error::ThrownValue(e)) => return Err(Error::ThrownValue(e)),
                    Err(_) => Value::Undefined,
                };
                return Ok(Some((value, depth)));
            }
        }

        if let Some(value) = p.get_local_stored(name, activation, is_slash_path) {
            return Ok(Some((value, depth)));
        }

        proto = p.proto(activation);
        depth += 1;
    }

    if let Some(resolve) = find_resolve_method(orig_proto, activation)? {
        let result = resolve.call("__resolve".into(), activation, this.into(), &[name.into()])?;
        return Ok(Some((result, 0)));
    }

    Ok(None)
}

/// Finds the appropriate `__resolve` method for an object, searching its hierarchy too.
pub fn find_resolve_method<'gc>(
    mut proto: Value<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Option<Object<'gc>>, Error<'gc>> {
    let mut depth = 0;

    while let Value::Object(p) = proto {
        if depth == 255 {
            return Err(Error::PrototypeRecursionLimit);
        }

        if let Some(value) = p.get_local_stored("__resolve", activation, false) {
            return Ok(Some(value.coerce_to_object(activation)));
        }

        proto = p.proto(activation);
        depth += 1;
    }

    Ok(None)
}
