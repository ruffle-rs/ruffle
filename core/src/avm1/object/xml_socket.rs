//! AVM1 object type to represent XML nodes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::TObject;
use crate::avm1::{Object, ScriptObject};
use crate::{avm_warn, impl_custom_object};
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A ScriptObject that is inherently tied to an XML node.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XmlSocket<'gc>(GcCell<'gc, XmlSocketData<'gc>>);

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct XmlSocketData<'gc> {
    base: ScriptObject<'gc>,
    pub id: u64,
}

impl<'gc> XmlSocket<'gc> {
    pub fn empty_socket(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
        id: u64,
    ) -> Object<'gc> {
        let base_object = ScriptObject::object(gc_context, proto);
        let object = XmlSocket(GcCell::allocate(
            gc_context,
            XmlSocketData {
                base: base_object,
                id,
            },
        ))
        .into();

        object
    }
}

impl fmt::Debug for XmlSocket<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("XmlSocket")
            .field("base", &this.base)
            .field("id", &this.id)
            .finish()
    }
}

impl<'gc> TObject<'gc> for XmlSocket<'gc> {
    impl_custom_object!(base);

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        avm_warn!(activation, "XMLSocket::create_bare_object()");
        activation.context.xml_socket.current_socket_id += 1;
        Ok(XmlSocket::empty_socket(
            activation.context.gc_context,
            Some(this),
            activation.context.xml_socket.current_socket_id,
        ))
    }

    fn as_xml_socket(&self) -> Option<XmlSocketData<'gc>> {
        Some(*self.0.read())
    }
}
