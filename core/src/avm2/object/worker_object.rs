use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::TObject;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct WorkerObject<'gc>(pub Gc<'gc, WorkerObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct WorkerObjectWeak<'gc>(pub GcWeak<'gc, WorkerObjectData<'gc>>);

impl fmt::Debug for WorkerObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct WorkerObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,
}

impl<'gc> TObject<'gc> for WorkerObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl<'gc> WorkerObject<'gc> {
    pub fn new(activation: &mut Activation<'_, 'gc>) -> Self {
        let class = activation.avm2().classes().worker;
        let base = ScriptObjectData::new(class);
        WorkerObject(Gc::new(activation.gc(), WorkerObjectData { base }))
    }
}
