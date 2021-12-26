//! AVM1 object type to represent XML documents

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::{Object, ScriptObject};
use crate::impl_custom_object;
use crate::xml::{XmlDocument, XmlNode};
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A ScriptObject that is inherently tied to an XML document.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XmlObject<'gc>(GcCell<'gc, XmlObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlObjectData<'gc> {
    base: ScriptObject<'gc>,
    document: XmlDocument<'gc>,
}

impl<'gc> XmlObject<'gc> {
    /// Construct a new XML document and object pair.
    pub fn empty(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        let document = XmlDocument::new(gc_context);
        let object = Self(GcCell::allocate(
            gc_context,
            XmlObjectData {
                base: ScriptObject::object(gc_context, proto),
                document,
            },
        ));

        document
            .as_node()
            .introduce_script_object(gc_context, object.into());

        object
    }
}

impl fmt::Debug for XmlObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("XmlObject")
            .field("base", &this.base)
            .field("document", &this.document)
            .finish()
    }
}

impl<'gc> TObject<'gc> for XmlObject<'gc> {
    impl_custom_object!(base);

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(Self::empty(activation.context.gc_context, Some(this)).into())
    }

    fn as_xml(&self) -> Option<XmlDocument<'gc>> {
        Some(self.0.read().document)
    }

    fn as_xml_node(&self) -> Option<XmlNode<'gc>> {
        Some(self.0.read().document.as_node())
    }
}
