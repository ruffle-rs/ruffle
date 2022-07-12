//! ActionScript Virtual Machine 2 (AS3) support

use crate::avm2::class::AllocatorFn;
use crate::avm2::globals::SystemClasses;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::EventObject;
use crate::avm2::script::{Script, TranslationUnit};
use crate::context::UpdateContext;
use crate::string::AvmString;
use crate::tag_utils::{SwfSlice, SwfStream};
use fnv::FnvHashMap;
use gc_arena::{Collect, MutationContext};
use std::rc::Rc;
use swf::avm2::read::Reader;
use swf::extensions::ReadSwfExt;

#[macro_export]
macro_rules! avm_debug {
    ($avm: expr, $($arg:tt)*) => (
        if $avm.show_debug_output() {
            log::debug!($($arg)*)
        }
    )
}

pub mod activation;
mod array;
pub mod bytearray;
mod class;
mod domain;
mod events;
mod function;
mod globals;
mod method;
pub mod names;
pub mod object;
mod property;
mod property_map;
mod regexp;
mod scope;
mod script;
mod string;
mod traits;
mod value;
mod vector;
mod vtable;

pub use crate::avm2::activation::Activation;
pub use crate::avm2::array::ArrayStorage;
pub use crate::avm2::domain::Domain;
pub use crate::avm2::events::{Event, EventData};
pub use crate::avm2::names::{Namespace, QName};
pub use crate::avm2::object::{
    ArrayObject, ClassObject, Object, ScriptObject, SoundChannelObject, StageObject, TObject,
};
pub use crate::avm2::value::Value;

const BROADCAST_WHITELIST: [&str; 3] = ["enterFrame", "exitFrame", "frameConstructed"];

/// Boxed error alias.
///
/// As AVM2 is a far stricter VM than AVM1, this may eventually be replaced
/// with a proper Avm2Error enum.
pub type Error = Box<dyn std::error::Error>;

/// The state of an AVM2 interpreter.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Avm2<'gc> {
    /// Values currently present on the operand stack.
    stack: Vec<Value<'gc>>,

    /// Global scope object.
    globals: Domain<'gc>,

    /// System classes.
    system_classes: Option<SystemClasses<'gc>>,

    #[collect(require_static)]
    native_method_table: &'static [Option<NativeMethodImpl>],

    #[collect(require_static)]
    native_instance_allocator_table: &'static [Option<AllocatorFn>],

    /// A list of objects which are capable of recieving broadcasts.
    ///
    /// Certain types of events are "broadcast events" that are emitted on all
    /// constructed objects in order of their creation, whether or not they are
    /// currently present on the display list. This list keeps track of that.
    ///
    /// TODO: These should be weak object pointers, but our current garbage
    /// collector does not support weak references.
    broadcast_list: FnvHashMap<AvmString<'gc>, Vec<Object<'gc>>>,

    #[cfg(feature = "avm_debug")]
    pub debug_output: bool,
}

impl<'gc> Avm2<'gc> {
    /// Construct a new AVM interpreter.
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        let globals = Domain::global_domain(mc);

        Self {
            stack: Vec::new(),
            globals,
            system_classes: None,
            native_method_table: Default::default(),
            native_instance_allocator_table: Default::default(),
            broadcast_list: Default::default(),

            #[cfg(feature = "avm_debug")]
            debug_output: false,
        }
    }

    pub fn load_player_globals(context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let globals = context.avm2.globals;
        let mut activation = Activation::from_nothing(context.reborrow());
        globals::load_player_globals(&mut activation, globals)
    }

    /// Return the current set of system classes.
    ///
    /// This function panics if the interpreter has not yet been initialized.
    pub fn classes(&self) -> &SystemClasses<'gc> {
        self.system_classes.as_ref().unwrap()
    }

    /// Run a script's initializer method.
    pub fn run_script_initializer(
        script: Script<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut init_activation = Activation::from_script(context.reborrow(), script)?;

        let (method, scope, _domain) = script.init();
        match method {
            Method::Native(method) => {
                //This exists purely to check if the builtin is OK with being called with
                //no parameters.
                init_activation.resolve_parameters(&method.name, &[], &method.signature)?;

                (method.method)(&mut init_activation, Some(scope), &[])?;
            }
            Method::Bytecode(method) => {
                init_activation.run_actions(method)?;
            }
        };

        Ok(())
    }

    /// Dispatch an event on an object.
    ///
    /// The `bool` parameter reads true if the event was cancelled.
    pub fn dispatch_event(
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: Event<'gc>,
        target: Object<'gc>,
    ) -> Result<bool, Error> {
        use crate::avm2::events::dispatch_event;

        let mut activation = Activation::from_nothing(context.reborrow());

        let event_object = EventObject::from_event(&mut activation, event)?;

        dispatch_event(&mut activation, target, event_object)
    }

    /// Add an object to the broadcast list.
    ///
    /// Each broadcastable event contains it's own broadcast list. You must
    /// register all objects that have event handlers with that event's
    /// broadcast list by calling this function. Attempting to register a
    /// broadcast listener for a non-broadcast event will do nothing.
    ///
    /// Attempts to register the same listener for the same event will also do
    /// nothing.
    pub fn register_broadcast_listener(
        context: &mut UpdateContext<'_, 'gc, '_>,
        object: Object<'gc>,
        event_name: AvmString<'gc>,
    ) {
        if !BROADCAST_WHITELIST
            .iter()
            .any(|x| AvmString::from(*x) == event_name)
        {
            return;
        }

        let bucket = context.avm2.broadcast_list.entry(event_name).or_default();

        if bucket.iter().any(|x| Object::ptr_eq(*x, object)) {
            return;
        }

        bucket.push(object);
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
    pub fn broadcast_event(
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: Event<'gc>,
        on_type: ClassObject<'gc>,
    ) -> Result<(), Error> {
        let event_name = event.event_type();
        if !BROADCAST_WHITELIST
            .iter()
            .any(|x| AvmString::from(*x) == event_name)
        {
            return Ok(());
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

            if let Some(object) = object {
                let mut activation = Activation::from_nothing(context.reborrow());

                if object.is_of_type(on_type, &mut activation)? {
                    Avm2::dispatch_event(&mut activation.context, event.clone(), object)?;
                }
            }
        }

        Ok(())
    }

    pub fn run_stack_frame_for_callable(
        callable: Object<'gc>,
        reciever: Option<Object<'gc>>,
        args: &[Value<'gc>],
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut evt_activation = Activation::from_nothing(context.reborrow());
        callable.call(reciever, args, &mut evt_activation)?;

        Ok(())
    }

    pub fn load_abc_from_do_abc(
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf: &SwfSlice,
        domain: Domain<'gc>,
        reader: &mut SwfStream<'_>,
        tag_len: usize,
    ) -> Result<(), Error> {
        let start = reader.as_slice();
        // Queue the actions.
        // TODO: The tag reader parses the entire ABC file, instead of just
        // giving us a `SwfSlice` for later parsing, so we have to replcate the
        // *entire* parsing code here. This sucks.
        let flags = reader.read_u32()?;
        let name = reader.read_str()?.to_string_lossy(reader.encoding());
        let is_lazy_initialize = flags & 1 != 0;
        let num_read = reader.pos(start);

        // The rest of the tag is an ABC file so we can take our SwfSlice now.
        let slice = swf
            .resize_to_reader(reader, tag_len - num_read)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid source or tag length when running init action",
                )
            })?;

        Avm2::load_abc(slice, &name, is_lazy_initialize, context, domain)?;
        Ok(())
    }

    /// Load an ABC file embedded in a `SwfSlice`.
    ///
    /// The `SwfSlice` must resolve to the contents of an ABC file.
    pub fn load_abc(
        abc: SwfSlice,
        _abc_name: &str,
        lazy_init: bool,
        context: &mut UpdateContext<'_, 'gc, '_>,
        domain: Domain<'gc>,
    ) -> Result<(), Error> {
        let mut read = Reader::new(abc.as_ref());

        let abc_file = Rc::new(read.read()?);
        let tunit = TranslationUnit::from_abc(abc_file.clone(), domain, context.gc_context);

        for i in (0..abc_file.scripts.len()).rev() {
            let mut script = tunit.load_script(i as u32, context)?;

            if !lazy_init {
                script.globals(context)?;
            }
        }

        Ok(())
    }

    pub fn global_domain(&self) -> Domain<'gc> {
        self.globals
    }

    /// Push a value onto the operand stack.
    fn push(&mut self, value: impl Into<Value<'gc>>) {
        let mut value = value.into();
        if let Value::Object(o) = value {
            if let Some(prim) = o.as_primitive() {
                value = *prim;
            }
        }

        avm_debug!(self, "Stack push {}: {:?}", self.stack.len(), value);
        self.stack.push(value);
    }

    /// Retrieve the top-most value on the operand stack.
    #[allow(clippy::let_and_return)]
    fn pop(&mut self) -> Value<'gc> {
        let value = self.stack.pop().unwrap_or_else(|| {
            log::warn!("Avm1::pop: Stack underflow");
            Value::Undefined
        });

        avm_debug!(self, "Stack pop {}: {:?}", self.stack.len(), value);

        value
    }

    fn pop_args(&mut self, arg_count: u32) -> Vec<Value<'gc>> {
        let mut args = vec![Value::Undefined; arg_count as usize];
        for arg in args.iter_mut().rev() {
            *arg = self.pop();
        }
        args
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
