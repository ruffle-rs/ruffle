use crate::avm1::globals::create_globals;
use crate::avm1::object::search_prototype;
use crate::avm1::return_value::ReturnValue;
use crate::backend::navigator::{NavigationMethod, RequestOptions};
use crate::context::UpdateContext;
use crate::prelude::*;
use gc_arena::{GcCell, MutationContext};
use std::collections::HashMap;
use url::form_urlencoded;

use swf::avm1::read::Reader;

use crate::display_object::{DisplayObject, MovieClip};
use crate::tag_utils::SwfSlice;

#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
pub mod listeners;

mod activation;
pub mod debug;
pub mod error;
mod fscommand;
pub mod function;
pub mod globals;
pub mod object;
mod property;
mod return_value;
mod scope;
pub mod script_object;
pub mod shared_object;
mod sound_object;
mod stack_frame;
mod stage_object;
mod super_object;
mod value;
mod value_object;
pub mod xml_attributes_object;
pub mod xml_idmap_object;
pub mod xml_object;

#[cfg(test)]
mod tests;

use crate::avm1::error::Error;
use crate::avm1::listeners::SystemListener;
use crate::avm1::stack_frame::StackFrame;
pub use activation::Activation;
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

    /// All activation records for the current execution context.
    stack_frames: Vec<GcCell<'gc, Activation<'gc>>>,

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
        self.stack_frames.trace(cc);
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
            stack_frames: vec![],
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

    #[allow(dead_code)]
    pub fn base_clip(&self) -> DisplayObject<'gc> {
        self.current_stack_frame().unwrap().read().base_clip()
    }

    /// The current target clip for the executing code.
    /// This is the movie clip that contains the bytecode.
    /// Timeline actions like `GotoFrame` use this because
    /// a goto after an invalid tellTarget has no effect.
    pub fn target_clip(&self) -> Option<DisplayObject<'gc>> {
        self.current_stack_frame().unwrap().read().target_clip()
    }

    /// The current target clip of the executing code, or `root` if there is none.
    /// Actions that affect `root` after an invalid `tellTarget` will use this.
    ///
    /// The `root` is determined relative to the base clip that defined the
    pub fn target_clip_or_root(&self) -> DisplayObject<'gc> {
        self.current_stack_frame()
            .unwrap()
            .read()
            .target_clip()
            .unwrap_or_else(|| self.base_clip().root())
    }

    /// Convert the current locals pool into a set of form values.
    ///
    /// This is necessary to support form submission from Flash via a couple of
    /// legacy methods, such as the `ActionGetURL2` opcode or `getURL` function.
    ///
    /// WARNING: This does not support user defined virtual properties!
    pub fn locals_into_form_values(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> HashMap<String, String> {
        let mut form_values = HashMap::new();
        let stack_frame = self.current_stack_frame().unwrap();
        let stack_frame = stack_frame.read();
        let scope = stack_frame.scope();
        let locals = scope.locals();
        let keys = locals.get_keys(self);

        for k in keys {
            let v = locals.get(&k, self, context);

            //TODO: What happens if an error occurs inside a virtual property?
            form_values.insert(
                k,
                v.ok()
                    .unwrap_or_else(|| Value::Undefined)
                    .coerce_to_string(self, context)
                    .unwrap_or_else(|_| Cow::Borrowed("undefined"))
                    .to_string(),
            );
        }

        form_values
    }

    /// Construct request options for a fetch operation that may send locals as
    /// form data in the request body or URL.
    pub fn locals_into_request_options<'a>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        url: Cow<'a, str>,
        method: Option<NavigationMethod>,
    ) -> (Cow<'a, str>, RequestOptions) {
        match method {
            Some(method) => {
                let vars = self.locals_into_form_values(context);
                let qstring = form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(vars.iter())
                    .finish();

                match method {
                    NavigationMethod::GET if url.find('?').is_none() => (
                        Cow::Owned(format!("{}?{}", url, qstring)),
                        RequestOptions::get(),
                    ),
                    NavigationMethod::GET => (
                        Cow::Owned(format!("{}&{}", url, qstring)),
                        RequestOptions::get(),
                    ),
                    NavigationMethod::POST => (
                        url,
                        RequestOptions::post(Some((
                            qstring.as_bytes().to_owned(),
                            "application/x-www-form-urlencoded".to_string(),
                        ))),
                    ),
                }
            }
            None => (url, RequestOptions::get()),
        }
    }

    /// Add a stack frame that executes code in timeline scope
    pub fn insert_stack_frame_for_action(
        &mut self,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        code: SwfSlice,
        action_context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        if self.halted {
            // We've been told to ignore all future execution.
            return;
        }

        let global_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::from_global_object(self.globals),
        );
        let clip_obj = active_clip.object().coerce_to_object(self, action_context);
        let child_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        self.stack_frames.push(GcCell::allocate(
            action_context.gc_context,
            Activation::from_action(
                swf_version,
                code,
                child_scope,
                self.constant_pool,
                active_clip,
                clip_obj,
                None,
            ),
        ));
    }

    /// Add a stack frame that executes code in timeline scope
    pub fn insert_stack_frame_for_display_object(
        &mut self,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        action_context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        use crate::tag_utils::SwfMovie;
        use std::sync::Arc;

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
        self.stack_frames.push(GcCell::allocate(
            action_context.gc_context,
            Activation::from_action(
                swf_version,
                SwfSlice {
                    movie: Arc::new(SwfMovie::empty(swf_version)),
                    start: 0,
                    end: 0,
                },
                child_scope,
                self.constant_pool,
                active_clip,
                clip_obj,
                None,
            ),
        ));
    }

    /// Add a stack frame that executes code in initializer scope
    pub fn insert_stack_frame_for_init_action(
        &mut self,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        code: SwfSlice,
        action_context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        if self.halted {
            // We've been told to ignore all future execution.
            return;
        }

        let global_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::from_global_object(self.globals),
        );
        let clip_obj = active_clip.object().coerce_to_object(self, action_context);
        let child_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        self.push(Value::Undefined);
        self.stack_frames.push(GcCell::allocate(
            action_context.gc_context,
            Activation::from_action(
                swf_version,
                code,
                child_scope,
                self.constant_pool,
                active_clip,
                clip_obj,
                None,
            ),
        ));
    }

    /// Add a stack frame that executes code in timeline scope for an object
    /// method, such as an event handler.
    pub fn insert_stack_frame_for_method(
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

        // Grab the property with the given name.
        // Requires a dummy stack frame.
        self.stack_frames.push(GcCell::allocate(
            context.gc_context,
            Activation::from_nothing(swf_version, self.globals, context.gc_context, active_clip),
        ));
        let search_result = search_prototype(Some(obj), name, self, context, obj)
            .and_then(|r| Ok((r.0.resolve(self, context)?, r.1)));
        self.stack_frames.pop();

        // Run the callback.
        // The function exec pushes its own stack frame.
        // The function is now ready to execute with `run_stack_till_empty`.
        if let Ok((callback, base_proto)) = search_result {
            let _ = callback.call(self, context, obj, base_proto, args);
        }
    }

    /// Add a stack frame for any arbitrary code.
    pub fn insert_stack_frame(&mut self, frame: GcCell<'gc, Activation<'gc>>) {
        self.stack_frames.push(frame);
    }

    /// Retrieve the current AVM execution frame.
    ///
    /// Yields None if there is no stack frame.
    pub fn current_stack_frame(&self) -> Option<GcCell<'gc, Activation<'gc>>> {
        self.stack_frames.last().copied()
    }

    /// Checks if there is currently a stack frame.
    ///
    /// This is an indicator if you are currently running from inside or outside the AVM.
    /// This method is cheaper than `current_stack_frame` as it doesn't need to perform a copy.
    pub fn has_stack_frame(&self) -> bool {
        !self.stack_frames.is_empty()
    }

    /// Get the currently executing SWF version.
    pub fn current_swf_version(&self) -> u8 {
        self.current_stack_frame()
            .map(|sf| sf.read().swf_version())
            .unwrap_or(self.player_version)
    }

    /// Returns whether property keys should be case sensitive based on the current SWF version.
    pub fn is_case_sensitive(&self) -> bool {
        is_swf_case_sensitive(self.current_swf_version())
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
        // Push a dummy stack frame.
        self.stack_frames.push(GcCell::allocate(
            context.gc_context,
            Activation::from_nothing(swf_version, self.globals, context.gc_context, active_clip),
        ));
        let listeners = self.system_listeners.get(listener);
        let mut handlers = listeners.prepare_handlers(self, context, method);
        self.stack_frames.pop();

        // Each callback exec pushes its own stack frame.
        // The functions are now ready to execute with `run_stack_till_empty`.
        for (listener, handler) in handlers.drain(..) {
            let _ = handler.call(self, context, listener, None, &args);
        }
    }

    /// Execute the AVM stack until it is exhausted.
    pub fn run_stack_till_empty(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        if self.halted {
            // We've been told to ignore all future execution.
            return Ok(());
        }
        while !self.stack_frames.is_empty() {
            let activation = self.current_stack_frame().ok_or(Error::FrameNotOnStack)?;
            if let Err(e) = self.run_activation(context, activation) {
                if let Error::ThrownValue(error) = &e {
                    let string = error
                        .coerce_to_string(self, context)
                        .unwrap_or_else(|_| Cow::Borrowed("undefined"));
                    log::info!(target: "avm_trace", "{}", string);
                }
                return Err(e);
            }
        }

        // Operand stack should be empty at this point.
        // This is probably a bug on our part,
        // although bytecode could in theory leave data on the stack.
        if !self.stack.is_empty() {
            log::warn!("Operand stack is not empty after execution");
            self.stack.clear();
        }

        Ok(())
    }

    /// Execute the AVM stack until a given activation returns.
    pub fn run_current_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        stop_frame: GcCell<'gc, Activation<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if self.halted {
            // We've been told to ignore all future execution.
            return Ok(());
        }

        let mut stop_frame_id = None;
        for (index, frame) in self.stack_frames.iter().enumerate() {
            if GcCell::ptr_eq(stop_frame, *frame) {
                stop_frame_id = Some(index);
            }
        }

        if let Some(stop_frame_id) = stop_frame_id {
            while self
                .stack_frames
                .get(stop_frame_id)
                .map(|fr| GcCell::ptr_eq(stop_frame, *fr))
                .unwrap_or(false)
            {
                let activation = self.current_stack_frame().ok_or(Error::FrameNotOnStack)?;
                self.run_activation(context, activation)?;
            }

            Ok(())
        } else {
            self.halt();
            Err(Error::FrameNotOnStack)
        }
    }

    /// Execute the AVM stack until a given activation returns.
    pub fn run_activation(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        activation: GcCell<'gc, Activation<'gc>>,
    ) -> Result<(), Error<'gc>> {
        match StackFrame::new(self, activation).run(context) {
            Ok(return_type) => {
                self.stack_frames.pop();

                let can_return = activation.read().can_return() && !self.stack_frames.is_empty();
                if can_return {
                    let return_value = return_type.value();
                    activation
                        .write(context.gc_context)
                        .set_return_value(return_value.clone());

                    self.push(return_value);
                }
                Ok(())
            }
            Err(error) => {
                self.stack_frames.pop();
                if error.is_halting() {
                    self.halt();
                }
                Err(error)
            }
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

    /// Resolves a target value to a display object, relative to a starting display object.
    ///
    /// This is used by any action/function with a parameter that can be either
    /// a display object or a string path referencing the display object.
    /// For example, `removeMovieClip(mc)` takes either a string or a display object.
    ///
    /// This can be an object, dot path, slash path, or weird combination thereof:
    /// `_root/movieClip`, `movieClip.child._parent`, `movieClip:child`, etc.
    /// See the `target_path` test for many examples.
    ///
    /// A target path always resolves via the display list. It can look
    /// at the prototype chain, but not the scope chain.
    pub fn resolve_target_display_object(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        start: DisplayObject<'gc>,
        target: Value<'gc>,
    ) -> Result<Option<DisplayObject<'gc>>, Error<'gc>> {
        // If the value you got was a display object, we can just toss it straight back.
        if let Value::Object(o) = target {
            if let Some(o) = o.as_display_object() {
                return Ok(Some(o));
            }
        }

        // Otherwise, we coerce it into a string and try to resolve it as a path.
        // This means that values like `undefined` will resolve to clips with an instance name of
        // `"undefined"`, for example.
        let path = target.coerce_to_string(self, context)?;
        let root = start.root();
        let start = start.object().coerce_to_object(self, context);
        Ok(self
            .resolve_target_path(context, root, start, &path)?
            .and_then(|o| o.as_display_object()))
    }

    /// Resolves a target path string to an object.
    /// This only returns `Object`; other values will bail out with `None`.
    ///
    /// This can be a dot path, slash path, or weird combination thereof:
    /// `_root/movieClip`, `movieClip.child._parent`, `movieClip:child`, etc.
    /// See the `target_path` test for many examples.
    ///
    /// A target path always resolves via the display list. It can look
    /// at the prototype chain, but not the scope chain.
    pub fn resolve_target_path(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        root: DisplayObject<'gc>,
        start: Object<'gc>,
        path: &str,
    ) -> Result<Option<Object<'gc>>, Error<'gc>> {
        // Empty path resolves immediately to start clip.
        if path.is_empty() {
            return Ok(Some(start));
        }

        // Starting / means an absolute path starting from root.
        // (`/bar` means `_root.bar`)
        let mut path = path.as_bytes();
        let (mut object, mut is_slash_path) = if path[0] == b'/' {
            path = &path[1..];
            (root.object().coerce_to_object(self, context), true)
        } else {
            (start, false)
        };

        let case_sensitive = self.is_case_sensitive();

        // Iterate through each token in the path.
        while !path.is_empty() {
            // Skip any number of leading :
            // `foo`, `:foo`, and `:::foo` are all the same
            while path.get(0) == Some(&b':') {
                path = &path[1..];
            }

            let val = if let b".." | b"../" | b"..:" = &path[..std::cmp::min(path.len(), 3)] {
                // Check for ..
                // SWF-4 style _parent
                if path.get(2) == Some(&b'/') {
                    is_slash_path = true;
                }
                path = path.get(3..).unwrap_or(&[]);
                if let Some(parent) = object.as_display_object().and_then(|o| o.parent()) {
                    parent.object()
                } else {
                    // Tried to get parent of root, bail out.
                    return Ok(None);
                }
            } else {
                // Step until the next delimiter.
                // : . / all act as path delimiters.
                // The only restriction is that after a / appears,
                // . is no longer considered a delimiter.
                // TODO: SWF4 is probably more restrictive.
                let mut pos = 0;
                while pos < path.len() {
                    match path[pos] {
                        b':' => break,
                        b'.' if !is_slash_path => break,
                        b'/' => {
                            is_slash_path = true;
                            break;
                        }
                        _ => (),
                    }
                    pos += 1;
                }

                // Slice out the identifier and step the cursor past the delimiter.
                let ident = &path[..pos];
                path = path.get(pos + 1..).unwrap_or(&[]);

                // Guaranteed to be valid UTF-8.
                let name = unsafe { std::str::from_utf8_unchecked(ident) };

                // Get the value from the object.
                // Resolves display object instances first, then local variables.
                // This is the opposite of general GetMember property access!
                if let Some(child) = object
                    .as_display_object()
                    .and_then(|o| o.get_child_by_name(name, case_sensitive))
                {
                    child.object()
                } else {
                    object.get(&name, self, context).unwrap()
                }
            };

            // Resolve the value to an object while traversing the path.
            object = if let Value::Object(o) = val {
                o
            } else {
                return Ok(None);
            };
        }

        Ok(Some(object))
    }

    /// Gets the value referenced by a target path string.
    ///
    /// This can be a raw variable name, a slash path, a dot path, or weird combination thereof.
    /// For example:
    /// `_root/movieClip.foo`, `movieClip:child:_parent`, `blah`
    /// See the `target_path` test for many examples.
    ///
    /// The string first tries to resolve as target path with a variable name, such as
    /// "a/b/c:foo". The right-most : or . delimits the variable name, with the left side
    /// identifying the target object path. Note that the variable name on the right can
    /// contain a slash in this case. This path is resolved on the scope chain; if
    /// the path does not resolve to an existing property on a scope, the parent scope is
    /// searched. Undefined is returned if no path resolves successfully.
    ///
    /// If there is no variable name, but the path contains slashes, the path will still try
    /// to resolve on the scope chain as above. If this fails to resolve, we consider
    /// it a simple variable name and fall through to the variable case
    /// (i.e. "a/b/c" would be a variable named "a/b/c", not a path).
    ///
    /// Finally, if none of the above applies, it is a normal variable name resovled via the
    /// scope chain.
    pub fn get_variable<'s>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        path: &'s str,
    ) -> Result<ReturnValue<'gc>, Error<'gc>> {
        // Resolve a variable path for a GetVariable action.
        let start = self.target_clip_or_root();

        // Find the right-most : or . in the path.
        // If we have one, we must resolve as a target path.
        // We also check for a / to skip some unnecessary work later.
        let mut has_slash = false;
        let mut var_iter = path.as_bytes().rsplitn(2, |c| match c {
            b':' | b'.' => true,
            b'/' => {
                has_slash = true;
                false
            }
            _ => false,
        });

        let b = var_iter.next();
        let a = var_iter.next();
        if let (Some(path), Some(var_name)) = (a, b) {
            // We have a . or :, so this is a path to an object plus a variable name.
            // We resolve it directly on the targeted object.
            let path = unsafe { std::str::from_utf8_unchecked(path) };
            let var_name = unsafe { std::str::from_utf8_unchecked(var_name) };

            let mut current_scope = Some(self.current_stack_frame().unwrap().read().scope_cell());
            while let Some(scope) = current_scope {
                if let Some(object) =
                    self.resolve_target_path(context, start.root(), *scope.read().locals(), path)?
                {
                    if object.has_property(self, context, var_name) {
                        return Ok(object.get(var_name, self, context)?.into());
                    }
                }
                current_scope = scope.read().parent_cell();
            }

            return Ok(Value::Undefined.into());
        }

        // If it doesn't have a trailing variable, it can still be a slash path.
        // We can skip this step if we didn't find a slash above.
        if has_slash {
            let mut current_scope = Some(self.current_stack_frame().unwrap().read().scope_cell());
            while let Some(scope) = current_scope {
                if let Some(object) =
                    self.resolve_target_path(context, start.root(), *scope.read().locals(), path)?
                {
                    return Ok(object.into());
                }
                current_scope = scope.read().parent_cell();
            }
        }

        // Finally! It's a plain old variable name.
        // Resolve using scope chain, as normal.
        self.current_stack_frame()
            .unwrap()
            .read()
            .resolve(&path, self, context)
    }

    /// Sets the value referenced by a target path string.
    ///
    /// This can be a raw variable name, a slash path, a dot path, or weird combination thereof.
    /// For example:
    /// `_root/movieClip.foo`, `movieClip:child:_parent`, `blah`
    /// See the `target_path` test for many examples.
    ///
    /// The string first tries to resolve as target path with a variable name, such as
    /// "a/b/c:foo". The right-most : or . delimits the variable name, with the left side
    /// identifying the target object path. Note that the variable name on the right can
    /// contain a slash in this case. This target path (sans variable) is resolved on the
    /// scope chain; if the path does not resolve to an existing property on a scope, the
    /// parent scope is searched. If the path does not resolve on any scope, the set fails
    /// and returns immediately. If the path does resolve, the variable name is created
    /// or overwritten on the target scope.
    ///
    /// This differs from `get_variable` because slash paths with no variable segment are invalid;
    /// For example, `foo/bar` sets a property named `foo/bar` on the current stack frame instead
    /// of drilling into the display list.
    ///
    /// If the string does not resolve as a path, the path is considered a normal variable
    /// name and is set on the scope chain as usual.
    pub fn set_variable<'s>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        path: &'s str,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        // Resolve a variable path for a GetVariable action.
        let start = self.target_clip_or_root();

        // If the target clip is invalid, we default to root for the variable path.
        if path.is_empty() {
            return Ok(());
        }

        // Find the right-most : or . in the path.
        // If we have one, we must resolve as a target path.
        let mut var_iter = path.as_bytes().rsplitn(2, |&c| c == b':' || c == b'.');
        let b = var_iter.next();
        let a = var_iter.next();

        if let (Some(path), Some(var_name)) = (a, b) {
            // We have a . or :, so this is a path to an object plus a variable name.
            // We resolve it directly on the targeted object.
            let path = unsafe { std::str::from_utf8_unchecked(path) };
            let var_name = unsafe { std::str::from_utf8_unchecked(var_name) };

            let mut current_scope = Some(self.current_stack_frame().unwrap().read().scope_cell());
            while let Some(scope) = current_scope {
                if let Some(object) =
                    self.resolve_target_path(context, start.root(), *scope.read().locals(), path)?
                {
                    object.set(var_name, value, self, context)?;
                    return Ok(());
                }
                current_scope = scope.read().parent_cell();
            }

            return Ok(());
        }

        // Finally! It's a plain old variable name.
        // Set using scope chain, as normal.
        // This will overwrite the value if the property exists somewhere
        // in the scope chain, otherwise it is created on the top-level object.
        let sf = self.current_stack_frame().unwrap();
        let stack_frame = sf.read();
        let this = stack_frame.this_cell();
        let scope = stack_frame.scope_cell();
        scope.read().set(path, value, self, context, this)?;
        Ok(())
    }

    /// Resolves a path for text field variable binding.
    /// Returns the parent object that owns the variable, and the variable name.
    /// Returns `None` if the path does not yet point to a valid object.
    /// TODO: This can probably be merged with some of the above `resolve_target_path` methods.
    pub fn resolve_text_field_variable_path<'s>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        text_field_parent: DisplayObject<'gc>,
        path: &'s str,
    ) -> Result<Option<(Object<'gc>, &'s str)>, Error<'gc>> {
        // Resolve a variable path for a GetVariable action.
        let start = text_field_parent;

        // Find the right-most : or . in the path.
        // If we have one, we must resolve as a target path.
        // We also check for a / to skip some unnecessary work later.
        let mut has_slash = false;
        let mut var_iter = path.as_bytes().rsplitn(2, |c| match c {
            b':' | b'.' => true,
            b'/' => {
                has_slash = true;
                false
            }
            _ => false,
        });

        let b = var_iter.next();
        let a = var_iter.next();
        if let (Some(path), Some(var_name)) = (a, b) {
            // We have a . or :, so this is a path to an object plus a variable name.
            // We resolve it directly on the targeted object.
            let path = unsafe { std::str::from_utf8_unchecked(path) };
            let var_name = unsafe { std::str::from_utf8_unchecked(var_name) };

            let mut current_scope = Some(self.current_stack_frame().unwrap().read().scope_cell());
            while let Some(scope) = current_scope {
                if let Some(object) =
                    self.resolve_target_path(context, start.root(), *scope.read().locals(), path)?
                {
                    return Ok(Some((object, var_name)));
                }
                current_scope = scope.read().parent_cell();
            }

            return Ok(None);
        }

        // Finally! It's a plain old variable name.
        // Resolve using scope chain, as normal.
        if let Value::Object(object) = start.object() {
            Ok(Some((object, path)))
        } else {
            Ok(None)
        }
    }

    /// Resolve a level by ID.
    ///
    /// If the level does not exist, then it will be created and instantiated
    /// with a script object.
    pub fn resolve_level(
        &mut self,
        level_id: u32,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> DisplayObject<'gc> {
        if let Some(level) = context.levels.get(&level_id) {
            *level
        } else {
            let mut level: DisplayObject<'_> = MovieClip::new(
                SwfSlice::empty(self.base_clip().movie().unwrap()),
                context.gc_context,
            )
            .into();

            level.set_depth(context.gc_context, level_id as i32);
            context.levels.insert(level_id, level);
            level.post_instantiation(self, context, level, None, false);

            level
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

    /// Retrieve a given register value.
    ///
    /// If a given register does not exist, this function yields
    /// Value::Undefined, which is also a valid register value.
    pub fn current_register(&self, id: u8) -> Value<'gc> {
        if self
            .current_stack_frame()
            .map(|sf| sf.read().has_local_register(id))
            .unwrap_or(false)
        {
            self.current_stack_frame()
                .unwrap()
                .read()
                .local_register(id)
                .unwrap_or(Value::Undefined)
        } else {
            self.registers
                .get(id as usize)
                .cloned()
                .unwrap_or(Value::Undefined)
        }
    }

    /// Set a register to a given value.
    ///
    /// If a given register does not exist, this function does nothing.
    pub fn set_current_register(
        &mut self,
        id: u8,
        value: Value<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        if self
            .current_stack_frame()
            .map(|sf| sf.read().has_local_register(id))
            .unwrap_or(false)
        {
            self.current_stack_frame()
                .unwrap()
                .write(context.gc_context)
                .set_local_register(id, value, context.gc_context);
        } else if let Some(v) = self.registers.get_mut(id as usize) {
            *v = value;
        }
    }

    /// Obtain the value of `_root`.
    pub fn root_object(&self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Value<'gc> {
        self.base_clip().root().object()
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

/// Returns whether the given SWF version is case-sensitive.
/// SWFv7 and above is case-sensitive.
pub fn is_swf_case_sensitive(swf_version: u8) -> bool {
    swf_version > 6
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
    avm: &mut Avm1<'gc>,
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
            .coerce_to_f64(avm, context)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut y_min = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(avm, context)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut x_max = args
            .get(3)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(avm, context)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut y_max = args
            .get(4)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(avm, context)
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
