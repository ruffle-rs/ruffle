use core::fmt;

use gc_arena::{Collect, GcCell, MutationContext};

use crate::{
    avm1::{Activation, Error, ScriptObject},
    impl_custom_object,
    socket::XmlSocketHandle,
};

use super::{Object, TObject};

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XmlSocketObject<'gc>(GcCell<'gc, XmlSocketObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlSocketObjectData<'gc> {
    base: ScriptObject<'gc>,
    #[collect(require_static)]
    handle: Option<XmlSocketHandle>,
}

impl<'gc> XmlSocketObject<'gc> {
    pub fn empty(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Object<'gc> {
        Self(GcCell::allocate(
            gc_context,
            XmlSocketObjectData {
                base: ScriptObject::new(gc_context, proto),
                handle: None,
            },
        ))
        .into()
    }

    pub fn handle(&self) -> Option<XmlSocketHandle> {
        self.0.read().handle
    }

    pub fn set_handle(
        &self,
        gc_context: MutationContext<'gc, '_>,
        handle: XmlSocketHandle,
    ) -> Option<XmlSocketHandle> {
        std::mem::replace(&mut self.0.write(gc_context).handle, Some(handle))
    }
}

impl fmt::Debug for XmlSocketObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("XmlSocketObject")
            .field("base", &this.base)
            .field("handle", &this.handle)
            .finish()
    }
}

impl<'gc> TObject<'gc> for XmlSocketObject<'gc> {
    impl_custom_object!(base);

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(Self(GcCell::allocate(
            activation.context.gc_context,
            XmlSocketObjectData {
                base: ScriptObject::new(activation.context.gc_context, Some(this)),
                handle: None,
            },
        ))
        .into())
    }

    fn as_xml_socket(&self) -> Option<XmlSocketObject<'gc>> {
        Some(*self)
    }
}
