use crate::avm1::globals::create_globals;
use crate::avm1::object::search_prototype;
use crate::context::UpdateContext;
use crate::prelude::*;
use gc_arena::{GcCell, MutationContext};

use swf::avm1::read::Reader;

use crate::display_object::DisplayObject;
use crate::tag_utils::SwfSlice;

#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
pub mod listeners;

pub mod activation;
pub mod color_transform_object;
pub mod debug;
pub mod error;
mod fscommand;
pub mod function;
pub mod globals;
pub mod object;
mod property;
mod scope;
pub mod script_object;
pub mod shared_object;
mod sound_object;
mod stage_object;
mod super_object;
mod value;
mod value_object;
pub mod xml_attributes_object;
pub mod xml_idmap_object;
pub mod xml_object;

#[cfg(test)]
mod tests;

use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::error::Error;
use crate::avm1::listeners::SystemListener;
pub use globals::SystemPrototypes;
pub use object::{Object, ObjectPtr, TObject};
use scope::Scope;
pub use script_object::ScriptObject;
use smallvec::alloc::borrow::Cow;
pub use sound_object::SoundObject;
pub use stage_object::StageObject;
pub use value::Value;

macro_rules! avm_debug {
    ($($arg:tt)*) => (
        #[cfg(feature = "avm_debug")]
        log::debug!($($arg)*)
    )
}

pub struct Avm1<'gc> {
    /// The Flash Player version we're emulating.
    player_version: u8,

    /// The constant pool to use for new activations from code sources that
    /// don't close over the constant pool they were defined with.
    constant_pool: GcCell<'gc, Vec<String>>,

    /// The global object.
    globals: Object<'gc>,

    /// System builtins that we use internally to construct new objects.
    prototypes: globals::SystemPrototypes<'gc>,

    /// System event listeners that will respond to native events (Mouse, Key, etc)
    system_listeners: listeners::SystemListeners<'gc>,

    /// DisplayObject property map.
    display_properties: GcCell<'gc, stage_object::DisplayPropertyMap<'gc>>,

    /// The operand stack (shared across functions).
    stack: Vec<Value<'gc>>,

    /// The register slots (also shared across functions).
    /// `ActionDefineFunction2` defined functions do not use these slots.
    registers: [Value<'gc>; 4],

    /// If a serious error has occured, or a user has requested it, the AVM may be halted.
    /// This will completely prevent any further actions from being executed.
    halted: bool,
}

unsafe impl<'gc> gc_arena::Collect for Avm1<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.globals.trace(cc);
        self.constant_pool.trace(cc);
        self.system_listeners.trace(cc);
        self.prototypes.trace(cc);
        self.display_properties.trace(cc);
        self.stack.trace(cc);

        for register in &self.registers {
            register.trace(cc);
        }
    }
}

impl<'gc> Avm1<'gc> {
    pub fn new(gc_context: MutationContext<'gc, '_>, player_version: u8) -> Self {
        let (prototypes, globals, system_listeners) = create_globals(gc_context);

        Self {
            player_version,
            constant_pool: GcCell::allocate(gc_context, vec![]),
            globals,
            prototypes,
            system_listeners,
            display_properties: stage_object::DisplayPropertyMap::new(gc_context),
            stack: vec![],
            registers: [
                Value::Undefined,
                Value::Undefined,
                Value::Undefined,
                Value::Undefined,
            ],
            halted: false,
        }
    }

    /// Add a stack frame that executes code in timeline scope
    ///
    /// This creates a new frame stack.
    pub fn run_stack_frame_for_action<S: Into<Cow<'static, str>>>(
        &mut self,
        active_clip: DisplayObject<'gc>,
        name: S,
        swf_version: u8,
        code: SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        if self.halted {
            // We've been told to ignore all future execution.
            return;
        }

        let mut parent_activation = Activation::from_nothing(
            self,
            ActivationIdentifier::root("[Actions Parent]"),
            swf_version,
            self.global_object_cell(),
            context.gc_context,
            active_clip,
        );

        let clip_obj = active_clip
            .object()
            .coerce_to_object(&mut parent_activation, context);
        let child_scope = GcCell::allocate(
            context.gc_context,
            Scope::new(
                parent_activation.scope_cell(),
                scope::ScopeClass::Target,
                clip_obj,
            ),
        );
        let constant_pool = parent_activation.avm.constant_pool;
        let mut child_activation = Activation::from_action(
            parent_activation.avm,
            parent_activation.id.child(name),
            swf_version,
            child_scope,
            constant_pool,
            active_clip,
            clip_obj,
            None,
        );
        if let Err(e) = child_activation.run_actions(context, code) {
            root_error_handler(&mut child_activation, context, e);
        }
    }

    /// Add a stack frame that executes code in initializer scope.
    ///
    /// This creates a new frame stack.
    pub fn run_with_stack_frame_for_display_object<'a, F, R>(
        &mut self,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        action_context: &mut UpdateContext<'a, 'gc, '_>,
        function: F,
    ) -> R
    where
        for<'b> F: FnOnce(&mut Activation<'b, 'gc>, &mut UpdateContext<'a, 'gc, '_>) -> R,
    {
        let clip_obj = match active_clip.object() {
            Value::Object(o) => o,
            _ => panic!("No script object for display object"),
        };
        let global_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::from_global_object(self.globals),
        );
        let child_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        let mut activation = Activation::from_action(
            self,
            ActivationIdentifier::root("[Display Object]"),
            swf_version,
            child_scope,
            self.constant_pool,
            active_clip,
            clip_obj,
            None,
        );
        function(&mut activation, action_context)
    }

    /// Add a stack frame that executes code in initializer scope.
    ///
    /// This creates a new frame stack.
    pub fn run_stack_frame_for_init_action(
        &mut self,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        code: SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        if self.halted {
            // We've been told to ignore all future execution.
            return;
        }

        let mut parent_activation = Activation::from_nothing(
            self,
            ActivationIdentifier::root("[Init Parent]"),
            swf_version,
            self.global_object_cell(),
            context.gc_context,
            active_clip,
        );

        let clip_obj = active_clip
            .object()
            .coerce_to_object(&mut parent_activation, context);
        let child_scope = GcCell::allocate(
            context.gc_context,
            Scope::new(
                parent_activation.scope_cell(),
                scope::ScopeClass::Target,
                clip_obj,
            ),
        );
        parent_activation.avm.push(Value::Undefined);
        let constant_pool = parent_activation.avm.constant_pool;
        let mut child_activation = Activation::from_action(
            parent_activation.avm,
            parent_activation.id.child("[Init]"),
            swf_version,
            child_scope,
            constant_pool,
            active_clip,
            clip_obj,
            None,
        );
        if let Err(e) = child_activation.run_actions(context, code) {
            root_error_handler(&mut child_activation, context, e);
        }
    }

    /// Add a stack frame that executes code in timeline scope for an object
    /// method, such as an event handler.
    ///
    /// This creates a new frame stack.
    pub fn run_stack_frame_for_method(
        &mut self,
        active_clip: DisplayObject<'gc>,
        obj: Object<'gc>,
        swf_version: u8,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
        args: &[Value<'gc>],
    ) {
        if self.halted {
            // We've been told to ignore all future execution.
            return;
        }

        let mut activation = Activation::from_nothing(
            self,
            ActivationIdentifier::root(name.to_owned()),
            swf_version,
            self.global_object_cell(),
            context.gc_context,
            active_clip,
        );

        let search_result =
            search_prototype(Some(obj), name, &mut activation, context, obj).map(|r| (r.0, r.1));

        if let Ok((callback, base_proto)) = search_result {
            let _ = callback.call(name, &mut activation, context, obj, base_proto, args);
        }
    }

    pub fn notify_system_listeners(
        &mut self,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        context: &mut UpdateContext<'_, 'gc, '_>,
        listener: SystemListener,
        method: &str,
        args: &[Value<'gc>],
    ) {
        let mut activation = Activation::from_nothing(
            self,
            ActivationIdentifier::root("[System Listeners]"),
            swf_version,
            self.global_object_cell(),
            context.gc_context,
            active_clip,
        );

        let listeners = activation.avm.system_listeners.get(listener);
        let mut handlers = listeners.prepare_handlers(&mut activation, context, method);

        for (listener, handler) in handlers.drain(..) {
            let _ = handler.call(method, &mut activation, context, listener, None, &args);
        }
    }

    /// Halts the AVM, preventing execution of any further actions.
    ///
    /// If the AVM is currently evaluating an action, it will continue until it realizes that it has
    /// been halted. If an immediate stop is required, an Error must be raised inside of the execution.
    ///
    /// This is most often used when serious errors or infinite loops are encountered.
    pub fn halt(&mut self) {
        if !self.halted {
            self.halted = true;
            log::error!("No more actions will be executed in this movie.")
        }
    }

    fn push(&mut self, value: impl Into<Value<'gc>>) {
        let value = value.into();
        avm_debug!("Stack push {}: {:?}", self.stack.len(), value);
        self.stack.push(value);
    }

    #[allow(clippy::let_and_return)]
    fn pop(&mut self) -> Value<'gc> {
        let value = self.stack.pop().unwrap_or_else(|| {
            log::warn!("Avm1::pop: Stack underflow");
            Value::Undefined
        });

        avm_debug!("Stack pop {}: {:?}", self.stack.len(), value);

        value
    }

    /// Obtain the value of `_global`.
    pub fn global_object(&self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Value<'gc> {
        Value::Object(self.globals)
    }

    /// Obtain a reference to `_global`.
    pub fn global_object_cell(&self) -> Object<'gc> {
        self.globals
    }

    /// Obtain system built-in prototypes for this instance.
    pub fn prototypes(&self) -> &globals::SystemPrototypes<'gc> {
        &self.prototypes
    }
}

pub fn root_error_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    error: Error<'gc>,
) {
    if let Error::ThrownValue(error) = &error {
        let string = error
            .coerce_to_string(activation, context)
            .unwrap_or_else(|_| Cow::Borrowed("undefined"));
        log::info!(target: "avm_trace", "{}", string);
    } else {
        log::error!("{}", error);
    }
    if error.is_halting() {
        activation.avm.halt();
    }
}

/// Utility function used by `Avm1::action_wait_for_frame` and
/// `Avm1::action_wait_for_frame_2`.
fn skip_actions(reader: &mut Reader<'_>, num_actions_to_skip: u8) {
    for _ in 0..num_actions_to_skip {
        if let Err(e) = reader.read_action() {
            log::warn!("Couldn't skip action: {}", e);
        }
    }
}

/// Starts draggining this display object, making it follow the cursor.
/// Runs via the `startDrag` method or `StartDrag` AVM1 action.
pub fn start_drag<'gc>(
    display_object: DisplayObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    args: &[Value<'gc>],
) {
    let lock_center = args
        .get(0)
        .map(|o| o.as_bool(context.swf.version()))
        .unwrap_or(false);

    let offset = if lock_center {
        // The object's origin point is locked to the mouse.
        Default::default()
    } else {
        // The object moves relative to current mouse position.
        // Calculate the offset from the mouse to the object in world space.
        let obj_pos = display_object.local_to_global(Default::default());
        (
            obj_pos.0 - context.mouse_position.0,
            obj_pos.1 - context.mouse_position.1,
        )
    };

    let constraint = if args.len() > 1 {
        // Invalid values turn into 0.
        let mut x_min = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation, context)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut y_min = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation, context)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut x_max = args
            .get(3)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation, context)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut y_max = args
            .get(4)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation, context)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();

        // Normalize the bounds.
        if x_max.get() < x_min.get() {
            std::mem::swap(&mut x_min, &mut x_max);
        }
        if y_max.get() < y_min.get() {
            std::mem::swap(&mut y_min, &mut y_max);
        }
        BoundingBox {
            valid: true,
            x_min,
            y_min,
            x_max,
            y_max,
        }
    } else {
        // No constraints.
        Default::default()
    };

    let drag_object = crate::player::DragObject {
        display_object,
        offset,
        constraint,
    };
    *context.drag_object = Some(drag_object);
}
