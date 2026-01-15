//! Object representation for Stage3D objects

use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, TObject};
use crate::context::UpdateContext;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use std::cell::Cell;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Stage3DObject<'gc>(pub Gc<'gc, Stage3DObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct Stage3DObjectWeak<'gc>(pub GcWeak<'gc, Stage3DObjectData<'gc>>);

impl fmt::Debug for Stage3DObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage3DObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

impl<'gc> Stage3DObject<'gc> {
    pub fn new(context: &mut UpdateContext<'gc>) -> Self {
        let class = context.avm2.classes().stage3d;
        Stage3DObject(Gc::new(
            context.gc(),
            Stage3DObjectData {
                base: ScriptObjectData::new(class),
                context3d: Lock::new(None),
                visible: Cell::new(true),
            },
        ))
    }

    pub fn context3d(self) -> Option<Object<'gc>> {
        self.0.context3d.get()
    }

    pub fn set_context3d(self, context3d: Option<Object<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), Stage3DObjectData, context3d).set(context3d)
    }

    pub fn visible(self) -> bool {
        self.0.visible.get()
    }

    pub fn set_visible(self, visible: bool) {
        self.0.visible.set(visible);
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct Stage3DObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The context3D object associated with this Stage3D object,
    /// if it's been created with `requestContext3D`
    context3d: Lock<Option<Object<'gc>>>,
    visible: Cell<bool>,
}

impl<'gc> TObject<'gc> for Stage3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
