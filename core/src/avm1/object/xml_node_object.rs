//! AVM1 object type to represent XML nodes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::{Object, ScriptObject};
use crate::impl_custom_object;
use crate::string::AvmString;
use crate::xml::{XmlDocument, XmlNode};
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A ScriptObject that is inherently tied to an XML node.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XmlNodeObject<'gc>(GcCell<'gc, XmlNodeObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlNodeObjectData<'gc> {
    base: ScriptObject<'gc>,
    node: XmlNode<'gc>,
}

impl<'gc> XmlNodeObject<'gc> {
    /// Construct a new XML node and object pair.
    pub fn empty_node(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let empty_document = XmlDocument::new(gc_context);
        let mut xml_node = XmlNode::new_text(gc_context, AvmString::default(), empty_document);
        let base_object = ScriptObject::object(gc_context, proto);
        let object = XmlNodeObject(GcCell::allocate(
            gc_context,
            XmlNodeObjectData {
                base: base_object,
                node: xml_node,
            },
        ))
        .into();

        xml_node.introduce_script_object(gc_context, object);

        object
    }

    /// Construct an XmlNodeObject for an already existing node.
    pub fn from_xml_node(
        gc_context: MutationContext<'gc, '_>,
        xml_node: XmlNode<'gc>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        XmlNodeObject(GcCell::allocate(
            gc_context,
            XmlNodeObjectData {
                base: ScriptObject::object(gc_context, proto),
                node: xml_node,
            },
        ))
        .into()
    }
}

impl fmt::Debug for XmlNodeObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("XmlNodeObject")
            .field("base", &this.base)
            .field("node", &this.node)
            .finish()
    }
}

impl<'gc> TObject<'gc> for XmlNodeObject<'gc> {
    impl_custom_object!(base);

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(XmlNodeObject::empty_node(
            activation.context.gc_context,
            Some(this),
        ))
    }

    fn as_xml_node(&self) -> Option<XmlNode<'gc>> {
        Some(self.0.read().node)
    }
}
