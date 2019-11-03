use crate::avm1::function::Avm1Function;
use crate::avm1::globals::create_globals;
use crate::backend::navigator::NavigationMethod;
use crate::context::UpdateContext;
use crate::prelude::*;
use gc_arena::{GcCell, MutationContext};
use rand::Rng;
use std::collections::HashMap;
use std::convert::TryInto;

use swf::avm1::read::Reader;
use swf::avm1::types::{Action, Function};

use crate::tag_utils::SwfSlice;

#[cfg(test)]
#[macro_use]
mod test_utils;

mod activation;
mod fscommand;
mod function;
pub mod globals;
pub mod object;
mod property;
mod return_value;
mod scope;
pub mod script_object;
mod value;
mod stage_object;

#[cfg(test)]
mod tests;

use activation::Activation;
pub use globals::SystemPrototypes;
pub use object::{Object, ObjectPtr, TObject};
use scope::Scope;
pub use script_object::ScriptObject;
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

    /// The currently installed constant pool.
    constant_pool: Vec<String>,

    /// The global object.
    globals: Object<'gc>,

    /// System builtins that we use internally to construct new objects.
    prototypes: globals::SystemPrototypes<'gc>,

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
        self.prototypes.trace(cc);
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
        let (prototypes, globals) = create_globals(gc_context);

        Self {
            player_version,
            constant_pool: vec![],
            globals,
            prototypes,
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
        let keys = locals.get_keys();

        for k in keys {
            let v = locals.get(&k, self, context);

            //TODO: What happens if an error occurs inside a virtual property?
            form_values.insert(
                k,
                v.ok()
                    .unwrap_or_else(|| Value::Undefined.into())
                    .resolve(self, context)
                    .ok()
                    .unwrap_or(Value::Undefined)
                    .clone()
                    .into_string(),
            );
        }

        form_values
    }

    /// Add a stack frame that executes code in timeline scope
    pub fn insert_stack_frame_for_action(
        &mut self,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        code: SwfSlice,
        action_context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        action_context.start_clip = active_clip;
        action_context.active_clip = active_clip;
        action_context.target_clip = Some(active_clip);
        let global_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::from_global_object(self.globals),
        );
        let clip_obj = active_clip.object().as_object().unwrap();
        let child_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        self.stack_frames.push(GcCell::allocate(
            action_context.gc_context,
            Activation::from_action(swf_version, code, child_scope, clip_obj, None),
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
        action_context.start_clip = active_clip;
        action_context.active_clip = active_clip;
        action_context.target_clip = Some(active_clip);
        let global_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::from_global_object(self.globals),
        );
        let clip_obj = active_clip.object().as_object().unwrap();
        let child_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        self.push(Value::Undefined);
        self.stack_frames.push(GcCell::allocate(
            action_context.gc_context,
            Activation::from_action(swf_version, code, child_scope, clip_obj, None),
        ));
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

    /// Get the currently executing SWF version.
    pub fn current_swf_version(&self) -> u8 {
        self.current_stack_frame()
            .map(|sf| sf.read().swf_version())
            .unwrap_or(self.player_version)
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
    ) -> Result<R, Error>
    where
        F: FnOnce(&mut Self, &mut Reader<'_>, &mut UpdateContext<'_, 'gc, '_>) -> Result<R, Error>,
    {
        let (frame_cell, swf_version, data, pc) = {
            let frame = self.stack_frames.last().ok_or("No stack frame to read!")?;
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
    ) -> Result<(), Error> {
        if let Some(frame) = self.current_stack_frame() {
            self.stack_frames.pop();

            let can_return = !self.stack_frames.is_empty();
            if can_return {
                frame
                    .write(context.gc_context)
                    .set_return_value(return_value.clone());
                self.stack.push(return_value);
            }
        }

        Ok(())
    }

    /// Execute the AVM stack until it is exhausted.
    pub fn run_stack_till_empty(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        while !self.stack_frames.is_empty() {
            self.with_current_reader_mut(context, |this, r, context| {
                this.do_next_action(context, r)
            })?;
        }

        Ok(())
    }

    /// Execute the AVM stack until a given activation returns.
    pub fn run_current_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        stop_frame: GcCell<'gc, Activation<'gc>>,
    ) -> Result<(), Error> {
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
            Err("Attempted to run a frame not on the current interpreter stack".into())
        }
    }

    /// Run a single action from a given action reader.
    fn do_next_action(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut Reader<'_>,
    ) -> Result<(), Error> {
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
            if let Err(ref e) = result {
                log::error!("AVM1 error: {}", e);
                return result;
            }
        } else {
            //The explicit end opcode was encountered so return here
            self.retire_stack_frame(context, Value::Undefined)?;
        }

        Ok(())
    }

    pub fn variable_name_is_slash_path(path: &str) -> bool {
        path.contains(':') || path.contains('.')
    }

    pub fn resolve_slash_path(
        start: DisplayObject<'gc>,
        root: DisplayObject<'gc>,
        mut path: &str,
    ) -> Option<DisplayObject<'gc>> {
        let mut cur_clip = if path.bytes().nth(0).unwrap_or(0) == b'/' {
            path = &path[1..];
            root
        } else {
            start
        };
        if !path.is_empty() {
            for name in path.split('/') {
                let next_clip = if let Some(clip) = cur_clip.as_movie_clip() {
                    if let Some(child) = clip.get_child_by_name(name) {
                        child
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };
                cur_clip = next_clip;
            }
        }
        Some(cur_clip)
    }

    pub fn resolve_slash_path_variable<'s>(
        start: Option<DisplayObject<'gc>>,
        root: DisplayObject<'gc>,
        path: &'s str,
    ) -> Option<(DisplayObject<'gc>, &'s str)> {
        // If the target clip is invalid, we default to root for the variable path.
        let start = start.unwrap_or(root);
        if !path.is_empty() {
            let mut var_iter = path.splitn(2, ':');
            match (var_iter.next(), var_iter.next()) {
                (Some(var_name), None) => return Some((start, var_name)),
                (Some(path), Some(var_name)) => {
                    if let Some(node) = Self::resolve_slash_path(start, root, path) {
                        return Some((node, var_name));
                    }
                }
                _ => (),
            }
        }

        None
    }

    fn push(&mut self, value: impl Into<Value<'gc>>) {
        let value = value.into();
        avm_debug!("Stack push {}: {:?}", self.stack.len(), value);
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value<'gc>, Error> {
        self.stack
            .pop()
            .ok_or_else(|| "Stack underflow".into())
            .map(|value| {
                avm_debug!("Stack pop {}: {:?}", self.stack.len(), value);
                value
            })
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
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(b.into_number_v1() + a.into_number_v1());
        Ok(())
    }

    fn action_add_2(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // ECMA-262 s. 11.6.1
        let a = self.pop()?;
        let b = self.pop()?;

        // TODO(Herschel):
        if let Value::String(a) = a {
            let mut s = b.coerce_to_string(self, context)?;
            s.push_str(&a);
            self.push(s);
        } else if let Value::String(mut b) = b {
            b.push_str(&a.coerce_to_string(self, context)?);
            self.push(b);
        } else {
            let result = b.as_number(self, context)? + a.as_number(self, context)?;
            self.push(result);
        }
        Ok(())
    }

    fn action_and(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 logical and
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() != 0.0 && a.into_number_v1() != 0.0;
        self.push(Value::from_bool_v1(result, self.current_swf_version()));
        Ok(())
    }

    fn action_ascii_to_char(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let val = (self.pop()?.as_f64()? as u8) as char;
        self.push(val.to_string());
        Ok(())
    }

    fn action_char_to_ascii(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let s = self.pop()?.into_string();
        let result = s.bytes().nth(0).unwrap_or(0);
        self.push(result);
        Ok(())
    }

    fn action_clone_sprite(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel)
        let _depth = self.pop()?;
        let _target = self.pop()?;
        let _source = self.pop()?;
        log::error!("Unimplemented action: CloneSprite");
        Ok(())
    }

    fn action_bit_and(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = a & b;
        self.push(result);
        Ok(())
    }

    fn action_bit_lshift(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop()?.as_i32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_i32()?;
        let result = b << a;
        self.push(result);
        Ok(())
    }

    fn action_bit_or(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = a | b;
        self.push(result);
        Ok(())
    }

    fn action_bit_rshift(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop()?.as_i32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_i32()?;
        let result = b >> a;
        self.push(result);
        Ok(())
    }

    fn action_bit_urshift(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_u32()?;
        let result = b >> a;
        self.push(result);
        Ok(())
    }

    fn action_bit_xor(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = b ^ a;
        self.push(result);
        Ok(())
    }

    fn action_call(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let _val = self.pop()?;
        // TODO(Herschel)
        Err("Unimplemented action: Call".into())
    }

    fn action_call_function(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let fn_name = self.pop()?;
        let mut args = Vec::new();
        let num_args = self.pop()?.as_i64()?; // TODO(Herschel): max arg count?
        for _ in 0..num_args {
            args.push(self.pop()?);
        }

        let target_fn = self
            .stack_frames
            .last()
            .unwrap()
            .clone()
            .read()
            .resolve(fn_name.as_string()?, self, context)?
            .resolve(self, context)?;
        let this = context.active_clip.object().as_object()?;
        target_fn.call(self, context, this, &args)?.push(self);

        Ok(())
    }

    fn action_call_method(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let method_name = self.pop()?;
        let object = self.pop()?;
        let num_args = self.pop()?.as_i64()?; // TODO(Herschel): max arg count?
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.pop()?);
        }

        match method_name {
            Value::Undefined | Value::Null => {
                let this = context.active_clip.object().as_object()?;
                object.call(self, context, this, &args)?.push(self);
            }
            Value::String(name) => {
                if name.is_empty() {
                    object
                        .call(self, context, object.as_object()?, &args)?
                        .push(self);
                } else {
                    let target = object.as_object()?;
                    let callable = object
                        .as_object()?
                        .get(&name, self, context)?
                        .resolve(self, context)?;

                    if let Value::Object(_) = callable {
                    } else {
                        log::warn!("Object method {} is not callable", name);
                    }

                    callable.call(self, context, target, &args)?.push(self);
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

    fn action_constant_pool(
        &mut self,
        _context: &mut UpdateContext,
        constant_pool: &[&str],
    ) -> Result<(), Error> {
        self.constant_pool = constant_pool.iter().map(|s| s.to_string()).collect();
        Ok(())
    }

    fn action_decrement(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop()?.as_number(self, context)?;
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
        let func = Avm1Function::from_df1(swf_version, func_data, name, params, scope);
        let prototype =
            ScriptObject::object(context.gc_context, Some(self.prototypes.object)).into();
        let func_obj = ScriptObject::function(
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
        let func = Avm1Function::from_df2(swf_version, func_data, action_func, scope);
        let prototype =
            ScriptObject::object(context.gc_context, Some(self.prototypes.object)).into();
        let func_obj = ScriptObject::function(
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
        let value = self.pop()?;
        let name = self.pop()?;
        self.current_stack_frame().unwrap().read().define(
            name.as_string()?,
            value,
            context.gc_context,
        );
        Ok(())
    }

    fn action_define_local_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let name = self.pop()?;
        self.current_stack_frame().unwrap().read().define(
            name.as_string()?,
            Value::Undefined,
            context.gc_context,
        );
        Ok(())
    }

    fn action_delete(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name_val = self.pop()?;
        let name = name_val.as_string()?;
        let object = self.pop()?.as_object()?;

        let success = object.delete(context.gc_context, name);
        self.push(success);

        Ok(())
    }

    fn action_delete_2(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name_val = self.pop()?;
        let name = name_val.as_string()?;

        //Fun fact: This isn't in the Adobe SWF19 spec, but this opcode returns
        //a boolean based on if the delete actually deleted something.
        let did_exist = self.current_stack_frame().unwrap().read().is_defined(name);

        self.current_stack_frame()
            .unwrap()
            .read()
            .scope()
            .delete(name, context.gc_context);
        self.push(did_exist);

        Ok(())
    }

    fn action_divide(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // AS1 divide
        let a = self.pop()?;
        let b = self.pop()?;

        // TODO(Herschel): SWF19: "If A is zero, the result NaN, Infinity, or -Infinity is pushed to the in SWF 5 and later.
        // In SWF 4, the result is the string #ERROR#.""
        // Seems to be untrue for SWF v4, I get 1.#INF.

        self.push(b.into_number_v1() / a.into_number_v1());
        Ok(())
    }

    fn action_end_drag(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel)
        log::error!("Unimplemented action: EndDrag");
        Ok(())
    }

    fn action_enumerate(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name_value = self.pop()?;
        let name = name_value.as_string()?;
        self.push(Value::Null); // Sentinel that indicates end of enumeration
        let object = self
            .current_stack_frame()
            .unwrap()
            .read()
            .resolve(name, self, context)?
            .resolve(self, context)?;

        match object {
            Value::Object(ob) => {
                for k in ob.get_keys() {
                    self.push(k);
                }
            }
            _ => log::error!("Cannot enumerate properties of {}", name_value.as_string()?),
        };

        Ok(())
    }

    fn action_enumerate_2(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let object = self.pop()?.as_object()?;

        self.push(Value::Null); // Sentinel that indicates end of enumeration
        for k in object.get_keys() {
            self.push(k);
        }

        Ok(())
    }

    #[allow(clippy::float_cmp)]
    fn action_equals(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 equality
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() == a.into_number_v1();
        self.push(Value::from_bool_v1(result, self.current_swf_version()));
        Ok(())
    }

    #[allow(clippy::float_cmp)]
    fn action_equals_2(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // Version >=5 equality
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.abstract_eq(a, self, context, false)?;
        self.push(result);
        Ok(())
    }

    fn action_get_member(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name_val = self.pop()?;
        let name = name_val.coerce_to_string(self, context)?;
        let object = self.pop()?.as_object()?;
        object.get(&name, self, context)?.push(self);

        Ok(())
    }

    fn action_get_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let prop_index = self.pop()?.into_number_v1() as usize;
        let clip_path = self.pop()?;
        let path = clip_path.as_string()?;
        let ret = if let Some(base_clip) = context.target_clip {
            if let Some(clip) = Avm1::resolve_slash_path(base_clip, context.root, path) {
                if let Some(clip) = clip.as_movie_clip() {
                    match prop_index {
                        0 => f64::from(clip.x()).into(),
                        1 => f64::from(clip.y()).into(),
                        2 => f64::from(clip.x_scale()).into(),
                        3 => f64::from(clip.y_scale()).into(),
                        4 => f64::from(clip.current_frame()).into(),
                        5 => f64::from(clip.total_frames()).into(),
                        10 => f64::from(clip.rotation()).into(),
                        11 => {
                            // _target
                            // TODO: This string should be built dynamically
                            // by traversing through parents. But this requires
                            // _name to work accurately.
                            context.target_path.clone()
                        }
                        12 => f64::from(clip.frames_loaded()).into(),
                        _ => {
                            log::error!("GetProperty: Unimplemented property index {}", prop_index);
                            Value::Undefined
                        }
                    }
                } else {
                    log::warn!("GetProperty: Target is not a movieclip");
                    Value::Undefined
                }
            } else {
                log::warn!("GetProperty: Invalid target {}", path);
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
        self.push(context.global_time as f64);
        Ok(())
    }

    /// Obtain the value of `_root`.
    pub fn root_object(&self, context: &mut UpdateContext<'_, 'gc, '_>) -> Value<'gc> {
        context.root.object()
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
        let var_path = self.pop()?;
        let path = var_path.as_string()?;

        let is_slashpath = Self::variable_name_is_slash_path(path);
        if is_slashpath {
            if let Some((node, var_name)) =
                Self::resolve_slash_path_variable(context.target_clip, context.root, path)
            {
                if let Some(clip) = node.as_movie_clip() {
                    let object = clip.object().as_object()?;
                    if object.has_property(var_name) {
                        object.get(var_name, self, context)?.push(self);
                    } else {
                        self.push(Value::Undefined);
                    }
                } else {
                    self.push(Value::Undefined);
                }
            }
        } else if self.current_stack_frame().unwrap().read().is_defined(path) {
            self.current_stack_frame()
                .unwrap()
                .read()
                .resolve(path, self, context)?
                .push(self);
        } else {
            self.push(Value::Undefined);
        }

        Ok(())
    }

    fn action_get_url(
        &mut self,
        context: &mut UpdateContext,
        url: &str,
        target: &str,
    ) -> Result<(), Error> {
        //TODO: support `_level0` thru `_level9`
        if target.starts_with("_level") {
            log::warn!(
                "Remote SWF loads into target {} not yet implemented",
                target
            );
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
        let target = self.pop()?.into_string();
        let url = self.pop()?.into_string();

        if let Some(fscommand) = fscommand::parse(&url) {
            return fscommand::handle(fscommand, self, context);
        }

        if is_target_sprite {
            log::warn!("GetURL into target sprite is not yet implemented");
            return Ok(()); //maybe error?
        }

        if is_load_vars {
            log::warn!("Reading AVM locals from forms is not yet implemented");
            return Ok(()); //maybe error?
        }

        let vars = match NavigationMethod::from_send_vars_method(swf_method) {
            Some(method) => Some((method, self.locals_into_form_values(context))),
            None => None,
        };

        context.navigator.navigate_to_url(url, Some(target), vars);

        Ok(())
    }

    fn action_goto_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        frame: u16,
    ) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
            if let Some(clip) = clip.as_movie_clip() {
                // The frame on the stack is 0-based, not 1-based.
                clip.goto_frame(context, frame + 1, true);
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
        if let Some(clip) = context.target_clip {
            if let Some(clip) = clip.as_movie_clip() {
                match self.pop()? {
                    Value::Number(frame) => {
                        // The frame on the stack is 1-based, not 0-based.
                        clip.goto_frame(context, scene_offset + (frame as u16), !set_playing)
                    }
                    Value::String(frame_label) => {
                        if let Some(frame) = clip.frame_label_to_number(&frame_label) {
                            clip.goto_frame(context, scene_offset + frame, !set_playing)
                        } else {
                            log::warn!(
                                "GotoFrame2: MovieClip {} does not contain frame label '{}'",
                                clip.id(),
                                frame_label
                            );
                        }
                    }
                    _ => log::warn!("GotoFrame2: Expected frame label or number"),
                }
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
        if let Some(clip) = context.target_clip {
            if let Some(clip) = clip.as_movie_clip() {
                if let Some(frame) = clip.frame_label_to_number(label) {
                    clip.goto_frame(context, frame, true);
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
        let val = self.pop()?;
        if val.as_bool(self.current_swf_version()) {
            reader.seek(jump_offset.into());
        }
        Ok(())
    }

    fn action_increment(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let a = self.pop()?.as_number(self, context)?;
        self.push(a + 1.0);
        Ok(())
    }

    fn action_init_array(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let num_elements = self.pop()?.as_i64()?;
        for _ in 0..num_elements {
            let _value = self.pop()?;
        }

        // TODO(Herschel)
        Err("Unimplemented action: InitArray".into())
    }

    fn action_init_object(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let num_props = self.pop()?.as_i64()?;
        let object = ScriptObject::object(context.gc_context, Some(self.prototypes.object));
        for _ in 0..num_props {
            let value = self.pop()?;
            let name = self.pop()?.into_string();
            object.set(&name, value, self, context)?;
        }

        self.push(Value::Object(object.into()));

        Ok(())
    }

    fn action_instance_of(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let constr = self.pop()?.as_object()?;
        let obj = self.pop()?.as_object()?;

        //TODO: Interface detection on SWF7
        let prototype = constr
            .get("prototype", self, context)?
            .resolve(self, context)?
            .as_object()?;
        let mut proto = obj.proto();

        while let Some(this_proto) = proto {
            if Object::ptr_eq(this_proto, prototype) {
                self.push(true);
                return Ok(());
            }

            proto = this_proto.proto();
        }

        self.push(false);
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
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() < a.into_number_v1();
        self.push(Value::from_bool_v1(result, self.current_swf_version()));
        Ok(())
    }

    fn action_less_2(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // ECMA-262 s. 11.8.1
        let a = self.pop()?;
        let b = self.pop()?;

        let result = b.abstract_lt(a, self, context)?;

        self.push(result);
        Ok(())
    }

    fn action_greater(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // ECMA-262 s. 11.8.2
        let a = self.pop()?;
        let b = self.pop()?;

        let result = a.abstract_lt(b, self, context)?;

        self.push(result);
        Ok(())
    }

    fn action_mb_ascii_to_char(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        use std::convert::TryFrom;
        let val = char::try_from(self.pop()?.as_f64()? as u32)?;
        self.push(val.to_string());
        Ok(())
    }

    fn action_mb_char_to_ascii(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let s = self.pop()?.into_string();
        let result = s.chars().nth(0).unwrap_or('\0') as u32;
        self.push(result);
        Ok(())
    }

    fn action_mb_string_extract(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop()?.as_f64()? as usize;
        let start = self.pop()?.as_f64()? as usize;
        let s = self.pop()?.into_string();
        let result = s[len..len + start].to_string(); // TODO(Herschel): Flash uses UTF-16 internally.
        self.push(result);
        Ok(())
    }

    fn action_mb_string_length(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop()?.into_string().len();
        self.push(val as f64);
        Ok(())
    }

    fn action_multiply(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // AS1 multiply
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(a.into_number_v1() * b.into_number_v1());
        Ok(())
    }

    fn action_modulo(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO: Wrong operands?
        let a = self.pop()?.as_f64()?;
        let b = self.pop()?.as_f64()?;
        self.push(a % b);
        Ok(())
    }

    fn action_not(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 logical not
        let val = self.pop()?;
        let result = val.into_number_v1() == 0.0;
        self.push(Value::from_bool_v1(result, self.current_swf_version()));
        Ok(())
    }

    fn action_next_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
            if let Some(clip) = clip.as_movie_clip() {
                clip.next_frame(context);
            } else {
                log::warn!("NextFrame: Target is not a MovieClip");
            }
        } else {
            log::warn!("NextFrame: Invalid target");
        }
        Ok(())
    }

    fn action_new_method(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let method_name = self.pop()?;
        let object = self.pop()?.as_object()?;
        let num_args = self.pop()?.as_i64()?;
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.pop()?);
        }

        let constructor = object
            .get(&method_name.as_string()?, self, context)?
            .resolve(self, context)?
            .as_object()?;
        let prototype = constructor
            .get("prototype", self, context)?
            .resolve(self, context)?
            .as_object()?;

        let this = prototype.new(self, context, prototype, &args)?;

        //TODO: What happens if you `ActionNewMethod` without a method name?
        constructor
            .call(self, context, this, &args)?
            .resolve(self, context)?;

        self.push(this);

        Ok(())
    }

    fn action_new_object(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let fn_name = self.pop()?;
        let num_args = self.pop()?.as_i64()?;
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.pop()?);
        }

        let constructor = self
            .stack_frames
            .last()
            .unwrap()
            .clone()
            .read()
            .resolve(fn_name.as_string()?, self, context)?
            .resolve(self, context)?
            .as_object()?;
        let prototype = constructor
            .get("prototype", self, context)?
            .resolve(self, context)?
            .as_object()?;

        let this = prototype.new(self, context, prototype, &args)?;

        constructor
            .call(self, context, this, &args)?
            .resolve(self, context)?;

        self.push(this);

        Ok(())
    }

    fn action_or(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        // AS1 logical or
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() != 0.0 || a.into_number_v1() != 0.0;
        self.push(Value::from_bool_v1(result, self.current_swf_version()));
        Ok(())
    }

    fn action_play(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
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
        if let Some(clip) = context.target_clip {
            if let Some(clip) = clip.as_movie_clip() {
                clip.prev_frame(context);
            } else {
                log::warn!("PrevFrame: Target is not a MovieClip");
            }
        } else {
            log::warn!("PrevFrame: Invalid target");
        }
        Ok(())
    }

    fn action_pop(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        self.pop()?;
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
                SwfValue::Str(v) => v.to_string().into(),
                SwfValue::Register(v) => self.current_register(*v),
                SwfValue::ConstantPool(i) => {
                    if let Some(value) = self.constant_pool.get(*i as usize) {
                        value.to_string().into()
                    } else {
                        log::warn!(
                            "ActionPush: Constant pool index {} out of range (len = {})",
                            i,
                            self.constant_pool.len()
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
        let max = self.pop()?.into_number_v1() as i32;
        let val = context.rng.gen_range(0, max);
        self.push(val);
        Ok(())
    }

    fn action_remove_sprite(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let _target = self.pop()?.into_string();
        // TODO(Herschel)
        log::error!("Unimplemented action: RemoveSprite");
        Ok(())
    }

    fn action_return(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let return_value = self.pop()?;
        self.retire_stack_frame(context, return_value)?;

        Ok(())
    }

    fn action_set_member(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let value = self.pop()?;
        let name_val = self.pop()?;
        let name = name_val.coerce_to_string(self, context)?;
        let object = self.pop()?.as_object()?;

        object.set(&name, value, self, context)?;
        Ok(())
    }

    fn action_set_property(&mut self, context: &mut UpdateContext) -> Result<(), Error> {
        let value = self.pop()?.into_number_v1() as f32;
        let prop_index = self.pop()?.as_u32()? as usize;
        let clip_path = self.pop()?;
        let path = clip_path.as_string()?;
        if let Some(base_clip) = context.target_clip {
            if let Some(clip) = Avm1::resolve_slash_path(base_clip, context.root, path) {
                if let Some(clip) = clip.as_movie_clip() {
                    match prop_index {
                        0 => clip.set_x(context.gc_context, value),
                        1 => clip.set_y(context.gc_context, value),
                        2 => clip.set_x_scale(context.gc_context, value),
                        3 => clip.set_y_scale(context.gc_context, value),
                        10 => clip.set_rotation(context.gc_context, value),
                        _ => {
                            log::error!("SetProperty: Unimplemented property index {}", prop_index)
                        }
                    }
                }
            } else {
                log::warn!("SetProperty: Invalid target {}", path);
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
        let value = self.pop()?;
        let var_path = self.pop()?;
        self.set_variable(context, var_path.as_string()?, value)
    }

    #[allow(clippy::float_cmp)]
    fn action_strict_equals(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // The same as normal equality but types must match
        let a = self.pop()?;
        let b = self.pop()?;
        let result = a == b;
        self.push(result);
        Ok(())
    }

    pub fn set_variable(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        var_path: &str,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        let is_slashpath = Self::variable_name_is_slash_path(var_path);

        if is_slashpath {
            if let Some((node, var_name)) =
                Self::resolve_slash_path_variable(context.target_clip, context.root, var_path)
            {
                if let Some(clip) = node.as_movie_clip() {
                    clip.object()
                        .as_object()?
                        .set(var_name, value.clone(), self, context)?;
                }
            }
        } else {
            let this = self.current_stack_frame().unwrap().read().this_cell();
            let scope = self.current_stack_frame().unwrap().read().scope_cell();
            let unused_value = scope
                .read()
                .overwrite(var_path, value, self, context, this)?;
            if let Some(value) = unused_value {
                self.current_stack_frame().unwrap().read().define(
                    var_path,
                    value,
                    context.gc_context,
                );
            }
        }

        Ok(())
    }

    fn action_set_target(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        target: &str,
    ) -> Result<(), Error> {
        if target.is_empty() {
            context.active_clip = context.start_clip;
            context.target_clip = Some(context.start_clip);
            context.target_path = target.into();
        } else if let Some(clip) =
            Avm1::resolve_slash_path(context.start_clip, context.root, target)
        {
            context.target_clip = Some(clip);
            context.active_clip = clip;
            context.target_path = target.into();
        } else {
            log::warn!("SetTarget failed: {} not found", target);
            // TODO: Emulate AVM1 trace error message.
            // log::info!(target: "avm_trace", "Target not found: Target=\"{}\" Base=\"{}\"", target, context.root.read().name());

            // When SetTarget has an invalid target, subsequent GetVariables act
            // as if they are targeting root, but subsequent Play/Stop/etc.
            // fail silenty.
            context.target_clip = None;
            context.active_clip = context.root;
            context.target_path = Value::Undefined;
        }

        let scope = self.current_stack_frame().unwrap().read().scope_cell();
        let clip_obj = context.active_clip.object().as_object().unwrap();

        self.current_stack_frame()
            .unwrap()
            .write(context.gc_context)
            .set_scope(Scope::new_target_scope(scope, clip_obj, context.gc_context));
        Ok(())
    }

    fn action_set_target2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let target = self.pop()?;
        if let Ok(target) = target.as_string() {
            self.action_set_target(context, target)?;
        } else {
            log::error!("SetTarget2: Path must be a string");
        }
        Ok(())
    }

    fn action_stack_swap(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(a);
        self.push(b);
        Ok(())
    }

    fn action_start_drag(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let _target = self.pop()?;
        let _lock_center = self.pop()?.as_bool(self.current_swf_version());
        let constrain = self.pop()?.as_bool(self.current_swf_version());
        if constrain {
            let _y2 = self.pop()?;
            let _x2 = self.pop()?;
            let _y1 = self.pop()?;
            let _x1 = self.pop()?;
        }
        log::error!("Unimplemented action: StartDrag");
        Ok(())
    }

    fn action_stop(&mut self, context: &mut UpdateContext) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
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

    fn action_string_add(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // SWFv4 string concatenation
        // TODO(Herschel): Result with non-string operands?
        let a = self.pop()?.into_string();
        let mut b = self.pop()?.into_string();
        b.push_str(&a);
        self.push(b);
        Ok(())
    }

    fn action_string_equals(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.coerce_to_string(self, context)? == a.coerce_to_string(self, context)?;
        self.push(Value::from_bool_v1(result, self.current_swf_version()));
        Ok(())
    }

    fn action_string_extract(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // SWFv4 substring
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop()?.as_f64()? as usize;
        let start = self.pop()?.as_f64()? as usize;
        let s = self.pop()?.into_string();
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
        let a = self.pop()?;
        let b = self.pop()?;
        // This is specifically a non-UTF8 aware comparison.
        let result = b
            .coerce_to_string(self, context)?
            .bytes()
            .gt(a.coerce_to_string(self, context)?.bytes());
        self.push(Value::from_bool_v1(result, self.current_swf_version()));
        Ok(())
    }

    fn action_string_length(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // AS1 strlen
        // Only returns byte length.
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop()?.into_string().bytes().len() as f64;
        self.push(val);
        Ok(())
    }

    fn action_string_less(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop()?;
        let b = self.pop()?;
        // This is specifically a non-UTF8 aware comparison.
        let result = b
            .coerce_to_string(self, context)?
            .bytes()
            .lt(a.coerce_to_string(self, context)?.bytes());
        self.push(Value::from_bool_v1(result, self.current_swf_version()));
        Ok(())
    }

    fn action_subtract(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(b.into_number_v1() - a.into_number_v1());
        Ok(())
    }

    fn action_target_path(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel)
        let _clip = self.pop()?.as_object()?;
        self.push(Value::Undefined);
        Err("Unimplemented action: TargetPath".into())
    }

    fn toggle_quality(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        // TODO(Herschel): Noop for now? Could chang anti-aliasing on render backend.
        Ok(())
    }

    fn action_to_integer(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let val = self.pop()?;
        self.push(val.into_number_v1().trunc());
        Ok(())
    }

    fn action_to_number(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let val = self.pop()?.as_number(self, context)?;
        self.push(val);
        Ok(())
    }

    fn action_to_string(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let val = self.pop()?.coerce_to_string(self, context)?;
        self.push(val);
        Ok(())
    }

    fn action_trace(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let val = self.pop()?.coerce_to_string(self, context)?;
        log::info!(target: "avm_trace", "{}", val);
        Ok(())
    }

    fn action_type_of(&mut self, _context: &mut UpdateContext) -> Result<(), Error> {
        let type_of = self.pop()?.type_of();
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
        _context: &mut UpdateContext,
        num_actions_to_skip: u8,
        r: &mut Reader<'_>,
    ) -> Result<(), Error> {
        // TODO(Herschel): Always true for now.
        let _frame_num = self.pop()?.as_f64()? as u16;
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
        let object = self.pop()?.as_object()?;
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

/// Utility function used by `Avm1::action_wait_for_frame` and
/// `Avm1::action_wait_for_frame_2`.
fn skip_actions(reader: &mut Reader<'_>, num_actions_to_skip: u8) -> Result<(), Error> {
    for _ in 0..num_actions_to_skip {
        reader.read_action()?;
    }

    Ok(())
}
