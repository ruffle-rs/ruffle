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
pub enum XMLObject<'gc> {
    /// An `XMLObject` that references a whole document.
    Document(ScriptObject<'gc>, XMLDocument<'gc>),

    /// An `XMLObject` that references a specific node of another document.
    Node(ScriptObject<'gc>, XMLNode<'gc>),
}

impl<'gc> XMLObject<'gc> {
    fn empty_document(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> XMLObject<'gc> {
        XMLObject::Document(
            ScriptObject::object(gc_context, proto),
            XMLDocument::new(gc_context),
        )
    }

    fn empty_node(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> XMLObject<'gc> {
        XMLObject::Node(
            ScriptObject::object(gc_context, proto),
            XMLNode::new_text(gc_context, ""),
        )
    }

    fn base(&self) -> ScriptObject<'gc> {
        match self {
            XMLObject::Document(base, ..) => *base,
            XMLObject::Node(base, ..) => *base,
        }
    }
}

impl fmt::Debug for XMLObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XMLObject::Document(base, ..) => {
                f.debug_tuple("XMLObject::Document")
                    .field(base)
                    //.field(document)
                    .finish()
            }
            XMLObject::Node(base, ..) => {
                f.debug_tuple("XMLObject::Node")
                    .field(base)
                    //.field(document)
                    .finish()
            }
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
        match self {
            XMLObject::Document(..) => {
                Ok(XMLObject::empty_document(context.gc_context, Some(this)).into())
            }
            XMLObject::Node(..) => Ok(XMLObject::empty_node(context.gc_context, Some(this)).into()),
        }
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

    fn has_property(&self, name: &str) -> bool {
        self.base().has_property(name)
    }

    fn has_own_property(&self, name: &str) -> bool {
        self.base().has_own_property(name)
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

    fn as_xml_document(&self) -> Option<XMLDocument<'gc>> {
        match self {
            XMLObject::Document(_base, document) => Some(*document),
            _ => None,
        }
    }

    fn as_xml_node(&self) -> Option<XMLNode<'gc>> {
        match self {
            XMLObject::Node(_base, node) => Some(*node),
            _ => None,
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
