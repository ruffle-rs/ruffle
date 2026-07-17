use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_3732;
use crate::avm2::object::TObject;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::context::UpdateContext;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::Avm2Enum;
use std::cell::Cell;

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

    kind: WorkerKind,
}

impl<'gc> TObject<'gc> for WorkerObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

/// Distinguishes the primordial worker (main SWF runtime) from workers created
/// via `WorkerDomain.createWorker`. Only `Spawned` carries lifecycle state;
/// the primordial worker is permanently `Running`.
#[derive(Collect)]
#[collect(require_static)]
pub enum WorkerKind {
    Primordial,
    Spawned { state: Cell<WorkerState> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Avm2Enum)]
pub enum WorkerState {
    #[avm2_variant("new")]
    New,
    #[avm2_variant("running")]
    Running,
    #[avm2_variant("terminated")]
    Terminated,
}

impl<'gc> WorkerObject<'gc> {
    pub fn new_regular(activation: &mut Activation<'_, 'gc>) -> Self {
        Self::new(
            activation.context,
            WorkerKind::Spawned {
                state: Cell::new(WorkerState::New),
            },
        )
    }

    pub fn new_primordial(context: &mut UpdateContext<'gc>) -> Self {
        Self::new(context, WorkerKind::Primordial)
    }

    fn new(context: &mut UpdateContext<'gc>, kind: WorkerKind) -> Self {
        let class = context.avm2.classes().worker;
        let base = ScriptObjectData::new(class);

        Self(Gc::new(context.gc(), WorkerObjectData { base, kind }))
    }

    pub fn is_primordial(self) -> bool {
        matches!(self.0.kind, WorkerKind::Primordial)
    }

    pub fn state(self) -> WorkerState {
        match &self.0.kind {
            WorkerKind::Primordial => WorkerState::Running,
            WorkerKind::Spawned { state } => state.get(),
        }
    }

    /// Transition from `New` to `Running`. Returns `true` if the state
    /// changed. Always returns `false` for the primordial worker, which is
    /// already `Running`.
    pub fn start(self) -> bool {
        let WorkerKind::Spawned { state } = &self.0.kind else {
            return false;
        };

        let mut changed = false;

        state.update(|s| match s {
            WorkerState::New => {
                changed = true;
                WorkerState::Running
            }
            s => s,
        });

        changed
    }

    /// Attempt to transition `Running` → `Terminated`. Returns `true` if the state
    /// changed. Matches Flash: a `New` worker that was never started cannot
    /// be terminated, and the primordial worker throws `Error #3732`.
    pub fn terminate(self, activation: &mut Activation<'_, 'gc>) -> Result<bool, Error<'gc>> {
        let state = match &self.0.kind {
            WorkerKind::Primordial => return Err(make_error_3732(activation)),
            WorkerKind::Spawned { state } => state,
        };

        let mut changed = false;

        state.update(|s| match s {
            WorkerState::Running => {
                changed = true;
                WorkerState::Terminated
            }
            s => s,
        });

        Ok(changed)
    }
}
