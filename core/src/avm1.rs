use crate::avm1::function::FunctionObject;
use crate::avm1::globals::create_globals;
use crate::avm1::object::stage_object;
use crate::avm1::property_map::PropertyMap;
use crate::context::UpdateContext;
use crate::prelude::*;
use gc_arena::{Collect, GcCell, MutationContext};

use swf::avm1::read::Reader;

use crate::display_object::DisplayObject;
use crate::tag_utils::SwfSlice;

#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
pub mod function;
#[macro_use]
pub mod property_decl;

pub mod activation;
mod callable_value;
pub mod debug;
pub mod error;
mod fscommand;
pub mod globals;
pub mod object;
pub mod property;
pub mod property_map;
mod scope;
mod timer;
mod value;

#[cfg(test)]
mod tests;

use crate::avm1::activation::{Activation, ActivationIdentifier};
pub use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::string::AvmString;
pub use globals::SystemPrototypes;
pub use object::array_object::ArrayObject;
pub use object::script_object::ScriptObject;
pub use object::sound_object::SoundObject;
pub use object::stage_object::StageObject;
pub use object::{Object, ObjectPtr, TObject};
use scope::Scope;
use smallvec::alloc::borrow::Cow;
pub use timer::Timers;
pub use value::Value;

macro_rules! avm_debug {
    ($avm: expr, $($arg:tt)*) => (
        if $avm.show_debug_output() {
            log::debug!($($arg)*)
        }
    )
}

#[macro_export]
macro_rules! avm_warn {
    ($activation: ident, $($arg:tt)*) => (
        if cfg!(feature = "avm_debug") {
            log::warn!("{} -- in {}", format!($($arg)*), $activation.id)
        } else {
            log::warn!($($arg)*)
        }
    )
}

#[macro_export]
macro_rules! avm_error {
    ($activation: ident, $($arg:tt)*) => (
        if cfg!(feature = "avm_debug") {
            log::error!("{} -- in {}", format!($($arg)*), $activation.id)
        } else {
            log::error!($($arg)*)
        }
    )
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct Avm1<'gc> {
    /// The Flash Player version we're emulating.
    player_version: u8,

    /// The constant pool to use for new activations from code sources that
    /// don't close over the constant pool they were defined with.
    constant_pool: GcCell<'gc, Vec<Value<'gc>>>,

    /// The global object.
    globals: Object<'gc>,

    /// System built-ins that we use internally to construct new objects.
    prototypes: globals::SystemPrototypes<'gc>,

    /// Cached functions for the AsBroadcaster
    broadcaster_functions: BroadcasterFunctions<'gc>,

    /// DisplayObject property map.
    display_properties: GcCell<'gc, stage_object::DisplayPropertyMap<'gc>>,

    /// The operand stack (shared across functions).
    stack: Vec<Value<'gc>>,

    /// The register slots (also shared across functions).
    /// `ActionDefineFunction2` defined functions do not use these slots.
    registers: [Value<'gc>; 4],

    /// If a serious error has occurred, or a user has requested it, the AVM may be halted.
    /// This will completely prevent any further actions from being executed.
    halted: bool,

    /// The maximum amount of functions that can be called before a `Error::FunctionRecursionLimit`
    /// is raised. This defaults to 256 but can be changed per movie.
    max_recursion_depth: u16,

    /// Whether a Mouse listener has been registered.
    /// Used to prevent scrolling on web.
    has_mouse_listener: bool,

    /// The list of all movie clips in execution order.
    clip_exec_list: Option<DisplayObject<'gc>>,

    /// The mappings between symbol names and constructors registered
    /// with `Object.registerClass()`.
    /// Because SWFs v6 and v7+ use different case-sensitivity rules, Flash
    /// keeps two separate registries, one case-sensitive, the other not.
    constructor_registry_case_insensitive: PropertyMap<'gc, FunctionObject<'gc>>,
    constructor_registry_case_sensitive: PropertyMap<'gc, FunctionObject<'gc>>,

    #[cfg(feature = "avm_debug")]
    pub debug_output: bool,
}

impl<'gc> Avm1<'gc> {
    pub fn new(gc_context: MutationContext<'gc, '_>, player_version: u8) -> Self {
        let (prototypes, globals, broadcaster_functions) = create_globals(gc_context);

        Self {
            player_version,
            constant_pool: GcCell::allocate(gc_context, vec![]),
            globals,
            prototypes,
            broadcaster_functions,
            display_properties: stage_object::DisplayPropertyMap::new(gc_context),
            stack: vec![],
            registers: [
                Value::Undefined,
                Value::Undefined,
                Value::Undefined,
                Value::Undefined,
            ],
            halted: false,
            max_recursion_depth: 255,
            has_mouse_listener: false,
            clip_exec_list: None,
            constructor_registry_case_insensitive: PropertyMap::new(),
            constructor_registry_case_sensitive: PropertyMap::new(),

            #[cfg(feature = "avm_debug")]
            debug_output: false,
        }
    }

    /// Add a stack frame that executes code in timeline scope
    ///
    /// This creates a new frame stack.
    pub fn run_stack_frame_for_action<S: Into<Cow<'static, str>>>(
        active_clip: DisplayObject<'gc>,
        name: S,
        swf_version: u8,
        code: SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        if context.avm1.halted {
            // We've been told to ignore all future execution.
            return;
        }

        let globals = context.avm1.global_object_cell();

        let mut parent_activation = Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[Actions Parent]"),
            swf_version,
            globals,
            active_clip,
        );

        let clip_obj = active_clip
            .object()
            .coerce_to_object(&mut parent_activation);
        let child_scope = GcCell::allocate(
            parent_activation.context.gc_context,
            Scope::new(
                parent_activation.scope_cell(),
                scope::ScopeClass::Target,
                clip_obj,
            ),
        );
        let constant_pool = parent_activation.context.avm1.constant_pool;
        let child_name = parent_activation.id.child(name);
        let mut child_activation = Activation::from_action(
            parent_activation.context.reborrow(),
            child_name,
            swf_version,
            child_scope,
            constant_pool,
            active_clip,
            clip_obj.into(),
            None,
            None,
        );
        if let Err(e) = child_activation.run_actions(code) {
            root_error_handler(&mut child_activation, e);
        }
    }

    /// Add a stack frame that executes code in initializer scope.
    ///
    /// This creates a new frame stack.
    pub fn run_with_stack_frame_for_display_object<'a, F, R>(
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        action_context: &mut UpdateContext<'_, 'gc, '_>,
        function: F,
    ) -> R
    where
        for<'b> F: FnOnce(&mut Activation<'b, 'gc, '_>) -> R,
    {
        let clip_obj = match active_clip.object() {
            Value::Object(o) => o,
            _ => panic!("No script object for display object"),
        };
        let global_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::from_global_object(action_context.avm1.globals),
        );
        let child_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        let constant_pool = action_context.avm1.constant_pool;
        let mut activation = Activation::from_action(
            action_context.reborrow(),
            ActivationIdentifier::root("[Display Object]"),
            swf_version,
            child_scope,
            constant_pool,
            active_clip,
            clip_obj.into(),
            None,
            None,
        );
        function(&mut activation)
    }

    /// Add a stack frame that executes code in initializer scope.
    ///
    /// This creates a new frame stack.
    pub fn run_stack_frame_for_init_action(
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        code: SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        if context.avm1.halted {
            // We've been told to ignore all future execution.
            return;
        }

        let globals = context.avm1.global_object_cell();

        let mut parent_activation = Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[Init Parent]"),
            swf_version,
            globals,
            active_clip,
        );

        let clip_obj = active_clip
            .object()
            .coerce_to_object(&mut parent_activation);
        let child_scope = GcCell::allocate(
            parent_activation.context.gc_context,
            Scope::new(
                parent_activation.scope_cell(),
                scope::ScopeClass::Target,
                clip_obj,
            ),
        );
        parent_activation.context.avm1.push(Value::Undefined);
        let constant_pool = parent_activation.context.avm1.constant_pool;
        let child_name = parent_activation.id.child("[Init]");
        let mut child_activation = Activation::from_action(
            parent_activation.context.reborrow(),
            child_name,
            swf_version,
            child_scope,
            constant_pool,
            active_clip,
            clip_obj.into(),
            None,
            None,
        );
        if let Err(e) = child_activation.run_actions(code) {
            root_error_handler(&mut child_activation, e);
        }
    }

    /// Add a stack frame that executes code in timeline scope for an object
    /// method, such as an event handler.
    ///
    /// This creates a new frame stack.
    pub fn run_stack_frame_for_method<'a, 'b>(
        active_clip: DisplayObject<'gc>,
        obj: Object<'gc>,
        swf_version: u8,
        context: &'a mut UpdateContext<'b, 'gc, '_>,
        name: AvmString<'gc>,
        args: &[Value<'gc>],
    ) {
        if context.avm1.halted {
            // We've been told to ignore all future execution.
            return;
        }

        let globals = context.avm1.global_object_cell();

        let mut activation = Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root(name.to_string()),
            swf_version,
            globals,
            active_clip,
        );

        let _ = obj.call_method(name, args, &mut activation);
    }

    pub fn notify_system_listeners(
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        context: &mut UpdateContext<'_, 'gc, '_>,
        broadcaster_name: AvmString<'gc>,
        method: AvmString<'gc>,
        args: &[Value<'gc>],
    ) {
        let global = context.avm1.global_object_cell();

        let mut activation = Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[System Listeners]"),
            swf_version,
            global,
            active_clip,
        );

        let broadcaster = global
            .get(broadcaster_name, &mut activation)
            .unwrap()
            .coerce_to_object(&mut activation);

        let has_listener =
            as_broadcaster::broadcast_internal(&mut activation, broadcaster, args, method)
                .unwrap_or(false);
        drop(activation);

        if &broadcaster_name == b"Mouse" {
            context.avm1.has_mouse_listener = has_listener;
        }
    }

    /// Returns true if the `Mouse` object has a listener registered.
    /// Used to prevent mouse wheel scrolling on web.
    pub fn has_mouse_listener(&self) -> bool {
        self.has_mouse_listener
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

    fn push(&mut self, value: Value<'gc>) {
        avm_debug!(self, "Stack push {}: {:?}", self.stack.len(), value);
        self.stack.push(value);
    }

    #[allow(clippy::let_and_return)]
    fn pop(&mut self) -> Value<'gc> {
        let value = self.stack.pop().unwrap_or_else(|| {
            log::warn!("Avm1::pop: Stack underflow");
            Value::Undefined
        });

        avm_debug!(self, "Stack pop {}: {:?}", self.stack.len(), value);

        value
    }

    /// Obtain the value of `_global`.
    pub fn global_object(&self) -> Value<'gc> {
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

    pub fn max_recursion_depth(&self) -> u16 {
        self.max_recursion_depth
    }

    pub fn set_max_recursion_depth(&mut self, max_recursion_depth: u16) {
        self.max_recursion_depth = max_recursion_depth
    }

    pub fn broadcaster_functions(&self) -> BroadcasterFunctions<'gc> {
        self.broadcaster_functions
    }

    /// Returns an iterator over all movie clips in execution order.
    pub fn clip_exec_iter(&self) -> DisplayObjectIter<'gc> {
        DisplayObjectIter {
            clip: self.clip_exec_list,
        }
    }

    /// Adds a movie clip to the execution list.
    ///
    /// This should be called whenever a movie clip is created, and controls the order of
    /// execution for AVM1 movies.
    pub fn add_to_exec_list(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        clip: DisplayObject<'gc>,
    ) {
        // Adding while iterating is safe, as this does not modify any active nodes.
        if clip.next_avm1_clip().is_none() && clip.prev_avm1_clip().is_none() {
            if let Some(head) = self.clip_exec_list {
                head.set_prev_avm1_clip(gc_context, Some(clip));
                clip.set_next_avm1_clip(gc_context, self.clip_exec_list);
            }
            self.clip_exec_list = Some(clip);
        }
    }

    /// Removes a display object from the execution list.
    pub fn remove_from_exec_list(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        clip: DisplayObject<'gc>,
    ) -> bool {
        let prev = clip.prev_avm1_clip();
        let next = clip.next_avm1_clip();
        let present_on_execution_list = prev.is_some() || next.is_some();

        if let Some(head) = self.clip_exec_list {
            if DisplayObject::ptr_eq(head, clip) {
                self.clip_exec_list = next;
            }
        }

        if let Some(prev) = prev {
            prev.set_next_avm1_clip(gc_context, next);
        }
        if let Some(next) = next {
            next.set_prev_avm1_clip(gc_context, prev);
        }

        clip.set_prev_avm1_clip(gc_context, None);
        clip.set_next_avm1_clip(gc_context, None);

        present_on_execution_list
    }

    pub fn get_registered_constructor(
        &self,
        swf_version: u8,
        symbol: AvmString<'gc>,
    ) -> Option<&FunctionObject<'gc>> {
        let is_case_sensitive = swf_version >= 7;
        let registry = if is_case_sensitive {
            &self.constructor_registry_case_sensitive
        } else {
            &self.constructor_registry_case_insensitive
        };
        registry.get(symbol, is_case_sensitive)
    }

    pub fn register_constructor(
        &mut self,
        swf_version: u8,
        symbol: AvmString<'gc>,
        constructor: Option<FunctionObject<'gc>>,
    ) {
        let is_case_sensitive = swf_version >= 7;
        let registry = if is_case_sensitive {
            &mut self.constructor_registry_case_sensitive
        } else {
            &mut self.constructor_registry_case_insensitive
        };
        if let Some(constructor) = constructor {
            registry.insert(symbol, constructor, is_case_sensitive);
        } else {
            registry.remove(symbol, is_case_sensitive);
        }
    }

    #[cfg(feature = "avm_debug")]
    #[inline]
    pub fn show_debug_output(&self) -> bool {
        self.debug_output
    }

    #[cfg(not(feature = "avm_debug"))]
    pub const fn show_debug_output(&self) -> bool {
        false
    }

    #[cfg(feature = "avm_debug")]
    pub fn set_show_debug_output(&mut self, visible: bool) {
        self.debug_output = visible;
    }

    #[cfg(not(feature = "avm_debug"))]
    pub const fn set_show_debug_output(&self, _visible: bool) {}
}

pub fn root_error_handler<'gc>(activation: &mut Activation<'_, 'gc, '_>, error: Error<'gc>) {
    match &error {
        Error::ThrownValue(value) => {
            let message = value
                .coerce_to_string(activation)
                .unwrap_or_else(|_| "undefined".into());
            activation.context.avm_trace(&message.to_utf8_lossy());
            // Continue execution without halting.
            return;
        }
        Error::InvalidSwf(swf_error) => {
            log::error!("{}: {}", error, swf_error);
        }
        _ => {
            log::error!("{}", error);
        }
    }
    activation.context.avm1.halt();
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

/// Starts dragging this display object, making it follow the cursor.
/// Runs via the `startDrag` method or `StartDrag` AVM1 action.
pub fn start_drag<'gc>(
    display_object: DisplayObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) {
    let lock_center = args
        .get(0)
        .map(|o| o.as_bool(activation.context.swf.version()))
        .unwrap_or(false);

    let offset = if lock_center {
        // The object's origin point is locked to the mouse.
        Default::default()
    } else {
        // The object moves relative to current mouse position.
        // Calculate the offset from the mouse to the object in world space.
        let (object_x, object_y) = display_object.local_to_global(Default::default());
        let (mouse_x, mouse_y) = *activation.context.mouse_position;
        (object_x - mouse_x, object_y - mouse_y)
    };

    let constraint = if args.len() > 1 {
        // Invalid values turn into 0.
        let mut x_min = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut y_min = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut x_max = args
            .get(3)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut y_max = args
            .get(4)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)
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
    *activation.context.drag_object = Some(drag_object);
}

pub struct DisplayObjectIter<'gc> {
    clip: Option<DisplayObject<'gc>>,
}

impl<'gc> Iterator for DisplayObjectIter<'gc> {
    type Item = DisplayObject<'gc>;

    fn next(&mut self) -> Option<Self::Item> {
        let clip = self.clip;
        self.clip = clip.and_then(|clip| clip.next_avm1_clip());
        clip
    }
}
