//! AVM1 object type to represent the attributes of XML nodes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::{ObjectPtr, TObject};
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, ScriptObject, Value};
use crate::avm_warn;
use crate::xml::{XmlDocument, XmlNode};
use gc_arena::{Collect, MutationContext};
use std::fmt;

/// An Object that is inherently tied to an XML document's ID map.
///
/// Note that this is *not* the same as the XML root object itself; and
/// furthermore this object is linked to the document, not the root node of the
/// document.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XmlIdMapObject<'gc>(ScriptObject<'gc>, XmlDocument<'gc>);

impl<'gc> XmlIdMapObject<'gc> {
    /// Construct an XmlIdMapObject for an already existing node's
    /// attributes.
    pub fn from_xml_document(
        gc_context: MutationContext<'gc, '_>,
        xml_doc: XmlDocument<'gc>,
    ) -> Object<'gc> {
        XmlIdMapObject(ScriptObject::object(gc_context, None), xml_doc).into()
    }

    fn base(&self) -> ScriptObject<'gc> {
        match self {
            XmlIdMapObject(base, ..) => *base,
        }
    }

    fn document(&self) -> XmlDocument<'gc> {
        match self {
            XmlIdMapObject(_, document) => *document,
        }
    }
}

impl fmt::Debug for XmlIdMapObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XmlIdMapObject(base, document) => f
                .debug_tuple("XmlIdMapObject")
                .field(base)
                .field(document)
                .finish(),
        }
    }
}

impl<'gc> TObject<'gc> for XmlIdMapObject<'gc> {
    fn get_local_stored(
        &self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Value<'gc>> {
        let name = name.into();
        if let Some(mut node) = self.document().get_node_by_id(name) {
            Some(node.script_object(activation).into())
        } else {
            self.base().get_local_stored(name, activation)
        }
    }

    fn set_local(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.base().set_local(name, value, activation, this)
    }

    fn call(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Value<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.base().call(name, activation, this, args)
    }

    fn getter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.base().getter(name, activation)
    }

    fn setter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
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

    fn delete(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        self.base().delete(activation, name)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: AvmString<'gc>,
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
        name: AvmString<'gc>,
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
        name: AvmString<'gc>,
        value: &mut Value<'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.base().call_watcher(activation, name, value, this)
    }

    fn watch(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.base().watch(activation, name, callback, user_data);
    }

    fn unwatch(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        self.base().unwatch(activation, name)
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: impl Into<AvmString<'gc>>,
        value: Value<'gc>,
        attributes: Attribute,
    ) {
        self.base()
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<AvmString<'gc>>,
        set_attributes: Attribute,
        clear_attributes: Attribute,
    ) {
        self.base()
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

    fn proto(&self, activation: &mut Activation<'_, 'gc, '_>) -> Value<'gc> {
        self.base().proto(activation)
    }

    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        self.base().has_property(activation, name)
    }

    fn has_own_property(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.document().get_node_by_id(name).is_some()
            || self.base().has_own_property(activation, name)
    }

    fn has_own_virtual(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.base().has_own_virtual(activation, name)
    }

    fn is_property_enumerable(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.base().is_property_enumerable(activation, name)
    }

    fn get_keys(&self, activation: &mut Activation<'_, 'gc, '_>) -> Vec<AvmString<'gc>> {
        let mut keys = self.base().get_keys(activation);
        keys.extend(self.document().get_node_ids().into_iter());
        keys
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
        None
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
