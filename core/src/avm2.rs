//! ActionScript Virtual Machine 2 (AS3) support

use crate::avm2::bytearray::ObjectEncoding;
use crate::avm2::class::{AllocatorFn, CustomConstructorFn};
use crate::avm2::e4x::XmlSettings;
use crate::avm2::error::{
    make_error_1014, make_error_1107, type_error, verify_error, Error1014Type,
};
use crate::avm2::function::exec;
use crate::avm2::globals::{
    init_builtin_system_class_defs, init_builtin_system_classes, init_native_system_classes,
    SystemClassDefs, SystemClasses,
};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::FunctionObject;
use crate::avm2::scope::ScopeChain;
use crate::avm2::script::{Script, TranslationUnit};
use crate::avm2::stack::Stack;
use crate::character::Character;
use crate::context::UpdateContext;
use crate::display_object::{MovieClip, TDisplayObject};
use crate::string::{AvmString, StringContext};
use crate::tag_utils::SwfMovie;
use crate::PlayerRuntime;

use fnv::FnvHashMap;
use gc_arena::lock::GcRefLock;
use gc_arena::{Collect, Gc, Mutation};
use std::sync::Arc;
use swf::avm2::read::Reader;
use swf::DoAbc2Flag;

#[macro_export]
macro_rules! avm_debug {
    ($avm: expr, $($arg:tt)*) => (
        if $avm.show_debug_output() {
            tracing::debug!($($arg)*)
        }
    )
}

pub mod activation;
mod amf;
pub mod api_version;
mod array;
pub mod bytearray;
mod call_stack;
mod class;
mod domain;
mod dynamic_map;
mod e4x;
pub mod error;
mod events;
mod filters;
mod flv;
mod function;
pub mod globals;
mod metadata;
mod method;
mod multiname;
mod namespace;
pub mod object;
mod op;
mod optimizer;
mod parameters;
pub mod property;
mod property_map;
mod qname;
mod regexp;
mod scope;
pub mod script;
#[cfg(feature = "known_stubs")]
pub mod specification;
mod stack;
mod string;
mod stubs;
mod traits;
mod value;
pub mod vector;
mod verify;
mod vtable;

pub use crate::avm2::activation::Activation;
pub use crate::avm2::array::ArrayStorage;
pub use crate::avm2::call_stack::CallStack;
pub use crate::avm2::class::Class;
#[allow(unused)] // For debug_ui
pub use crate::avm2::domain::{Domain, DomainPtr};
pub use crate::avm2::error::Error;
pub use crate::avm2::flv::FlvValueAvm2Ext;
pub use crate::avm2::function::FunctionArgs;
pub use crate::avm2::globals::flash::ui::context_menu::make_context_menu_state;
pub use crate::avm2::multiname::Multiname;
pub use crate::avm2::namespace::{CommonNamespaces, Namespace};
pub use crate::avm2::object::{
    ArrayObject, BitmapDataObject, ClassObject, EventObject, LoaderInfoObject, Object,
    SoundChannelObject, StageObject, TObject,
};
pub use crate::avm2::qname::QName;
pub use crate::avm2::value::Value;

use self::api_version::ApiVersion;
use self::object::WeakObject;
use self::scope::Scope;

const BROADCAST_WHITELIST: [&[u8]; 4] =
    [b"enterFrame", b"exitFrame", b"frameConstructed", b"render"];

/// The state of an AVM2 interpreter.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Avm2<'gc> {
    /// The Flash Player version we're emulating.
    player_version: u8,

    /// The player runtime we're emulating
    #[collect(require_static)]
    pub player_runtime: PlayerRuntime,

    /// Values currently present on the operand stack.
    stack: Stack<'gc>,

    /// Scopes currently present of the scope stack.
    scope_stack: Vec<Scope<'gc>>,

    /// The current call stack of the player.
    call_stack: GcRefLock<'gc, CallStack<'gc>>,

    /// This domain is used exclusively for classes from playerglobals
    playerglobals_domain: Domain<'gc>,

    /// The domain associated with 'stage.loaderInfo.applicationDomain'.
    /// Note that this is a parent of the root movie clip's domain
    /// (which can be observed from ActionScript)
    stage_domain: Domain<'gc>,

    /// System classes.
    system_classes: Option<SystemClasses<'gc>>,

    /// System class definitions.
    system_class_defs: Option<SystemClassDefs<'gc>>,

    /// Top-level global object. It contains most top-level types (Object, Class) and functions.
    /// However, it's not strictly defined which items end up there.
    toplevel_global_object: Option<Object<'gc>>,

    /// Pre-created known namespaces.
    namespaces: Gc<'gc, CommonNamespaces<'gc>>,

    #[collect(require_static)]
    native_method_table: &'static [Option<NativeMethodImpl>],

    #[collect(require_static)]
    native_instance_allocator_table: &'static [Option<AllocatorFn>],

    #[collect(require_static)]
    native_call_handler_table: &'static [Option<NativeMethodImpl>],

    #[collect(require_static)]
    native_custom_constructor_table: &'static [Option<CustomConstructorFn>],

    native_fast_call_list: &'static [usize],

    /// A list of objects which are capable of receiving broadcasts.
    ///
    /// Certain types of events are "broadcast events" that are emitted on all
    /// constructed objects in order of their creation, whether or not they are
    /// currently present on the display list. This list keeps track of that.
    broadcast_list: FnvHashMap<AvmString<'gc>, Vec<WeakObject<'gc>>>,

    alias_to_class_map: FnvHashMap<AvmString<'gc>, ClassObject<'gc>>,
    class_to_alias_map: FnvHashMap<Class<'gc>, AvmString<'gc>>,

    #[collect(require_static)]
    pub xml_settings: XmlSettings,

    pub default_bytearray_encoding: ObjectEncoding,

    /// The api version of our root movie clip. Note - this is used as the
    /// api version for swfs loaded via `Loader`, overriding the api version
    /// specified in the loaded SWF. This is only used for API versioning (hiding
    /// definitions from playerglobals) - version-specific behavior in things like
    /// `gotoAndPlay` uses the current movie clip's SWF version.
    #[collect(require_static)]
    pub root_api_version: ApiVersion,

    #[cfg(feature = "avm_debug")]
    pub debug_output: bool,

    pub optimizer_enabled: bool,
}

impl<'gc> Avm2<'gc> {
    /// Construct a new AVM interpreter.
    pub fn new(
        context: &mut StringContext<'gc>,
        player_version: u8,
        player_runtime: PlayerRuntime,
    ) -> Self {
        let mc = context.gc();

        let playerglobals_domain = Domain::uninitialized_domain(mc, None);
        let stage_domain = Domain::uninitialized_domain(mc, Some(playerglobals_domain));

        let namespaces = CommonNamespaces::new(context);

        Self {
            player_version,
            player_runtime,
            stack: Stack::new(mc),
            scope_stack: Vec::new(),
            call_stack: GcRefLock::new(mc, CallStack::new().into()),
            playerglobals_domain,
            stage_domain,
            system_classes: None,
            system_class_defs: None,
            toplevel_global_object: None,

            namespaces: Gc::new(mc, namespaces),

            native_method_table: Default::default(),
            native_instance_allocator_table: Default::default(),
            native_call_handler_table: Default::default(),
            native_custom_constructor_table: Default::default(),
            native_fast_call_list: Default::default(),
            broadcast_list: Default::default(),

            alias_to_class_map: Default::default(),
            class_to_alias_map: Default::default(),

            xml_settings: XmlSettings::new_default(),
            default_bytearray_encoding: ObjectEncoding::Amf3,

            // Set the lowest version for now - this will be overridden when we set our movie
            root_api_version: ApiVersion::AllVersions,

            #[cfg(feature = "avm_debug")]
            debug_output: false,

            optimizer_enabled: true,
        }
    }

    pub fn load_player_globals(context: &mut UpdateContext<'gc>) {
        let globals = context.avm2.playerglobals_domain;
        let mut activation = Activation::from_domain(context, globals);
        globals::load_playerglobal(&mut activation, globals);
    }

    pub fn playerglobals_domain(&self) -> Domain<'gc> {
        self.playerglobals_domain
    }

    /// Return the current set of system classes.
    ///
    /// This function panics if the interpreter has not yet been initialized.
    pub fn classes(&self) -> &SystemClasses<'gc> {
        self.system_classes.as_ref().unwrap()
    }

    /// Return the current set of system class definitions.
    ///
    /// This function panics if the interpreter has not yet been initialized.
    pub fn class_defs(&self) -> &SystemClassDefs<'gc> {
        self.system_class_defs.as_ref().unwrap()
    }

    pub fn toplevel_global_object(&self) -> Option<Object<'gc>> {
        self.toplevel_global_object
    }

    pub fn register_class_alias(&mut self, name: AvmString<'gc>, class_object: ClassObject<'gc>) {
        self.alias_to_class_map.insert(name, class_object);
        self.class_to_alias_map
            .insert(class_object.inner_class_definition(), name);
    }

    pub fn get_class_by_alias(&self, name: AvmString<'gc>) -> Option<ClassObject<'gc>> {
        self.alias_to_class_map.get(&name).copied()
    }

    pub fn get_alias_by_class(&self, cls: Class<'gc>) -> Option<AvmString<'gc>> {
        self.class_to_alias_map.get(&cls).copied()
    }

    /// Run a script's initializer method.
    #[inline(never)]
    pub fn run_script_initializer(
        script: Script<'gc>,
        context: &mut UpdateContext<'gc>,
    ) -> Result<(), Error<'gc>> {
        // TODO can we skip creating this temporary Activation?
        let mut activation = Activation::from_nothing(context);

        let (method, global_object, domain) = script.init();

        let scope = ScopeChain::new(domain);
        // Script `global` classes extend Object
        let bound_superclass = Some(activation.avm2().classes().object);

        // Provide a callee object if necessary
        let callee = if method.needs_arguments_object() {
            Some(FunctionObject::from_method(
                &mut activation,
                method,
                scope,
                Some(global_object.into()),
                bound_superclass,
            ))
        } else {
            None
        };

        exec(
            method,
            scope,
            global_object.into(),
            bound_superclass,
            FunctionArgs::empty(),
            &mut activation,
            callee,
        )?;

        Ok(())
    }

    /// Dispatch an event on an object.
    ///
    /// This will become its own self-contained activation and swallow
    /// any resulting error (after logging).
    ///
    /// Attempts to dispatch a non-event object will panic.
    ///
    /// Returns `true` if the event has been handled.
    pub fn dispatch_event(
        context: &mut UpdateContext<'gc>,
        event: EventObject<'gc>,
        target: Object<'gc>,
    ) -> bool {
        Self::dispatch_event_internal(context, event, target, false)
    }

    /// Simulate dispatching an event.
    ///
    /// This method is similar to [`Self::dispatch_event`],
    /// but it does not execute event handlers.
    ///
    /// Returns `true` when the event would have been handled if not simulated.
    pub fn simulate_event_dispatch(
        context: &mut UpdateContext<'gc>,
        event: EventObject<'gc>,
        target: Object<'gc>,
    ) -> bool {
        Self::dispatch_event_internal(context, event, target, true)
    }

    fn dispatch_event_internal(
        context: &mut UpdateContext<'gc>,
        event: EventObject<'gc>,
        target: Object<'gc>,
        simulate_dispatch: bool,
    ) -> bool {
        let mut activation = Activation::from_nothing(context);
        match events::dispatch_event(&mut activation, target, event, simulate_dispatch) {
            Err(err) => {
                let event_name = event.event().event_type();

                tracing::error!(
                    "Encountered AVM2 error when dispatching `{}` event: {:?}",
                    event_name,
                    err,
                );
                // TODO: push the error onto `loaderInfo.uncaughtErrorEvents`
                false
            }
            Ok(handled) => handled,
        }
    }

    /// Add an object to the broadcast list.
    ///
    /// Each broadcastable event contains its own broadcast list. You must
    /// register all objects that have event handlers with that event's
    /// broadcast list by calling this function. Attempting to register a
    /// broadcast listener for a non-broadcast event will do nothing.
    ///
    /// Attempts to register the same listener for the same event will also do
    /// nothing.
    pub fn register_broadcast_listener(
        context: &mut UpdateContext<'gc>,
        object: Object<'gc>,
        event_name: AvmString<'gc>,
    ) {
        if !BROADCAST_WHITELIST.iter().any(|x| *x == &event_name) {
            return;
        }

        let bucket = context.avm2.broadcast_list.entry(event_name).or_default();

        for entry in bucket.iter() {
            // Note: comparing pointers is correct because GcWeak keeps its allocation alive,
            // so the pointers can't overlap by accident.
            if std::ptr::eq(entry.as_ptr(), object.as_ptr()) {
                return;
            }
        }

        bucket.push(object.downgrade());
    }

    /// Dispatch an event on all objects in the current execution list.
    ///
    /// `on_type` specifies a class or interface constructor whose instances,
    /// implementers, and/or subclasses define the set of objects that will
    /// receive the event. You can broadcast to just display objects, or
    /// specific interfaces, and so on.
    ///
    /// Attempts to broadcast a non-broadcast event will do nothing. To add a
    /// new broadcast type, you must add it to the `BROADCAST_WHITELIST` first.
    ///
    /// Attempts to broadcast a non-event object will panic.
    pub fn broadcast_event(
        context: &mut UpdateContext<'gc>,
        event: EventObject<'gc>,
        on_type: ClassObject<'gc>,
    ) {
        let event_name = event.event().event_type();

        if !BROADCAST_WHITELIST.iter().any(|x| *x == &event_name) {
            return;
        }

        let el_length = context
            .avm2
            .broadcast_list
            .entry(event_name)
            .or_default()
            .len();

        for i in 0..el_length {
            let object = context
                .avm2
                .broadcast_list
                .get(&event_name)
                .unwrap()
                .get(i)
                .copied();

            if let Some(object) = object.and_then(|obj| obj.upgrade(context.gc())) {
                let mut activation = Activation::from_nothing(context);

                if object.is_of_type(on_type.inner_class_definition()) {
                    if let Err(err) = events::broadcast_event(&mut activation, object, event) {
                        tracing::error!(
                            "Encountered AVM2 error when broadcasting `{}` event: {:?}",
                            event_name,
                            err,
                        );
                        // TODO: push the error onto `loaderInfo.uncaughtErrorEvents`
                    }
                }
            }
        }
        // Once we're done iterating, remove dead weak references from the list.
        context
            .avm2
            .broadcast_list
            .entry(event_name)
            .or_default()
            .retain(|x| x.upgrade(context.gc_context).is_some());
    }

    pub fn run_stack_frame_for_callable(
        callable: Object<'gc>,
        receiver: Value<'gc>,
        domain: Domain<'gc>,
        context: &mut UpdateContext<'gc>,
    ) -> Result<(), String> {
        let mut evt_activation = Activation::from_domain(context, domain);
        Value::from(callable)
            .call(&mut evt_activation, receiver, FunctionArgs::empty())
            .map_err(|e| format!("{e:?}"))?;

        Ok(())
    }

    pub fn lookup_class_for_character(
        activation: &mut Activation<'_, 'gc>,
        movie_clip: MovieClip<'gc>,
        domain: Domain<'gc>,
        name: AvmString<'gc>,
        id: u16,
    ) -> Result<ClassObject<'gc>, Error<'gc>> {
        let movie = movie_clip.movie().clone();

        let class_object = domain
            .get_defined_value_handling_vector(activation, name)?
            .as_object()
            .and_then(|o| o.as_class_object())
            .ok_or_else(|| make_error_1014(activation, Error1014Type::ReferenceError, name))?;

        let class = class_object.inner_class_definition();

        let library = activation.context.library.library_for_movie_mut(movie);
        let character = library.character_by_id(id);

        if let Some(character) = character {
            if matches!(
                character,
                Character::EditText(_)
                    | Character::Graphic(_)
                    | Character::MovieClip(_)
                    | Character::Avm2Button(_)
            ) {
                // The class must extend DisplayObject to ensure that events
                // can properly be dispatched to them
                if !class.has_class_in_chain(activation.avm2().class_defs().display_object) {
                    return Err(Error::avm_error(type_error(
                        activation,
                        &format!("Error #2022: Class {}$ must inherit from DisplayObject to link to a symbol.", class.name().local_name()),
                        2022,
                    )?));
                }
            }
        } else if movie_clip.avm2_class().is_none() {
            // If this ID doesn't correspond to any character, and the MC that
            // we're processing doesn't have an AVM2 class set, then this
            // ClassObject is going to be the class of the MC. Ensure it
            // subclasses Sprite.
            if !class.has_class_in_chain(activation.avm2().class_defs().sprite) {
                return Err(Error::avm_error(type_error(
                    activation,
                    &format!(
                        "Error #2023: Class {}$ must inherit from Sprite to link to the root.",
                        class.name().local_name(),
                    ),
                    2023,
                )?));
            }
        }

        Ok(class_object)
    }

    /// Load an ABC file embedded in a `DoAbc` or `DoAbc2` tag.
    pub fn do_abc(
        context: &mut UpdateContext<'gc>,
        data: &[u8],
        name: Option<AvmString<'gc>>,
        flags: DoAbc2Flag,
        domain: Domain<'gc>,
        movie: Arc<SwfMovie>,
    ) -> Result<Option<Script<'gc>>, Error<'gc>> {
        let mut reader = Reader::new(data);
        let abc = match reader.read() {
            Ok(abc) => abc,
            Err(_) => {
                let mut activation = Activation::from_nothing(context);
                return Err(make_error_1107(&mut activation));
            }
        };

        let mut activation = Activation::from_domain(context, domain);
        // Make sure we have the correct domain for code that tries to access it
        // using `activation.domain()`
        activation.set_outer(ScopeChain::new(domain));

        if abc.scripts.is_empty() {
            return Err(Error::avm_error(verify_error(
                &mut activation,
                "Error #1047: No entry point was found.",
                1047,
            )?));
        }

        let num_scripts = abc.scripts.len();
        let tunit = TranslationUnit::from_abc(abc, domain, name, movie, activation.gc());
        tunit.load_classes(&mut activation)?;
        for i in 0..num_scripts {
            tunit.load_script(i as u32, &mut activation)?;
        }

        if !flags.contains(DoAbc2Flag::LAZY_INITIALIZE) {
            return Ok(Some(tunit.get_script(num_scripts - 1).unwrap()));
        }
        Ok(None)
    }

    /// Load the playerglobal ABC file.
    pub fn load_builtin_abc(
        context: &mut UpdateContext<'gc>,
        data: &[u8],
        domain: Domain<'gc>,
        movie: Arc<SwfMovie>,
    ) {
        let mut reader = Reader::new(data);
        let abc = match reader.read() {
            Ok(abc) => abc,
            Err(_) => panic!("Builtin ABC should be valid"),
        };

        let mut activation = Activation::from_domain(context, domain);
        // Make sure we have the correct domain for code that tries to access its
        // domain using `activation.domain()`
        activation.set_outer(ScopeChain::new(domain));

        let tunit = TranslationUnit::from_abc(abc, domain, None, movie, activation.gc());

        globals::init_early_classes(&mut activation, tunit).expect("Early classes should load");

        // At this point we have everything necessary to load scripts and classes.

        tunit
            .load_classes(&mut activation)
            .expect("Classes should load");

        // These Classes are absolutely critical to the runtime, so make sure
        // we've registered them before anything else.
        init_builtin_system_class_defs(&mut activation);

        // The second script (script #1) is Toplevel.as, and includes important
        // builtin classes such as Namespace, QName, and XML.
        let toplevel_script = tunit
            .load_script(1, &mut activation)
            .expect("Script should load");

        // We intentionally avoid running the script initializer here
        let (_, toplevel_global, _) = toplevel_script.init();

        activation.avm2().toplevel_global_object = Some(toplevel_global);

        // HACK: Replace ScopeChains on the class vtable of `Object` to include
        // the toplevel global.
        let mc = activation.gc();

        let new_scope = ScopeChain::new(tunit.domain());
        let new_scope = new_scope.chain(mc, &[Scope::new(toplevel_global.into())]);

        activation
            .avm2()
            .classes()
            .object
            .vtable()
            .replace_scopes_with(mc, new_scope);

        // The scopes must be correct before we run the script initializer from
        // `init_builtin_system_classes`.
        init_builtin_system_classes(&mut activation);

        // The first script (script #0) is globals.as, and includes other builtin
        // classes that are less critical for the AVM to load.
        tunit
            .load_script(0, &mut activation)
            .expect("Script should load");
        init_native_system_classes(&mut activation);
    }

    pub fn stage_domain(&self) -> Domain<'gc> {
        self.stage_domain
    }

    /// Pushes an executable on the call stack
    pub fn push_call(&self, mc: &Mutation<'gc>, method: Method<'gc>) {
        self.call_stack.borrow_mut(mc).push(method)
    }

    /// Pops an executable off the call stack
    pub fn pop_call(&self, mc: &Mutation<'gc>) {
        self.call_stack.borrow_mut(mc).pop();
    }

    pub fn call_stack(&self) -> GcRefLock<'gc, CallStack<'gc>> {
        self.call_stack
    }

    fn push_scope(&mut self, scope: Scope<'gc>) {
        self.scope_stack.push(scope);
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
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

    /// Gets the public namespace, versioned based on the current root SWF.
    /// See `AvmCore::findPublicNamespace()`
    /// https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/AvmCore.cpp#L5809C25-L5809C25
    pub fn find_public_namespace(&self) -> Namespace<'gc> {
        self.namespaces.public_for(self.root_api_version)
    }

    pub fn optimizer_enabled(&self) -> bool {
        self.optimizer_enabled
    }

    pub fn set_optimizer_enabled(&mut self, value: bool) {
        self.optimizer_enabled = value;
    }
}
