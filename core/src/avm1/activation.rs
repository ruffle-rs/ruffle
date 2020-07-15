use crate::avm1::error::Error;
use crate::avm1::function::{Avm1Function, ExecutionReason, FunctionObject};
use crate::avm1::object::{value_object, Object, TObject};
use crate::avm1::property::Attribute;
use crate::avm1::scope::Scope;
use crate::avm1::value::f64_to_wrapping_u32;
use crate::avm1::{
    fscommand, globals, scope, skip_actions, start_drag, Avm1, AvmString, ScriptObject, Value,
};
use crate::backend::navigator::{NavigationMethod, RequestOptions};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, MovieClip, TDisplayObject};
use crate::tag_utils::SwfSlice;
use enumset::EnumSet;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use rand::Rng;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::fmt;
use swf::avm1::read::Reader;
use swf::avm1::types::{Action, CatchVar, Function, TryBlock};
use url::form_urlencoded;

macro_rules! avm_debug {
    ($($arg:tt)*) => (
        #[cfg(feature = "avm_debug")]
        log::debug!($($arg)*)
    )
}

/// Represents a particular register set.
///
/// This type exists primarily because SmallVec isn't garbage-collectable.
#[derive(Clone)]
pub struct RegisterSet<'gc>(SmallVec<[Value<'gc>; 8]>);

unsafe impl<'gc> gc_arena::Collect for RegisterSet<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for register in &self.0 {
            register.trace(cc);
        }
    }
}

impl<'gc> RegisterSet<'gc> {
    /// Create a new register set with a given number of specified registers.
    ///
    /// The given registers will be set to `undefined`.
    pub fn new(num: u8) -> Self {
        Self(smallvec![Value::Undefined; num as usize])
    }

    /// Return a reference to a given register, if it exists.
    pub fn get(&self, num: u8) -> Option<&Value<'gc>> {
        self.0.get(num as usize)
    }

    /// Return a mutable reference to a given register, if it exists.
    pub fn get_mut(&mut self, num: u8) -> Option<&mut Value<'gc>> {
        self.0.get_mut(num as usize)
    }

    pub fn len(&self) -> u8 {
        self.0.len() as u8
    }
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

#[derive(Debug, Clone)]
pub struct ActivationIdentifier<'a> {
    parent: Option<&'a ActivationIdentifier<'a>>,
    name: Cow<'static, str>,
    depth: u16,
    function_count: u16,
    special_count: u8,
}

impl fmt::Display for ActivationIdentifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "{} / ", parent)?;
        }

        f.write_str(&self.name)?;

        Ok(())
    }
}

impl<'a> ActivationIdentifier<'a> {
    pub fn root<S: Into<Cow<'static, str>>>(name: S) -> Self {
        Self {
            parent: None,
            name: name.into(),
            depth: 0,
            function_count: 0,
            special_count: 0,
        }
    }

    pub fn child<S: Into<Cow<'static, str>>>(&'a self, name: S) -> Self {
        Self {
            parent: Some(self),
            name: name.into(),
            depth: self.depth + 1,
            function_count: self.function_count,
            special_count: self.special_count,
        }
    }

    pub fn function<'gc, S: Into<Cow<'static, str>>>(
        &'a self,
        name: S,
        reason: ExecutionReason,
        max_recursion_depth: u16,
    ) -> Result<Self, Error<'gc>> {
        let (function_count, special_count) = match reason {
            ExecutionReason::FunctionCall => {
                if self.function_count >= max_recursion_depth - 1 {
                    return Err(Error::FunctionRecursionLimit(max_recursion_depth));
                }
                (self.function_count + 1, self.special_count)
            }
            ExecutionReason::Special => {
                if self.special_count == 65 {
                    return Err(Error::SpecialRecursionLimit);
                }
                (self.function_count, self.special_count + 1)
            }
        };
        Ok(Self {
            parent: Some(self),
            name: name.into(),
            depth: self.depth + 1,
            function_count,
            special_count,
        })
    }

    pub fn depth(&self) -> u16 {
        self.depth
    }
}

unsafe impl<'gc> gc_arena::Collect for ActivationIdentifier<'gc> {
    fn needs_trace() -> bool {
        false
    }

    #[inline]
    fn trace(&self, _cc: gc_arena::CollectionContext) {}
}

#[derive(Collect)]
#[collect(unsafe_drop)]
pub struct Activation<'a, 'gc: 'a> {
    pub avm: &'a mut Avm1<'gc>,

    /// Represents the SWF version of a given function.
    ///
    /// Certain AVM1 operations change behavior based on the version of the SWF
    /// file they were defined in. For example, case sensitivity changes based
    /// on the SWF version.
    swf_version: u8,

    /// All defined local variables in this stack frame.
    scope: GcCell<'gc, Scope<'gc>>,

    /// The currently in use constant pool.
    constant_pool: GcCell<'gc, Vec<String>>,

    /// The immutable value of `this`.
    this: Object<'gc>,

    /// The arguments this function was called by.
    pub arguments: Option<Object<'gc>>,

    /// Local registers, if any.
    ///
    /// None indicates a function executing out of the global register set.
    /// Some indicates the existence of local registers, even if none exist.
    /// i.e. None(Vec::new()) means no registers should exist at all.
    ///
    /// Registers are numbered from 1; r0 does not exist. Therefore this vec,
    /// while nominally starting from zero, actually starts from r1.
    ///
    /// Registers are stored in a `GcCell` so that rescopes (e.g. with) use the
    /// same register set.
    local_registers: Option<GcCell<'gc, RegisterSet<'gc>>>,

    /// The base clip of this stack frame.
    /// This will be the movieclip that contains the bytecode.
    base_clip: DisplayObject<'gc>,

    /// The current target display object of this stack frame.
    /// This can be changed with `tellTarget` (via `ActionSetTarget` and `ActionSetTarget2`).
    target_clip: Option<DisplayObject<'gc>>,

    /// An identifier to refer to this activation by, when debugging.
    /// This is often the name of a function (if known), or some static name to indicate where
    /// in the code it is (for example, a with{} block).
    pub id: ActivationIdentifier<'a>,
}

impl Drop for Activation<'_, '_> {
    fn drop(&mut self) {
        avm_debug!("END {}", self.id);
    }
}

impl<'a, 'gc: 'a> Activation<'a, 'gc> {
    #[allow(clippy::too_many_arguments)]
    pub fn from_action(
        avm: &'a mut Avm1<'gc>,
        id: ActivationIdentifier<'a>,
        swf_version: u8,
        scope: GcCell<'gc, Scope<'gc>>,
        constant_pool: GcCell<'gc, Vec<String>>,
        base_clip: DisplayObject<'gc>,
        this: Object<'gc>,
        arguments: Option<Object<'gc>>,
    ) -> Self {
        avm_debug!("START {}", id);
        Self {
            avm,
            id,
            swf_version,
            scope,
            constant_pool,
            base_clip,
            target_clip: Some(base_clip),
            this,
            arguments,
            local_registers: None,
        }
    }

    /// Create a new activation to run a block of code with a given scope.
    pub fn with_new_scope<'b, S: Into<Cow<'static, str>>>(
        &'b mut self,
        name: S,
        scope: GcCell<'gc, Scope<'gc>>,
    ) -> Activation<'b, 'gc> {
        let id = self.id.child(name);
        avm_debug!("START {}", id);
        Activation {
            avm: self.avm,
            id,
            swf_version: self.swf_version,
            scope,
            constant_pool: self.constant_pool,
            base_clip: self.base_clip,
            target_clip: self.target_clip,
            this: self.this,
            arguments: self.arguments,
            local_registers: self.local_registers,
        }
    }

    /// Construct an empty stack frame with no code.
    ///
    /// This is used by tests and by callback methods (`onEnterFrame`) to create a base
    /// activation frame with access to the global context.
    pub fn from_nothing(
        avm: &'a mut Avm1<'gc>,
        id: ActivationIdentifier<'a>,
        swf_version: u8,
        globals: Object<'gc>,
        mc: MutationContext<'gc, '_>,
        base_clip: DisplayObject<'gc>,
    ) -> Self {
        let global_scope = GcCell::allocate(mc, Scope::from_global_object(globals));
        let child_scope = GcCell::allocate(mc, Scope::new_local_scope(global_scope, mc));
        let empty_constant_pool = GcCell::allocate(mc, Vec::new());
        avm_debug!("START {}", id);

        Self {
            avm,
            id,
            swf_version,
            scope: child_scope,
            constant_pool: empty_constant_pool,
            base_clip,
            target_clip: Some(base_clip),
            this: globals,
            arguments: None,
            local_registers: None,
        }
    }

    /// Add a stack frame that executes code in timeline scope
    pub fn run_child_frame_for_action<S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        code: SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnType<'gc>, Error<'gc>> {
        let mut parent_activation = Activation::from_nothing(
            self.avm,
            self.id.child("[Actions Parent]"),
            swf_version,
            self.avm.globals,
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
        child_activation.run_actions(context, code)
    }

    /// Add a stack frame that executes code in initializer scope.
    pub fn run_with_child_frame_for_display_object<'c, F, R, S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        action_context: &mut UpdateContext<'c, 'gc, '_>,
        function: F,
    ) -> R
    where
        for<'b> F: FnOnce(&mut Activation<'b, 'gc>, &mut UpdateContext<'c, 'gc, '_>) -> R,
    {
        let clip_obj = match active_clip.object() {
            Value::Object(o) => o,
            _ => panic!("No script object for display object"),
        };
        let global_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::from_global_object(self.avm.globals),
        );
        let child_scope = GcCell::allocate(
            action_context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        let constant_pool = self.avm.constant_pool;
        let mut activation = Activation::from_action(
            self.avm,
            self.id.child(name),
            swf_version,
            child_scope,
            constant_pool,
            active_clip,
            clip_obj,
            None,
        );
        function(&mut activation, action_context)
    }

    pub fn run_actions(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        code: SwfSlice,
    ) -> Result<ReturnType<'gc>, Error<'gc>> {
        let mut read = Reader::new(code.as_ref(), self.swf_version());

        loop {
            let result = self.do_action(&code, context, &mut read);
            match result {
                Ok(FrameControl::Return(return_type)) => break Ok(return_type),
                Ok(FrameControl::Continue) => {}
                Err(e) => break Err(e),
            }
        }
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
            avm_debug!("({}) Action: {:?}", self.id.depth(), action);

            match action {
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
                } => self.action_define_function(
                    context,
                    &name,
                    &params[..],
                    data.to_subslice(actions).unwrap(),
                ),
                Action::DefineFunction2(func) => {
                    self.action_define_function_2(context, &func, &data)
                }
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
                Action::With { actions } => {
                    self.action_with(context, data.to_subslice(actions).unwrap())
                }
                Action::Throw => self.action_throw(context),
                Action::Try(try_block) => self.action_try(context, &try_block, &data),
                _ => self.unknown_op(context, action),
            }
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
            let mut s = b.coerce_to_string(self, context)?.to_string();
            s.push_str(&a);
            self.avm.push(AvmString::new(context.gc_context, s));
        } else if let Value::String(b) = b {
            let mut b = b.to_string();
            b.push_str(&a.coerce_to_string(self, context)?);
            self.avm.push(AvmString::new(context.gc_context, b));
        } else {
            let result = b.coerce_to_f64(self, context)? + a.coerce_to_f64(self, context)?;
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
        let version = self.current_swf_version();
        let result = b.as_bool(version) && a.as_bool(version);
        self.avm
            .push(Value::from_bool(result, self.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_ascii_to_char(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Results on incorrect operands?
        let val = (self.avm.pop().coerce_to_f64(self, context)? as u8) as char;
        self.avm
            .push(AvmString::new(context.gc_context, val.to_string()));
        Ok(FrameControl::Continue)
    }

    fn action_char_to_ascii(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Results on incorrect operands?
        let val = self.avm.pop();
        let string = val.coerce_to_string(self, context)?;
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

        Ok(FrameControl::Continue)
    }

    fn action_bit_and(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_u32(self, context)?;
        let b = self.avm.pop().coerce_to_u32(self, context)?;
        let result = a & b;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_lshift(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_i32(self, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.avm.pop().coerce_to_i32(self, context)?;
        let result = b << a;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_or(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_u32(self, context)?;
        let b = self.avm.pop().coerce_to_u32(self, context)?;
        let result = a | b;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_rshift(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_i32(self, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.avm.pop().coerce_to_i32(self, context)?;
        let result = b >> a;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_urshift(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_u32(self, context)? & 0b11111; // Only 5 bits used for shift count
        let b = self.avm.pop().coerce_to_u32(self, context)?;
        let result = b >> a;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_xor(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_u32(self, context)?;
        let b = self.avm.pop().coerce_to_u32(self, context)?;
        let result = b ^ a;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_call(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Runs any actions on the given frame.
        let arg = self.avm.pop();
        let target = self.target_clip_or_root();

        // The parameter can be a frame # or a path to a movie clip with a frame number.
        let mut call_frame = None;
        if let Value::Number(frame) = arg {
            // Frame # on the current clip.
            if let Some(target) = target.as_movie_clip() {
                call_frame = Some((target, f64_to_wrapping_u32(frame)));
            }
        } else {
            // An optional path to a movieclip and a frame #/label, such as "/clip:framelabel".
            let frame_path = arg.coerce_to_string(self, context)?;
            if let Some((clip, frame)) = self.resolve_variable_path(context, target, &frame_path)? {
                if let Some(clip) = clip.as_display_object().and_then(|o| o.as_movie_clip()) {
                    if let Ok(frame) = frame.parse().map(f64_to_wrapping_u32) {
                        // First try to parse as a frame number.
                        call_frame = Some((clip, frame));
                    } else if let Some(frame) = clip.frame_label_to_number(&frame) {
                        // Otherwise, it's a frame label.
                        call_frame = Some((clip, frame.into()));
                    }
                }
            }
        };

        if let Some((clip, frame)) = call_frame {
            if frame <= u32::from(std::u16::MAX) {
                for action in clip.actions_on_frame(context, frame as u16) {
                    let _ = self.run_child_frame_for_action(
                        "[Frame Call]",
                        clip.into(),
                        self.current_swf_version(),
                        action,
                        context,
                    )?;
                }
            }
        } else {
            log::warn!("Call: Invalid call");
        }

        Ok(FrameControl::Continue)
    }

    fn action_call_function(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let fn_name_value = self.avm.pop();
        let fn_name = fn_name_value.coerce_to_string(self, context)?;
        let mut args = Vec::new();
        let num_args = self.avm.pop().coerce_to_f64(self, context)? as i64; // TODO(Herschel): max arg count?
        for _ in 0..num_args {
            args.push(self.avm.pop());
        }

        let target_fn = self.get_variable(context, &fn_name)?;

        let this = self
            .target_clip_or_root()
            .object()
            .coerce_to_object(self, context);
        let result = target_fn.call(&fn_name, self, context, this, None, &args)?;
        self.avm.push(result);

        Ok(FrameControl::Continue)
    }

    fn action_call_method(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let method_name = self.avm.pop();
        let object_val = self.avm.pop();
        let object = value_object::ValueObject::boxed(self, context, object_val);
        let num_args = self.avm.pop().coerce_to_f64(self, context)? as i64; // TODO(Herschel): max arg count?
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.avm.pop());
        }

        match method_name {
            Value::Undefined | Value::Null => {
                let this = self
                    .target_clip_or_root()
                    .object()
                    .coerce_to_object(self, context);
                let result = object.call("[Anonymous]", self, context, this, None, &args)?;
                self.avm.push(result);
            }
            Value::String(name) => {
                if name.is_empty() {
                    let result = object.call("[Anonymous]", self, context, object, None, &args)?;
                    self.avm.push(result);
                } else {
                    let result = object.call_method(&name, &args, self, context)?;
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
        let obj = self.avm.pop();
        let constr = self.avm.pop().coerce_to_object(self, context);

        let is_instance_of = if let Value::Object(obj) = obj {
            let prototype = constr
                .get("prototype", self, context)?
                .coerce_to_object(self, context);
            obj.is_instance_of(self, context, constr, prototype)?
        } else {
            false
        };

        let ret = if is_instance_of { obj } else { Value::Null };
        self.avm.push(ret);

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
        self.set_constant_pool(self.avm.constant_pool);

        Ok(FrameControl::Continue)
    }

    fn action_decrement(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_f64(self, context)?;
        self.avm.push(a - 1.0);
        Ok(FrameControl::Continue)
    }

    fn action_define_function(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
        params: &[&str],
        actions: SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let swf_version = self.swf_version();
        let scope = Scope::new_closure_scope(self.scope_cell(), context.gc_context);
        let constant_pool = self.constant_pool();
        let func = Avm1Function::from_df1(
            swf_version,
            actions,
            name,
            params,
            scope,
            constant_pool,
            self.target_clip_or_root(),
        );
        let prototype =
            ScriptObject::object(context.gc_context, Some(self.avm.prototypes.object)).into();
        let func_obj = FunctionObject::function(
            context.gc_context,
            Gc::allocate(context.gc_context, func),
            Some(self.avm.prototypes.function),
            Some(prototype),
        );
        if name == "" {
            self.avm.push(func_obj);
        } else {
            self.define(name, func_obj, context.gc_context);
        }

        Ok(FrameControl::Continue)
    }

    fn action_define_function_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        action_func: &Function,
        parent_data: &SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let swf_version = self.swf_version();
        let func_data = parent_data.to_subslice(action_func.actions).unwrap();
        let scope = Scope::new_closure_scope(self.scope_cell(), context.gc_context);
        let constant_pool = self.constant_pool();
        let func = Avm1Function::from_df2(
            swf_version,
            func_data,
            action_func,
            scope,
            constant_pool,
            self.base_clip(),
        );
        let prototype =
            ScriptObject::object(context.gc_context, Some(self.avm.prototypes.object)).into();
        let func_obj = FunctionObject::function(
            context.gc_context,
            Gc::allocate(context.gc_context, func),
            Some(self.avm.prototypes.function),
            Some(prototype),
        );
        if action_func.name == "" {
            self.avm.push(func_obj);
        } else {
            self.define(action_func.name, func_obj, context.gc_context);
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
        let name = name_val.coerce_to_string(self, context)?;
        let scope = self.scope_cell();
        scope
            .write(context.gc_context)
            .locals()
            .set(&name, value, self, context)?;
        Ok(FrameControl::Continue)
    }

    fn action_define_local_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // If the property does not exist on the local object's prototype chain, it is created on the local object.
        // Otherwise, the property is unchanged.
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self, context)?;
        let scope = self.scope_cell();
        if !scope.read().locals().has_property(self, context, &name) {
            scope
                .write(context.gc_context)
                .locals()
                .set(&name, Value::Undefined, self, context)?;
        }
        Ok(FrameControl::Continue)
    }

    fn action_delete(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self, context)?;
        let object = self.avm.pop();

        if let Value::Object(object) = object {
            let success = object.delete(self, context.gc_context, &name);
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
        let name = name_val.coerce_to_string(self, context)?;

        //Fun fact: This isn't in the Adobe SWF19 spec, but this opcode returns
        //a boolean based on if the delete actually deleted something.
        let did_exist = self.is_defined(context, &name);

        self.scope_cell()
            .read()
            .delete(self, context, &name, context.gc_context);
        self.avm.push(did_exist);

        Ok(FrameControl::Continue)
    }

    fn action_divide(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 divide
        let a = self.avm.pop().coerce_to_f64(self, context)?;
        let b = self.avm.pop().coerce_to_f64(self, context)?;

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
        let name = name_value.coerce_to_string(self, context)?;
        self.avm.push(Value::Null); // Sentinel that indicates end of enumeration
        let object = self.resolve(&name, context)?;

        match object {
            Value::Object(ob) => {
                for k in ob.get_keys(self).into_iter().rev() {
                    self.avm.push(AvmString::new(context.gc_context, k));
                }
            }
            _ => log::error!("Cannot enumerate properties of {}", name),
        };

        Ok(FrameControl::Continue)
    }

    fn action_enumerate_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.avm.pop();

        self.avm.push(Value::Null); // Sentinel that indicates end of enumeration

        if let Value::Object(object) = value {
            for k in object.get_keys(self).into_iter().rev() {
                self.avm.push(AvmString::new(context.gc_context, k));
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
            .push(Value::from_bool(result, self.current_swf_version()));
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
        let result = b.abstract_eq(a, self, context, false)?;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_extends(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let superclass = self.avm.pop().coerce_to_object(self, context);
        let subclass = self.avm.pop().coerce_to_object(self, context);

        //TODO: What happens if we try to extend an object which has no `prototype`?
        //e.g. `class Whatever extends Object.prototype` or `class Whatever extends 5`
        let super_proto = superclass
            .get("prototype", self, context)?
            .coerce_to_object(self, context);

        let mut sub_prototype: Object<'gc> =
            ScriptObject::object(context.gc_context, Some(super_proto)).into();

        sub_prototype.set("constructor", superclass.into(), self, context)?;
        sub_prototype.set_attributes(
            context.gc_context,
            Some("constructor"),
            Attribute::DontEnum.into(),
            EnumSet::empty(),
        );

        sub_prototype.set("__constructor__", superclass.into(), self, context)?;
        sub_prototype.set_attributes(
            context.gc_context,
            Some("__constructor__"),
            Attribute::DontEnum.into(),
            EnumSet::empty(),
        );

        subclass.set("prototype", sub_prototype.into(), self, context)?;

        Ok(FrameControl::Continue)
    }

    fn action_get_member(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_val = self.avm.pop();
        let name = name_val.coerce_to_string(self, context)?;
        let object_val = self.avm.pop();
        let object = value_object::ValueObject::boxed(self, context, object_val);

        let result = object.get(&name, self, context)?;
        self.avm.push(result);

        Ok(FrameControl::Continue)
    }

    fn action_get_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let prop_index = self.avm.pop().into_number_v1() as usize;
        let path = self.avm.pop();
        let ret = if let Some(target) = self.target_clip() {
            if let Some(clip) = self.resolve_target_display_object(context, target, path)? {
                let display_properties = self.avm.display_properties;
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
        let path = var_path.coerce_to_string(self, context)?;

        let value = self.get_variable(context, &path)?;
        self.avm.push(value);

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
                    let level = self.resolve_level(level_id, context);

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
            fscommand::handle(fscommand, self, context)?;
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
        let url = url_val.coerce_to_string(self, context)?;

        if let Some(fscommand) = fscommand::parse(&url) {
            fscommand::handle(fscommand, self, context)?;
            return Ok(FrameControl::Continue);
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
                    Cow::Borrowed(&url),
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
                let (url, opts) = self.locals_into_request_options(
                    context,
                    Cow::Borrowed(&url),
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
                Some(method) => Some((method, self.locals_into_form_values(context))),
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
        if let Some(clip) = self.target_clip() {
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
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                let frame = self.avm.pop();
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
        Ok(FrameControl::Continue)
    }

    fn action_goto_label(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        label: &str,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
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
        if val.as_bool(self.current_swf_version()) {
            reader.seek(jump_offset.into());
        }
        Ok(FrameControl::Continue)
    }

    fn action_increment(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_f64(self, context)?;
        self.avm.push(a + 1.0);
        Ok(FrameControl::Continue)
    }

    fn action_init_array(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let num_elements = self.avm.pop().coerce_to_f64(self, context)? as i64;
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
        let num_props = self.avm.pop().coerce_to_f64(self, context)? as i64;
        let object = ScriptObject::object(context.gc_context, Some(self.avm.prototypes.object));
        for _ in 0..num_props {
            let value = self.avm.pop();
            let name_val = self.avm.pop();
            let name = name_val.coerce_to_string(self, context)?;
            object.set(&name, value, self, context)?;
        }

        self.avm.push(Value::Object(object.into()));

        Ok(FrameControl::Continue)
    }

    fn action_implements_op(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let constr = self.avm.pop().coerce_to_object(self, context);
        let count = self.avm.pop().coerce_to_f64(self, context)? as i64; //TODO: Is this coercion actually performed by Flash?
        let mut interfaces = vec![];

        //TODO: If one of the interfaces is not an object, do we leave the
        //whole stack dirty, or...?
        for _ in 0..count {
            interfaces.push(self.avm.pop().coerce_to_object(self, context));
        }

        let mut prototype = constr
            .get("prototype", self, context)?
            .coerce_to_object(self, context);

        prototype.set_interfaces(context.gc_context, interfaces);

        Ok(FrameControl::Continue)
    }

    fn action_instance_of(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let constr = self.avm.pop().coerce_to_object(self, context);
        let obj = self.avm.pop();

        let is_instance_of = if let Value::Object(obj) = obj {
            let prototype = constr
                .get("prototype", self, context)?
                .coerce_to_object(self, context);
            obj.is_instance_of(self, context, constr, prototype)?
        } else {
            false
        };

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
            .push(Value::from_bool(result, self.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_less_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // ECMA-262 s. 11.8.1
        let a = self.avm.pop();
        let b = self.avm.pop();

        let result = b.abstract_lt(a, self, context)?;

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

        let result = a.abstract_lt(b, self, context)?;

        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_mb_ascii_to_char(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Results on incorrect operands?
        use std::convert::TryFrom;
        let result = char::try_from(self.avm.pop().coerce_to_f64(self, context)? as u32);
        match result {
            Ok(val) => self
                .avm
                .push(AvmString::new(context.gc_context, val.to_string())),
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
        let s = val.coerce_to_string(self, context)?;
        let result = s.chars().next().unwrap_or('\0') as u32;
        self.avm.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_mb_string_extract(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Result with incorrect operands?
        let len = self.avm.pop().coerce_to_f64(self, context)? as usize;
        let start = self.avm.pop().coerce_to_f64(self, context)? as usize;
        let val = self.avm.pop();
        let s = val.coerce_to_string(self, context)?;
        let result = s[len..len + start].to_string(); // TODO(Herschel): Flash uses UTF-16 internally.
        self.avm.push(AvmString::new(context.gc_context, result));
        Ok(FrameControl::Continue)
    }

    fn action_mb_string_length(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Result with non-string operands?
        let val = self.avm.pop();
        let len = val.coerce_to_string(self, context)?.len();
        self.avm.push(len as f64);
        Ok(FrameControl::Continue)
    }

    fn action_multiply(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_f64(self, context)?;
        let b = self.avm.pop().coerce_to_f64(self, context)?;
        self.avm.push(a * b);
        Ok(FrameControl::Continue)
    }

    fn action_modulo(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO: Wrong operands?
        let a = self.avm.pop().coerce_to_f64(self, context)?;
        let b = self.avm.pop().coerce_to_f64(self, context)?;
        self.avm.push(b % a);
        Ok(FrameControl::Continue)
    }

    fn action_not(
        &mut self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let version = self.current_swf_version();
        let val = !self.avm.pop().as_bool(version);
        self.avm.push(Value::from_bool(val, version));
        Ok(FrameControl::Continue)
    }

    fn action_next_frame(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
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
        let num_args = self.avm.pop().coerce_to_f64(self, context)? as i64;
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.avm.pop());
        }

        let object = value_object::ValueObject::boxed(self, context, object_val);
        let constructor =
            object.get(&method_name.coerce_to_string(self, context)?, self, context)?;
        if let Value::Object(constructor) = constructor {
            let prototype = constructor
                .get("prototype", self, context)?
                .coerce_to_object(self, context);

            let mut this = prototype.new(self, context, prototype, &args)?;
            this.set("__constructor__", constructor.into(), self, context)?;
            this.set_attributes(
                context.gc_context,
                Some("__constructor__"),
                Attribute::DontEnum.into(),
                EnumSet::empty(),
            );
            if self.current_swf_version() < 7 {
                this.set("constructor", constructor.into(), self, context)?;
                this.set_attributes(
                    context.gc_context,
                    Some("constructor"),
                    Attribute::DontEnum.into(),
                    EnumSet::empty(),
                );
            }

            //TODO: What happens if you `ActionNewMethod` without a method name?
            constructor.call("[ctor]", self, context, this, None, &args)?;

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
        let fn_name = fn_name_val.coerce_to_string(self, context)?;
        let num_args = self.avm.pop().coerce_to_f64(self, context)? as i64;
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.avm.pop());
        }

        let constructor = self
            .resolve(&fn_name, context)?
            .coerce_to_object(self, context);
        let prototype = constructor
            .get("prototype", self, context)?
            .coerce_to_object(self, context);

        let mut this = prototype.new(self, context, prototype, &args)?;
        this.set("__constructor__", constructor.into(), self, context)?;
        this.set_attributes(
            context.gc_context,
            Some("__constructor__"),
            Attribute::DontEnum.into(),
            EnumSet::empty(),
        );
        if self.current_swf_version() < 7 {
            this.set("constructor", constructor.into(), self, context)?;
            this.set_attributes(
                context.gc_context,
                Some("constructor"),
                Attribute::DontEnum.into(),
                EnumSet::empty(),
            );
        }

        constructor.call("[ctor]", self, context, this, None, &args)?;

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
        let version = self.current_swf_version();
        let result = b.as_bool(version) || a.as_bool(version);
        self.avm.push(Value::from_bool(result, version));
        Ok(FrameControl::Continue)
    }

    fn action_play(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
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
        if let Some(clip) = self.target_clip() {
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
                SwfValue::Str(v) => AvmString::new(context.gc_context, (*v).to_string()).into(),
                SwfValue::Register(v) => self.current_register(*v),
                SwfValue::ConstantPool(i) => {
                    if let Some(value) = self.constant_pool().read().get(*i as usize) {
                        AvmString::new(context.gc_context, value.to_string()).into()
                    } else {
                        log::warn!(
                            "ActionPush: Constant pool index {} out of range (len = {})",
                            i,
                            self.constant_pool().read().len()
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
        let start_clip = self.target_clip_or_root();
        let target_clip = self.resolve_target_display_object(context, start_clip, target)?;

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
        let name = name_val.coerce_to_string(self, context)?;

        let object = self.avm.pop().coerce_to_object(self, context);
        object.set(&name, value, self, context)?;

        Ok(FrameControl::Continue)
    }

    fn action_set_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.avm.pop();
        let prop_index = self.avm.pop().coerce_to_u32(self, context)? as usize;
        let path = self.avm.pop();
        if let Some(target) = self.target_clip() {
            if let Some(clip) = self.resolve_target_display_object(context, target, path)? {
                let display_properties = self.avm.display_properties;
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
        Ok(FrameControl::Continue)
    }

    fn action_set_variable(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Flash 4-style variable
        let value = self.avm.pop();
        let var_path_val = self.avm.pop();
        let var_path = var_path_val.coerce_to_string(self, context)?;
        self.set_variable(context, &var_path, value)?;
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

        self.set_target_clip(new_target_clip);

        let scope = self.scope_cell();
        let clip_obj = self
            .target_clip()
            .unwrap_or_else(|| self.base_clip().root())
            .object()
            .coerce_to_object(self, context);

        self.set_scope(Scope::new_target_scope(scope, clip_obj, context.gc_context));
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
                let base_clip = self.base_clip();
                self.set_target_clip(Some(base_clip));
            }
            Value::Object(o) => {
                if let Some(clip) = o.as_display_object() {
                    // Movieclips can be targetted directly
                    self.set_target_clip(Some(clip));
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

        let scope = self.scope_cell();
        let clip_obj = self
            .target_clip()
            .unwrap_or_else(|| self.base_clip().root())
            .object()
            .coerce_to_object(self, context);
        self.set_scope(Scope::new_target_scope(scope, clip_obj, context.gc_context));
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
        let start_clip = self.target_clip_or_root();
        let display_object = self.resolve_target_display_object(context, start_clip, target)?;
        if let Some(display_object) = display_object {
            let lock_center = self.avm.pop();
            let constrain = self.avm.pop().as_bool(self.current_swf_version());
            if constrain {
                let y2 = self.avm.pop();
                let x2 = self.avm.pop();
                let y1 = self.avm.pop();
                let x1 = self.avm.pop();
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
        Ok(FrameControl::Continue)
    }

    fn action_stop(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
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
        self.set_current_register(register, val, context);

        Ok(FrameControl::Continue)
    }

    fn action_string_add(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWFv4 string concatenation
        // TODO(Herschel): Result with non-string operands?
        let a = self.avm.pop();
        let mut b = self.avm.pop().coerce_to_string(self, context)?.to_string();
        b.push_str(&a.coerce_to_string(self, context)?);
        self.avm.push(AvmString::new(context.gc_context, b));
        Ok(FrameControl::Continue)
    }

    fn action_string_equals(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strcmp
        let a = self.avm.pop();
        let b = self.avm.pop();
        let result = b.coerce_to_string(self, context)? == a.coerce_to_string(self, context)?;
        self.avm
            .push(Value::from_bool(result, self.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_string_extract(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWFv4 substring
        // TODO(Herschel): Result with incorrect operands?
        let len = self.avm.pop().coerce_to_f64(self, context)? as usize;
        let start = self.avm.pop().coerce_to_f64(self, context)? as usize;
        let val = self.avm.pop();
        let s = val.coerce_to_string(self, context)?;
        // This is specifically a non-UTF8 aware substring.
        // SWFv4 only used ANSI strings.
        let result = s
            .bytes()
            .skip(start)
            .take(len)
            .map(|c| c as char)
            .collect::<String>();
        self.avm.push(AvmString::new(context.gc_context, result));
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
            .coerce_to_string(self, context)?
            .bytes()
            .gt(a.coerce_to_string(self, context)?.bytes());
        self.avm
            .push(Value::from_bool(result, self.current_swf_version()));
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
        let len = val.coerce_to_string(self, context)?.bytes().len() as f64;
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
            .coerce_to_string(self, context)?
            .bytes()
            .lt(a.coerce_to_string(self, context)?.bytes());
        self.avm
            .push(Value::from_bool(result, self.current_swf_version()));
        Ok(FrameControl::Continue)
    }

    fn action_subtract(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.avm.pop().coerce_to_f64(self, context)?;
        let b = self.avm.pop().coerce_to_f64(self, context)?;
        self.avm.push(b - a);
        Ok(FrameControl::Continue)
    }

    fn action_target_path(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel)
        let _clip = self.avm.pop().coerce_to_object(self, context);
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
        let val = self.avm.pop().coerce_to_f64(self, context)?;
        self.avm.push(val.trunc());
        Ok(FrameControl::Continue)
    }

    fn action_to_number(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.avm.pop().coerce_to_f64(self, context)?;
        self.avm.push(val);
        Ok(FrameControl::Continue)
    }

    fn action_to_string(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.avm.pop();
        let string = val.coerce_to_string(self, context)?;
        self.avm
            .push(AvmString::new(context.gc_context, string.to_string()));
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
            "undefined".into()
        } else {
            val.coerce_to_string(self, context)?
        };
        log::info!(target: "avm_trace", "{}", out);
        Ok(FrameControl::Continue)
    }

    fn action_type_of(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let type_of = self.avm.pop().type_of();
        self.avm
            .push(AvmString::new(context.gc_context, type_of.to_string()));
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
        let _frame_num = self.avm.pop().coerce_to_f64(self, context)? as u16;
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
                .coerce_to_string(self, context)
                .unwrap_or_else(|_| Cow::Borrowed("undefined"))
        );
        Err(Error::ThrownValue(value))
    }

    fn action_with(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        code: SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.avm.pop();
        match value {
            // Undefined/null with is ignored.
            Value::Undefined | Value::Null => {
                // Mimic Flash's error output.
                log::info!(target: "avm_trace", "Error: A 'with' action failed because the specified object did not exist.\n");
                Ok(FrameControl::Continue)
            }

            value => {
                // Note that primitives get boxed at this point.
                let object = value.coerce_to_object(self, context);
                let with_scope =
                    Scope::new_with_scope(self.scope_cell(), object, context.gc_context);
                let mut new_activation = self.with_new_scope("[With]", with_scope);
                if let ReturnType::Explicit(value) = new_activation.run_actions(context, code)? {
                    Ok(FrameControl::Return(ReturnType::Explicit(value)))
                } else {
                    Ok(FrameControl::Continue)
                }
            }
        }
    }

    fn action_try(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        try_block: &TryBlock,
        parent_data: &SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let mut result = self.run_actions(
            context,
            parent_data.to_subslice(try_block.try_actions).unwrap(),
        );

        if let Some((catch_vars, actions)) = &try_block.catch {
            if let Err(Error::ThrownValue(value)) = &result {
                let mut activation = Activation::from_action(
                    self.avm,
                    self.id.child("[Catch]"),
                    self.swf_version,
                    self.scope,
                    self.constant_pool,
                    self.base_clip,
                    self.this,
                    self.arguments,
                );

                match catch_vars {
                    CatchVar::Var(name) => {
                        activation.set_variable(context, name, value.to_owned())?
                    }
                    CatchVar::Register(id) => {
                        activation.set_current_register(*id, value.to_owned(), context)
                    }
                }

                result = activation.run_actions(context, parent_data.to_subslice(actions).unwrap());
            }
        }

        if let Some(actions) = try_block.finally {
            if let ReturnType::Explicit(value) =
                self.run_actions(context, parent_data.to_subslice(actions).unwrap())?
            {
                return Ok(FrameControl::Return(ReturnType::Explicit(value)));
            }
        }

        match result? {
            ReturnType::Implicit => Ok(FrameControl::Continue),
            ReturnType::Explicit(value) => Ok(FrameControl::Return(ReturnType::Explicit(value))),
        }
    }

    /// Retrieve a given register value.
    ///
    /// If a given register does not exist, this function yields
    /// Value::Undefined, which is also a valid register value.
    pub fn current_register(&self, id: u8) -> Value<'gc> {
        if self.has_local_register(id) {
            self.local_register(id).unwrap_or(Value::Undefined)
        } else {
            self.avm
                .registers
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
        if self.has_local_register(id) {
            self.set_local_register(id, value, context.gc_context);
        } else if let Some(v) = self.avm.registers.get_mut(id as usize) {
            *v = value;
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
        let scope = self.scope_cell();
        let locals = scope.read().locals_cell();
        let keys = locals.get_keys(self);

        for k in keys {
            let v = locals.get(&k, self, context);

            //TODO: What happens if an error occurs inside a virtual property?
            form_values.insert(
                k,
                v.ok()
                    .unwrap_or_else(|| Value::Undefined)
                    .coerce_to_string(self, context)
                    .unwrap_or_else(|_| "undefined".into())
                    .to_string(),
            );
        }

        form_values
    }

    /// Construct request options for a fetch operation that may send locals as
    /// form data in the request body or URL.
    pub fn locals_into_request_options<'b>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        url: Cow<'b, str>,
        method: Option<NavigationMethod>,
    ) -> (Cow<'b, str>, RequestOptions) {
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

    /// Resolves a path for text field variable binding.
    /// Returns the parent object that owns the variable, and the variable name.
    /// Returns `None` if the path does not yet point to a valid object.
    /// TODO: This can probably be merged with some of the above `resolve_target_path` methods.
    pub fn resolve_variable_path<'s>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        start: DisplayObject<'gc>,
        path: &'s str,
    ) -> Result<Option<(Object<'gc>, &'s str)>, Error<'gc>> {
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

            let mut current_scope = Some(self.scope_cell());
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
    ) -> Result<Value<'gc>, Error<'gc>> {
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

            let mut current_scope = Some(self.scope_cell());
            while let Some(scope) = current_scope {
                if let Some(object) =
                    self.resolve_target_path(context, start.root(), *scope.read().locals(), path)?
                {
                    if object.has_property(self, context, var_name) {
                        return Ok(object.get(var_name, self, context)?);
                    }
                }
                current_scope = scope.read().parent_cell();
            }

            return Ok(Value::Undefined);
        }

        // If it doesn't have a trailing variable, it can still be a slash path.
        // We can skip this step if we didn't find a slash above.
        if has_slash {
            let mut current_scope = Some(self.scope_cell());
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
        self.resolve(&path, context)
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

            let mut current_scope = Some(self.scope_cell());
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
        let this = self.this_cell();
        let scope = self.scope_cell();
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
            level.post_instantiation(self.avm, context, level, None, false);

            level
        }
    }

    /// The current target clip of the executing code.
    /// Actions that affect `root` after an invalid `tellTarget` will use this.
    ///
    /// The `root` is determined relative to the base clip that defined the
    pub fn target_clip_or_root(&self) -> DisplayObject<'gc> {
        self.target_clip()
            .unwrap_or_else(|| self.base_clip().root())
    }

    /// Obtain the value of `_root`.
    pub fn root_object(&self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Value<'gc> {
        self.base_clip().root().object()
    }

    /// Get the currently executing SWF version.
    pub fn current_swf_version(&self) -> u8 {
        self.swf_version()
    }

    /// Returns whether property keys should be case sensitive based on the current SWF version.
    pub fn is_case_sensitive(&self) -> bool {
        self.current_swf_version() > 6
    }

    /// Resolve a particular named local variable within this activation.
    ///
    /// Because scopes are object chains, the same rules for `Object::get`
    /// still apply here.
    pub fn resolve(
        &mut self,
        name: &str,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if name == "this" {
            return Ok(Value::Object(self.this_cell()));
        }

        if name == "arguments" && self.arguments.is_some() {
            return Ok(Value::Object(self.arguments.unwrap()));
        }

        self.scope_cell()
            .read()
            .resolve(name, self, context, self.this_cell())
    }

    /// Check if a particular property in the scope chain is defined.
    pub fn is_defined(&mut self, context: &mut UpdateContext<'_, 'gc, '_>, name: &str) -> bool {
        if name == "this" {
            return true;
        }

        if name == "arguments" && self.arguments.is_some() {
            return true;
        }

        self.scope_cell().read().is_defined(self, context, name)
    }

    /// Returns the SWF version of the action or function being executed.
    pub fn swf_version(&self) -> u8 {
        self.swf_version
    }

    /// Returns AVM local variable scope.
    pub fn scope(&self) -> Ref<Scope<'gc>> {
        self.scope.read()
    }

    /// Returns AVM local variable scope for mutation.
    #[allow(dead_code)]
    pub fn scope_mut(&mut self, mc: MutationContext<'gc, '_>) -> RefMut<Scope<'gc>> {
        self.scope.write(mc)
    }

    /// Returns AVM local variable scope for reference.
    pub fn scope_cell(&self) -> GcCell<'gc, Scope<'gc>> {
        self.scope
    }

    /// Completely replace the current scope with a new one.
    pub fn set_scope(&mut self, scope: GcCell<'gc, Scope<'gc>>) {
        self.scope = scope;
    }

    /// Gets the base clip of this stack frame.
    /// This is the movie clip that contains the executing bytecode.
    pub fn base_clip(&self) -> DisplayObject<'gc> {
        self.base_clip
    }

    /// Gets the current target clip of this stack frame.
    /// This is the movie clip to which `GotoFrame` and other actions apply.
    /// Changed via `ActionSetTarget`/`ActionSetTarget2`.
    pub fn target_clip(&self) -> Option<DisplayObject<'gc>> {
        self.target_clip
    }

    /// Changes the target clip.
    pub fn set_target_clip(&mut self, value: Option<DisplayObject<'gc>>) {
        self.target_clip = value;
    }

    /// Define a named local variable within this activation.
    pub fn define(&self, name: &str, value: impl Into<Value<'gc>>, mc: MutationContext<'gc, '_>) {
        self.scope().define(name, value, mc)
    }

    /// Returns value of `this` as a reference.
    pub fn this_cell(&self) -> Object<'gc> {
        self.this
    }

    /// Returns true if this activation has a given local register ID.
    pub fn has_local_register(&self, id: u8) -> bool {
        self.local_registers
            .map(|rs| id < rs.read().len())
            .unwrap_or(false)
    }

    pub fn allocate_local_registers(&mut self, num: u8, mc: MutationContext<'gc, '_>) {
        self.local_registers = match num {
            0 => None,
            num => Some(GcCell::allocate(mc, RegisterSet::new(num))),
        };
    }

    /// Retrieve a local register.
    pub fn local_register(&self, id: u8) -> Option<Value<'gc>> {
        if let Some(local_registers) = self.local_registers {
            local_registers.read().get(id).cloned()
        } else {
            None
        }
    }

    /// Set a local register.
    pub fn set_local_register(
        &mut self,
        id: u8,
        value: impl Into<Value<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) {
        if let Some(ref mut local_registers) = self.local_registers {
            if let Some(r) = local_registers.write(mc).get_mut(id) {
                *r = value.into();
            }
        }
    }

    pub fn constant_pool(&self) -> GcCell<'gc, Vec<String>> {
        self.constant_pool
    }

    pub fn set_constant_pool(&mut self, constant_pool: GcCell<'gc, Vec<String>>) {
        self.constant_pool = constant_pool;
    }
}
