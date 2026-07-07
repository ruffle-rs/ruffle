//! Object representation for Stage3D objects

use crate::avm2::Avm2;
use crate::avm2::object::kind;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Context3DObject, EventObject, TObject};
use crate::context::UpdateContext;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::backend::Context3DProfile;
use std::cell::Cell;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Stage3DObject<'gc>(pub Gc<'gc, Stage3DObjectData<'gc>>);

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
                context3d_status: Lock::new(Context3DStatus::None),
                visible: Cell::new(true),
                x: Cell::new(0.0),
                y: Cell::new(0.0),
            },
        ))
    }

    pub fn context3d(self) -> Option<Context3DObject<'gc>> {
        match self.0.context3d_status.get() {
            Context3DStatus::Ready(object) => Some(object),
            _ => None,
        }
    }

    pub fn set_requesting_context3d(self, mc: &Mutation<'gc>, profile: Context3DProfile) {
        self.set_status(mc, Context3DStatus::Requested { profile });
    }

    pub fn clear_context3d(self, mc: &Mutation<'gc>) {
        self.set_status(mc, Context3DStatus::None);
    }

    pub fn update_context3d_status(self, context: &mut UpdateContext<'gc>) {
        if let Context3DStatus::Requested { profile } = self.0.context3d_status.get() {
            let context3d = match context.renderer.create_context3d(profile) {
                Ok(context3d) => context3d,
                Err(err) => {
                    tracing::error!("Failed to create Context3d: {}", err);
                    // TODO the docs say FP dispatches an "error" event here
                    return;
                }
            };

            let context3d_obj = Context3DObject::from_context(context, context3d, self);
            self.set_status(context.gc(), Context3DStatus::Ready(context3d_obj));

            let event = EventObject::bare_default_event(context, "context3DCreate");

            Avm2::dispatch_event(context, event, self.into());
        }
    }

    fn set_status(self, mc: &Mutation<'gc>, status: Context3DStatus<'gc>) {
        unlock!(Gc::write(mc, self.0), Stage3DObjectData, context3d_status).set(status);
    }

    pub fn visible(self) -> bool {
        self.0.visible.get()
    }

    pub fn set_visible(self, visible: bool) {
        self.0.visible.set(visible);
    }

    pub fn x(self) -> f64 {
        self.0.x.get()
    }

    pub fn set_x(self, x: f64) {
        self.0.x.set(x);
    }

    pub fn y(self) -> f64 {
        self.0.y.get()
    }

    pub fn set_y(self, y: f64) {
        self.0.y.set(y);
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct Stage3DObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc, kind::Stage3DObject>,

    /// The state context3D object associated with this Stage3D object.
    context3d_status: Lock<Context3DStatus<'gc>>,

    visible: Cell<bool>,

    x: Cell<f64>,
    y: Cell<f64>,
}

impl<'gc> TObject<'gc> for Stage3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        ScriptObjectData::erase_kind(HasPrefixField::as_prefix_gc(self.0))
    }
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub enum Context3DStatus<'gc> {
    None,
    Requested {
        #[collect(require_static)]
        profile: Context3DProfile,
    },
    Ready(Context3DObject<'gc>),
}
