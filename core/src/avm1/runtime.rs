use crate::avm1::function::{ExecutionReason, FunctionObject};
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::globals::{as_broadcaster, create_globals};
use crate::avm1::object::stage_object;
use crate::avm1::object::TObject;
use crate::avm1::property_map::PropertyMap;
use crate::avm1::scope::Scope;
use crate::avm1::{scope, Activation, ActivationIdentifier, Error, Object, Value};
use crate::context::UpdateContext;
use crate::frame_lifecycle::FramePhase;
use crate::prelude::*;
use crate::string::AvmString;
use crate::tag_utils::SwfSlice;
use crate::{avm1, avm_debug};
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::Cow;
use swf::avm1::read::Reader;

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
    prototypes: avm1::globals::SystemPrototypes<'gc>,

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
            active_clip.swf_version(),
            child_scope,
            constant_pool,
            active_clip,
            clip_obj.into(),
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
            Scope::from_global_object(action_context.avm1.global_object_cell()),
        );
        let child_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        let constant_pool = action_context.avm1.constant_pool;
        let mut activation = Activation::from_action(
            action_context.reborrow(),
            ActivationIdentifier::root("[Display Object]"),
            active_clip.swf_version(),
            child_scope,
            constant_pool,
            active_clip,
            clip_obj.into(),
            None,
        );
        function(&mut activation)
    }

    /// Add a stack frame that executes code in initializer scope.
    ///
    /// This creates a new frame stack.
    pub fn run_stack_frame_for_init_action(
        active_clip: DisplayObject<'gc>,
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
            active_clip.swf_version(),
            child_scope,
            constant_pool,
            active_clip,
            clip_obj.into(),
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
            globals,
            active_clip,
        );

        let _ = obj.call_method(name, args, &mut activation, ExecutionReason::Special);
    }

    pub fn notify_system_listeners(
        active_clip: DisplayObject<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        broadcaster_name: AvmString<'gc>,
        method: AvmString<'gc>,
        args: &[Value<'gc>],
    ) {
        let global = context.avm1.global_object_cell();

        let mut activation = Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[System Listeners]"),
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

    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, value: Value<'gc>) {
        avm_debug!(self, "Stack push {}: {:?}", self.stack.len(), value);
        self.stack.push(value);
    }

    #[allow(clippy::let_and_return)]
    pub fn pop(&mut self) -> Value<'gc> {
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
    pub fn prototypes(&self) -> &avm1::globals::SystemPrototypes<'gc> {
        &self.prototypes
    }

    /// Obtains the constant pool to use for new activations from code sources that
    /// don't close over the constant pool they were defined with.
    pub fn constant_pool(&self) -> GcCell<'gc, Vec<Value<'gc>>> {
        self.constant_pool
    }

    /// Sets the constant pool to use for new activations from code sources that
    /// don't close over the constant pool they were defined with.
    pub fn set_constant_pool(&mut self, constant_pool: GcCell<'gc, Vec<Value<'gc>>>) {
        self.constant_pool = constant_pool;
    }

    /// DisplayObject property map.
    pub fn display_properties(&self) -> GcCell<'gc, stage_object::DisplayPropertyMap<'gc>> {
        self.display_properties
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

    /// The Flash Player version we're emulating.
    pub fn player_version(&self) -> u8 {
        self.player_version
    }

    pub fn get_register(&self, id: usize) -> Option<&Value<'gc>> {
        self.registers.get(id)
    }

    pub fn get_register_mut(&mut self, id: usize) -> Option<&mut Value<'gc>> {
        self.registers.get_mut(id)
    }

    // Run a single frame.
    pub fn run_frame(context: &mut UpdateContext<'_, 'gc, '_>) {
        // In AVM1, we only ever execute the update phase, and all the work that
        // would ordinarily be phased is instead run all at once in whatever order
        // the SWF requests it.
        *context.frame_phase = FramePhase::Update;

        // AVM1 execution order is determined by the global execution list, based on instantiation order.
        let mut prev: Option<DisplayObject<'gc>> = None;
        let mut next = context.avm1.clip_exec_list;
        while let Some(clip) = next {
            next = clip.next_avm1_clip();
            if clip.removed() {
                // Clean up removed clips from this frame or a previous frame.
                if let Some(prev) = prev {
                    prev.set_next_avm1_clip(context.gc_context, next);
                } else {
                    context.avm1.clip_exec_list = next;
                }
                clip.set_next_avm1_clip(context.gc_context, None);
            } else {
                clip.run_frame(context);
                prev = Some(clip);
            }
        }

        // Fire "onLoadInit" events.
        context
            .load_manager
            .movie_clip_on_load(context.action_queue);

        *context.frame_phase = FramePhase::Idle;
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
        if clip.next_avm1_clip().is_none() {
            clip.set_next_avm1_clip(gc_context, self.clip_exec_list);
            self.clip_exec_list = Some(clip);
        }
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

/// Utility function used by `Avm1::action_wait_for_frame` and
/// `Avm1::action_wait_for_frame_2`.
pub fn skip_actions(reader: &mut Reader<'_>, num_actions_to_skip: u8) {
    for _ in 0..num_actions_to_skip {
        if let Err(e) = reader.read_action() {
            log::warn!("Couldn't skip action: {}", e);
        }
    }
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
