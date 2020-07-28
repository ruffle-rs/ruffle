//! AVM1 object type to represent XML nodes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::{Object, ScriptObject, UpdateContext, Value};
use crate::impl_custom_object;
use crate::xml::{XMLDocument, XMLNode};
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A ScriptObject that is inherently tied to an XML node.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XMLObject<'gc>(GcCell<'gc, XMLObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XMLObjectData<'gc> {
    base: ScriptObject<'gc>,
    node: XMLNode<'gc>,
}

impl<'gc> XMLObject<'gc> {
    /// Construct a new XML node and object pair.
    pub fn empty_node(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let empty_document = XMLDocument::new(gc_context);
        let mut xml_node = XMLNode::new_text(gc_context, "", empty_document);
        let base_object = ScriptObject::object(gc_context, proto);
        let object = XMLObject(GcCell::allocate(
            gc_context,
            XMLObjectData {
                base: base_object,
                node: xml_node,
            },
        ))
        .into();

        xml_node.introduce_script_object(gc_context, object);

        object
    }

    /// Construct an XMLObject for an already existing node.
    pub fn from_xml_node(
        gc_context: MutationContext<'gc, '_>,
        xml_node: XMLNode<'gc>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        XMLObject(GcCell::allocate(
            gc_context,
            XMLObjectData {
                base: ScriptObject::object(gc_context, proto),
                node: xml_node,
            },
        ))
        .into()
    }
}

impl fmt::Debug for XMLObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("XMLObject")
            .field("base", &this.base)
            .field("node", &this.node)
            .finish()
    }
}

impl<'gc> TObject<'gc> for XMLObject<'gc> {
    impl_custom_object!(base);

    #[allow(clippy::new_ret_no_self)]
    fn create_bare_object(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(XMLObject::empty_node(context.gc_context, Some(this)))
    }

    fn as_xml_node(&self) -> Option<XMLNode<'gc>> {
        Some(self.0.read().node)
    }
}
