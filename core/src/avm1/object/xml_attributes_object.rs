//! AVM1 object type to represent the attributes of XML nodes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::{ObjectPtr, TObject};
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, Value};
use crate::string::AvmString;
use crate::xml::{XmlName, XmlNode};
use gc_arena::{Collect, MutationContext};
use std::borrow::Cow;
use std::fmt;

use crate::avm_warn;
/// A ScriptObject that is inherently tied to an XML node's attributes.
///
/// Note that this is *not* the same as the XMLNode object itself; for example,
/// `XMLNode`s must store both their base object and attributes object
/// separately.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XmlAttributesObject<'gc>(ScriptObject<'gc>, XmlNode<'gc>);

impl<'gc> XmlAttributesObject<'gc> {
    /// Construct an XmlAttributesObject for an already existing node's
    /// attributes.
    pub fn from_xml_node(
        gc_context: MutationContext<'gc, '_>,
        xml_node: XmlNode<'gc>,
    ) -> Object<'gc> {
        XmlAttributesObject(ScriptObject::object(gc_context, None), xml_node).into()
    }

    fn base(&self) -> ScriptObject<'gc> {
        match self {
            XmlAttributesObject(base, ..) => *base,
        }
    }

    fn node(&self) -> XmlNode<'gc> {
        match self {
            XmlAttributesObject(_, node) => *node,
        }
    }
}

impl fmt::Debug for XmlAttributesObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XmlAttributesObject(base, node) => f
                .debug_tuple("XmlAttributesObject")
                .field(base)
                .field(node)
                .finish(),
        }
    }
}

impl<'gc> TObject<'gc> for XmlAttributesObject<'gc> {
    fn get_local_stored(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Value<'gc>> {
        self.node()
            .attribute_value(&XmlName::from_str(name))
            .map(|s| AvmString::new(activation.context.gc_context, s).into())
    }

    fn set_local(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
        _base_proto: Option<Object<'gc>>,
    ) -> Result<(), Error<'gc>> {
        self.node().set_attribute_value(
            activation.context.gc_context,
            &XmlName::from_str(name),
            &value.coerce_to_string(activation)?,
        );
        Ok(())
    }

    fn call(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.base().call(name, activation, this, base_proto, args)
    }

    fn getter(&self, name: &str, activation: &mut Activation<'_, 'gc, '_>) -> Option<Object<'gc>> {
        self.base().getter(name, activation)
    }

    fn setter(&self, name: &str, activation: &mut Activation<'_, 'gc, '_>) -> Option<Object<'gc>> {
        self.base().setter(name, activation)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        //TODO: `new xmlnode.attributes()` returns undefined, not an object
        avm_warn!(activation, "Cannot create new XML Attributes object");
        Ok(Value::Undefined.coerce_to_object(activation))
    }

    fn delete(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.node()
            .delete_attribute(activation.context.gc_context, &XmlName::from_str(name));
        self.base().delete(activation, name)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.base()
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
        self.base()
            .add_property_with_case(activation, name, get, set, attributes)
    }

    fn call_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: &str,
        value: &mut Value<'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.base().call_watcher(activation, name, value, this)
    }

    fn watch(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: Cow<str>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.base().watch(activation, name, callback, user_data);
    }

    fn unwatch(&self, activation: &mut Activation<'_, 'gc, '_>, name: Cow<str>) -> bool {
        self.base().unwatch(activation, name)
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: Attribute,
    ) {
        self.base()
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: Attribute,
        clear_attributes: Attribute,
    ) {
        self.base()
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

    fn proto(&self, activation: &mut Activation<'_, 'gc, '_>) -> Value<'gc> {
        self.base().proto(activation)
    }

    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.base().has_property(activation, name)
    }

    fn has_own_property(&self, _activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.node()
            .attribute_value(&XmlName::from_str(name))
            .is_some()
    }

    fn has_own_virtual(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.base().has_own_virtual(activation, name)
    }

    fn is_property_enumerable(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.base().is_property_enumerable(activation, name)
    }

    fn get_keys(&self, activation: &mut Activation<'_, 'gc, '_>) -> Vec<String> {
        let mut base = self.base().get_keys(activation);
        base.extend(self.node().attribute_keys());
        base
    }

    fn type_of(&self) -> &'static str {
        self.base().type_of()
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.base().interfaces()
    }

    fn set_interfaces(&self, gc_context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.base().set_interfaces(gc_context, iface_list)
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.base())
    }

    fn as_xml_node(&self) -> Option<XmlNode<'gc>> {
        Some(self.node())
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.base().as_ptr() as *const ObjectPtr
    }

    fn length(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<i32, Error<'gc>> {
        self.base().length(activation)
    }

    fn set_length(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        length: i32,
    ) -> Result<(), Error<'gc>> {
        self.base().set_length(activation, length)
    }

    fn has_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> bool {
        self.base().has_element(activation, index)
    }

    fn get_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> Value<'gc> {
        self.base().get_element(activation, index)
    }

    fn set_element(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        index: i32,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.base().set_element(activation, index, value)
    }

    fn delete_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> bool {
        self.base().delete_element(activation, index)
    }
}
