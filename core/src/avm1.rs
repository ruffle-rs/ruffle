use crate::avm1::function::{Avm1Function, FunctionObject};
use crate::avm1::globals::create_globals;
use crate::avm1::object::search_prototype;
use crate::avm1::return_value::ReturnValue;
use crate::backend::navigator::{NavigationMethod, RequestOptions};
use crate::context::UpdateContext;
use crate::prelude::*;
use gc_arena::{GcCell, MutationContext};
use rand::Rng;
use std::collections::HashMap;
use std::convert::TryInto;
use url::form_urlencoded;

use swf::avm1::read::Reader;
use swf::avm1::types::{Action, Function};

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

use crate::avm1::error::ExecutionError;
use crate::avm1::listeners::SystemListener;
use crate::avm1::value::f64_to_wrapping_u32;
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

type Error = Box<dyn std::error::Error>;

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

    /// Add a stack frame that executes code in initializer scope
    pub fn insert_stack_frame_for_init_action(
        &mut self,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        code: SwfSlice,
        action_context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
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

    /// Perform some action with the current stack frame's reader.
    ///
    /// This function constructs a reader based off the current stack frame's
    /// reader. You are permitted to mutate the stack frame as you wish. If the
    /// stack frame we started with still exists in the same location on the
    /// stack, it's PC will be updated to the Reader's current PC.
    ///
    /// Stack frame identity (for the purpose of the above paragraph) is
    /// determined by the data pointed to by the `SwfSlice` of a given frame.
    ///
    /// # Warnings
    ///
    /// It is incorrect to call this function multiple times in the same stack.
    /// Doing so will result in any changes in duplicate readers being ignored.
    /// Always pass the borrowed reader into functions that need it.
    fn with_current_reader_mut<F, R>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        func: F,
    ) -> Result<R, ExecutionError>
    where
        F: FnOnce(
            &mut Self,
            &mut Reader<'_>,
            &mut UpdateContext<'_, 'gc, '_>,
        ) -> Result<R, ExecutionError>,
    {
        let (frame_cell, swf_version, data, pc) = {
            let frame = self
                .stack_frames
                .last()
                .ok_or(ExecutionError::NoStackFrame)?;
            let mut frame_ref = frame.write(context.gc_context);
            frame_ref.lock()?;

            (
                *frame,
                frame_ref.swf_version(),
                frame_ref.data(),
                frame_ref.pc(),
            )
        };

        let mut read = Reader::new(data.as_ref(), swf_version);
        read.seek(pc.try_into().unwrap());

        let r = func(self, &mut read, context);

        let mut frame_ref = frame_cell.write(context.gc_context);
        frame_ref.unlock_execution();
        frame_ref.set_pc(read.pos());

        r
    }

    /// Destroy the current stack frame (if there is one).
    ///
    /// The given return value will be pushed on the stack if there is a
    /// function to return it to. Otherwise, it will be discarded.
    ///
    /// NOTE: This means that if you are starting a brand new AVM stack just to
    /// get it's return value, you won't get that value. Instead, retain a cell
    /// referencing the oldest activation frame and use that to retrieve the
    /// return value.
    fn retire_stack_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        return_value: Value<'gc>,
    ) -> Result<(), ExecutionError> {
        if let Some(frame) = self.current_stack_frame() {
            self.stack_frames.pop();

            let can_return = frame.read().can_return() && !self.stack_frames.is_empty();
            if can_return {
                frame
                    .write(context.gc_context)
                    .set_return_value(return_value.clone());

                self.push(return_value);
            }
        }

        Ok(())
    }

    /// Execute the AVM stack until it is exhausted.
    pub fn run_stack_till_empty(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), ExecutionError> {
        while !self.stack_frames.is_empty() {
            self.with_current_reader_mut(context, |this, r, context| {
                this.do_next_action(context, r)
            })?;
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
    ) -> Result<(), ExecutionError> {
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
                self.with_current_reader_mut(context, |this, r, context| {
                    this.do_next_action(context, r)
                })?;
            }

            Ok(())
        } else {
            Err(ExecutionError::FrameNotOnStack)
        }
    }

    /// Run a single action from a given action reader.
    fn do_next_action(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut Reader<'_>,
    ) -> Result<(), ExecutionError> {
        let data = self.current_stack_frame().unwrap().read().data();

        if reader.pos() >= (data.end - data.start) {
            //Executing beyond the end of a function constitutes an implicit return.
            self.retire_stack_frame(context, Value::Undefined)?;
        } else if let Some(action) = reader.read_action()? {
            avm_debug!("Action: {:?}", action);

            let result = match action {
                Action::Add => self.action_add(context),
                Action::Add2 => self.action_add_2(context),
                Action::And => self.action_and(context),
                Action::AsciiToChar => self.action_ascii_to_char(context),
                Action::BitAnd => self.action_bit_and(context),
                Action::BitLShift => self.action_bit_lshift(context),
                Action::BitOr => self.action_bit_or(context),
                Action::BitRShift => self.action_bit_rshift(context),
                Action::BitURShift => self.action_bit_urshift(context),
                Action::BitXor => self.action_bit_xor(context),
                Action::Call => self.action_call(context),
                Action::CallFunction => self.action_call_function(context),
                Action::CallMethod => self.action_call_method(context),
                Action::CastOp => self.action_cast_op(context),
                Action::CharToAscii => self.action_char_to_ascii(context),
                Action::CloneSprite => self.action_clone_sprite(context),
                Action::ConstantPool(constant_pool) => {
                    self.action_constant_pool(context, &constant_pool[..])
                }
                Action::Decrement => self.action_decrement(context),
                Action::DefineFunction {
                    name,
                    params,
                    actions,
                } => self.action_define_function(context, &name, &params[..], actions),
                Action::DefineFunction2(func) => self.action_define_function_2(context, &func),
                Action::DefineLocal => self.action_define_local(context),
                Action::DefineLocal2 => self.action_define_local_2(context),
                Action::Delete => self.action_delete(context),
                Action::Delete2 => self.action_delete_2(context),
                Action::Divide => self.action_divide(context),
                Action::EndDrag => self.action_end_drag(context),
                Action::Enumerate => self.action_enumerate(context),
                Action::Enumerate2 => self.action_enumerate_2(context),
                Action::Equals => self.action_equals(context),
                Action::Equals2 => self.action_equals_2(context),
                Action::Extends => self.action_extends(context),
                Action::GetMember => self.action_get_member(context),
                Action::GetProperty => self.action_get_property(context),
                Action::GetTime => self.action_get_time(context),
                Action::GetVariable => self.action_get_variable(context),
                Action::GetUrl { url, target } => self.action_get_url(context, &url, &target),
                Action::GetUrl2 {
                    send_vars_method,
                    is_target_sprite,
                    is_load_vars,
                } => {
                    self.action_get_url_2(context, send_vars_method, is_target_sprite, is_load_vars)
                }
                Action::GotoFrame(frame) => self.action_goto_frame(context, frame),
                Action::GotoFrame2 {
                    set_playing,
                    scene_offset,
                } => self.action_goto_frame_2(context, set_playing, scene_offset),
                Action::Greater => self.action_greater(context),
                Action::GotoLabel(label) => self.action_goto_label(context, &label),
                Action::If { offset } => self.action_if(context, offset, reader),
                Action::Increment => self.action_increment(context),
                Action::InitArray => self.action_init_array(context),
                Action::InitObject => self.action_init_object(context),
                Action::ImplementsOp => self.action_implements_op(context),
                Action::InstanceOf => self.action_instance_of(context),
                Action::Jump { offset } => self.action_jump(context, offset, reader),
                Action::Less => self.action_less(context),
                Action::Less2 => self.action_less_2(context),
                Action::MBAsciiToChar => self.action_mb_ascii_to_char(context),
                Action::MBCharToAscii => self.action_mb_char_to_ascii(context),
                Action::MBStringLength => self.action_mb_string_length(context),
                Action::MBStringExtract => self.action_mb_string_extract(context),
                Action::Modulo => self.action_modulo(context),
                Action::Multiply => self.action_multiply(context),
                Action::NextFrame => self.action_next_frame(context),
                Action::NewMethod => self.action_new_method(context),
                Action::NewObject => self.action_new_object(context),
                Action::Not => self.action_not(context),
                Action::Or => self.action_or(context),
                Action::Play => self.action_play(context),
                Action::Pop => self.action_pop(context),
                Action::PreviousFrame => self.action_prev_frame(context),
                Action::Push(values) => self.action_push(context, &values[..]),
                Action::PushDuplicate => self.action_push_duplicate(context),
                Action::RandomNumber => self.action_random_number(context),
                Action::RemoveSprite => self.action_remove_sprite(context),
                Action::Return => self.action_return(context),
                Action::SetMember => self.action_set_member(context),
                Action::SetProperty => self.action_set_property(context),
                Action::SetTarget(target) => self.action_set_target(context, &target),
                Action::SetTarget2 => self.action_set_target2(context),
                Action::SetVariable => self.action_set_variable(context),
                Action::StackSwap => self.action_stack_swap(context),
                Action::StartDrag => self.action_start_drag(context),
                Action::Stop => self.action_stop(context),
                Action::StopSounds => self.action_stop_sounds(context),
                Action::StoreRegister(register) => self.action_store_register(context, register),
                Action::StrictEquals => self.action_strict_equals(context),
                Action::StringAdd => self.action_string_add(context),
                Action::StringEquals => self.action_string_equals(context),
                Action::StringExtract => self.action_string_extract(context),
                Action::StringGreater => self.action_string_greater(context),
                Action::StringLength => self.action_string_length(context),
                Action::StringLess => self.action_string_less(context),
                Action::Subtract => self.action_subtract(context),
                Action::TargetPath => self.action_target_path(context),
                Action::ToggleQuality => self.toggle_quality(context),
                Action::ToInteger => self.action_to_integer(context),
                Action::ToNumber => self.action_to_number(context),
                Action::ToString => self.action_to_string(context),
                Action::Trace => self.action_trace(context),
                Action::TypeOf => self.action_type_of(context),
                Action::WaitForFrame {
                    frame,
                    num_actions_to_skip,
                } => self.action_wait_for_frame(context, frame, num_actions_to_skip, reader),
                Action::WaitForFrame2 {
                    num_actions_to_skip,
                } => self.action_wait_for_frame_2(context, num_actions_to_skip, reader),
                Action::With { actions } => self.action_with(context, actions),
                _ => self.unknown_op(context, action),
            };
            if let Err(e) = result {
                log::error!("AVM1 error: {}", e);
                return Err(ExecutionError::ScriptError(e));
            }
        } else {
            //The explicit end opcode was encountered so return here
            self.retire_stack_frame(context, Value::Undefined)?;
        }

        Ok(())
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
    ) -> Result<Option<DisplayObject<'gc>>, Error> {
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
    ) -> Result<Option<Object<'gc>>, Error> {
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
    ) -> Result<ReturnValue<'gc>, Error> {
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
    ) -> Result<(), Error> {
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

    fn unknown_op(
        &mut self,
        _context: &mut UpdateContext,
        action: swf::avm1::types::Action,
    ) -> Result<(), Error> {
        log::error!("Unknown AVM1 opcode: {:?}", action);
        Err("Unknown op".into())
    }

    fn action_add(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop();
        let b = self.pop();
        self.push(b.into_number_v1() + a.into_number_v1());
        Ok(())
    }

    fn action_add_2(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // ECMA-262 s. 11.6.1
        let a = self.pop();
        let b = self.pop();

        // TODO(Herschel):
        if let Value::String(a) = a {
            let mut s = b.coerce_to_string(self, context)?.to_string();
            s.push_str(&a);
            self.push(s);
        } else if let Value::String(mut b) = b {
            b.push_str(&a.coerce_to_string(self, context)?);
            self.push(b);
        } else {
            let result = b.coerce_to_f64(self, context)? + a.coerce_to_f64(self, context)?;
            self.push(result);
        }
        Ok(())
    }

    fn action_and(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 logical and
        let a = self.pop();
        let b = self.pop();
        let version = self.current_swf_version();
        let result = b.as_bool(version) && a.as_bool(version);
        self.push(Value::from_bool(result, self.current_swf_version()));
        Ok(())
    }

    fn action_ascii_to_char(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let val = (self.pop().coerce_to_f64(self, context)? as u8) as char;
        self.push(val.to_string());
        Ok(())
    }

    fn action_char_to_ascii(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let val = self.pop();
        let string = val.coerce_to_string(self, context)?;
        let result = string.bytes().next().unwrap_or(0);
        self.push(result);
        Ok(())
    }

    fn action_clone_sprite(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let depth = self.pop();
        let target = self.pop();
        let source = self.pop();
        let start_clip = self.target_clip_or_root();
        let source_clip = self.resolve_target_display_object(context, start_clip, source)?;

        if let Some(movie_clip) = source_clip.and_then(|o| o.as_movie_clip()) {
            let _ = globals::movie_clip::duplicate_movie_clip_with_bias(
                movie_clip,
                self,
                context,
                &[target, depth],
                0,
            );
        } else {
            log::warn!("CloneSprite: Source is not a movie clip");
        }

        Ok(())
    }

    fn action_bit_and(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_u32(self, context)?;
        let b = self.pop().coerce_to_u32(self, context)?;
        let result = a & b;
        self.push(result);
        Ok(())
    }

    fn action_bit_lshift(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_i32(self, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop().coerce_to_i32(self, context)?;
        let result = b << a;
        self.push(result);
        Ok(())
    }

    fn action_bit_or(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_u32(self, context)?;
        let b = self.pop().coerce_to_u32(self, context)?;
        let result = a | b;
        self.push(result);
        Ok(())
    }

    fn action_bit_rshift(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_i32(self, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop().coerce_to_i32(self, context)?;
        let result = b >> a;
        self.push(result);
        Ok(())
    }

    fn action_bit_urshift(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let a = self.pop().coerce_to_u32(self, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop().coerce_to_u32(self, context)?;
        let result = b >> a;
        self.push(result);
        Ok(())
    }

    fn action_bit_xor(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_u32(self, context)?;
        let b = self.pop().coerce_to_u32(self, context)?;
        let result = b ^ a;
        self.push(result);
        Ok(())
    }

    fn action_call(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // Runs any actions on the given frame.
        let frame = self.pop();
        let clip = self.target_clip_or_root();
        if let Some(clip) = clip.as_movie_clip() {
            // Use frame # if parameter is a number, otherwise cast to string and check for frame labels.
            let frame = if let Value::Number(frame) = frame {
                let frame = f64_to_wrapping_u32(frame);
                if frame >= 1 && frame <= u32::from(clip.total_frames()) {
                    Some(frame as u16)
                } else {
                    None
                }
            } else {
                let frame_label = frame.coerce_to_string(self, context)?;
                clip.frame_label_to_number(&frame_label)
            };

            if let Some(frame) = frame {
                // We must run the actions in the order that the tags appear,
                // so we want to push the stack frames in reverse order.
                for action in clip.actions_on_frame(context, frame).rev() {
                    self.insert_stack_frame_for_action(
                        self.target_clip_or_root(),
                        self.current_swf_version(),
                        action,
                        context,
                    );
                }
            } else {
                log::warn!("Call: Invalid frame {:?}", frame);
            }
        } else {
            log::warn!("Call: Expected MovieClip");
        }
        Ok(())
    }

    fn action_call_function(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let fn_name_value = self.pop();
        let fn_name = fn_name_value.coerce_to_string(self, context)?;
        let mut args = Vec::new();
        let num_args = self.pop().coerce_to_f64(self, context)? as i64; // TODO(Herschel): max arg count?
        for _ in 0..num_args {
            args.push(self.pop());
        }

        let target_fn = self
            .get_variable(context, &fn_name)?
            .resolve(self, context)?;

        let this = self
            .target_clip_or_root()
            .object()
            .coerce_to_object(self, context);
        let result = target_fn.call(self, context, this, None, &args)?;
        self.push(result);

        Ok(())
    }

    fn action_call_method(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let method_name = self.pop();
        let object_val = self.pop();
        let object = value_object::ValueObject::boxed(self, context, object_val);
        let num_args = self.pop().coerce_to_f64(self, context)? as i64; // TODO(Herschel): max arg count?
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.pop());
        }

        match method_name {
            Value::Undefined | Value::Null => {
                let this = self
                    .target_clip_or_root()
                    .object()
                    .coerce_to_object(self, context);
                let result = object.call(self, context, this, None, &args)?;
                self.push(result);
            }
            Value::String(name) => {
                if name.is_empty() {
                    let result = object.call(self, context, object, None, &args)?;
                    self.push(result);
                } else {
                    let result = object.call_method(&name, &args, self, context)?;
                    self.push(result);
                }
            }
            _ => {
                return Err(format!(
                    "Invalid method name, expected string but found {:?}",
                    method_name
                )
                .into())
            }
        }

        Ok(())
    }

    fn action_cast_op(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let obj = self.pop().coerce_to_object(self, context);
        let constr = self.pop().coerce_to_object(self, context);

        let prototype = constr
            .get("prototype", self, context)?
            .coerce_to_object(self, context);

        if obj.is_instance_of(self, context, constr, prototype)? {
            self.push(obj);
        } else {
            self.push(Value::Null);
        }

        Ok(())
    }

    fn action_constant_pool(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        constant_pool: &[&str],
    ) -> Result<(), Error> {
        self.constant_pool = GcCell::allocate(
            context.gc_context,
            constant_pool.iter().map(|s| (*s).to_string()).collect(),
        );
        self.current_stack_frame()
            .unwrap()
            .write(context.gc_context)
            .set_constant_pool(self.constant_pool);

        Ok(())
    }

    fn action_decrement(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_f64(self, context)?;
        self.push(a - 1.0);
        Ok(())
    }

    fn action_define_function(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
        params: &[&str],
        actions: &[u8],
    ) -> Result<(), Error> {
        let swf_version = self.current_stack_frame().unwrap().read().swf_version();
        let func_data = self
            .current_stack_frame()
            .unwrap()
            .read()
            .data()
            .to_subslice(actions)
            .unwrap();
        let scope = Scope::new_closure_scope(
            self.current_stack_frame().unwrap().read().scope_cell(),
            context.gc_context,
        );
        let constant_pool = self.current_stack_frame().unwrap().read().constant_pool();
        let func = Avm1Function::from_df1(
            swf_version,
            func_data,
            name,
            params,
            scope,
            constant_pool,
            self.target_clip_or_root(),
        );
        let prototype =
            ScriptObject::object(context.gc_context, Some(self.prototypes.object)).into();
        let func_obj = FunctionObject::function(
            context.gc_context,
            func,
            Some(self.prototypes.function),
            Some(prototype),
        );
        if name == "" {
            self.push(func_obj);
        } else {
            self.current_stack_frame()
                .unwrap()
                .read()
                .define(name, func_obj, context.gc_context);
        }

        Ok(())
    }

    fn action_define_function_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        action_func: &Function,
    ) -> Result<(), Error> {
        let swf_version = self.current_stack_frame().unwrap().read().swf_version();
        let func_data = self
            .current_stack_frame()
            .unwrap()
            .read()
            .data()
            .to_subslice(action_func.actions)
            .unwrap();
        let scope = Scope::new_closure_scope(
            self.current_stack_frame().unwrap().read().scope_cell(),
            context.gc_context,
        );
        let constant_pool = self.current_stack_frame().unwrap().read().constant_pool();
        let func = Avm1Function::from_df2(
            swf_version,
            func_data,
            action_func,
            scope,
            constant_pool,
            self.base_clip(),
        );
        let prototype =
            ScriptObject::object(context.gc_context, Some(self.prototypes.object)).into();
        let func_obj = FunctionObject::function(
            context.gc_context,
            func,
            Some(self.prototypes.function),
            Some(prototype),
        );
        if action_func.name == "" {
            self.push(func_obj);
        } else {
            self.current_stack_frame().unwrap().read().define(
                action_func.name,
                func_obj,
                context.gc_context,
            );
        }

        Ok(())
    }

    fn action_define_local(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let value = self.pop();
        let name_val = self.pop();
        let name = name_val.coerce_to_string(self, context)?;
        self.current_stack_frame()
            .unwrap()
            .read()
            .define(&name, value, context.gc_context);
        Ok(())
    }

    fn action_define_local_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let name_val = self.pop();
        let name = name_val.coerce_to_string(self, context)?;
        self.current_stack_frame().unwrap().read().define(
            &name,
            Value::Undefined,
            context.gc_context,
        );
        Ok(())
    }

    fn action_delete(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name_val = self.pop();
        let name = name_val.coerce_to_string(self, context)?;
        let object = self.pop();

        if let Value::Object(object) = object {
            let success = object.delete(self, context.gc_context, &name);
            self.push(success);
        } else {
            log::warn!("Cannot delete property {} from {:?}", name, object);
            self.push(false);
        }

        Ok(())
    }

    fn action_delete_2(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name_val = self.pop();
        let name = name_val.coerce_to_string(self, context)?;

        //Fun fact: This isn't in the Adobe SWF19 spec, but this opcode returns
        //a boolean based on if the delete actually deleted something.
        let did_exist = self
            .current_stack_frame()
            .unwrap()
            .read()
            .is_defined(self, context, &name);

        self.current_stack_frame().unwrap().read().scope().delete(
            self,
            context,
            &name,
            context.gc_context,
        );
        self.push(did_exist);

        Ok(())
    }

    fn action_divide(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 divide
        let a = self.pop().coerce_to_f64(self, context)?;
        let b = self.pop().coerce_to_f64(self, context)?;

        // TODO(Herschel): SWF19: "If A is zero, the result NaN, Infinity, or -Infinity is pushed to the in SWF 5 and later.
        // In SWF 4, the result is the string #ERROR#.""
        // Seems to be untrue for SWF v4, I get 1.#INF.

        self.push(b / a);
        Ok(())
    }

    fn action_end_drag(&mut self, context: &mut UpdateContext) -> Result<(), Error> {
        *context.drag_object = None;
        Ok(())
    }

    fn action_enumerate(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name_value = self.pop();
        let name = name_value.coerce_to_string(self, context)?;
        self.push(Value::Null); // Sentinel that indicates end of enumeration
        let object = self
            .current_stack_frame()
            .unwrap()
            .read()
            .resolve(&name, self, context)?
            .resolve(self, context)?;

        match object {
            Value::Object(ob) => {
                for k in ob.get_keys(self).into_iter().rev() {
                    self.push(k);
                }
            }
            _ => log::error!("Cannot enumerate properties of {}", name),
        };

        Ok(())
    }

    fn action_enumerate_2(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let value = self.pop();

        self.push(Value::Null); // Sentinel that indicates end of enumeration

        if let Value::Object(object) = value {
            for k in object.get_keys(self).into_iter().rev() {
                self.push(k);
            }
        } else {
            log::warn!("Cannot enumerate {:?}", value);
        }

        Ok(())
    }

    #[allow(clippy::float_cmp)]
    fn action_equals(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 equality
        let a = self.pop();
        let b = self.pop();
        let result = b.into_number_v1() == a.into_number_v1();
        self.push(Value::from_bool(result, self.current_swf_version()));
        Ok(())
    }

    #[allow(clippy::float_cmp)]
    fn action_equals_2(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // Version >=5 equality
        let a = self.pop();
        let b = self.pop();
        let result = b.abstract_eq(a, self, context, false)?;
        self.push(result);
        Ok(())
    }

    fn action_extends(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let superclass = self.pop().coerce_to_object(self, context);
        let subclass = self.pop().coerce_to_object(self, context);

        //TODO: What happens if we try to extend an object which has no `prototype`?
        //e.g. `class Whatever extends Object.prototype` or `class Whatever extends 5`
        let super_proto = superclass
            .get("prototype", self, context)?
            .coerce_to_object(self, context);

        let sub_prototype: Object<'gc> =
            ScriptObject::object(context.gc_context, Some(super_proto)).into();

        sub_prototype.set("constructor", superclass.into(), self, context)?;
        sub_prototype.set("__constructor__", superclass.into(), self, context)?;
        subclass.set("prototype", sub_prototype.into(), self, context)?;

        Ok(())
    }

    fn action_get_member(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name_val = self.pop();
        let name = name_val.coerce_to_string(self, context)?;
        let object_val = self.pop();
        let object = value_object::ValueObject::boxed(self, context, object_val);

        let result = object.get(&name, self, context)?;
        self.push(result);

        Ok(())
    }

    fn action_get_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let prop_index = self.pop().into_number_v1() as usize;
        let path = self.pop();
        let ret = if let Some(target) = self.target_clip() {
            if let Some(clip) = self.resolve_target_display_object(context, target, path)? {
                let display_properties = self.display_properties;
                let props = display_properties.write(context.gc_context);
                if let Some(property) = props.get_by_index(prop_index) {
                    property.get(self, context, clip)?
                } else {
                    log::warn!("GetProperty: Invalid property index {}", prop_index);
                    Value::Undefined
                }
            } else {
                //log::warn!("GetProperty: Invalid target {}", path);
                Value::Undefined
            }
        } else {
            log::warn!("GetProperty: Invalid base clip");
            Value::Undefined
        };
        self.push(ret);
        Ok(())
    }

    fn action_get_time(&mut self, context: &mut UpdateContext) -> Result<(), Error> {
        let time = context.navigator.time_since_launch().as_millis() as u32;
        self.push(time);
        Ok(())
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

    fn action_get_variable(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let var_path = self.pop();
        let path = var_path.coerce_to_string(self, context)?;

        self.get_variable(context, &path)?.push(self);

        Ok(())
    }

    fn action_get_url(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        url: &str,
        target: &str,
    ) -> Result<(), Error> {
        if target.starts_with("_level") && target.len() > 6 {
            let url = url.to_string();
            let level_id = target[6..].parse::<u32>()?;
            let fetch = context.navigator.fetch(&url, RequestOptions::get());
            let level = self.resolve_level(level_id, context);

            let process = context.load_manager.load_movie_into_clip(
                context.player.clone().unwrap(),
                level,
                fetch,
                None,
            );
            context.navigator.spawn_future(process);

            return Ok(());
        }

        if let Some(fscommand) = fscommand::parse(url) {
            return fscommand::handle(fscommand, self, context);
        }

        context
            .navigator
            .navigate_to_url(url.to_owned(), Some(target.to_owned()), None);

        Ok(())
    }

    fn action_get_url_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_method: swf::avm1::types::SendVarsMethod,
        is_target_sprite: bool,
        is_load_vars: bool,
    ) -> Result<(), Error> {
        // TODO: Support `LoadVariablesFlag`, `LoadTargetFlag`
        // TODO: What happens if there's only one string?
        let target = self.pop();
        let url_val = self.pop();
        let url = url_val.coerce_to_string(self, context)?;

        if let Some(fscommand) = fscommand::parse(&url) {
            return fscommand::handle(fscommand, self, context);
        }

        let window_target = target.coerce_to_string(self, context)?;
        let clip_target: Option<DisplayObject<'gc>> = if is_target_sprite {
            if let Value::Object(target) = target {
                target.as_display_object()
            } else {
                let start = self.target_clip_or_root();
                self.resolve_target_display_object(context, start, target.clone())?
            }
        } else {
            Some(self.target_clip_or_root())
        };

        if is_load_vars {
            if let Some(clip_target) = clip_target {
                let target_obj = clip_target
                    .as_movie_clip()
                    .unwrap()
                    .object()
                    .coerce_to_object(self, context);
                let (url, opts) = self.locals_into_request_options(
                    context,
                    url,
                    NavigationMethod::from_send_vars_method(swf_method),
                );
                let fetch = context.navigator.fetch(&url, opts);
                let process = context.load_manager.load_form_into_object(
                    context.player.clone().unwrap(),
                    target_obj,
                    fetch,
                );

                context.navigator.spawn_future(process);
            }

            return Ok(());
        } else if is_target_sprite {
            if let Some(clip_target) = clip_target {
                let (url, opts) = self.locals_into_request_options(
                    context,
                    url,
                    NavigationMethod::from_send_vars_method(swf_method),
                );
                let fetch = context.navigator.fetch(&url, opts);
                let process = context.load_manager.load_movie_into_clip(
                    context.player.clone().unwrap(),
                    clip_target,
                    fetch,
                    None,
                );
                context.navigator.spawn_future(process);
            }

            return Ok(());
        } else {
            let vars = match NavigationMethod::from_send_vars_method(swf_method) {
                Some(method) => Some((method, self.locals_into_form_values(context))),
                None => None,
            };

            context.navigator.navigate_to_url(
                url.to_string(),
                Some(window_target.to_string()),
                vars,
            );
        }

        Ok(())
    }

    fn action_goto_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        frame: u16,
    ) -> Result<(), Error> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                // The frame on the stack is 0-based, not 1-based.
                clip.goto_frame(self, context, frame + 1, true);
            } else {
                log::error!("GotoFrame failed: Target is not a MovieClip");
            }
        } else {
            log::error!("GotoFrame failed: Invalid target");
        }
        Ok(())
    }

    fn action_goto_frame_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        set_playing: bool,
        scene_offset: u16,
    ) -> Result<(), Error> {
        // Version 4+ gotoAndPlay/gotoAndStop
        // Param can either be a frame number or a frame label.
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                let frame = self.pop();
                let _ = globals::movie_clip::goto_frame(
                    clip,
                    self,
                    context,
                    &[frame],
                    !set_playing,
                    scene_offset,
                );
            } else {
                log::warn!("GotoFrame2: Target is not a MovieClip");
            }
        } else {
            log::warn!("GotoFrame2: Invalid target");
        }
        Ok(())
    }

    fn action_goto_label(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        label: &str,
    ) -> Result<(), Error> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                if let Some(frame) = clip.frame_label_to_number(label) {
                    clip.goto_frame(self, context, frame, true);
                } else {
                    log::warn!("GoToLabel: Frame label '{}' not found", label);
                }
            } else {
                log::warn!("GoToLabel: Target is not a MovieClip");
            }
        } else {
            log::warn!("GoToLabel: Invalid target");
        }
        Ok(())
    }

    fn action_if(
        &mut self,
        _context: &mut UpdateContext,
        jump_offset: i16,
        reader: &mut Reader<'_>,
    ) -> Result<(), Error> {
        let val = self.pop();
        if val.as_bool(self.current_swf_version()) {
            reader.seek(jump_offset.into());
        }
        Ok(())
    }

    fn action_increment(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_f64(self, context)?;
        self.push(a + 1.0);
        Ok(())
    }

    fn action_init_array(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let num_elements = self.pop().coerce_to_f64(self, context)? as i64;
        let array = ScriptObject::array(context.gc_context, Some(self.prototypes.array));

        for i in 0..num_elements {
            array.set_array_element(i as usize, self.pop(), context.gc_context);
        }

        self.push(Value::Object(array.into()));
        Ok(())
    }

    fn action_init_object(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let num_props = self.pop().coerce_to_f64(self, context)? as i64;
        let object = ScriptObject::object(context.gc_context, Some(self.prototypes.object));
        for _ in 0..num_props {
            let value = self.pop();
            let name_val = self.pop();
            let name = name_val.coerce_to_string(self, context)?;
            object.set(&name, value, self, context)?;
        }

        self.push(Value::Object(object.into()));

        Ok(())
    }

    fn action_implements_op(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let constr = self.pop().coerce_to_object(self, context);
        let count = self.pop().coerce_to_f64(self, context)? as i64; //TODO: Is this coercion actually performed by Flash?
        let mut interfaces = vec![];

        //TODO: If one of the interfaces is not an object, do we leave the
        //whole stack dirty, or...?
        for _ in 0..count {
            interfaces.push(self.pop().coerce_to_object(self, context));
        }

        let mut prototype = constr
            .get("prototype", self, context)?
            .coerce_to_object(self, context);

        prototype.set_interfaces(context.gc_context, interfaces);

        Ok(())
    }

    fn action_instance_of(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let constr = self.pop().coerce_to_object(self, context);
        let obj = self.pop().coerce_to_object(self, context);

        let prototype = constr
            .get("prototype", self, context)?
            .coerce_to_object(self, context);
        let is_instance_of = obj.is_instance_of(self, context, constr, prototype)?;

        self.push(is_instance_of);
        Ok(())
    }

    fn action_jump(
        &mut self,
        _context: &mut UpdateContext,
        jump_offset: i16,
        reader: &mut Reader<'_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Handle out-of-bounds.
        reader.seek(jump_offset.into());
        Ok(())
    }

    fn action_less(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 less than
        let a = self.pop();
        let b = self.pop();
        let result = b.into_number_v1() < a.into_number_v1();
        self.push(Value::from_bool(result, self.current_swf_version()));
        Ok(())
    }

    fn action_less_2(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // ECMA-262 s. 11.8.1
        let a = self.pop();
        let b = self.pop();

        let result = b.abstract_lt(a, self, context)?;

        self.push(result);
        Ok(())
    }

    fn action_greater(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // ECMA-262 s. 11.8.2
        let a = self.pop();
        let b = self.pop();

        let result = a.abstract_lt(b, self, context)?;

        self.push(result);
        Ok(())
    }

    fn action_mb_ascii_to_char(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        use std::convert::TryFrom;
        let val = char::try_from(self.pop().coerce_to_f64(self, context)? as u32)?;
        self.push(val.to_string());
        Ok(())
    }

    fn action_mb_char_to_ascii(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let val = self.pop();
        let s = val.coerce_to_string(self, context)?;
        let result = s.chars().next().unwrap_or('\0') as u32;
        self.push(result);
        Ok(())
    }

    fn action_mb_string_extract(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop().coerce_to_f64(self, context)? as usize;
        let start = self.pop().coerce_to_f64(self, context)? as usize;
        let val = self.pop();
        let s = val.coerce_to_string(self, context)?;
        let result = s[len..len + start].to_string(); // TODO(Herschel): Flash uses UTF-16 internally.
        self.push(result);
        Ok(())
    }

    fn action_mb_string_length(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop();
        let len = val.coerce_to_string(self, context)?.len();
        self.push(len as f64);
        Ok(())
    }

    fn action_multiply(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_f64(self, context)?;
        let b = self.pop().coerce_to_f64(self, context)?;
        self.push(a * b);
        Ok(())
    }

    fn action_modulo(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // TODO: Wrong operands?
        let a = self.pop().coerce_to_f64(self, context)?;
        let b = self.pop().coerce_to_f64(self, context)?;
        self.push(b % a);
        Ok(())
    }

    fn action_not(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let version = self.current_swf_version();
        let val = !self.pop().as_bool(version);
        self.push(Value::from_bool(val, version));
        Ok(())
    }

    fn action_next_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.next_frame(self, context);
            } else {
                log::warn!("NextFrame: Target is not a MovieClip");
            }
        } else {
            log::warn!("NextFrame: Invalid target");
        }
        Ok(())
    }

    fn action_new_method(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let method_name = self.pop();
        let object_val = self.pop();
        let num_args = self.pop().coerce_to_f64(self, context)? as i64;
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.pop());
        }

        let object = value_object::ValueObject::boxed(self, context, object_val);
        let constructor =
            object.get(&method_name.coerce_to_string(self, context)?, self, context)?;
        if let Value::Object(constructor) = constructor {
            let prototype = constructor
                .get("prototype", self, context)?
                .coerce_to_object(self, context);

            let this = prototype.new(self, context, prototype, &args)?;

            this.set("__constructor__", constructor.into(), self, context)?;
            if self.current_swf_version() < 7 {
                this.set("constructor", constructor.into(), self, context)?;
            }

            //TODO: What happens if you `ActionNewMethod` without a method name?
            constructor.call(self, context, this, None, &args)?;

            self.push(this);
        } else {
            log::warn!(
                "Tried to construct with non-object constructor {:?}",
                constructor
            );
            self.push(Value::Undefined);
        }

        Ok(())
    }

    fn action_new_object(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let fn_name_val = self.pop();
        let fn_name = fn_name_val.coerce_to_string(self, context)?;
        let num_args = self.pop().coerce_to_f64(self, context)? as i64;
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.pop());
        }

        let constructor = self
            .stack_frames
            .last()
            .unwrap()
            .clone()
            .read()
            .resolve(&fn_name, self, context)?
            .resolve(self, context)?
            .coerce_to_object(self, context);
        let prototype = constructor
            .get("prototype", self, context)?
            .coerce_to_object(self, context);
        let this = prototype.new(self, context, prototype, &args)?;

        this.set("__constructor__", constructor.into(), self, context)?;
        if self.current_swf_version() < 7 {
            this.set("constructor", constructor.into(), self, context)?;
        }

        constructor.call(self, context, this, None, &args)?;

        self.push(this);

        Ok(())
    }

    fn action_or(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 logical or
        let a = self.pop();
        let b = self.pop();
        let version = self.current_swf_version();
        let result = b.as_bool(version) || a.as_bool(version);
        self.push(Value::from_bool(result, version));
        Ok(())
    }

    fn action_play(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.play(context)
            } else {
                log::warn!("Play: Target is not a MovieClip");
            }
        } else {
            log::warn!("Play: Invalid target");
        }
        Ok(())
    }

    fn action_prev_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.prev_frame(self, context);
            } else {
                log::warn!("PrevFrame: Target is not a MovieClip");
            }
        } else {
            log::warn!("PrevFrame: Invalid target");
        }
        Ok(())
    }

    fn action_pop(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        self.pop();
        Ok(())
    }

    fn action_push(
        &mut self,
        _context: &mut UpdateContext,
        values: &[swf::avm1::types::Value],
    ) -> Result<(), Error> {
        for value in values {
            use swf::avm1::types::Value as SwfValue;
            let value = match value {
                SwfValue::Undefined => Value::Undefined,
                SwfValue::Null => Value::Null,
                SwfValue::Bool(v) => Value::Bool(*v),
                SwfValue::Int(v) => f64::from(*v).into(),
                SwfValue::Float(v) => f64::from(*v).into(),
                SwfValue::Double(v) => (*v).into(),
                SwfValue::Str(v) => (*v).to_string().into(),
                SwfValue::Register(v) => self.current_register(*v),
                SwfValue::ConstantPool(i) => {
                    if let Some(value) = self
                        .current_stack_frame()
                        .unwrap()
                        .read()
                        .constant_pool()
                        .read()
                        .get(*i as usize)
                    {
                        value.to_string().into()
                    } else {
                        log::warn!(
                            "ActionPush: Constant pool index {} out of range (len = {})",
                            i,
                            self.current_stack_frame()
                                .unwrap()
                                .read()
                                .constant_pool()
                                .read()
                                .len()
                        );
                        Value::Undefined
                    }
                }
            };
            self.push(value);
        }
        Ok(())
    }

    fn action_push_duplicate(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let val = self.stack.last().ok_or("Stack underflow")?.clone();
        self.push(val);
        Ok(())
    }

    fn action_random_number(&mut self, context: &mut UpdateContext) -> Result<(), Error> {
        // A max value < 0 will always return 0,
        // and the max value gets converted into an i32, so any number > 2^31 - 1 will return 0.
        let max = self.pop().into_number_v1() as i32;
        let val = if max > 0 {
            context.rng.gen_range(0, max)
        } else {
            0
        };
        self.push(val);
        Ok(())
    }

    fn action_remove_sprite(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let target = self.pop();
        let start_clip = self.target_clip_or_root();
        let target_clip = self.resolve_target_display_object(context, start_clip, target)?;

        if let Some(target_clip) = target_clip.and_then(|o| o.as_movie_clip()) {
            let _ = globals::movie_clip::remove_movie_clip_with_bias(target_clip, context, 0);
        } else {
            log::warn!("RemoveSprite: Source is not a movie clip");
        }
        Ok(())
    }

    fn action_return(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let return_value = self.pop();
        self.retire_stack_frame(context, return_value)?;

        Ok(())
    }

    fn action_set_member(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let value = self.pop();
        let name_val = self.pop();
        let name = name_val.coerce_to_string(self, context)?;

        let object = self.pop().coerce_to_object(self, context);
        object.set(&name, value, self, context)?;

        Ok(())
    }

    fn action_set_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let value = self.pop();
        let prop_index = self.pop().coerce_to_u32(self, context)? as usize;
        let path = self.pop();
        if let Some(target) = self.target_clip() {
            if let Some(clip) = self.resolve_target_display_object(context, target, path)? {
                let display_properties = self.display_properties;
                let props = display_properties.read();
                if let Some(property) = props.get_by_index(prop_index) {
                    property.set(self, context, clip, value)?;
                }
            } else {
                log::warn!("SetProperty: Invalid target");
            }
        } else {
            log::warn!("SetProperty: Invalid base clip");
        }
        Ok(())
    }

    fn action_set_variable(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // Flash 4-style variable
        let value = self.pop();
        let var_path_val = self.pop();
        let var_path = var_path_val.coerce_to_string(self, context)?;
        self.set_variable(context, &var_path, value)
    }

    #[allow(clippy::float_cmp)]
    fn action_strict_equals(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // The same as normal equality but types must match
        let a = self.pop();
        let b = self.pop();
        let result = a == b;
        self.push(result);
        Ok(())
    }

    fn action_set_target(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        target: &str,
    ) -> Result<(), Error> {
        let base_clip = self.base_clip();
        let new_target_clip;
        let root = base_clip.root();
        let start = base_clip.object().coerce_to_object(self, context);
        if target.is_empty() {
            new_target_clip = Some(base_clip);
        } else if let Some(clip) = self
            .resolve_target_path(context, root, start, target)?
            .and_then(|o| o.as_display_object())
        {
            new_target_clip = Some(clip);
        } else {
            log::warn!("SetTarget failed: {} not found", target);
            // TODO: Emulate AVM1 trace error message.
            log::info!(target: "avm_trace", "Target not found: Target=\"{}\" Base=\"{}\"", target, base_clip.path());

            // When SetTarget has an invalid target, subsequent GetVariables act
            // as if they are targeting root, but subsequent Play/Stop/etc.
            // fail silenty.
            new_target_clip = None;
        }

        let stack_frame = self.current_stack_frame().unwrap();
        let mut sf = stack_frame.write(context.gc_context);
        sf.set_target_clip(new_target_clip);

        let scope = sf.scope_cell();
        let clip_obj = sf
            .target_clip()
            .unwrap_or_else(|| sf.base_clip().root())
            .object()
            .coerce_to_object(self, context);

        sf.set_scope(Scope::new_target_scope(scope, clip_obj, context.gc_context));
        Ok(())
    }

    fn action_set_target2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let target = self.pop();
        match target {
            Value::String(target) => {
                return self.action_set_target(context, &target);
            }
            Value::Undefined => {
                // Reset
                let stack_frame = self.current_stack_frame().unwrap();
                let mut sf = stack_frame.write(context.gc_context);
                let base_clip = sf.base_clip();
                sf.set_target_clip(Some(base_clip));
            }
            Value::Object(o) => {
                if let Some(clip) = o.as_display_object() {
                    let stack_frame = self.current_stack_frame().unwrap();
                    let mut sf = stack_frame.write(context.gc_context);
                    // Movieclips can be targetted directly
                    sf.set_target_clip(Some(clip));
                } else {
                    // Other objects get coerced to string
                    let target = target.coerce_to_string(self, context)?;
                    return self.action_set_target(context, &target);
                }
            }
            _ => {
                let target = target.coerce_to_string(self, context)?;
                return self.action_set_target(context, &target);
            }
        };

        let stack_frame = self.current_stack_frame().unwrap();
        let mut sf = stack_frame.write(context.gc_context);
        let scope = sf.scope_cell();
        let clip_obj = sf
            .target_clip()
            .unwrap_or_else(|| sf.base_clip().root())
            .object()
            .coerce_to_object(self, context);
        sf.set_scope(Scope::new_target_scope(scope, clip_obj, context.gc_context));
        Ok(())
    }

    fn action_stack_swap(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop();
        let b = self.pop();
        self.push(a);
        self.push(b);
        Ok(())
    }

    fn action_start_drag(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let target = self.pop();
        let start_clip = self.target_clip_or_root();
        let display_object = self.resolve_target_display_object(context, start_clip, target)?;
        if let Some(display_object) = display_object {
            let lock_center = self.pop();
            let constrain = self.pop().as_bool(self.current_swf_version());
            if constrain {
                let y2 = self.pop();
                let x2 = self.pop();
                let y1 = self.pop();
                let x1 = self.pop();
                start_drag(
                    display_object,
                    self,
                    context,
                    &[lock_center, x1, y1, x2, y2],
                );
            } else {
                start_drag(display_object, self, context, &[lock_center]);
            };
        } else {
            log::warn!("StartDrag: Invalid target");
        }
        Ok(())
    }

    fn action_stop(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.stop(context);
            } else {
                log::warn!("Stop: Target is not a MovieClip");
            }
        } else {
            log::warn!("Stop: Invalid target");
        }
        Ok(())
    }

    fn action_stop_sounds(&mut self, context: &mut UpdateContext) -> Result<(), Error> {
        context.audio.stop_all_sounds();
        Ok(())
    }

    fn action_store_register(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        register: u8,
    ) -> Result<(), Error> {
        // Does NOT pop the value from the stack.
        let val = self.stack.last().ok_or("Stack underflow")?.clone();
        self.set_current_register(register, val, context);

        Ok(())
    }

    fn action_string_add(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // SWFv4 string concatenation
        // TODO(Herschel): Result with non-string operands?
        let a = self.pop();
        let mut b = self.pop().coerce_to_string(self, context)?.to_string();
        b.push_str(&a.coerce_to_string(self, context)?);
        self.push(b);
        Ok(())
    }

    fn action_string_equals(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop();
        let b = self.pop();
        let result = b.coerce_to_string(self, context)? == a.coerce_to_string(self, context)?;
        self.push(Value::from_bool(result, self.current_swf_version()));
        Ok(())
    }

    fn action_string_extract(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // SWFv4 substring
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop().coerce_to_f64(self, context)? as usize;
        let start = self.pop().coerce_to_f64(self, context)? as usize;
        let val = self.pop();
        let s = val.coerce_to_string(self, context)?;
        // This is specifically a non-UTF8 aware substring.
        // SWFv4 only used ANSI strings.
        let result = s
            .bytes()
            .skip(start)
            .take(len)
            .map(|c| c as char)
            .collect::<String>();
        self.push(result);
        Ok(())
    }

    fn action_string_greater(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop();
        let b = self.pop();
        // This is specifically a non-UTF8 aware comparison.
        let result = b
            .coerce_to_string(self, context)?
            .bytes()
            .gt(a.coerce_to_string(self, context)?.bytes());
        self.push(Value::from_bool(result, self.current_swf_version()));
        Ok(())
    }

    fn action_string_length(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // AS1 strlen
        // Only returns byte length.
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop();
        let len = val.coerce_to_string(self, context)?.bytes().len() as f64;
        self.push(len);
        Ok(())
    }

    fn action_string_less(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop();
        let b = self.pop();
        // This is specifically a non-UTF8 aware comparison.
        let result = b
            .coerce_to_string(self, context)?
            .bytes()
            .lt(a.coerce_to_string(self, context)?.bytes());
        self.push(Value::from_bool(result, self.current_swf_version()));
        Ok(())
    }

    fn action_subtract(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop().coerce_to_f64(self, context)?;
        let b = self.pop().coerce_to_f64(self, context)?;
        self.push(b - a);
        Ok(())
    }

    fn action_target_path(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel)
        let _clip = self.pop().coerce_to_object(self, context);
        self.push(Value::Undefined);
        Err("Unimplemented action: TargetPath".into())
    }

    fn toggle_quality(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel): Noop for now? Could chang anti-aliasing on render backend.
        Ok(())
    }

    fn action_to_integer(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let val = self.pop().coerce_to_f64(self, context)?;
        self.push(val.trunc());
        Ok(())
    }

    fn action_to_number(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let val = self.pop().coerce_to_f64(self, context)?;
        self.push(val);
        Ok(())
    }

    fn action_to_string(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let val = self.pop();
        let string = val.coerce_to_string(self, context)?;
        self.push(string);
        Ok(())
    }

    fn action_trace(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let val = self.pop();
        // trace always prints "undefined" even though SWF6 and below normally
        // coerce undefined to "".
        let out = if val == Value::Undefined {
            Cow::Borrowed("undefined")
        } else {
            val.coerce_to_string(self, context)?
        };
        log::info!(target: "avm_trace", "{}", out);
        Ok(())
    }

    fn action_type_of(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let type_of = self.pop().type_of();
        self.push(type_of);
        Ok(())
    }

    fn action_wait_for_frame(
        &mut self,
        _context: &mut UpdateContext,
        _frame: u16,
        num_actions_to_skip: u8,
        r: &mut Reader<'_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Always true for now.
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            skip_actions(r, num_actions_to_skip)?;
        }
        Ok(())
    }

    fn action_wait_for_frame_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        num_actions_to_skip: u8,
        r: &mut Reader<'_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Always true for now.
        let _frame_num = self.pop().coerce_to_f64(self, context)? as u16;
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            skip_actions(r, num_actions_to_skip)?;
        }
        Ok(())
    }

    fn action_with(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        actions: &[u8],
    ) -> Result<(), Error> {
        let object = self.pop().coerce_to_object(self, context);
        let block = self
            .current_stack_frame()
            .unwrap()
            .read()
            .data()
            .to_subslice(actions)
            .unwrap();
        let with_scope = Scope::new_with_scope(
            self.current_stack_frame().unwrap().read().scope_cell(),
            object,
            context.gc_context,
        );
        let new_activation = self
            .current_stack_frame()
            .unwrap()
            .read()
            .to_rescope(block, with_scope);
        self.stack_frames
            .push(GcCell::allocate(context.gc_context, new_activation));
        Ok(())
    }
}

/// Returns whether the given SWF version is case-sensitive.
/// SWFv7 and above is case-sensitive.
pub fn is_swf_case_sensitive(swf_version: u8) -> bool {
    swf_version > 6
}

/// Utility function used by `Avm1::action_wait_for_frame` and
/// `Avm1::action_wait_for_frame_2`.
fn skip_actions(reader: &mut Reader<'_>, num_actions_to_skip: u8) -> Result<(), Error> {
    for _ in 0..num_actions_to_skip {
        reader.read_action()?;
    }

    Ok(())
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
