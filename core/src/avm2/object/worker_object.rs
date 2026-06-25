use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::error::{make_error_1511, make_error_3732};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{EventObject, TObject};
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
    Spawned {
        state: Cell<WorkerState>,
        /// Whether `start()` has been called. Kept separate from `state`
        /// because Flash boots the worker thread asynchronously: right after
        /// `start()` the public `state` is still `new`, but `terminate()`
        /// already succeeds.
        started: Cell<bool>,
        /// The state last reported through a `workerState` event.
        announced_state: Cell<WorkerState>,
    },
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
                started: Cell::new(false),
                announced_state: Cell::new(WorkerState::New),
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
            WorkerKind::Spawned { state, .. } => state.get(),
        }
    }

    /// Mark the worker as started. Matches Flash: the primordial worker and
    /// workers that were already started (including terminated ones) throw
    /// `Error #1511`.
    pub fn start(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        match &self.0.kind {
            WorkerKind::Spawned { started, .. } if !started.get() => {
                started.set(true);
                Ok(())
            }
            _ => Err(make_error_1511(activation)),
        }
    }

    /// Attempt to terminate the worker. Returns `true` if the state changed
    /// to `Terminated`. Matches Flash: a worker that was never `start()`ed
    /// cannot be terminated, a terminated worker can't be terminated again,
    /// and the primordial worker throws `Error #3732`.
    pub fn terminate(self, activation: &mut Activation<'_, 'gc>) -> Result<bool, Error<'gc>> {
        let WorkerKind::Spawned { state, started, .. } = &self.0.kind else {
            return Err(make_error_3732(activation));
        };

        if !started.get() || state.get() == WorkerState::Terminated {
            return Ok(false);
        }

        state.set(WorkerState::Terminated);
        Ok(true)
    }

    /// Moves a started worker from `New` to `Running`, and dispatches a
    /// `workerState` event if its state changed since the last one.
    pub fn update_state(self, context: &mut UpdateContext<'gc>) {
        let WorkerKind::Spawned {
            state,
            started,
            announced_state,
        } = &self.0.kind
        else {
            return;
        };

        if started.get() && state.get() == WorkerState::New {
            state.set(WorkerState::Running);
        }

        if announced_state.replace(state.get()) != state.get() {
            let event = EventObject::bare_default_event(context, "workerState");

            Avm2::dispatch_event(context, event, self.into());
        }
    }
}
