use crate::avm1::globals::create_globals;
use crate::avm1::object::Object;
use crate::backend::navigator::NavigationMethod;

use crate::prelude::*;
use gc_arena::{GcCell, MutationContext};
use rand::{rngs::SmallRng, Rng};
use std::convert::TryInto;
use std::collections::HashMap;

use swf::avm1::read::Reader;
use swf::avm1::types::Action;

use crate::tag_utils::SwfSlice;

mod fscommand;
mod scope;
mod activation;
mod globals;
pub mod movie_clip;
pub mod object;
mod value;

pub use value::Value;
use scope::Scope;
use activation::Activation;

pub struct ActionContext<'a, 'gc, 'gc_context> {
    pub gc_context: gc_arena::MutationContext<'gc, 'gc_context>,
    pub global_time: u64,

    /// The root of the current timeline.
    /// This will generally be `_level0`, except for loadMovie/loadMovieNum.
    pub root: DisplayNode<'gc>,

    /// The object that this code is running in.
    /// Used by `GetVariable`/`SetVariable`/`this`.
    pub active_clip: DisplayNode<'gc>,

    /// The base clip for Flash 4-era actions.
    /// Used by `Play`, `GetProperty`, etc.
    pub start_clip: DisplayNode<'gc>,

    /// The object targeted with `tellTarget`.
    /// This is used for Flash 4-era actions, such as
    /// `Play`, `GetProperty`, etc.
    /// This will be `None` after an invalid tell target.
    pub target_clip: Option<DisplayNode<'gc>>,

    /// The last path string used by `tellTarget`.
    /// Returned by `GetProperty`.
    /// TODO: This should actually be built dynamically upon
    /// request, but this requires us to implement auto-generated
    /// _names ("instanceN" etc. for unnamed clips).
    pub target_path: Value<'gc>,

    pub rng: &'a mut SmallRng,
    pub audio: &'a mut dyn crate::backend::audio::AudioBackend,
    pub navigator: &'a mut dyn crate::backend::navigator::NavigatorBackend,
}

pub struct Avm1<'gc> {
    /// The currently installed constant pool.
    constant_pool: Vec<String>,

    /// The global object.
    globals: GcCell<'gc, Object<'gc>>,

    /// All activation records for the current execution context.
    stack_frames: Vec<Activation<'gc>>,

    /// The operand stack (shared across functions).
    stack: Vec<Value<'gc>>
}

unsafe impl<'gc> gc_arena::Collect for Avm1<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.globals.trace(cc);
        self.stack_frames.trace(cc);
        self.stack.trace(cc);
    }
}

type Error = Box<dyn std::error::Error>;

impl<'gc> Avm1<'gc> {
    pub fn new(gc_context: MutationContext<'gc, '_>) -> Self {
        Self {
            constant_pool: vec![],
            globals: GcCell::allocate(gc_context, create_globals(gc_context)),
            stack_frames: vec![],
            stack: vec![]
        }
    }

    /// Convert the current locals pool into a set of form values.
    ///
    /// This is necessary to support form submission from Flash via a couple of
    /// legacy methods, such as the `ActionGetURL2` opcode or `getURL` function.
    pub fn locals_into_form_values(&self) -> HashMap<String, String> {
        let mut form_values = HashMap::new();

        for (k, v) in self.current_stack_frame().unwrap().scope().locals().iter_values() {
            form_values.insert(k.clone(), v.clone().into_string());
        }

        form_values
    }

    /// Add a stack frame that executes code directly from a SwfSlice.
    pub fn insert_stack_frame_from_action(&mut self, swf_version: u8, code: SwfSlice, gc_context: MutationContext<'gc, '_>) {
        let scope = GcCell::allocate(gc_context, Scope::from_global_object(self.globals));
        self.stack_frames.push(Activation::from_action(swf_version, code, scope));
    }

    /// Retrieve the current AVM execution frame.
    /// 
    /// Yields None if there is no stack frame.
    pub fn current_stack_frame(&self) -> Option<&Activation<'gc>> {
        self.stack_frames.last()
    }

    /// Retrieve the current AVM execution frame for mutation.
    /// 
    /// Yields None if there is no stack frame.
    pub fn current_stack_frame_mut(&mut self) -> Option<&mut Activation<'gc>> {
        self.stack_frames.last_mut()
    }

    /// Get the currently executing SWF version, if there is one.
    fn current_swf_version(&self) -> Option<u8> {
        self.current_stack_frame().map(|sf| sf.swf_version())
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
    fn with_current_reader_mut<F, R>(&mut self, func: F) -> Option<R> where F: FnOnce(&mut Self, &mut Reader<'_>) -> R {
        let current_stack_id = self.stack_frames.len() - 1;
        let (swf_version, data, pc) = self.stack_frames.last().map(|frame| (frame.swf_version(), frame.data(), frame.pc()))?;
        let mut read = Reader::new(data.as_ref(), swf_version);
        read.seek(pc.try_into().unwrap());

        let r = func(self, &mut read);
        
        //this took an hour of fighting borrowck to figure out was necessary
        if let Some(new_stack) = self.stack_frames.get_mut(current_stack_id) {
            if new_stack.is_identical_fn(&data) {
                new_stack.set_pc(read.pos());
            }
        }

        Some(r)
    }

    /// Destroy the current stack frame (if there is one).
    fn retire_stack_frame(&mut self) {
        self.stack_frames.pop();
    }

    /// Execute the AVM stack until it is exhausted.
    pub fn run_stack_till_empty(&mut self, context: &mut ActionContext<'_, 'gc, '_>) -> Result<(), Error> {
        while self.stack_frames.len() > 0 {
            self.with_current_reader_mut(|this, r| this.do_next_action(context, r)).unwrap()?;
        }

        Ok(())
    }

    /// Run a single action from a given action reader.
    fn do_next_action(&mut self, context: &mut ActionContext<'_, 'gc, '_>, reader: &mut Reader<'_>) -> Result<(), Error> {
        if let Some(action) = reader.read_action()? {
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
                Action::ConstantPool(constant_pool) => {
                    self.action_constant_pool(context, &constant_pool[..])
                }
                Action::Decrement => self.action_decrement(context),
                Action::DefineFunction {
                    name,
                    params,
                    actions,
                } => self.action_define_function(context, &name, &params[..], actions),
                Action::DefineLocal => self.action_define_local(context),
                Action::DefineLocal2 => self.action_define_local_2(context),
                Action::Delete => self.action_delete(context),
                Action::Delete2 => self.action_delete_2(context),
                Action::Divide => self.action_divide(context),
                Action::EndDrag => self.action_end_drag(context),
                Action::Enumerate => self.action_enumerate(context),
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
                Action::GotoLabel(label) => self.action_goto_label(context, &label),
                Action::If { offset } => self.action_if(context, offset),
                Action::Increment => self.action_increment(context),
                Action::InitArray => self.action_init_array(context),
                Action::InitObject => self.action_init_object(context),
                Action::Jump { offset } => self.action_jump(context, offset),
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
                Action::StringAdd => self.action_string_add(context),
                Action::StringEquals => self.action_string_equals(context),
                Action::StringExtract => self.action_string_extract(context),
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
                } => self.action_wait_for_frame(context, frame, num_actions_to_skip),
                Action::WaitForFrame2 {
                    num_actions_to_skip,
                } => self.action_wait_for_frame_2(context, num_actions_to_skip),
                Action::With { .. } => self.action_with(context),
                _ => self.unknown_op(context, action),
            };
            if let Err(ref e) = result {
                log::error!("AVM1 error: {}", e);
                return result;
            }
        } else {
            //Implicit return undefined
            self.retire_stack_frame();
            self.push(Value::Undefined); //TODO: What if we don't have any more code?
        }

        Ok(())
    }

    pub fn resolve_slash_path(
        start: DisplayNode<'gc>,
        root: DisplayNode<'gc>,
        mut path: &str,
    ) -> Option<DisplayNode<'gc>> {
        let mut cur_clip = if path.bytes().nth(0).unwrap_or(0) == b'/' {
            path = &path[1..];
            root
        } else {
            start
        };
        if !path.is_empty() {
            for name in path.split('/') {
                let next_clip = if let Some(clip) = cur_clip.read().as_movie_clip() {
                    if let Some(child) = clip.get_child_by_name(name) {
                        *child
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
        start: Option<DisplayNode<'gc>>,
        root: DisplayNode<'gc>,
        path: &'s str,
    ) -> Option<(DisplayNode<'gc>, &'s str)> {
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
        self.stack.push(value.into());
    }

    fn pop(&mut self) -> Result<Value<'gc>, Error> {
        self.stack.pop().ok_or_else(|| "Stack underflow".into())
    }

    fn unknown_op(
        &mut self,
        _context: &mut ActionContext,
        action: swf::avm1::types::Action,
    ) -> Result<(), Error> {
        log::error!("Unknown AVM1 opcode: {:?}", action);
        Err("Unknown op".into())
    }

    fn action_add(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(b.into_number_v1() + a.into_number_v1()));
        Ok(())
    }

    fn action_add_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // ECMA-262 s. 11.6.1
        let a = self.pop()?;
        let b = self.pop()?;
        // TODO(Herschel):
        if let Value::String(a) = a {
            let mut s = b.into_string();
            s.push_str(&a);
            self.push(Value::String(s));
        } else if let Value::String(mut b) = b {
            b.push_str(&a.into_string());
            self.push(Value::String(b));
        } else {
            self.push(Value::Number(b.as_number() + a.as_number()));
        }
        Ok(())
    }

    fn action_and(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical and
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() != 0.0 && a.into_number_v1() != 0.0;
        self.push(Value::from_bool_v1(result, self.current_swf_version().unwrap()));
        Ok(())
    }

    fn action_ascii_to_char(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let val = (self.pop()?.as_f64()? as u8) as char;
        self.push(Value::String(val.to_string()));
        Ok(())
    }

    fn action_char_to_ascii(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let s = self.pop()?.into_string();
        let result = s.bytes().nth(0).unwrap_or(0);
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_clone_sprite(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel)
        let _depth = self.pop()?;
        let _target = self.pop()?;
        let _source = self.pop()?;
        log::error!("Unimplemented action: CloneSprite");
        Ok(())
    }

    fn action_bit_and(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = a & b;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_lshift(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_i32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_i32()?;
        let result = b << a;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_or(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = a | b;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_rshift(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_i32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_i32()?;
        let result = b >> a;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_urshift(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()? & 0b11111; // Only 5 bits used for shift count
        let b = self.pop()?.as_u32()?;
        let result = b >> a;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_bit_xor(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_u32()?;
        let b = self.pop()?.as_u32()?;
        let result = b ^ a;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_call(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _val = self.pop()?;
        // TODO(Herschel)
        Err("Unimplemented action: Call".into())
    }

    fn action_call_function(&mut self, context: &mut ActionContext<'_, 'gc, '_>) -> Result<(), Error> {
        let fn_name = self.pop()?;
        let mut args = Vec::new();
        let num_args = self.pop()?.as_i64()?; // TODO(Herschel): max arg count?
        for _ in 0..num_args {
            args.push(self.pop()?);
        }
        
        let target_fn = self.current_stack_frame_mut().unwrap().resolve(fn_name.as_string()?);
        let return_value = target_fn.call(self, context, self.globals, &args)?;
        if let Some(instant_return) = return_value {
            self.push(instant_return);
        }

        Ok(())
    }

    fn action_call_method(&mut self, context: &mut ActionContext<'_, 'gc, '_>) -> Result<(), Error> {
        let method_name = self.pop()?;
        let object = self.pop()?;
        let num_args = self.pop()?.as_i64()?; // TODO(Herschel): max arg count?
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.pop()?);
        }

        match method_name {
            Value::Undefined | Value::Null => {
                let this = context.active_clip.read().object().as_object()?.to_owned();
                let return_value = object.call(self, context, this, &args)?;
                if let Some(instant_return) = return_value {
                    self.push(instant_return);
                }
            }
            Value::String(name) => {
                if name.is_empty() {
                    let return_value = object.call(self, context, object.as_object()?.to_owned(), &args)?;
                    if let Some(instant_return) = return_value {
                        self.push(instant_return);
                    }
                } else {
                    let callable = object.as_object()?.read().get(
                        &name,
                        self,
                        context,
                        object.as_object()?.to_owned(),
                    );

                    if let Value::Undefined = callable {
                        return Err(format!("Object method {} is not defined", name).into());
                    }

                    let return_value = callable.call(self, context, object.as_object()?.to_owned(), &args)?;
                    if let Some(instant_return) = return_value {
                        self.push(instant_return);
                    }
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
        _context: &mut ActionContext,
        constant_pool: &[&str],
    ) -> Result<(), Error> {
        self.constant_pool = constant_pool.iter().map(|s| s.to_string()).collect();
        Ok(())
    }

    fn action_decrement(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_number();
        self.push(Value::Number(a - 1.0));
        Ok(())
    }

    fn action_define_function(
        &mut self,
        context: &mut ActionContext<'_, 'gc, '_>,
        name: &str,
        params: &[&str],
        actions: &[u8],
    ) -> Result<(), Error> {
        let swf_version = self.current_stack_frame().unwrap().swf_version();
        let func_data = self.current_stack_frame().unwrap().data().to_subslice(actions).unwrap();
        let func = Value::Object(GcCell::allocate(context.gc_context, Object::action_function(swf_version, func_data, name, params)));
        
        if name == "" {
            self.push(func);
        } else {
            self.current_stack_frame_mut().unwrap().define(name, func, context.gc_context);
        }

        Ok(())
    }

    fn action_define_local(&mut self, context: &mut ActionContext<'_, 'gc, '_>) -> Result<(), Error> {
        let value = self.pop()?;
        let name = self.pop()?;
        self.current_stack_frame_mut().unwrap().define(name.as_string()?, value, context.gc_context);
        Ok(())
    }

    fn action_define_local_2(&mut self, context: &mut ActionContext<'_, 'gc, '_>) -> Result<(), Error> {
        let name = self.pop()?;
        self.current_stack_frame_mut().unwrap().define(name.as_string()?, Value::Undefined, context.gc_context);
        Ok(())
    }

    fn action_delete(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        let _object = self.pop()?.as_object()?;
        Err("Unimplemented action: Delete".into())
        // TODO(Herschel)
    }

    fn action_delete_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        Err("Unimplemented action: Delete2".into())
        // TODO(Herschel)
    }

    fn action_divide(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 divide
        let a = self.pop()?;
        let b = self.pop()?;

        // TODO(Herschel): SWF19: "If A is zero, the result NaN, Infinity, or -Infinity is pushed to the in SWF 5 and later.
        // In SWF 4, the result is the string #ERROR#.""
        // Seems to be untrue for SWF v4, I get 1.#INF.

        self.push(Value::Number(b.into_number_v1() / a.into_number_v1()));
        Ok(())
    }

    fn action_end_drag(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel)
        log::error!("Unimplemented action: EndDrag");
        Ok(())
    }

    fn action_enumerate(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        self.push(Value::Null); // Sentinel that indicates end of enumeration
                                // TODO(Herschel): Push each property name onto the stack
        Err("Unimplemented action: Enumerate".into())
    }

    #[allow(clippy::float_cmp)]
    fn action_equals(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 equality
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() == a.into_number_v1();
        self.push(Value::from_bool_v1(result, self.current_swf_version().unwrap()));
        Ok(())
    }

    #[allow(clippy::float_cmp)]
    fn action_equals_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // Version >=5 equality
        let a = self.pop()?;
        let b = self.pop()?;
        let result = match (b, a) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Null, Value::Undefined) => true,
            (Value::Undefined, Value::Null) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Object(_a), Value::Object(_b)) => false, // TODO(Herschel)
            (Value::String(a), Value::Number(b)) => a.parse().unwrap_or(std::f64::NAN) == b,
            (Value::Number(a), Value::String(b)) => a == b.parse().unwrap_or(std::f64::NAN),
            _ => false,
        };
        self.push(Value::Bool(result));
        Ok(())
    }

    fn action_get_member(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        let _object = self.pop()?.as_object()?;
        // TODO(Herschel)
        Err("Unimplemented action: GetMember".into())
    }

    fn action_get_property(
        &mut self,
        context: &mut ActionContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let prop_index = self.pop()?.into_number_v1() as usize;
        let clip_path = self.pop()?;
        let path = clip_path.as_string()?;
        let ret = if let Some(base_clip) = context.target_clip {
            if let Some(clip) = Avm1::resolve_slash_path(base_clip, context.root, path) {
                if let Some(clip) = clip.read().as_movie_clip() {
                    match prop_index {
                        0 => Value::Number(f64::from(clip.x())),
                        1 => Value::Number(f64::from(clip.y())),
                        2 => Value::Number(f64::from(clip.x_scale())),
                        3 => Value::Number(f64::from(clip.y_scale())),
                        4 => Value::Number(f64::from(clip.current_frame())),
                        5 => Value::Number(f64::from(clip.total_frames())),
                        10 => Value::Number(f64::from(clip.rotation())),
                        11 => {
                            // _target
                            // TODO: This string should be built dynamically
                            // by traversing through parents. But this requires
                            // _name to work accurately.
                            context.target_path.clone()
                        }
                        12 => Value::Number(f64::from(clip.frames_loaded())),
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

    fn action_get_time(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        self.push(Value::Number(context.global_time as f64));
        Ok(())
    }

    fn action_get_variable(
        &mut self,
        context: &mut ActionContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // Flash 4-style variable
        let var_path = self.pop()?;
        let path = var_path.as_string()?;
        let globals = self.globals;

        // Special hardcoded variables
        if path == "_root" || path == "this" {
            self.push(context.start_clip.read().object());
            return Ok(());
        } else if path == "_global" {
            self.push(Value::Object(globals));
            return Ok(());
        }

        let mut result = None;
        if self.current_stack_frame().unwrap().is_defined(path) {
            result = Some(self.current_stack_frame().unwrap().resolve(path));
        } else {
            if let Some((node, var_name)) =
                Self::resolve_slash_path_variable(context.target_clip, context.root, path)
            {
                if let Some(clip) = node.read().as_movie_clip() {
                    let object = clip.object().as_object()?;
                    if object.read().has_property(var_name) {
                        result = Some(object.read().get(var_name, self, context, object));
                    }
                }
            };
        }

        if result.is_none() && globals.read().has_property(path) {
            result = Some(globals.read().get(path, self, context, globals));
        }
        self.push(result.unwrap_or(Value::Undefined));
        Ok(())
    }

    fn action_get_url(
        &mut self,
        context: &mut ActionContext,
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
        context: &mut ActionContext,
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
            Some(method) => Some((method, self.locals_into_form_values())),
            None => None,
        };

        context.navigator.navigate_to_url(url, Some(target), vars);

        Ok(())
    }

    fn action_goto_frame(&mut self, context: &mut ActionContext, frame: u16) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
            let mut display_object = clip.write(context.gc_context);
            if let Some(clip) = display_object.as_movie_clip_mut() {
                // The frame on the stack is 0-based, not 1-based.
                clip.goto_frame(frame + 1, true);
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
        context: &mut ActionContext,
        set_playing: bool,
        scene_offset: u16,
    ) -> Result<(), Error> {
        // Version 4+ gotoAndPlay/gotoAndStop
        // Param can either be a frame number or a frame label.
        if let Some(clip) = context.target_clip {
            let mut display_object = clip.write(context.gc_context);
            if let Some(clip) = display_object.as_movie_clip_mut() {
                match self.pop()? {
                    Value::Number(frame) => {
                        // The frame on the stack is 1-based, not 0-based.
                        clip.goto_frame(scene_offset + (frame as u16), !set_playing)
                    }
                    Value::String(frame_label) => {
                        if let Some(frame) = clip.frame_label_to_number(&frame_label) {
                            clip.goto_frame(scene_offset + frame, !set_playing)
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

    fn action_goto_label(&mut self, context: &mut ActionContext, label: &str) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
            let mut display_object = clip.write(context.gc_context);
            if let Some(clip) = display_object.as_movie_clip_mut() {
                if let Some(frame) = clip.frame_label_to_number(label) {
                    clip.goto_frame(frame, true);
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
        _context: &mut ActionContext,
        jump_offset: i16
    ) -> Result<(), Error> {
        let val = self.pop()?;
        if val.as_bool() {
            self.with_current_reader_mut(|_this, r| r.seek(jump_offset.into()));
        }
        Ok(())
    }

    fn action_increment(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?.as_number();
        self.push(Value::Number(a + 1.0));
        Ok(())
    }

    fn action_init_array(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let num_elements = self.pop()?.as_i64()?;
        for _ in 0..num_elements {
            let _value = self.pop()?;
        }

        // TODO(Herschel)
        Err("Unimplemented action: InitArray".into())
    }

    fn action_init_object(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let num_props = self.pop()?.as_i64()?;
        for _ in 0..num_props {
            let _value = self.pop()?;
            let _name = self.pop()?;
        }

        // TODO(Herschel)
        Err("Unimplemented action: InitObject".into())
    }

    fn action_jump(
        &mut self,
        _context: &mut ActionContext,
        jump_offset: i16
    ) -> Result<(), Error> {
        // TODO(Herschel): Handle out-of-bounds.
        self.with_current_reader_mut(|_this, r| r.seek(jump_offset.into()));
        Ok(())
    }

    fn action_less(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 less than
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() < a.into_number_v1();
        self.push(Value::from_bool_v1(result, self.current_swf_version().unwrap()));
        Ok(())
    }

    fn action_less_2(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // ECMA-262 s. 11.8.5
        let a = self.pop()?;
        let b = self.pop()?;

        let result = match (a, b) {
            (Value::String(a), Value::String(b)) => b.to_string().bytes().lt(a.to_string().bytes()),
            (a, b) => b.as_number() < a.as_number(),
        };

        self.push(Value::Bool(result));
        Ok(())
    }

    fn action_mb_ascii_to_char(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        use std::convert::TryFrom;
        let val = char::try_from(self.pop()?.as_f64()? as u32)?;
        self.push(Value::String(val.to_string()));
        Ok(())
    }

    fn action_mb_char_to_ascii(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Results on incorrect operands?
        let s = self.pop()?.into_string();
        let result = s.chars().nth(0).unwrap_or('\0') as u32;
        self.push(Value::Number(result.into()));
        Ok(())
    }

    fn action_mb_string_extract(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Result with incorrect operands?
        let len = self.pop()?.as_f64()? as usize;
        let start = self.pop()?.as_f64()? as usize;
        let s = self.pop()?.into_string();
        let result = s[len..len + start].to_string(); // TODO(Herschel): Flash uses UTF-16 internally.
        self.push(Value::String(result));
        Ok(())
    }

    fn action_mb_string_length(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop()?.into_string().len();
        self.push(Value::Number(val as f64));
        Ok(())
    }

    fn action_multiply(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 multiply
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(a.into_number_v1() * b.into_number_v1()));
        Ok(())
    }

    fn action_modulo(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO: Wrong operands?
        let a = self.pop()?.as_f64()?;
        let b = self.pop()?.as_f64()?;
        self.push(Value::Number(a % b));
        Ok(())
    }

    fn action_not(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical not
        let val = self.pop()?;
        let result = val.into_number_v1() == 0.0;
        self.push(Value::from_bool_v1(result, self.current_swf_version().unwrap()));
        Ok(())
    }

    fn action_next_frame(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
            let mut display_object = clip.write(context.gc_context);
            if let Some(clip) = display_object.as_movie_clip_mut() {
                clip.next_frame();
            } else {
                log::warn!("NextFrame: Target is not a MovieClip");
            }
        } else {
            log::warn!("NextFrame: Invalid target");
        }
        Ok(())
    }

    fn action_new_method(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _name = self.pop()?.as_string()?;
        let _object = self.pop()?.as_object()?;
        let _num_args = self.pop()?.as_i64()?;
        self.push(Value::Undefined);
        // TODO(Herschel)
        Err("Unimplemented action: NewMethod".into())
    }

    fn action_new_object(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _object = self.pop()?.as_string()?;
        let num_args = self.pop()?.as_i64()?;
        for _ in 0..num_args {
            let _arg = self.pop()?;
        }
        self.push(Value::Undefined);
        // TODO(Herschel)
        Err("Unimplemented action: NewObject".into())
    }

    fn action_or(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 logical or
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_number_v1() != 0.0 || a.into_number_v1() != 0.0;
        self.push(Value::from_bool_v1(result, self.current_swf_version().unwrap()));
        Ok(())
    }

    fn action_play(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
            let mut display_object = clip.write(context.gc_context);
            if let Some(clip) = display_object.as_movie_clip_mut() {
                clip.play()
            } else {
                log::warn!("Play: Target is not a MovieClip");
            }
        } else {
            log::warn!("Play: Invalid target");
        }
        Ok(())
    }

    fn action_prev_frame(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
            let mut display_object = clip.write(context.gc_context);
            if let Some(clip) = display_object.as_movie_clip_mut() {
                clip.prev_frame();
            } else {
                log::warn!("PrevFrame: Target is not a MovieClip");
            }
        } else {
            log::warn!("PrevFrame: Invalid target");
        }
        Ok(())
    }

    fn action_pop(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        self.pop()?;
        Ok(())
    }

    fn action_push(
        &mut self,
        _context: &mut ActionContext,
        values: &[swf::avm1::types::Value],
    ) -> Result<(), Error> {
        for value in values {
            use swf::avm1::types::Value as SwfValue;
            let value = match value {
                SwfValue::Undefined => Value::Undefined,
                SwfValue::Null => Value::Null,
                SwfValue::Bool(v) => Value::Bool(*v),
                SwfValue::Int(v) => Value::Number(f64::from(*v)),
                SwfValue::Float(v) => Value::Number(f64::from(*v)),
                SwfValue::Double(v) => Value::Number(*v),
                SwfValue::Str(v) => Value::String(v.to_string()),
                SwfValue::Register(_v) => {
                    log::error!("Register push unimplemented");
                    Value::Undefined
                }
                SwfValue::ConstantPool(i) => {
                    if let Some(value) = self.constant_pool.get(*i as usize) {
                        Value::String(value.to_string())
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

    fn action_push_duplicate(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.stack.last().ok_or("Stack underflow")?.clone();
        self.push(val);
        Ok(())
    }

    fn action_random_number(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        // A max value < 0 will always return 0,
        // and the max value gets converted into an i32, so any number > 2^31 - 1 will return 0.
        let max = self.pop()?.into_number_v1() as i32;
        let val = context.rng.gen_range(0, max);
        self.push(Value::Number(val.into()));
        Ok(())
    }

    fn action_remove_sprite(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _target = self.pop()?.into_string();
        // TODO(Herschel)
        log::error!("Unimplemented action: RemoveSprite");
        Ok(())
    }

    fn action_return(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let result = self.pop()?;

        if self.stack_frames.len() > 1 {
            self.retire_stack_frame();
            self.push(result);
        }

        Ok(())
    }

    fn action_set_member(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _value = self.pop()?;
        let _name = self.pop()?;
        let _object = self.pop()?;
        // TODO(Herschel)
        Err("Unimplemented action: SetMember".into())
    }

    fn action_set_property(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        let value = self.pop()?.into_number_v1() as f32;
        let prop_index = self.pop()?.as_u32()? as usize;
        let clip_path = self.pop()?;
        let path = clip_path.as_string()?;
        if let Some(base_clip) = context.target_clip {
            if let Some(clip) = Avm1::resolve_slash_path(base_clip, context.root, path) {
                if let Some(clip) = clip.write(context.gc_context).as_movie_clip_mut() {
                    match prop_index {
                        0 => clip.set_x(value),
                        1 => clip.set_y(value),
                        2 => clip.set_x_scale(value),
                        3 => clip.set_y_scale(value),
                        10 => clip.set_rotation(value),
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
        context: &mut ActionContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // Flash 4-style variable
        let value = self.pop()?;
        let var_path = self.pop()?;
        self.set_variable(context, var_path.as_string()?, value)
    }

    pub fn set_variable(
        &mut self,
        context: &mut ActionContext<'_, 'gc, '_>,
        var_path: &str,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        if let Some((node, var_name)) =
            Self::resolve_slash_path_variable(context.target_clip, context.root, var_path)
        {
            if let Some(clip) = node.write(context.gc_context).as_movie_clip_mut() {
                let object = clip.object().as_object()?;
                object
                    .write(context.gc_context)
                    .set(var_name, value, self, context, object);
            }
        }
        Ok(())
    }

    fn action_set_target(
        &mut self,
        context: &mut ActionContext,
        target: &str,
    ) -> Result<(), Error> {
        if target.is_empty() {
            context.active_clip = context.start_clip;
            context.target_clip = Some(context.start_clip);
            context.target_path = Value::String(target.to_string());
        } else if let Some(clip) =
            Avm1::resolve_slash_path(context.start_clip, context.root, target)
        {
            context.target_clip = Some(clip);
            context.active_clip = clip;
            context.target_path = Value::String(target.to_string());
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
        Ok(())
    }

    fn action_set_target2(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        let target = self.pop()?;
        if let Ok(target) = target.as_string() {
            self.action_set_target(context, target)?;
        } else {
            log::error!("SetTarget2: Path must be a string");
        }
        Ok(())
    }

    fn action_stack_swap(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(a);
        self.push(b);
        Ok(())
    }

    fn action_start_drag(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _target = self.pop()?;
        let _lock_center = self.pop()?.as_bool();
        let constrain = self.pop()?.as_bool();
        if constrain {
            let _y2 = self.pop()?;
            let _x2 = self.pop()?;
            let _y1 = self.pop()?;
            let _x1 = self.pop()?;
        }
        log::error!("Unimplemented action: StartDrag");
        Ok(())
    }

    fn action_stop(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        if let Some(clip) = context.target_clip {
            let mut display_object = clip.write(context.gc_context);
            if let Some(clip) = display_object.as_movie_clip_mut() {
                clip.stop();
            } else {
                log::warn!("Stop: Target is not a MovieClip");
            }
        } else {
            log::warn!("Stop: Invalid target");
        }
        Ok(())
    }

    fn action_stop_sounds(&mut self, context: &mut ActionContext) -> Result<(), Error> {
        context.audio.stop_all_sounds();
        Ok(())
    }

    fn action_store_register(
        &mut self,
        _context: &mut ActionContext,
        _register: u8,
    ) -> Result<(), Error> {
        // Does NOT pop the value from the stack.
        let _val = self.stack.last().ok_or("Stack underflow")?;
        Err("Unimplemented action: StoreRegister".into())
    }

    fn action_string_add(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // SWFv4 string concatenation
        // TODO(Herschel): Result with non-string operands?
        let a = self.pop()?.into_string();
        let mut b = self.pop()?.into_string();
        b.push_str(&a);
        self.push(Value::String(b));
        Ok(())
    }

    fn action_string_equals(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop()?;
        let b = self.pop()?;
        let result = b.into_string() == a.into_string();
        self.push(Value::from_bool_v1(result, self.current_swf_version().unwrap()));
        Ok(())
    }

    fn action_string_extract(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
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
        self.push(Value::String(result));
        Ok(())
    }

    fn action_string_length(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strlen
        // Only returns byte length.
        // TODO(Herschel): Result with non-string operands?
        let val = self.pop()?.into_string().bytes().len() as f64;
        self.push(Value::Number(val));
        Ok(())
    }

    fn action_string_less(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // AS1 strcmp
        let a = self.pop()?;
        let b = self.pop()?;
        // This is specifically a non-UTF8 aware comparison.
        let result = b.into_string().bytes().lt(a.into_string().bytes());
        self.push(Value::from_bool_v1(result, self.current_swf_version().unwrap()));
        Ok(())
    }

    fn action_subtract(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let a = self.pop()?;
        let b = self.pop()?;
        self.push(Value::Number(b.into_number_v1() - a.into_number_v1()));
        Ok(())
    }

    fn action_target_path(
        &mut self,
        _context: &mut ActionContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        // TODO(Herschel)
        let _clip = self.pop()?.as_object()?;
        self.push(Value::Undefined);
        Err("Unimplemented action: TargetPath".into())
    }

    fn toggle_quality(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        // TODO(Herschel): Noop for now? Could chang anti-aliasing on render backend.
        Ok(())
    }

    fn action_to_integer(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        self.push(Value::Number(val.into_number_v1().trunc()));
        Ok(())
    }

    fn action_to_number(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        self.push(Value::Number(val.as_number()));
        Ok(())
    }

    fn action_to_string(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        self.push(Value::String(val.into_string()));
        Ok(())
    }

    fn action_trace(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let val = self.pop()?;
        log::info!(target: "avm_trace", "{}", val.into_string());
        Ok(())
    }

    fn action_type_of(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let type_of = self.pop()?.type_of();
        self.push(type_of);
        Ok(())
    }

    fn action_wait_for_frame(
        &mut self,
        _context: &mut ActionContext,
        _frame: u16,
        num_actions_to_skip: u8
    ) -> Result<(), Error> {
        // TODO(Herschel): Always true for now.
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            self.with_current_reader_mut(|_this, r| skip_actions(r, num_actions_to_skip)).unwrap()?;
        }
        Ok(())
    }

    fn action_wait_for_frame_2(
        &mut self,
        _context: &mut ActionContext,
        num_actions_to_skip: u8
    ) -> Result<(), Error> {
        // TODO(Herschel): Always true for now.
        let _frame_num = self.pop()?.as_f64()? as u16;
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            self.with_current_reader_mut(|_this, r| skip_actions(r, num_actions_to_skip)).unwrap()?;
        }
        Ok(())
    }

    fn action_with(&mut self, _context: &mut ActionContext) -> Result<(), Error> {
        let _object = self.pop()?.as_object()?;
        Err("Unimplemented action: With".into())
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