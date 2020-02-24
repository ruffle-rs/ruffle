//! AVM1 object type to represent XML nodes

use crate::avm1::function::Executable;
use crate::avm1::object::{ObjectPtr, TObject};
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, UpdateContext, Value};
use crate::xml::{XMLDocument, XMLNode};
use enumset::EnumSet;
use gc_arena::{Collect, MutationContext};
use std::collections::HashSet;
use std::fmt;

/// A ScriptObject that is inherently tied to an XML node.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XMLObject<'gc>(ScriptObject<'gc>, XMLNode<'gc>);

impl<'gc> XMLObject<'gc> {
    /// Construct a new XML node and object pair.
    pub fn empty_node(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let empty_document = XMLDocument::new(gc_context);
        let mut xml_node = XMLNode::new_text(gc_context, "", empty_document);
        let base_object = ScriptObject::object(gc_context, proto);
        let object = XMLObject(base_object, xml_node).into();

        xml_node.introduce_script_object(gc_context, object);

        object
    }

    /// Construct an XMLObject for an already existing node.
    pub fn from_xml_node(
        gc_context: MutationContext<'gc, '_>,
        xml_node: XMLNode<'gc>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        XMLObject(ScriptObject::object(gc_context, proto), xml_node).into()
    }

    fn base(&self) -> ScriptObject<'gc> {
        match self {
            XMLObject(base, ..) => *base,
        }
    }
}

impl fmt::Debug for XMLObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XMLObject(base, node) => f.debug_tuple("XMLObject").field(base).field(node).finish(),
        }
    }
}

impl<'gc> TObject<'gc> for XMLObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base().get_local(name, avm, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.base().set(name, value, avm, context)
    }

    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base().call(avm, context, this, args)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        _avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        Ok(XMLObject::empty_node(context.gc_context, Some(this)))
    }

    fn delete(&self, gc_context: MutationContext<'gc, '_>, name: &str) -> bool {
        self.base().delete(gc_context, name)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .add_property(gc_context, name, get, set, attributes)
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: EnumSet<Attribute>,
        clear_attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.base().proto()
    }

    fn has_property(&self, context: &mut UpdateContext<'_, 'gc, '_>, name: &str) -> bool {
        self.base().has_property(context, name)
    }

    fn has_own_property(&self, context: &mut UpdateContext<'_, 'gc, '_>, name: &str) -> bool {
        self.base().has_own_property(context, name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.base().is_property_overwritable(name)
    }

    fn is_property_enumerable(&self, name: &str) -> bool {
        self.base().is_property_enumerable(name)
    }

    fn get_keys(&self) -> HashSet<String> {
        self.base().get_keys()
    }

    fn as_string(&self) -> String {
        self.base().as_string()
    }

    fn type_of(&self) -> &'static str {
        self.base().type_of()
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.base().interfaces()
    }

    fn set_interfaces(&mut self, context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.base().set_interfaces(context, iface_list)
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.base())
    }

    fn as_xml_node(&self) -> Option<XMLNode<'gc>> {
        match self {
            XMLObject(_base, node) => Some(*node),
        }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.base().as_ptr() as *const ObjectPtr
    }

    fn length(&self) -> usize {
        self.base().length()
    }

    fn array(&self) -> Vec<Value<'gc>> {
        self.base().array()
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, length: usize) {
        self.base().set_length(gc_context, length)
    }

    fn array_element(&self, index: usize) -> Value<'gc> {
        self.base().array_element(index)
    }

    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        self.base().set_array_element(index, value, gc_context)
    }

    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>) {
        self.base().delete_array_element(index, gc_context)
    }
}
