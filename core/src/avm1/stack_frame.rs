use crate::avm1::error::Error;
use crate::avm1::function::{Avm1Function, FunctionObject};
use crate::avm1::object::{Object, TObject};
use crate::avm1::property::Attribute;
use crate::avm1::scope::Scope;
use crate::avm1::value::f64_to_wrapping_u32;
use crate::avm1::{
    fscommand, globals, skip_actions, start_drag, value_object, Activation, Avm1, ScriptObject,
    Value,
};
use crate::backend::navigator::{NavigationMethod, RequestOptions};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::tag_utils::SwfSlice;
use enumset::EnumSet;
use gc_arena::{Collect, GcCell};
use rand::Rng;
use std::borrow::Cow;
use std::convert::TryInto;
use swf::avm1::read::Reader;
use swf::avm1::types::{Action, Function};

macro_rules! avm_debug {
    ($($arg:tt)*) => (
        #[cfg(feature = "avm_debug")]
        log::debug!($($arg)*)
    )
}

#[derive(Debug, Clone)]
pub enum ReturnType<'gc> {
    Implicit,
    Explicit(Value<'gc>),
}

impl<'gc> ReturnType<'gc> {
    pub fn value(self) -> Value<'gc> {
        match self {
            ReturnType::Implicit => Value::Undefined,
            ReturnType::Explicit(value) => value,
        }
    }
}

#[derive(Debug, Clone)]
enum FrameControl<'gc> {
    Continue,
    Return(ReturnType<'gc>),
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct StackFrame<'a, 'gc: 'a> {
    avm: &'a mut Avm1<'gc>,
    activation: GcCell<'gc, Activation<'gc>>,
}

impl<'a, 'gc: 'a> StackFrame<'a, 'gc> {
    pub fn new(avm: &'a mut Avm1<'gc>, activation: GcCell<'gc, Activation<'gc>>) -> Self {
        Self { avm, activation }
    }

    pub fn run(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnType<'gc>, Error<'gc>> {
        let mut activation = self.activation.write(context.gc_context);
        activation.lock()?;
        let data = activation.data();
        let mut read = Reader::new(data.as_ref(), activation.swf_version());
        read.seek(activation.pc().try_into().unwrap());
        drop(activation);

        let result = loop {
            let result = self.do_action(&data, context, &mut read);
            match result {
                Ok(FrameControl::Return(return_type)) => break Ok(return_type),
                Ok(FrameControl::Continue) => {}
                Err(e) => break Err(e),
            }
        };

        let mut activation = self.activation.write(context.gc_context);
        activation.unlock_execution();
        activation.set_pc(read.pos());
        drop(activation);

        result
    }

    /// Run a single action from a given action reader.
    fn do_action(
        &mut self,
        data: &SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut Reader<'_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if reader.pos() >= (data.end - data.start) {
            //Executing beyond the end of a function constitutes an implicit return.
            Ok(FrameControl::Return(ReturnType::Implicit))
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
                Action::Return => self.action_return(),
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
                Action::Throw => self.action_throw(context),
                _ => self.unknown_op(context, action),
            };
            if let Err(e) = result {
                match &e {
                    Error::ThrownValue(_) => {}
                    e => log::error!("AVM1 error: {}", e),
                }
                if e.is_halting() {
                    self.avm.halt();
                }
                return Err(e);
            }
            result
        } else {
            //The explicit end opcode was encountered so return here
            Ok(FrameControl::Return(ReturnType::Implicit))
        }
    }

    fn unknown_op(
        &mut self,
        _context: &mut UpdateContext,
        action: swf::avm1::types::Action,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        log::error!("Unknown AVM1 opcode: {:?}", action);
        Ok(FrameControl::Continue)
    }

    fn action_add(
        &mut self,
        _context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop();
        let b = self.avm.pop();
        self.avm.push(b.into_number_v1() + a.into_number_v1());
        Ok(FrameControl::Continue)
    }

    fn action_add_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // ECMA-262 s. 11.6.1
        let a = self.avm.pop();
        let b = self.avm.pop();

        // TODO(Herschel):
        if let Value::String(a) = a {
            let mut s = b.coerce_to_string(self.avm, context)?.to_string();
            s.push_str(&a);
            self.avm.push(s);
        } else if let Value::String(mut b) = b {
            b.push_str(&a.coerce_to_string(self.avm, context)?);
            self.avm.push(b);
        } else {
            let result =
                b.coerce_to_f64(self.avm, context)? + a.coerce_to_f64(self.avm, context)?;
            self.avm.push(result);
        }
        Ok(FrameControl::Continue)
    }

    fn action_and(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 logical and
        let a = self.avm.pop();
        let b = self.avm.pop();
        let version = self.avm.current_swf_version();
        let result = b.as_bool(version) && a.as_bool(version);
        self.avm
            .push(Value::from_bool(result, self.avm.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_ascii_to_char(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Results on incorrect operands?
        let val = (self.avm.pop().coerce_to_f64(self.avm, context)? as u8) as char;
        self.avm.push(val.to_string());
        Ok(FrameControl::Continue)
    }

    fn action_char_to_ascii(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Results on incorrect operands?
        let val = self.avm.pop();
        let string = val.coerce_to_string(self.avm, context)?;
        let result = string.bytes().next().unwrap_or(0);
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_clone_sprite(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let depth = self.avm.pop();
        let target = self.avm.pop();
        let source = self.avm.pop();
        let start_clip = self.avm.target_clip_or_root();
        let source_clip = self
            .avm
            .resolve_target_display_object(context, start_clip, source)?;

        if let Some(movie_clip) = source_clip.and_then(|o| o.as_movie_clip()) {
            let _ = globals::movie_clip::duplicate_movie_clip_with_bias(
                movie_clip,
                self.avm,
                context,
                &[target, depth],
                0,
            );
        } else {
            log::warn!("CloneSprite: Source is not a movie clip");
        }

        Ok(FrameControl::Continue)
    }

    fn action_bit_and(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_u32(self.avm, context)?;
        let b = self.avm.pop().coerce_to_u32(self.avm, context)?;
        let result = a & b;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_lshift(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_i32(self.avm, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.avm.pop().coerce_to_i32(self.avm, context)?;
        let result = b << a;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_or(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_u32(self.avm, context)?;
        let b = self.avm.pop().coerce_to_u32(self.avm, context)?;
        let result = a | b;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_rshift(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_i32(self.avm, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.avm.pop().coerce_to_i32(self.avm, context)?;
        let result = b >> a;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_urshift(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_u32(self.avm, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.avm.pop().coerce_to_u32(self.avm, context)?;
        let result = b >> a;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_xor(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_u32(self.avm, context)?;
        let b = self.avm.pop().coerce_to_u32(self.avm, context)?;
        let result = b ^ a;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_call(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Runs any actions on the given frame.
        let frame = self.avm.pop();
        let clip = self.avm.target_clip_or_root();
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
                let frame_label = frame.coerce_to_string(self.avm, context)?;
                clip.frame_label_to_number(&frame_label)
            };

            if let Some(frame) = frame {
                for action in clip.actions_on_frame(context, frame) {
                    self.avm.insert_stack_frame_for_action(
                        self.avm.target_clip_or_root(),
                        self.avm.current_swf_version(),
                        action,
                        context,
                    );
                    let frame = self.avm.current_stack_frame().unwrap();
                    self.avm.run_activation(context, frame)?;
                }
            } else {
                log::warn!("Call: Invalid frame {:?}", frame);
            }
        } else {
            log::warn!("Call: Expected MovieClip");
        }
        Ok(FrameControl::Continue)
    }

    fn action_call_function(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let fn_name_value = self.avm.pop();
        let fn_name = fn_name_value.coerce_to_string(self.avm, context)?;
        let mut args = Vec::new();
        let num_args = self.avm.pop().coerce_to_f64(self.avm, context)? as i64; // TODO(Herschel): max arg count?
        for _ in 0..num_args {
            args.push(self.avm.pop());
        }

        let target_fn = self
            .avm
            .get_variable(context, &fn_name)?
            .resolve(self.avm, context)?;

        let this = self
            .avm
            .target_clip_or_root()
            .object()
            .coerce_to_object(self.avm, context);
        let result = target_fn.call(self.avm, context, this, None, &args)?;
        self.avm.push(result);

        Ok(FrameControl::Continue)
    }

    fn action_call_method(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let method_name = self.avm.pop();
        let object_val = self.avm.pop();
        let object = value_object::ValueObject::boxed(self.avm, context, object_val);
        let num_args = self.avm.pop().coerce_to_f64(self.avm, context)? as i64; // TODO(Herschel): max arg count?
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.avm.pop());
        }

        match method_name {
            Value::Undefined | Value::Null => {
                let this = self
                    .avm
                    .target_clip_or_root()
                    .object()
                    .coerce_to_object(self.avm, context);
                let result = object.call(self.avm, context, this, None, &args)?;
                self.avm.push(result);
            }
            Value::String(name) => {
                if name.is_empty() {
                    let result = object.call(self.avm, context, object, None, &args)?;
                    self.avm.push(result);
                } else {
                    let result = object.call_method(&name, &args, self.avm, context)?;
                    self.avm.push(result);
                }
            }
            _ => {
                self.avm.push(Value::Undefined);
                log::warn!(
                    "Invalid method name, expected string but found {:?}",
                    method_name
                );
            }
        }

        Ok(FrameControl::Continue)
    }

    fn action_cast_op(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let obj = self.avm.pop().coerce_to_object(self.avm, context);
        let constr = self.avm.pop().coerce_to_object(self.avm, context);

        let prototype = constr
            .get("prototype", self.avm, context)?
            .coerce_to_object(self.avm, context);

        if obj.is_instance_of(self.avm, context, constr, prototype)? {
            self.avm.push(obj);
        } else {
            self.avm.push(Value::Null);
        }

        Ok(FrameControl::Continue)
    }

    fn action_constant_pool(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        constant_pool: &[&str],
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.avm.constant_pool = GcCell::allocate(
            context.gc_context,
            constant_pool.iter().map(|s| (*s).to_string()).collect(),
        );
        self.activation
            .write(context.gc_context)
            .set_constant_pool(self.avm.constant_pool);

        Ok(FrameControl::Continue)
    }

    fn action_decrement(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_f64(self.avm, context)?;
        self.avm.push(a - 1.0);
        Ok(FrameControl::Continue)
    }

    fn action_define_function(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
        params: &[&str],
        actions: &[u8],
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let swf_version = self.activation.read().swf_version();
        let func_data = self.activation.read().data().to_subslice(actions).unwrap();
        let scope =
            Scope::new_closure_scope(self.activation.read().scope_cell(), context.gc_context);
        let constant_pool = self.activation.read().constant_pool();
        let func = Avm1Function::from_df1(
            swf_version,
            func_data,
            name,
            params,
            scope,
            constant_pool,
            self.avm.target_clip_or_root(),
        );
        let prototype =
            ScriptObject::object(context.gc_context, Some(self.avm.prototypes.object)).into();
        let func_obj = FunctionObject::function(
            context.gc_context,
            func,
            Some(self.avm.prototypes.function),
            Some(prototype),
        );
        if name == "" {
            self.avm.push(func_obj);
        } else {
            self.activation
                .read()
                .define(name, func_obj, context.gc_context);
        }

        Ok(FrameControl::Continue)
    }

    fn action_define_function_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        action_func: &Function,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let swf_version = self.activation.read().swf_version();
        let func_data = self
            .activation
            .read()
            .data()
            .to_subslice(action_func.actions)
            .unwrap();
        let scope =
            Scope::new_closure_scope(self.activation.read().scope_cell(), context.gc_context);
        let constant_pool = self.activation.read().constant_pool();
        let func = Avm1Function::from_df2(
            swf_version,
            func_data,
            action_func,
            scope,
            constant_pool,
            self.avm.base_clip(),
        );
        let prototype =
            ScriptObject::object(context.gc_context, Some(self.avm.prototypes.object)).into();
        let func_obj = FunctionObject::function(
            context.gc_context,
            func,
            Some(self.avm.prototypes.function),
            Some(prototype),
        );
        if action_func.name == "" {
            self.avm.push(func_obj);
        } else {
            self.activation
                .read()
                .define(action_func.name, func_obj, context.gc_context);
        }

        Ok(FrameControl::Continue)
    }

    fn action_define_local(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // If the property does not exist on the local object's prototype chain, it is created on the local object.
        // Otherwise, the property is set (including calling virtual setters).
        let value = self.avm.pop();
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self.avm, context)?;
        let stack_frame = self.activation.read();
        let scope = stack_frame.scope();
        scope.locals().set(&name, value, self.avm, context)?;
        Ok(FrameControl::Continue)
    }

    fn action_define_local_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // If the property does not exist on the local object's prototype chain, it is created on the local object.
        // Otherwise, the property is unchanged.
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self.avm, context)?;
        let stack_frame = self.activation.read();
        let scope = stack_frame.scope();
        if !scope.locals().has_property(self.avm, context, &name) {
            scope
                .locals()
                .set(&name, Value::Undefined, self.avm, context)?;
        }
        Ok(FrameControl::Continue)
    }

    fn action_delete(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self.avm, context)?;
        let object = self.avm.pop();

        if let Value::Object(object) = object {
            let success = object.delete(self.avm, context.gc_context, &name);
            self.avm.push(success);
        } else {
            log::warn!("Cannot delete property {} from {:?}", name, object);
            self.avm.push(false);
        }

        Ok(FrameControl::Continue)
    }

    fn action_delete_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self.avm, context)?;

        //Fun fact: This isn't in the Adobe SWF19 spec, but this opcode returns
        //a boolean based on if the delete actually deleted something.
        let did_exist = self.activation.read().is_defined(self.avm, context, &name);

        self.activation
            .read()
            .scope()
            .delete(self.avm, context, &name, context.gc_context);
        self.avm.push(did_exist);

        Ok(FrameControl::Continue)
    }

    fn action_divide(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 divide
        let a = self.avm.pop().coerce_to_f64(self.avm, context)?;
        let b = self.avm.pop().coerce_to_f64(self.avm, context)?;

        // TODO(Herschel): SWF19: "If A is zero, the result NaN, Infinity, or -Infinity is pushed to the in SWF 5 and later.
        // In SWF 4, the result is the string #ERROR#.""
        // Seems to be untrue for SWF v4, I get 1.#INF.

        self.avm.push(b / a);
        Ok(FrameControl::Continue)
    }

    fn action_end_drag(
        &mut self,
        context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        *context.drag_object = None;
        Ok(FrameControl::Continue)
    }

    fn action_enumerate(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_value = self.avm.pop();
        let name = name_value.coerce_to_string(self.avm, context)?;
        self.avm.push(Value::Null); // Sentinel that indicates end of enumeration
        let object = self
            .activation
            .read()
            .resolve(&name, self.avm, context)?
            .resolve(self.avm, context)?;

        match object {
            Value::Object(ob) => {
                for k in ob.get_keys(self.avm).into_iter().rev() {
                    self.avm.push(k);
                }
            }
            _ => log::error!("Cannot enumerate properties of {}", name),
        };

        Ok(FrameControl::Continue)
    }

    fn action_enumerate_2(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.avm.pop();

        self.avm.push(Value::Null); // Sentinel that indicates end of enumeration

        if let Value::Object(object) = value {
            for k in object.get_keys(self.avm).into_iter().rev() {
                self.avm.push(k);
            }
        } else {
            log::warn!("Cannot enumerate {:?}", value);
        }

        Ok(FrameControl::Continue)
    }

    #[allow(clippy::float_cmp)]
    fn action_equals(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 equality
        let a = self.avm.pop();
        let b = self.avm.pop();
        let result = b.into_number_v1() == a.into_number_v1();
        self.avm
            .push(Value::from_bool(result, self.avm.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    #[allow(clippy::float_cmp)]
    fn action_equals_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Version >=5 equality
        let a = self.avm.pop();
        let b = self.avm.pop();
        let result = b.abstract_eq(a, self.avm, context, false)?;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_extends(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let superclass = self.avm.pop().coerce_to_object(self.avm, context);
        let subclass = self.avm.pop().coerce_to_object(self.avm, context);

        //TODO: What happens if we try to extend an object which has no `prototype`?
        //e.g. `class Whatever extends Object.prototype` or `class Whatever extends 5`
        let super_proto = superclass
            .get("prototype", self.avm, context)?
            .coerce_to_object(self.avm, context);

        let mut sub_prototype: Object<'gc> =
            ScriptObject::object(context.gc_context, Some(super_proto)).into();

        sub_prototype.set("constructor", superclass.into(), self.avm, context)?;
        sub_prototype.set_attributes(
            context.gc_context,
            Some("constructor"),
            Attribute::DontEnum.into(),
            EnumSet::empty(),
        );

        sub_prototype.set("__constructor__", superclass.into(), self.avm, context)?;
        sub_prototype.set_attributes(
            context.gc_context,
            Some("__constructor__"),
            Attribute::DontEnum.into(),
            EnumSet::empty(),
        );

        subclass.set("prototype", sub_prototype.into(), self.avm, context)?;

        Ok(FrameControl::Continue)
    }

    fn action_get_member(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self.avm, context)?;
        let object_val = self.avm.pop();
        let object = value_object::ValueObject::boxed(self.avm, context, object_val);

        let result = object.get(&name, self.avm, context)?;
        self.avm.push(result);

        Ok(FrameControl::Continue)
    }

    fn action_get_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let prop_index = self.avm.pop().into_number_v1() as usize;
        let path = self.avm.pop();
        let ret = if let Some(target) = self.avm.target_clip() {
            if let Some(clip) = self
                .avm
                .resolve_target_display_object(context, target, path)?
            {
                let display_properties = self.avm.display_properties;
                let props = display_properties.write(context.gc_context);
                if let Some(property) = props.get_by_index(prop_index) {
                    property.get(self.avm, context, clip)?
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
        self.avm.push(ret);
        Ok(FrameControl::Continue)
    }

    fn action_get_time(
        &mut self,
        context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let time = context.navigator.time_since_launch().as_millis() as u32;
        self.avm.push(time);
        Ok(FrameControl::Continue)
    }

    fn action_get_variable(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let var_path = self.avm.pop();
        let path = var_path.coerce_to_string(self.avm, context)?;

        self.avm.get_variable(context, &path)?.push(self.avm);

        Ok(FrameControl::Continue)
    }

    fn action_get_url(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        url: &str,
        target: &str,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if target.starts_with("_level") && target.len() > 6 {
            let url = url.to_string();
            match target[6..].parse::<u32>() {
                Ok(level_id) => {
                    let fetch = context.navigator.fetch(&url, RequestOptions::get());
                    let level = self.avm.resolve_level(level_id, context);

                    let process = context.load_manager.load_movie_into_clip(
                        context.player.clone().unwrap(),
                        level,
                        fetch,
                        None,
                    );
                    context.navigator.spawn_future(process);
                }
                Err(e) => log::warn!(
                    "Couldn't parse level id {} for action_get_url: {}",
                    target,
                    e
                ),
            }

            return Ok(FrameControl::Continue);
        }

        if let Some(fscommand) = fscommand::parse(url) {
            fscommand::handle(fscommand, self.avm, context)?;
        } else {
            context
                .navigator
                .navigate_to_url(url.to_owned(), Some(target.to_owned()), None);
        }

        Ok(FrameControl::Continue)
    }

    fn action_get_url_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_method: swf::avm1::types::SendVarsMethod,
        is_target_sprite: bool,
        is_load_vars: bool,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO: Support `LoadVariablesFlag`, `LoadTargetFlag`
        // TODO: What happens if there's only one string?
        let target = self.avm.pop();
        let url_val = self.avm.pop();
        let url = url_val.coerce_to_string(self.avm, context)?;

        if let Some(fscommand) = fscommand::parse(&url) {
            fscommand::handle(fscommand, self.avm, context)?;
            return Ok(FrameControl::Continue);
        }

        let window_target = target.coerce_to_string(self.avm, context)?;
        let clip_target: Option<DisplayObject<'gc>> = if is_target_sprite {
            if let Value::Object(target) = target {
                target.as_display_object()
            } else {
                let start = self.avm.target_clip_or_root();
                self.avm
                    .resolve_target_display_object(context, start, target.clone())?
            }
        } else {
            Some(self.avm.target_clip_or_root())
        };

        if is_load_vars {
            if let Some(clip_target) = clip_target {
                let target_obj = clip_target
                    .as_movie_clip()
                    .unwrap()
                    .object()
                    .coerce_to_object(self.avm, context);
                let (url, opts) = self.avm.locals_into_request_options(
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

            return Ok(FrameControl::Continue);
        } else if is_target_sprite {
            if let Some(clip_target) = clip_target {
                let (url, opts) = self.avm.locals_into_request_options(
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

            return Ok(FrameControl::Continue);
        } else {
            let vars = match NavigationMethod::from_send_vars_method(swf_method) {
                Some(method) => Some((method, self.avm.locals_into_form_values(context))),
                None => None,
            };

            context.navigator.navigate_to_url(
                url.to_string(),
                Some(window_target.to_string()),
                vars,
            );
        }

        Ok(FrameControl::Continue)
    }

    fn action_goto_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        frame: u16,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.avm.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                // The frame on the stack is 0-based, not 1-based.
                clip.goto_frame(self.avm, context, frame + 1, true);
            } else {
                log::error!("GotoFrame failed: Target is not a MovieClip");
            }
        } else {
            log::error!("GotoFrame failed: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_goto_frame_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        set_playing: bool,
        scene_offset: u16,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Version 4+ gotoAndPlay/gotoAndStop
        // Param can either be a frame number or a frame label.
        if let Some(clip) = self.avm.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                let frame = self.avm.pop();
                let _ = globals::movie_clip::goto_frame(
                    clip,
                    self.avm,
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
        Ok(FrameControl::Continue)
    }

    fn action_goto_label(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        label: &str,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.avm.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                if let Some(frame) = clip.frame_label_to_number(label) {
                    clip.goto_frame(self.avm, context, frame, true);
                } else {
                    log::warn!("GoToLabel: Frame label '{}' not found", label);
                }
            } else {
                log::warn!("GoToLabel: Target is not a MovieClip");
            }
        } else {
            log::warn!("GoToLabel: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_if(
        &mut self,
        _context: &mut UpdateContext,
        jump_offset: i16,
        reader: &mut Reader<'_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.avm.pop();
        if val.as_bool(self.avm.current_swf_version()) {
            reader.seek(jump_offset.into());
        }
        Ok(FrameControl::Continue)
    }

    fn action_increment(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_f64(self.avm, context)?;
        self.avm.push(a + 1.0);
        Ok(FrameControl::Continue)
    }

    fn action_init_array(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let num_elements = self.avm.pop().coerce_to_f64(self.avm, context)? as i64;
        let array = ScriptObject::array(context.gc_context, Some(self.avm.prototypes.array));

        for i in 0..num_elements {
            array.set_array_element(i as usize, self.avm.pop(), context.gc_context);
        }

        self.avm.push(Value::Object(array.into()));
        Ok(FrameControl::Continue)
    }

    fn action_init_object(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let num_props = self.avm.pop().coerce_to_f64(self.avm, context)? as i64;
        let object = ScriptObject::object(context.gc_context, Some(self.avm.prototypes.object));
        for _ in 0..num_props {
            let value = self.avm.pop();
            let name_val = self.avm.pop();
            let name = name_val.coerce_to_string(self.avm, context)?;
            object.set(&name, value, self.avm, context)?;
        }

        self.avm.push(Value::Object(object.into()));

        Ok(FrameControl::Continue)
    }

    fn action_implements_op(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let constr = self.avm.pop().coerce_to_object(self.avm, context);
        let count = self.avm.pop().coerce_to_f64(self.avm, context)? as i64; //TODO: Is this coercion actually performed by Flash?
        let mut interfaces = vec![];

        //TODO: If one of the interfaces is not an object, do we leave the
        //whole stack dirty, or...?
        for _ in 0..count {
            interfaces.push(self.avm.pop().coerce_to_object(self.avm, context));
        }

        let mut prototype = constr
            .get("prototype", self.avm, context)?
            .coerce_to_object(self.avm, context);

        prototype.set_interfaces(context.gc_context, interfaces);

        Ok(FrameControl::Continue)
    }

    fn action_instance_of(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let constr = self.avm.pop().coerce_to_object(self.avm, context);
        let obj = self.avm.pop().coerce_to_object(self.avm, context);

        let prototype = constr
            .get("prototype", self.avm, context)?
            .coerce_to_object(self.avm, context);
        let is_instance_of = obj.is_instance_of(self.avm, context, constr, prototype)?;

        self.avm.push(is_instance_of);
        Ok(FrameControl::Continue)
    }

    fn action_jump(
        &mut self,
        _context: &mut UpdateContext,
        jump_offset: i16,
        reader: &mut Reader<'_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Handle out-of-bounds.
        reader.seek(jump_offset.into());
        Ok(FrameControl::Continue)
    }

    fn action_less(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 less than
        let a = self.avm.pop();
        let b = self.avm.pop();
        let result = b.into_number_v1() < a.into_number_v1();
        self.avm
            .push(Value::from_bool(result, self.avm.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_less_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // ECMA-262 s. 11.8.1
        let a = self.avm.pop();
        let b = self.avm.pop();

        let result = b.abstract_lt(a, self.avm, context)?;

        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_greater(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // ECMA-262 s. 11.8.2
        let a = self.avm.pop();
        let b = self.avm.pop();

        let result = a.abstract_lt(b, self.avm, context)?;

        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_mb_ascii_to_char(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Results on incorrect operands?
        use std::convert::TryFrom;
        let result = char::try_from(self.avm.pop().coerce_to_f64(self.avm, context)? as u32);
        match result {
            Ok(val) => self.avm.push(val.to_string()),
            Err(e) => log::warn!("Couldn't parse char for action_mb_ascii_to_char: {}", e),
        }
        Ok(FrameControl::Continue)
    }

    fn action_mb_char_to_ascii(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Results on incorrect operands?
        let val = self.avm.pop();
        let s = val.coerce_to_string(self.avm, context)?;
        let result = s.chars().next().unwrap_or('\0') as u32;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_mb_string_extract(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Result with incorrect operands?
        let len = self.avm.pop().coerce_to_f64(self.avm, context)? as usize;
        let start = self.avm.pop().coerce_to_f64(self.avm, context)? as usize;
        let val = self.avm.pop();
        let s = val.coerce_to_string(self.avm, context)?;
        let result = s[len..len + start].to_string(); // TODO(Herschel): Flash uses UTF-16 internally.
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_mb_string_length(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Result with non-string operands?
        let val = self.avm.pop();
        let len = val.coerce_to_string(self.avm, context)?.len();
        self.avm.push(len as f64);
        Ok(FrameControl::Continue)
    }

    fn action_multiply(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_f64(self.avm, context)?;
        let b = self.avm.pop().coerce_to_f64(self.avm, context)?;
        self.avm.push(a * b);
        Ok(FrameControl::Continue)
    }

    fn action_modulo(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO: Wrong operands?
        let a = self.avm.pop().coerce_to_f64(self.avm, context)?;
        let b = self.avm.pop().coerce_to_f64(self.avm, context)?;
        self.avm.push(b % a);
        Ok(FrameControl::Continue)
    }

    fn action_not(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let version = self.avm.current_swf_version();
        let val = !self.avm.pop().as_bool(version);
        self.avm.push(Value::from_bool(val, version));
        Ok(FrameControl::Continue)
    }

    fn action_next_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.avm.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.next_frame(self.avm, context);
            } else {
                log::warn!("NextFrame: Target is not a MovieClip");
            }
        } else {
            log::warn!("NextFrame: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_new_method(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let method_name = self.avm.pop();
        let object_val = self.avm.pop();
        let num_args = self.avm.pop().coerce_to_f64(self.avm, context)? as i64;
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.avm.pop());
        }

        let object = value_object::ValueObject::boxed(self.avm, context, object_val);
        let constructor = object.get(
            &method_name.coerce_to_string(self.avm, context)?,
            self.avm,
            context,
        )?;
        if let Value::Object(constructor) = constructor {
            let prototype = constructor
                .get("prototype", self.avm, context)?
                .coerce_to_object(self.avm, context);

            let mut this = prototype.new(self.avm, context, prototype, &args)?;
            this.set("__constructor__", constructor.into(), self.avm, context)?;
            this.set_attributes(
                context.gc_context,
                Some("__constructor__"),
                Attribute::DontEnum.into(),
                EnumSet::empty(),
            );
            if self.avm.current_swf_version() < 7 {
                this.set("constructor", constructor.into(), self.avm, context)?;
                this.set_attributes(
                    context.gc_context,
                    Some("constructor"),
                    Attribute::DontEnum.into(),
                    EnumSet::empty(),
                );
            }

            //TODO: What happens if you `ActionNewMethod` without a method name?
            constructor.call(self.avm, context, this, None, &args)?;

            self.avm.push(this);
        } else {
            log::warn!(
                "Tried to construct with non-object constructor {:?}",
                constructor
            );
            self.avm.push(Value::Undefined);
        }

        Ok(FrameControl::Continue)
    }

    fn action_new_object(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let fn_name_val = self.avm.pop();
        let fn_name = fn_name_val.coerce_to_string(self.avm, context)?;
        let num_args = self.avm.pop().coerce_to_f64(self.avm, context)? as i64;
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.avm.pop());
        }

        let constructor = self
            .activation
            .read()
            .resolve(&fn_name, self.avm, context)?
            .resolve(self.avm, context)?
            .coerce_to_object(self.avm, context);
        let prototype = constructor
            .get("prototype", self.avm, context)?
            .coerce_to_object(self.avm, context);

        let mut this = prototype.new(self.avm, context, prototype, &args)?;
        this.set("__constructor__", constructor.into(), self.avm, context)?;
        this.set_attributes(
            context.gc_context,
            Some("__constructor__"),
            Attribute::DontEnum.into(),
            EnumSet::empty(),
        );
        if self.avm.current_swf_version() < 7 {
            this.set("constructor", constructor.into(), self.avm, context)?;
            this.set_attributes(
                context.gc_context,
                Some("constructor"),
                Attribute::DontEnum.into(),
                EnumSet::empty(),
            );
        }

        constructor.call(self.avm, context, this, None, &args)?;

        self.avm.push(this);

        Ok(FrameControl::Continue)
    }

    fn action_or(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 logical or
        let a = self.avm.pop();
        let b = self.avm.pop();
        let version = self.avm.current_swf_version();
        let result = b.as_bool(version) || a.as_bool(version);
        self.avm.push(Value::from_bool(result, version));
        Ok(FrameControl::Continue)
    }

    fn action_play(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.avm.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.play(context)
            } else {
                log::warn!("Play: Target is not a MovieClip");
            }
        } else {
            log::warn!("Play: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_prev_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.avm.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.prev_frame(self.avm, context);
            } else {
                log::warn!("PrevFrame: Target is not a MovieClip");
            }
        } else {
            log::warn!("PrevFrame: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_pop(
        &mut self,
        _context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.avm.pop();
        Ok(FrameControl::Continue)
    }

    fn action_push(
        &mut self,
        _context: &mut UpdateContext,
        values: &[swf::avm1::types::Value],
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
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
                SwfValue::Register(v) => self.avm.current_register(*v),
                SwfValue::ConstantPool(i) => {
                    if let Some(value) = self
                        .activation
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
                            self.activation.read().constant_pool().read().len()
                        );
                        Value::Undefined
                    }
                }
            };
            self.avm.push(value);
        }
        Ok(FrameControl::Continue)
    }

    fn action_push_duplicate(
        &mut self,
        _context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.avm.pop();
        self.avm.push(val.clone());
        self.avm.push(val);
        Ok(FrameControl::Continue)
    }

    fn action_random_number(
        &mut self,
        context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // A max value < 0 will always return 0,
        // and the max value gets converted into an i32, so any number > 2^31 - 1 will return 0.
        let max = self.avm.pop().into_number_v1() as i32;
        let val = if max > 0 {
            context.rng.gen_range(0, max)
        } else {
            0
        };
        self.avm.push(val);
        Ok(FrameControl::Continue)
    }

    fn action_remove_sprite(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let target = self.avm.pop();
        let start_clip = self.avm.target_clip_or_root();
        let target_clip = self
            .avm
            .resolve_target_display_object(context, start_clip, target)?;

        if let Some(target_clip) = target_clip.and_then(|o| o.as_movie_clip()) {
            let _ = globals::movie_clip::remove_movie_clip_with_bias(target_clip, context, 0);
        } else {
            log::warn!("RemoveSprite: Source is not a movie clip");
        }
        Ok(FrameControl::Continue)
    }

    fn action_return(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let return_value = self.avm.pop();

        Ok(FrameControl::Return(ReturnType::Explicit(return_value)))
    }

    fn action_set_member(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.avm.pop();
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self.avm, context)?;

        let object = self.avm.pop().coerce_to_object(self.avm, context);
        object.set(&name, value, self.avm, context)?;

        Ok(FrameControl::Continue)
    }

    fn action_set_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.avm.pop();
        let prop_index = self.avm.pop().coerce_to_u32(self.avm, context)? as usize;
        let path = self.avm.pop();
        if let Some(target) = self.avm.target_clip() {
            if let Some(clip) = self
                .avm
                .resolve_target_display_object(context, target, path)?
            {
                let display_properties = self.avm.display_properties;
                let props = display_properties.read();
                if let Some(property) = props.get_by_index(prop_index) {
                    property.set(self.avm, context, clip, value)?;
                }
            } else {
                log::warn!("SetProperty: Invalid target");
            }
        } else {
            log::warn!("SetProperty: Invalid base clip");
        }
        Ok(FrameControl::Continue)
    }

    fn action_set_variable(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Flash 4-style variable
        let value = self.avm.pop();
        let var_path_val = self.avm.pop();
        let var_path = var_path_val.coerce_to_string(self.avm, context)?;
        self.avm.set_variable(context, &var_path, value)?;
        Ok(FrameControl::Continue)
    }

    #[allow(clippy::float_cmp)]
    fn action_strict_equals(
        &mut self,
        _context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // The same as normal equality but types must match
        let a = self.avm.pop();
        let b = self.avm.pop();
        let result = a == b;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_set_target(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        target: &str,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let base_clip = self.avm.base_clip();
        let new_target_clip;
        let root = base_clip.root();
        let start = base_clip.object().coerce_to_object(self.avm, context);
        if target.is_empty() {
            new_target_clip = Some(base_clip);
        } else if let Some(clip) = self
            .avm
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

        let mut sf = self.activation.write(context.gc_context);
        sf.set_target_clip(new_target_clip);

        let scope = sf.scope_cell();
        let clip_obj = sf
            .target_clip()
            .unwrap_or_else(|| sf.base_clip().root())
            .object()
            .coerce_to_object(self.avm, context);

        sf.set_scope(Scope::new_target_scope(scope, clip_obj, context.gc_context));
        Ok(FrameControl::Continue)
    }

    fn action_set_target2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let target = self.avm.pop();
        match target {
            Value::String(target) => {
                return self.action_set_target(context, &target);
            }
            Value::Undefined => {
                // Reset
                let mut sf = self.activation.write(context.gc_context);
                let base_clip = sf.base_clip();
                sf.set_target_clip(Some(base_clip));
            }
            Value::Object(o) => {
                if let Some(clip) = o.as_display_object() {
                    let mut sf = self.activation.write(context.gc_context);
                    // Movieclips can be targetted directly
                    sf.set_target_clip(Some(clip));
                } else {
                    // Other objects get coerced to string
                    let target = target.coerce_to_string(self.avm, context)?;
                    return self.action_set_target(context, &target);
                }
            }
            _ => {
                let target = target.coerce_to_string(self.avm, context)?;
                return self.action_set_target(context, &target);
            }
        };

        let mut sf = self.activation.write(context.gc_context);
        let scope = sf.scope_cell();
        let clip_obj = sf
            .target_clip()
            .unwrap_or_else(|| sf.base_clip().root())
            .object()
            .coerce_to_object(self.avm, context);
        sf.set_scope(Scope::new_target_scope(scope, clip_obj, context.gc_context));
        Ok(FrameControl::Continue)
    }

    fn action_stack_swap(
        &mut self,
        _context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop();
        let b = self.avm.pop();
        self.avm.push(a);
        self.avm.push(b);
        Ok(FrameControl::Continue)
    }

    fn action_start_drag(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let target = self.avm.pop();
        let start_clip = self.avm.target_clip_or_root();
        let display_object = self
            .avm
            .resolve_target_display_object(context, start_clip, target)?;
        if let Some(display_object) = display_object {
            let lock_center = self.avm.pop();
            let constrain = self.avm.pop().as_bool(self.avm.current_swf_version());
            if constrain {
                let y2 = self.avm.pop();
                let x2 = self.avm.pop();
                let y1 = self.avm.pop();
                let x1 = self.avm.pop();
                start_drag(
                    display_object,
                    self.avm,
                    context,
                    &[lock_center, x1, y1, x2, y2],
                );
            } else {
                start_drag(display_object, self.avm, context, &[lock_center]);
            };
        } else {
            log::warn!("StartDrag: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_stop(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.avm.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.stop(context);
            } else {
                log::warn!("Stop: Target is not a MovieClip");
            }
        } else {
            log::warn!("Stop: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_stop_sounds(
        &mut self,
        context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        context.audio.stop_all_sounds();
        Ok(FrameControl::Continue)
    }

    fn action_store_register(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        register: u8,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // The value must remain on the stack.
        let val = self.avm.pop();
        self.avm.push(val.clone());
        self.avm.set_current_register(register, val, context);

        Ok(FrameControl::Continue)
    }

    fn action_string_add(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWFv4 string concatenation
        // TODO(Herschel): Result with non-string operands?
        let a = self.avm.pop();
        let mut b = self
            .avm
            .pop()
            .coerce_to_string(self.avm, context)?
            .to_string();
        b.push_str(&a.coerce_to_string(self.avm, context)?);
        self.avm.push(b);
        Ok(FrameControl::Continue)
    }

    fn action_string_equals(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strcmp
        let a = self.avm.pop();
        let b = self.avm.pop();
        let result =
            b.coerce_to_string(self.avm, context)? == a.coerce_to_string(self.avm, context)?;
        self.avm
            .push(Value::from_bool(result, self.avm.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_string_extract(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWFv4 substring
        // TODO(Herschel): Result with incorrect operands?
        let len = self.avm.pop().coerce_to_f64(self.avm, context)? as usize;
        let start = self.avm.pop().coerce_to_f64(self.avm, context)? as usize;
        let val = self.avm.pop();
        let s = val.coerce_to_string(self.avm, context)?;
        // This is specifically a non-UTF8 aware substring.
        // SWFv4 only used ANSI strings.
        let result = s
            .bytes()
            .skip(start)
            .take(len)
            .map(|c| c as char)
            .collect::<String>();
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_string_greater(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strcmp
        let a = self.avm.pop();
        let b = self.avm.pop();
        // This is specifically a non-UTF8 aware comparison.
        let result = b
            .coerce_to_string(self.avm, context)?
            .bytes()
            .gt(a.coerce_to_string(self.avm, context)?.bytes());
        self.avm
            .push(Value::from_bool(result, self.avm.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_string_length(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strlen
        // Only returns byte length.
        // TODO(Herschel): Result with non-string operands?
        let val = self.avm.pop();
        let len = val.coerce_to_string(self.avm, context)?.bytes().len() as f64;
        self.avm.push(len);
        Ok(FrameControl::Continue)
    }

    fn action_string_less(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strcmp
        let a = self.avm.pop();
        let b = self.avm.pop();
        // This is specifically a non-UTF8 aware comparison.
        let result = b
            .coerce_to_string(self.avm, context)?
            .bytes()
            .lt(a.coerce_to_string(self.avm, context)?.bytes());
        self.avm
            .push(Value::from_bool(result, self.avm.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_subtract(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_f64(self.avm, context)?;
        let b = self.avm.pop().coerce_to_f64(self.avm, context)?;
        self.avm.push(b - a);
        Ok(FrameControl::Continue)
    }

    fn action_target_path(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel)
        let _clip = self.avm.pop().coerce_to_object(self.avm, context);
        self.avm.push(Value::Undefined);
        log::warn!("Unimplemented action: TargetPath");
        Ok(FrameControl::Continue)
    }

    fn toggle_quality(
        &mut self,
        _context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Noop for now? Could chang anti-aliasing on render backend.
        Ok(FrameControl::Continue)
    }

    fn action_to_integer(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.avm.pop().coerce_to_f64(self.avm, context)?;
        self.avm.push(val.trunc());
        Ok(FrameControl::Continue)
    }

    fn action_to_number(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.avm.pop().coerce_to_f64(self.avm, context)?;
        self.avm.push(val);
        Ok(FrameControl::Continue)
    }

    fn action_to_string(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.avm.pop();
        let string = val.coerce_to_string(self.avm, context)?;
        self.avm.push(string);
        Ok(FrameControl::Continue)
    }

    fn action_trace(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.avm.pop();
        // trace always prints "undefined" even though SWF6 and below normally
        // coerce undefined to "".
        let out = if val == Value::Undefined {
            Cow::Borrowed("undefined")
        } else {
            val.coerce_to_string(self.avm, context)?
        };
        log::info!(target: "avm_trace", "{}", out);
        Ok(FrameControl::Continue)
    }

    fn action_type_of(
        &mut self,
        _context: &mut UpdateContext,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let type_of = self.avm.pop().type_of();
        self.avm.push(type_of);
        Ok(FrameControl::Continue)
    }

    fn action_wait_for_frame(
        &mut self,
        _context: &mut UpdateContext,
        _frame: u16,
        num_actions_to_skip: u8,
        r: &mut Reader<'_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Always true for now.
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            skip_actions(r, num_actions_to_skip);
        }
        Ok(FrameControl::Continue)
    }

    fn action_wait_for_frame_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        num_actions_to_skip: u8,
        r: &mut Reader<'_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Always true for now.
        let _frame_num = self.avm.pop().coerce_to_f64(self.avm, context)? as u16;
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            skip_actions(r, num_actions_to_skip);
        }
        Ok(FrameControl::Continue)
    }

    #[allow(unused_variables)]
    fn action_throw(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.avm.pop();
        avm_debug!(
            "Thrown exception: {}",
            value
                .coerce_to_string(self.avm, context)
                .unwrap_or_else(|_| Cow::Borrowed("undefined"))
        );
        Err(Error::ThrownValue(value))
    }

    fn action_with(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        actions: &[u8],
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let object = self.avm.pop().coerce_to_object(self.avm, context);
        let block = self.activation.read().data().to_subslice(actions).unwrap();
        let with_scope = Scope::new_with_scope(
            self.activation.read().scope_cell(),
            object,
            context.gc_context,
        );
        let new_activation = GcCell::allocate(
            context.gc_context,
            self.activation.read().to_rescope(block, with_scope),
        );
        self.avm.stack_frames.push(new_activation);
        self.avm.run_activation(context, new_activation)?;
        Ok(FrameControl::Continue)
    }
}
