use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::TObject;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct MessageChannelObject<'gc>(pub Gc<'gc, MessageChannelObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct MessageChannelObjectWeak<'gc>(pub GcWeak<'gc, MessageChannelObjectData<'gc>>);

impl fmt::Debug for MessageChannelObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageChannelObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct MessageChannelObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,
}

impl<'gc> TObject<'gc> for MessageChannelObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl<'gc> MessageChannelObject<'gc> {
    pub fn new(activation: &mut Activation<'_, 'gc>) -> Self {
        let class = activation.avm2().classes().messagechannel;
        let base = ScriptObjectData::new(class);
        MessageChannelObject(Gc::new(activation.gc(), MessageChannelObjectData { base }))
    }
}
