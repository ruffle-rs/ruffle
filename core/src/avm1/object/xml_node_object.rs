//! AVM1 object type to represent XML nodes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::{Object, ScriptObject};
use crate::impl_custom_object;
use crate::xml::{XmlNode, TEXT_NODE};
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
    /// Construct an XmlNodeObject for an already existing node.
    pub fn from_xml_node(
        gc_context: MutationContext<'gc, '_>,
        mut node: XmlNode<'gc>,
        proto: Object<'gc>,
    ) -> Self {
        let object = Self(GcCell::allocate(
            gc_context,
            XmlNodeObjectData {
                base: ScriptObject::new(gc_context, Some(proto)),
                node,
            },
        ));
        node.introduce_script_object(gc_context, object.into());
        object
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
        Ok(Self::from_xml_node(
            activation.context.gc_context,
            XmlNode::new(activation.context.gc_context, TEXT_NODE, Some("".into())),
            this,
        )
        .into())
    }

    fn as_xml_node(&self) -> Option<XmlNode<'gc>> {
        Some(self.0.read().node)
    }
}
