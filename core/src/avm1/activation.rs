use crate::avm1::callable_value::CallableValue;
use crate::avm1::error::Error;
use crate::avm1::function::{Avm1Function, ExecutionReason, FunctionObject};
use crate::avm1::object::{Object, TObject};
use crate::avm1::property::Attribute;
use crate::avm1::runtime::skip_actions;
use crate::avm1::scope::Scope;
use crate::avm1::{fscommand, globals, scope, ArrayObject, ScriptObject, Value};
use crate::backend::navigator::{NavigationMethod, Request};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, MovieClip, TDisplayObject, TDisplayObjectContainer};
use crate::ecma_conversions::f64_to_wrapping_u32;
use crate::string::{AvmString, WStr, WString};
use crate::tag_utils::SwfSlice;
use crate::vminterface::Instantiator;
use crate::{avm_error, avm_warn};
use gc_arena::{Gc, GcCell, MutationContext};
use indexmap::IndexMap;
use instant::Instant;
use rand::Rng;
use ruffle_render::bounding_box::BoundingBox;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::{Ref, RefMut};
use std::fmt;
use swf::avm1::read::Reader;
use swf::avm1::types::*;
use swf::Twips;
use url::form_urlencoded;

macro_rules! avm_debug {
    ($avm: expr, $($arg:tt)*) => (
        if $avm.show_debug_output() {
            log::debug!($($arg)*)
        }
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

pub struct Activation<'a, 'gc: 'a, 'gc_context: 'a> {
    /// Represents the SWF version of a given function.
    ///
    /// Certain AVM1 operations change behavior based on the version of the SWF
    /// file they were defined in. For example, case sensitivity changes based
    /// on the SWF version.
    swf_version: u8,

    /// All defined local variables in this stack frame.
    scope: GcCell<'gc, Scope<'gc>>,

    /// The currently in use constant pool.
    constant_pool: GcCell<'gc, Vec<Value<'gc>>>,

    /// The immutable value of `this`.
    ///
    /// This differs from Flash Player, where `this` is mutable and seems
    /// to be part of the scope chain (e.g. a function with the `suppress_this` flag
    /// set can modify the `this` value of its closure).
    ///
    /// Fortunately, ActionScript syntax prevents mutating `this` altogether, so
    /// observing this behavior requires manually-crafted bytecode.
    ///
    /// TODO: implement correct semantics for mutable `this`.
    this: Value<'gc>,

    /// The function object being called.
    pub callee: Option<Object<'gc>>,

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
    /// This will be the MovieClip that contains the bytecode.
    base_clip: DisplayObject<'gc>,

    /// The current target display object of this stack frame.
    /// This can be changed with `tellTarget` (via `ActionSetTarget` and `ActionSetTarget2`).
    target_clip: Option<DisplayObject<'gc>>,

    /// Whether the base clip was removed when we started this frame.
    base_clip_unloaded: bool,

    pub context: UpdateContext<'a, 'gc, 'gc_context>,

    /// An identifier to refer to this activation by, when debugging.
    /// This is often the name of a function (if known), or some static name to indicate where
    /// in the code it is (for example, a with{} block).
    pub id: ActivationIdentifier<'a>,
}

impl Drop for Activation<'_, '_, '_> {
    fn drop(&mut self) {
        avm_debug!(self.context.avm1, "END {}", self.id);
    }
}

impl<'a, 'gc, 'gc_context> Activation<'a, 'gc, 'gc_context> {
    #[allow(clippy::too_many_arguments)]
    pub fn from_action(
        context: UpdateContext<'a, 'gc, 'gc_context>,
        id: ActivationIdentifier<'a>,
        swf_version: u8,
        scope: GcCell<'gc, Scope<'gc>>,
        constant_pool: GcCell<'gc, Vec<Value<'gc>>>,
        base_clip: DisplayObject<'gc>,
        this: Value<'gc>,
        callee: Option<Object<'gc>>,
    ) -> Self {
        avm_debug!(context.avm1, "START {}", id);
        Self {
            context,
            id,
            swf_version,
            scope,
            constant_pool,
            base_clip,
            target_clip: Some(base_clip),
            base_clip_unloaded: base_clip.removed(),
            this,
            callee,
            local_registers: None,
        }
    }

    /// Create a new activation to run a block of code with a given scope.
    pub fn with_new_scope<'b, S: Into<Cow<'static, str>>>(
        &'b mut self,
        name: S,
        scope: GcCell<'gc, Scope<'gc>>,
    ) -> Activation<'b, 'gc, 'gc_context> {
        let id = self.id.child(name);
        avm_debug!(self.context.avm1, "START {}", id);
        Activation {
            id,
            context: self.context.reborrow(),
            swf_version: self.swf_version,
            scope,
            constant_pool: self.constant_pool,
            base_clip: self.base_clip,
            target_clip: self.target_clip,
            base_clip_unloaded: self.base_clip_unloaded,
            this: self.this,
            callee: self.callee,
            local_registers: self.local_registers,
        }
    }

    /// Construct an empty stack frame with no code.
    ///
    /// This is used by tests and by callback methods (`onEnterFrame`) to create a base
    /// activation frame with access to the global context.
    pub fn from_nothing(
        context: UpdateContext<'a, 'gc, 'gc_context>,
        id: ActivationIdentifier<'a>,
        globals: Object<'gc>,
        base_clip: DisplayObject<'gc>,
    ) -> Self {
        let global_scope = GcCell::allocate(context.gc_context, Scope::from_global_object(globals));
        let swf_version = base_clip.swf_version();
        let child_scope = GcCell::allocate(
            context.gc_context,
            Scope::new_local_scope(global_scope, context.gc_context),
        );
        let empty_constant_pool = GcCell::allocate(context.gc_context, Vec::new());
        avm_debug!(context.avm1, "START {}", id);

        Self {
            context,
            id,
            swf_version,
            scope: child_scope,
            constant_pool: empty_constant_pool,
            base_clip,
            target_clip: Some(base_clip),
            base_clip_unloaded: base_clip.removed(),
            this: globals.into(),
            callee: None,
            local_registers: None,
        }
    }

    /// Construct an empty stack frame with no code running on the root move in
    /// layer 0.
    pub fn from_stub(
        context: UpdateContext<'a, 'gc, 'gc_context>,
        id: ActivationIdentifier<'a>,
    ) -> Self {
        let globals = context.avm1.global_object_cell();
        let level0 = context.stage.root_clip();

        Self::from_nothing(context, id, globals, level0)
    }

    /// Add a stack frame that executes code in timeline scope
    pub fn run_child_frame_for_action<S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        active_clip: DisplayObject<'gc>,
        code: SwfSlice,
    ) -> Result<ReturnType<'gc>, Error<'gc>> {
        let globals = self.context.avm1.global_object_cell();
        let mut parent_activation = Activation::from_nothing(
            self.context.reborrow(),
            self.id.child("[Actions Parent]"),
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
        let constant_pool = parent_activation.context.avm1.constant_pool();
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
        child_activation.run_actions(code)
    }

    /// Add a stack frame that executes code in initializer scope.
    pub fn run_with_child_frame_for_display_object<F, R, S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        active_clip: DisplayObject<'gc>,
        swf_version: u8,
        function: F,
    ) -> R
    where
        for<'c> F: FnOnce(&mut Activation<'c, 'gc, '_>) -> R,
    {
        let clip_obj = match active_clip.object() {
            Value::Object(o) => o,
            _ => panic!("No script object for display object"),
        };
        let global_scope = GcCell::allocate(
            self.context.gc_context,
            Scope::from_global_object(self.context.avm1.global_object_cell()),
        );
        let child_scope = GcCell::allocate(
            self.context.gc_context,
            Scope::new(global_scope, scope::ScopeClass::Target, clip_obj),
        );
        let constant_pool = self.context.avm1.constant_pool();
        let mut activation = Activation::from_action(
            self.context.reborrow(),
            self.id.child(name),
            swf_version,
            child_scope,
            constant_pool,
            active_clip,
            clip_obj.into(),
            None,
        );
        function(&mut activation)
    }

    pub fn run_actions(&mut self, code: SwfSlice) -> Result<ReturnType<'gc>, Error<'gc>> {
        let mut read = Reader::new(&code.movie.data()[code.start..], self.swf_version());

        loop {
            let result = self.do_action(&code, &mut read);
            match result {
                Ok(FrameControl::Return(return_type)) => break Ok(return_type),
                Ok(FrameControl::Continue) => {}
                Err(e) => break Err(e),
            }
        }
    }

    /// Run a single action from a given action reader.
    fn do_action<'b>(
        &mut self,
        data: &'b SwfSlice,
        reader: &mut Reader<'b>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        *self.context.actions_since_timeout_check += 1;
        if *self.context.actions_since_timeout_check >= 2000 {
            *self.context.actions_since_timeout_check = 0;
            if self.context.update_start.elapsed() >= self.context.max_execution_duration {
                return Err(Error::ExecutionTimeout);
            }
        }

        if reader.get_ref().as_ptr() as usize >= data.as_ref().as_ptr_range().end as usize {
            //Executing beyond the end of a function constitutes an implicit return.
            Ok(FrameControl::Return(ReturnType::Implicit))
        } else {
            let action = reader.read_action()?;
            avm_debug!(
                self.context.avm1,
                "({}) Action: {:?}",
                self.id.depth(),
                action
            );

            match action {
                Action::Add => self.action_add(),
                Action::Add2 => self.action_add_2(),
                Action::And => self.action_and(),
                Action::AsciiToChar => self.action_ascii_to_char(),
                Action::BitAnd => self.action_bit_and(),
                Action::BitLShift => self.action_bit_lshift(),
                Action::BitOr => self.action_bit_or(),
                Action::BitRShift => self.action_bit_rshift(),
                Action::BitURShift => self.action_bit_urshift(),
                Action::BitXor => self.action_bit_xor(),
                Action::Call => self.action_call(),
                Action::CallFunction => self.action_call_function(),
                Action::CallMethod => self.action_call_method(),
                Action::CastOp => self.action_cast_op(),
                Action::CharToAscii => self.action_char_to_ascii(),
                Action::CloneSprite => self.action_clone_sprite(),
                Action::ConstantPool(action) => self.action_constant_pool(action),
                Action::Decrement => self.action_decrement(),
                Action::DefineFunction(action) => self.action_define_function(action.into(), data),
                Action::DefineFunction2(action) => self.action_define_function(action, data),
                Action::DefineLocal => self.action_define_local(),
                Action::DefineLocal2 => self.action_define_local_2(),
                Action::Delete => self.action_delete(),
                Action::Delete2 => self.action_delete_2(),
                Action::Divide => self.action_divide(),
                Action::End => self.action_end(),
                Action::EndDrag => self.action_end_drag(),
                Action::Enumerate => self.action_enumerate(),
                Action::Enumerate2 => self.action_enumerate_2(),
                Action::Equals => self.action_equals(),
                Action::Equals2 => self.action_equals_2(),
                Action::Extends => self.action_extends(),
                Action::GetMember => self.action_get_member(),
                Action::GetProperty => self.action_get_property(),
                Action::GetTime => self.action_get_time(),
                Action::GetVariable => self.action_get_variable(),
                Action::GetUrl(action) => self.action_get_url(action),
                Action::GetUrl2(action) => self.action_get_url_2(action),
                Action::GotoFrame(action) => self.action_goto_frame(action),
                Action::GotoFrame2(action) => self.action_goto_frame_2(action),
                Action::Greater => self.action_greater(),
                Action::GotoLabel(action) => self.action_goto_label(action),
                Action::If(action) => self.action_if(action, reader, data),
                Action::Increment => self.action_increment(),
                Action::InitArray => self.action_init_array(),
                Action::InitObject => self.action_init_object(),
                Action::ImplementsOp => self.action_implements_op(),
                Action::InstanceOf => self.action_instance_of(),
                Action::Jump(action) => self.action_jump(action, reader, data),
                Action::Less => self.action_less(),
                Action::Less2 => self.action_less_2(),
                Action::MBAsciiToChar => self.action_mb_ascii_to_char(),
                Action::MBCharToAscii => self.action_mb_char_to_ascii(),
                Action::MBStringLength => self.action_mb_string_length(),
                Action::MBStringExtract => self.action_mb_string_extract(),
                Action::Modulo => self.action_modulo(),
                Action::Multiply => self.action_multiply(),
                Action::NextFrame => self.action_next_frame(),
                Action::NewMethod => self.action_new_method(),
                Action::NewObject => self.action_new_object(),
                Action::Not => self.action_not(),
                Action::Or => self.action_or(),
                Action::Play => self.action_play(),
                Action::Pop => self.action_pop(),
                Action::PreviousFrame => self.action_prev_frame(),
                Action::Push(action) => self.action_push(action),
                Action::PushDuplicate => self.action_push_duplicate(),
                Action::RandomNumber => self.action_random_number(),
                Action::RemoveSprite => self.action_remove_sprite(),
                Action::Return => self.action_return(),
                Action::SetMember => self.action_set_member(),
                Action::SetProperty => self.action_set_property(),
                Action::SetTarget(action) => self.action_set_target(action),
                Action::SetTarget2 => self.action_set_target_2(),
                Action::SetVariable => self.action_set_variable(),
                Action::StackSwap => self.action_stack_swap(),
                Action::StartDrag => self.action_start_drag(),
                Action::Stop => self.action_stop(),
                Action::StopSounds => self.action_stop_sounds(),
                Action::StoreRegister(action) => self.action_store_register(action),
                Action::StrictEquals => self.action_strict_equals(),
                Action::StringAdd => self.action_string_add(),
                Action::StringEquals => self.action_string_equals(),
                Action::StringExtract => self.action_string_extract(),
                Action::StringGreater => self.action_string_greater(),
                Action::StringLength => self.action_string_length(),
                Action::StringLess => self.action_string_less(),
                Action::Subtract => self.action_subtract(),
                Action::TargetPath => self.action_target_path(),
                Action::Throw => self.action_throw(),
                Action::ToggleQuality => self.action_toggle_quality(),
                Action::ToInteger => self.action_to_integer(),
                Action::ToNumber => self.action_to_number(),
                Action::ToString => self.action_to_string(),
                Action::Trace => self.action_trace(),
                Action::Try(action) => self.action_try(&action, data),
                Action::TypeOf => self.action_type_of(),
                Action::WaitForFrame(action) => self.action_wait_for_frame(action, reader),
                Action::WaitForFrame2(action) => self.action_wait_for_frame_2(action, reader),
                Action::With(action) => self.action_with(action, data),
                Action::Unknown(action) => self.action_unknown(action),
            }
        }
    }

    fn action_add(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.coerce_to_f64(self)? + a.coerce_to_f64(self)?;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_add_2(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // ECMA-262 s. 11.6.1
        let a = self.context.avm1.pop().to_primitive(self)?;
        let b = self.context.avm1.pop().to_primitive(self)?;
        let result: Value<'_> = match (a, b) {
            (Value::String(a), Value::String(b)) => {
                AvmString::concat(self.context.gc_context, b, a).into()
            }
            (Value::String(a), b) => {
                AvmString::concat(self.context.gc_context, b.coerce_to_string(self)?, a).into()
            }
            (a, Value::String(b)) => {
                AvmString::concat(self.context.gc_context, b, a.coerce_to_string(self)?).into()
            }
            _ => (b.coerce_to_f64(self)? + a.coerce_to_f64(self)?).into(),
        };
        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_and(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 logical and
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.as_bool(self.swf_version()) && a.as_bool(self.swf_version());
        self.context.avm1.push(result.into()); // Diverges from spec: returns a boolean even in SWF 4
        Ok(FrameControl::Continue)
    }

    fn action_ascii_to_char(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // In SWF6+, this operates on UTF-16 code units.
        // In SWF5 and below, this operates on bytes, regardless of the locale encoding.
        let char_code = self.context.avm1.pop().coerce_to_u16(self)?;
        let result = match char_code {
            0 => WString::default(),
            c if self.swf_version() < 6 || char::try_from(c as u32).is_ok() => {
                WString::from_unit(c)
            }
            _ => WString::from_unit(char::REPLACEMENT_CHARACTER as u16),
        };
        self.context
            .avm1
            .push(AvmString::new(self.context.gc_context, result).into());
        Ok(FrameControl::Continue)
    }

    fn action_char_to_ascii(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWF4 ord function
        // In SWF6+, this operates on UTF-16 code units.
        // In SWF5 and below, this operates on bytes, regardless of the locale.
        let val = self.context.avm1.pop();
        let s = val.coerce_to_string(self)?;
        let char_code = s.get(0).unwrap_or(0);
        // Unpaired surrogate characters should return the code point for the replacement character.
        // Try to convert the code unit back to a character, which will fail if this is invalid UTF-16 (unpaired surrogate).
        // TODO: Should this happen in SWF5 and below?
        let c = crate::string::utils::utf16_code_unit_to_char(char_code);
        self.context.avm1.push(u32::from(c).into());
        Ok(FrameControl::Continue)
    }

    fn action_clone_sprite(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let depth = self.context.avm1.pop();
        let target = self.context.avm1.pop();
        let source = self.context.avm1.pop();
        let start_clip = self.target_clip_or_root();
        let source_clip = self.resolve_target_display_object(start_clip, source, true)?;

        if let Some(movie_clip) = source_clip.and_then(|o| o.as_movie_clip()) {
            let _ = globals::movie_clip::duplicate_movie_clip_with_bias(
                movie_clip,
                self,
                &[target, depth],
                0,
            );
        } else {
            avm_warn!(self, "CloneSprite: Source is not a movie clip");
        }

        Ok(FrameControl::Continue)
    }

    fn action_bit_and(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_i32(self)?;
        let b = self.context.avm1.pop().coerce_to_i32(self)?;
        let result = a & b;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_bit_lshift(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_i32(self)? & 0b11111; // Only 5 bits used for shift count
        let b = self.context.avm1.pop().coerce_to_i32(self)?;
        let result = b << a;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_bit_or(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_i32(self)?;
        let b = self.context.avm1.pop().coerce_to_i32(self)?;
        let result = a | b;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_bit_rshift(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_u32(self)? & 0b11111; // Only 5 bits used for shift count
        let b = self.context.avm1.pop().coerce_to_i32(self)?;
        let result = b >> a;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_bit_urshift(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_u32(self)? & 0b11111; // Only 5 bits used for shift count
        let b = self.context.avm1.pop().coerce_to_u32(self)?;
        let result = b >> a;
        let result = if matches!(self.swf_version(), 8..=9) {
            // In SWF8 and SWF9, unsigned right shift actually has a signed result.
            (result as i32).into()
        } else {
            result.into()
        };
        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_bit_xor(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_i32(self)?;
        let b = self.context.avm1.pop().coerce_to_i32(self)?;
        let result = b ^ a;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_call(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Runs any actions on the given frame.
        let arg = self.context.avm1.pop();
        let target = self.target_clip_or_root();

        // The parameter can be a frame # or a path to a movie clip with a frame number.
        let mut call_frame = None;
        if let Value::Number(frame) = arg {
            // Frame # on the current clip.
            if let Some(target) = target.as_movie_clip() {
                call_frame = Some((target, f64_to_wrapping_u32(frame)));
            }
        } else {
            // An optional path to a MovieClip and a frame #/label, such as "/clip:framelabel".
            let frame_path = arg.coerce_to_string(self)?;
            if let Some((clip, frame)) = self.resolve_variable_path(target, &frame_path)? {
                if let Some(clip) = clip.as_display_object().and_then(|o| o.as_movie_clip()) {
                    if let Ok(frame) = frame.parse().map(f64_to_wrapping_u32) {
                        // First try to parse as a frame number.
                        call_frame = Some((clip, frame));
                    } else if let Some(frame) = clip.frame_label_to_number(frame) {
                        // Otherwise, it's a frame label.
                        call_frame = Some((clip, frame.into()));
                    }
                }
            }
        };

        if let Some((clip, frame)) = call_frame {
            if frame <= u16::MAX.into() {
                for action in clip.actions_on_frame(&mut self.context, frame as u16) {
                    let _ = self.run_child_frame_for_action("[Frame Call]", clip.into(), action)?;
                }
            }
        } else {
            avm_warn!(self, "Call: Invalid call");
        }

        Ok(FrameControl::Continue)
    }

    fn action_call_function(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let fn_name_value = self.context.avm1.pop();
        let fn_name = fn_name_value.coerce_to_string(self)?;
        let num_args = self.context.avm1.pop().coerce_to_u32(self)? as usize;
        let num_args = num_args.min(self.context.avm1.stack_len());
        let mut args = Vec::with_capacity(num_args);
        for _ in 0..num_args {
            args.push(self.context.avm1.pop());
        }

        let variable = self.get_variable(fn_name)?;

        let result = variable.call_with_default_this(
            self.target_clip_or_root().object().coerce_to_object(self),
            fn_name,
            self,
            &args,
        )?;
        self.context.avm1.push(result);

        // After any function call, execution of this frame stops if the base clip doesn't exist.
        // For example, a _root.gotoAndStop moves the timeline to a frame where the clip was removed.
        self.continue_if_base_clip_exists()
    }

    fn action_call_method(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let method_name = self.context.avm1.pop();
        let object_val = self.context.avm1.pop();
        let num_args = self.context.avm1.pop().coerce_to_u32(self)? as usize;
        let num_args = num_args.min(self.context.avm1.stack_len());
        let mut args = Vec::with_capacity(num_args);
        for _ in 0..num_args {
            args.push(self.context.avm1.pop());
        }

        // Can not call method on undefined/null.
        if matches!(object_val, Value::Undefined | Value::Null) {
            self.context.avm1.push(Value::Undefined);
            return Ok(FrameControl::Continue);
        }

        let object = object_val.coerce_to_object(self);

        let method_name = if method_name == Value::Undefined {
            "".into()
        } else {
            method_name.coerce_to_string(self)?
        };

        let result = if method_name.is_empty() {
            // Undefined/empty method name; call `this` as a function.
            object.call("[Anonymous]".into(), self, Value::Undefined, &args)?
        } else {
            // Call `this[method_name]`.
            object.call_method(method_name, &args, self, ExecutionReason::FunctionCall)?
        };
        self.context.avm1.push(result);

        self.continue_if_base_clip_exists()
    }

    fn action_cast_op(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let obj = self.context.avm1.pop();
        let constr = self.context.avm1.pop().coerce_to_object(self);

        let is_instance_of = if let Value::Object(obj) = obj {
            let prototype = constr.get("prototype", self)?.coerce_to_object(self);
            obj.is_instance_of(self, constr, prototype)?
        } else {
            false
        };

        let result = if is_instance_of { obj } else { Value::Null };
        self.context.avm1.push(result);

        Ok(FrameControl::Continue)
    }

    fn action_constant_pool(
        &mut self,
        action: ConstantPool,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.context.avm1.set_constant_pool(GcCell::allocate(
            self.context.gc_context,
            action
                .strings
                .iter()
                .map(|s| {
                    AvmString::new_utf8(self.context.gc_context, s.to_str_lossy(self.encoding()))
                        .into()
                })
                .collect(),
        ));
        self.set_constant_pool(self.context.avm1.constant_pool());

        Ok(FrameControl::Continue)
    }

    fn action_decrement(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_f64(self)?;
        let result = a - 1.0;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_define_function(
        &mut self,
        action: DefineFunction2,
        parent_data: &SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let swf_version = self.swf_version();
        let func_data = parent_data.to_unbounded_subslice(action.actions);
        let constant_pool = self.constant_pool();
        let func = Avm1Function::from_swf_function(
            self.context.gc_context,
            swf_version,
            func_data,
            action,
            self.scope_cell(),
            constant_pool,
            self.base_clip(),
        );
        let name = func.name();
        let prototype = ScriptObject::new(
            self.context.gc_context,
            Some(self.context.avm1.prototypes().object),
        )
        .into();
        let func_obj = FunctionObject::function(
            self.context.gc_context,
            Gc::allocate(self.context.gc_context, func),
            Some(self.context.avm1.prototypes().function),
            prototype,
        );
        if let Some(name) = name {
            self.define_local(name, func_obj.into())?;
        } else {
            self.context.avm1.push(func_obj.into());
        }

        Ok(FrameControl::Continue)
    }

    fn action_define_local(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // If the property does not exist on the local object's prototype chain, it is created on the local object.
        // Otherwise, the property is set (including calling virtual setters).
        // Though this isn't in the SWF19 spec, dot paths and slash paths are also supported and affect the related
        // object in the same way as Action::SetVariable.
        let value = self.context.avm1.pop();
        let name_val = self.context.avm1.pop();
        let name = name_val.coerce_to_string(self)?;
        self.define_local(name, value)?;
        Ok(FrameControl::Continue)
    }

    fn action_define_local_2(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // If the property does not exist on the local object's prototype chain, it is created on the local object.
        // Otherwise, the property is unchanged.
        // Though this isn't in the SWF19 spec, dot paths and slash paths are also supported and affect the related
        // object in the same way as Action::SetVariable, if the variable doesn't already exist on the mentioned object.
        let name_val = self.context.avm1.pop();
        let name = name_val.coerce_to_string(self)?;
        if !self.in_local_scope() && name.find(b":.".as_ref()).is_some() {
            if matches!(
                self.get_variable(name)?,
                CallableValue::UnCallable(Value::Undefined)
            ) {
                self.set_variable(name, Value::Undefined)?;
            }
        } else if !self.scope_cell().read().locals().has_property(self, name) {
            let scope = self.scope;
            let scope = scope.write(self.context.gc_context);
            scope.define_local(name, Value::Undefined, self)?;
        };
        Ok(FrameControl::Continue)
    }

    fn action_delete(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_val = self.context.avm1.pop();
        let name = name_val.coerce_to_string(self)?;
        let object = self.context.avm1.pop();

        let success = if let Value::Object(object) = object {
            object.delete(self, name)
        } else {
            avm_warn!(self, "Cannot delete property {} from {:?}", name, object);
            false
        };
        self.context.avm1.push(success.into());

        Ok(FrameControl::Continue)
    }

    fn action_delete_2(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_val = self.context.avm1.pop();
        let name = name_val.coerce_to_string(self)?;

        // Fun fact: This isn't in the Adobe SWF19 spec, but this opcode returns
        // a boolean based on if the delete actually deleted something.
        let success = self.scope_cell().read().delete(self, name);
        self.context.avm1.push(success.into());

        Ok(FrameControl::Continue)
    }

    fn action_divide(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_f64(self)?;
        let b = self.context.avm1.pop().coerce_to_f64(self)?;

        // SWF19: "If A is zero, the result NaN, Infinity, or -Infinity is pushed to the stack in SWF 5 and later.
        // In SWF 4, the result is the string #ERROR#."
        let result: Value<'gc> = if a == 0.0 && self.swf_version() < 5 {
            "#ERROR#".into()
        } else {
            (b / a).into()
        };

        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_end(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        Ok(FrameControl::Return(ReturnType::Implicit))
    }

    fn action_end_drag(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // we might not have had an opportunity to call `update_drag`
        // if AS did `startDrag(mc);stopDrag();` in one go
        // so let's do it here
        crate::player::Player::update_drag(&mut self.context);

        *self.context.drag_object = None;
        Ok(FrameControl::Continue)
    }

    fn action_enumerate(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_value = self.context.avm1.pop();
        let name = name_value.coerce_to_string(self)?;
        let object: Value<'gc> = self.get_variable(name)?.into();
        self.context.avm1.push(Value::Undefined); // Sentinel that indicates end of enumeration

        match object {
            Value::Object(ob) => {
                for k in ob.get_keys(self).into_iter().rev() {
                    self.context.avm1.push(k.into());
                }
            }
            _ => avm_error!(self, "Cannot enumerate properties of {}", name),
        };

        Ok(FrameControl::Continue)
    }

    fn action_enumerate_2(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.context.avm1.pop();

        self.context.avm1.push(Value::Undefined); // Sentinel that indicates end of enumeration

        if let Value::Object(object) = value {
            for k in object.get_keys(self).into_iter().rev() {
                self.context.avm1.push(k.into());
            }
        } else {
            avm_warn!(self, "Cannot enumerate {:?}", value);
        }

        Ok(FrameControl::Continue)
    }

    #[allow(clippy::float_cmp)]
    fn action_equals(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 equality
        // If both of the values to compare coerce to `NaN`, the result will always be false.
        // This differs from the behavior used in `Action::Equals2`.
        let a = self.context.avm1.pop().coerce_to_f64(self)?;
        let b = self.context.avm1.pop().coerce_to_f64(self)?;
        let result = b == a;
        self.context.avm1.push(result.into()); // Diverges from spec: returns a boolean even in SWF 4
        Ok(FrameControl::Continue)
    }

    fn action_equals_2(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Version >=5 equality
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.abstract_eq(a, self)?;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_extends(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let superclass = self.context.avm1.pop().coerce_to_object(self);
        let subclass = self.context.avm1.pop().coerce_to_object(self);

        //TODO: What happens if we try to extend an object which has no `prototype`?
        //e.g. `class Whatever extends Object.prototype` or `class Whatever extends 5`

        // Use `create_bare_object` to ensure the proper underlying object type when
        // extending native objects.
        // TODO: This doesn't work if the user manually wires up `prototype`/`__proto__`.
        // The native object needs to be created later by the superclass's constructor.
        // (see #701)
        let super_prototype = superclass.get("prototype", self)?.coerce_to_object(self);
        let sub_prototype = super_prototype.create_bare_object(self, super_prototype)?;

        sub_prototype.define_value(
            self.context.gc_context,
            "constructor",
            superclass.into(),
            Attribute::DONT_ENUM,
        );

        sub_prototype.define_value(
            self.context.gc_context,
            "__constructor__",
            superclass.into(),
            Attribute::DONT_ENUM,
        );

        subclass.set("prototype", sub_prototype.into(), self)?;

        Ok(FrameControl::Continue)
    }

    fn action_get_member(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let name_val = self.context.avm1.pop();
        let name = name_val.coerce_to_string(self)?;
        let object_val = self.context.avm1.pop();
        let object = object_val.coerce_to_object(self);

        let result = object.get(name, self)?;
        self.context.avm1.push(result);

        Ok(FrameControl::Continue)
    }

    fn action_get_property(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let prop_value = self.context.avm1.pop();
        let prop_index = prop_value.coerce_to_f64(self)?;
        let path = self.context.avm1.pop();
        let result = if prop_index.is_nan() || prop_index <= -1.0 {
            avm_warn!(self, "GetProperty: Invalid property {:?}", prop_value);
            Value::Undefined
        } else if let Some(target) = self.target_clip() {
            let prop_index = prop_index as usize;
            if let Some(clip) = self.resolve_target_display_object(target, path, true)? {
                let display_properties = self.context.avm1.display_properties();
                let props = display_properties.read();
                if let Some(property) = props.get_by_index(prop_index) {
                    property.get(self, clip)
                } else {
                    avm_warn!(self, "GetProperty: Invalid property index {}", prop_index);
                    Value::Undefined
                }
            } else {
                avm_warn!(self, "GetProperty: Invalid target {:?}", path);
                Value::Undefined
            }
        } else {
            avm_warn!(self, "GetProperty: Invalid base clip");
            Value::Undefined
        };
        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_get_time(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.context.times_get_time_called += 1;
        // heuristic to detect busy loops used for delays and slowly progress fake time
        if self.context.times_get_time_called >= 20 && self.context.times_get_time_called % 5 == 0 {
            *self.context.time_offset += 1;
        }

        let time = Instant::now()
            .duration_since(self.context.start_time)
            .as_millis() as u32;
        let result = time.wrapping_add(*self.context.time_offset);
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_get_variable(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let var_path = self.context.avm1.pop();
        let path = var_path.coerce_to_string(self)?;

        let value: Value<'gc> = self.get_variable(path)?.into();
        self.context.avm1.push(value);

        Ok(FrameControl::Continue)
    }

    fn action_get_url(&mut self, action: GetUrl) -> Result<FrameControl<'gc>, Error<'gc>> {
        let target = action.target.to_str_lossy(self.encoding());
        let url = action.url.to_string_lossy(self.encoding());
        // TODO: Use `StageObject::get_level_by_path`.
        if target.starts_with("_level") && target.len() > 6 {
            match target[6..].parse::<i32>() {
                Ok(level_id) => {
                    let level = self.resolve_level(level_id);

                    if url.is_empty() {
                        //Blank URL on movie loads = unload!
                        if let Some(mut mc) = level.as_movie_clip() {
                            mc.replace_with_movie(&mut self.context, None, None)
                        }
                    } else {
                        let future = self.context.load_manager.load_movie_into_clip(
                            self.context.player.clone(),
                            level,
                            Request::get(url),
                            None,
                            None,
                        );
                        self.context.navigator.spawn_future(future);
                    }
                }
                Err(e) => avm_warn!(
                    self,
                    "Couldn't parse level id {} for action_get_url: {}",
                    target,
                    e
                ),
            }
            return Ok(FrameControl::Continue);
        }

        if let Some(fscommand) = fscommand::parse(&WString::from_utf8(&url)) {
            let fsargs = WString::from_utf8(&target);
            fscommand::handle(fscommand, &fsargs, self)?;
        } else {
            self.context
                .navigator
                .navigate_to_url(url.to_owned(), target.into_owned(), None);
        }

        Ok(FrameControl::Continue)
    }

    fn action_get_url_2(&mut self, action: GetUrl2) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO: Support `LoadVariablesFlag`, `LoadTargetFlag`
        // TODO: What happens if there's only one string?
        let target_val = self.context.avm1.pop();
        let target = target_val.coerce_to_string(self)?;
        let url_val = self.context.avm1.pop();
        let url = url_val.coerce_to_string(self)?;

        if let Some(fscommand) = fscommand::parse(&url) {
            // `target` = fscommand arguments!
            fscommand::handle(fscommand, &target, self)?;
            return Ok(FrameControl::Continue);
        }

        // TODO: Use `StageObject::get_level_by_path`.
        let level_target = if target.starts_with(WStr::from_units(b"_level")) && target.len() >= 6 {
            match target[6..].parse::<f64>() {
                Ok(level_id) => level_id as i32,
                Err(_) => {
                    if target.len() == 6 {
                        0
                    } else {
                        -1
                    }
                }
            }
        } else {
            -1
        };

        let clip_target: Option<DisplayObject<'gc>> = if level_target > -1 {
            Some(self.resolve_level(level_target))
        } else if action.is_load_vars() || action.is_target_sprite() {
            if let Value::Object(target) = target_val {
                target.as_display_object()
            } else {
                let start = self.target_clip_or_root();
                self.resolve_target_display_object(start, target_val, true)?
            }
        } else {
            None
        };

        if action.is_load_vars() {
            // `loadVariables` or `loadVariablesNum` call.
            // Depending on the situation, it will open a link in the browser instead.
            let mut is_load_vars = true;
            if !(action.is_target_sprite() || level_target > -1) {
                is_load_vars = false;
                if matches!(target_val, Value::Object(_)) {
                    if let Some(clip) = clip_target {
                        is_load_vars = DisplayObject::ptr_eq(clip, self.base_clip().avm1_root());
                    }
                }
            }
            if is_load_vars {
                if let Some(clip_target) = clip_target {
                    let target_obj = clip_target
                        .as_movie_clip()
                        .unwrap()
                        .object()
                        .coerce_to_object(self);
                    let request = self.locals_into_request(
                        url,
                        NavigationMethod::from_send_vars_method(action.send_vars_method()),
                    );
                    let future = self.context.load_manager.load_form_into_object(
                        self.context.player.clone(),
                        target_obj,
                        request,
                    );
                    self.context.navigator.spawn_future(future);
                }
                return Ok(FrameControl::Continue);
            }
        } else if action.is_target_sprite() {
            // `loadMovie`, `unloadMovie` or `unloadMovieNum` call.
            if let Some(clip_target) = clip_target {
                if url.is_empty() {
                    // Blank URL on movie loads = unload!
                    if let Some(mut mc) = clip_target.as_movie_clip() {
                        mc.replace_with_movie(&mut self.context, None, None)
                    }
                } else {
                    let request = self.locals_into_request(
                        url,
                        NavigationMethod::from_send_vars_method(action.send_vars_method()),
                    );
                    let future = self.context.load_manager.load_movie_into_clip(
                        self.context.player.clone(),
                        clip_target,
                        request,
                        None,
                        None,
                    );
                    self.context.navigator.spawn_future(future);
                }
            }
            return Ok(FrameControl::Continue);
        } else if level_target > -1 {
            // `loadMovieNum` call.
            if let Some(clip_target) = clip_target {
                if url.is_empty() {
                    // Blank URL on movie loads = unload!
                    if let Some(mut mc) = clip_target.as_movie_clip() {
                        mc.replace_with_movie(&mut self.context, None, None)
                    }
                } else {
                    let future = self.context.load_manager.load_movie_into_clip(
                        self.context.player.clone(),
                        clip_target,
                        Request::get(url.to_utf8_lossy().into_owned()),
                        None,
                        None,
                    );
                    self.context.navigator.spawn_future(future);
                }
            }
            return Ok(FrameControl::Continue);
        }

        // `getURL` call.
        let vars = match NavigationMethod::from_send_vars_method(action.send_vars_method()) {
            Some(method) => Some((method, self.locals_into_form_values())),
            None => None,
        };

        self.context
            .navigator
            .navigate_to_url(url.to_string(), target.to_string(), vars);

        Ok(FrameControl::Continue)
    }

    fn action_goto_frame(&mut self, action: GotoFrame) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                // The frame on the stack is 0-based, not 1-based.
                clip.goto_frame(&mut self.context, action.frame + 1, true);
            } else {
                avm_error!(self, "GotoFrame failed: Target is not a MovieClip");
            }
        } else {
            avm_error!(self, "GotoFrame failed: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_goto_frame_2(&mut self, action: GotoFrame2) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Version 4+ gotoAndPlay/gotoAndStop
        // Param can either be a frame number or a frame label.
        if let Some(clip) = self.target_clip_or_root().as_movie_clip() {
            let frame = self.context.avm1.pop();
            let _ = globals::movie_clip::goto_frame(
                clip,
                self,
                &[frame],
                !action.set_playing,
                action.scene_offset,
            );
        } else {
            avm_warn!(self, "GotoFrame2: Target is not a MovieClip");
        }
        Ok(FrameControl::Continue)
    }

    fn action_goto_label(&mut self, action: GotoLabel) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                let label = WString::from_utf8(&action.label.to_str_lossy(self.encoding()));
                if let Some(frame) = clip.frame_label_to_number(&label) {
                    clip.goto_frame(&mut self.context, frame, true);
                } else {
                    avm_warn!(self, "GoToLabel: Frame label '{:?}' not found", label);
                }
            } else {
                avm_warn!(self, "GoToLabel: Target is not a MovieClip");
            }
        } else {
            avm_warn!(self, "GoToLabel: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_if<'b>(
        &mut self,
        action: If,
        reader: &mut Reader<'b>,
        data: &'b SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.context.avm1.pop();
        if val.as_bool(self.swf_version()) {
            reader.seek(data.movie.data(), action.offset);
        }
        Ok(FrameControl::Continue)
    }

    fn action_increment(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_f64(self)?;
        let result = a + 1.0;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_init_array(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let num_elements = self.context.avm1.pop().coerce_to_f64(self)?;
        let result = if num_elements < 0.0 || num_elements > i32::MAX.into() {
            // InitArray pops no args and pushes undefined if num_elements is out of range.
            Value::Undefined
        } else {
            ArrayObject::new(
                self.context.gc_context,
                self.context.avm1.prototypes().array,
                (0..num_elements as i32).map(|_| self.context.avm1.pop()),
            )
            .into()
        };

        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_init_object(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let num_props = self.context.avm1.pop().coerce_to_f64(self)?;
        let result = if num_props < 0.0 || num_props > i32::MAX.into() {
            // InitArray pops no args and pushes undefined if num_props is out of range.
            Value::Undefined
        } else {
            let object = ScriptObject::new(
                self.context.gc_context,
                Some(self.context.avm1.prototypes().object),
            );
            for _ in 0..num_props as usize {
                let value = self.context.avm1.pop();
                let name_val = self.context.avm1.pop();
                let name = name_val.coerce_to_string(self)?;
                object.set(name, value, self)?;
            }
            Value::Object(object.into())
        };

        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_implements_op(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let constructor = self.context.avm1.pop().coerce_to_object(self);
        let count = self.context.avm1.pop();
        // Old Flash Players (at least FP9) used to coerce objects as well. However, this was
        // changed at some point and instead the following is logged:
        // "Parameters of type Object are no longer coerced into the required primitive type - number."
        // Newer Flash Players coerce only primitives, and treat objects as 0.
        let count = if count.is_primitive() {
            count.coerce_to_i32(self)? as usize
        } else {
            avm_warn!(self, "ImplementsOp: Object not coerced into number");
            0
        };
        let count = count.min(self.context.avm1.stack_len());
        let mut interfaces = Vec::with_capacity(count);

        // TODO: If one of the interfaces is not an object, do we leave the
        // whole stack dirty, or...?
        for _ in 0..count {
            interfaces.push(self.context.avm1.pop().coerce_to_object(self));
        }

        let prototype = constructor.get("prototype", self)?.coerce_to_object(self);
        prototype.set_interfaces(self.context.gc_context, interfaces);

        Ok(FrameControl::Continue)
    }

    fn action_instance_of(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let constr = self.context.avm1.pop().coerce_to_object(self);
        let obj = self.context.avm1.pop();

        let result = if let Value::Object(obj) = obj {
            let prototype = constr.get("prototype", self)?.coerce_to_object(self);
            obj.is_instance_of(self, constr, prototype)?
        } else {
            false
        };

        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_jump<'b>(
        &mut self,
        action: Jump,
        reader: &mut Reader<'b>,
        data: &'b SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        reader.seek(data.movie.data(), action.offset);
        Ok(FrameControl::Continue)
    }

    fn action_less(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 less than
        // If one of the values to compare coerces to `NaN`, the result will be false.
        // This differs from the behavior used in `Action::Less2`.
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.coerce_to_f64(self)? < a.coerce_to_f64(self)?;
        self.context.avm1.push(result.into()); // Diverges from spec: returns a boolean even in SWF 4
        Ok(FrameControl::Continue)
    }

    fn action_less_2(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // ECMA-262 s. 11.8.1
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.abstract_lt(a, self)?;
        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_greater(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // ECMA-262 s. 11.8.2
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = a.abstract_lt(b, self)?;
        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_mb_ascii_to_char(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // In SWF6+, this operates on UTF-16 code units.
        // TODO: In SWF5 and below, this operates on locale-dependent characters.
        let char_code: u32 = self.context.avm1.pop().coerce_to_u16(self)?.into();
        let result = if char_code != 0 {
            // Unpaired surrogates turn into replacement char.
            char::try_from(char_code)
                .unwrap_or(char::REPLACEMENT_CHARACTER)
                .to_string()
        } else {
            String::default()
        };
        self.context
            .avm1
            .push(AvmString::new_utf8(self.context.gc_context, result).into());
        Ok(FrameControl::Continue)
    }

    fn action_mb_char_to_ascii(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWF4 mbord function
        // In SWF6+, this operates on UTF-16 code units.
        // In SWF5 and below, this operates on locale-dependent characters.
        let val = self.context.avm1.pop();
        let s = val.coerce_to_string(self)?;
        let char_code = s.get(0).unwrap_or(0);
        let c = if self.swf_version() < 6 {
            char::from(char_code as u8)
        } else {
            // Unpaired surrogate characters should return the code point for the replacement character.
            // Try to convert the code unit back to a character, which will fail if this is invalid UTF-16 (unpaired surrogate).
            crate::string::utils::utf16_code_unit_to_char(char_code)
        };
        self.context.avm1.push(u32::from(c).into());
        Ok(FrameControl::Continue)
    }

    fn action_mb_string_extract(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWF4 mbsubstring
        // In SWF6+, this operates on UTF-16 code units.
        // In SWF5 and below, this operates on locale-dependent characters.
        let len = self.context.avm1.pop().coerce_to_i32(self)?;
        let len = usize::try_from(len).ok();

        // Index is 1-based for this opcode.
        let start = self.context.avm1.pop().coerce_to_i32(self)?;
        let start = if start >= 1 { start as usize - 1 } else { 0 };

        let val = self.context.avm1.pop();
        let s = val.coerce_to_string(self)?;

        let end = len
            .and_then(|l| start.checked_add(l))
            .filter(|l| *l <= s.len())
            .unwrap_or_else(|| s.len());

        let result = &s[start.min(end)..end];
        self.context
            .avm1
            .push(AvmString::new(self.context.gc_context, result).into());
        Ok(FrameControl::Continue)
    }

    fn action_mb_string_length(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.context.avm1.pop();
        let len = val.coerce_to_string(self)?.len();
        self.context.avm1.push((len as f64).into());
        Ok(FrameControl::Continue)
    }

    fn action_multiply(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_f64(self)?;
        let b = self.context.avm1.pop().coerce_to_f64(self)?;
        let result = b * a;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_modulo(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO: Wrong operands?
        let a = self.context.avm1.pop().coerce_to_f64(self)?;
        let b = self.context.avm1.pop().coerce_to_f64(self)?;
        let result = b % a;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_not(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop();
        let result = !a.as_bool(self.swf_version());
        self.context.avm1.push(result.into()); // Diverges from spec: returns a boolean even in SWF 4
        Ok(FrameControl::Continue)
    }

    fn action_next_frame(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.next_frame(&mut self.context);
            } else {
                avm_warn!(self, "NextFrame: Target is not a MovieClip");
            }
        } else {
            avm_warn!(self, "NextFrame: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_new_method(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let method_name = self.context.avm1.pop();
        let object_val = self.context.avm1.pop();
        let num_args = self.context.avm1.pop().coerce_to_u32(self)? as usize;
        let num_args = num_args.min(self.context.avm1.stack_len());
        let mut args = Vec::with_capacity(num_args);
        for _ in 0..num_args {
            args.push(self.context.avm1.pop());
        }

        // Can not call method on undefined/null.
        if matches!(object_val, Value::Undefined | Value::Null) {
            self.context.avm1.push(Value::Undefined);
            return Ok(FrameControl::Continue);
        }

        let object = object_val.coerce_to_object(self);

        let method_name = if method_name == Value::Undefined {
            "".into()
        } else {
            method_name.coerce_to_string(self)?
        };

        let result = if method_name.is_empty() {
            // Undefined/empty method name; construct `this` as a function.
            object.construct(self, &args)?
        } else {
            let constructor = object.get(method_name, self)?;
            if let Value::Object(constructor) = constructor {
                // Construct `this[method_name]`.
                constructor.construct(self, &args)?
            } else {
                avm_warn!(
                    self,
                    "Tried to construct with non-object constructor {:?}",
                    constructor
                );
                Value::Undefined
            }
        };

        self.context.avm1.push(result);

        self.continue_if_base_clip_exists()
    }

    fn action_new_object(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let fn_name_val = self.context.avm1.pop();
        let fn_name = fn_name_val.coerce_to_string(self)?;
        let num_args = self.context.avm1.pop().coerce_to_u32(self)? as usize;
        let num_args = num_args.min(self.context.avm1.stack_len());
        let mut args = Vec::with_capacity(num_args);
        for _ in 0..num_args {
            args.push(self.context.avm1.pop());
        }

        let name_value: Value<'gc> = self.resolve(fn_name)?.into();
        let constructor = name_value.coerce_to_object(self);
        let result = constructor.construct(self, &args)?;
        self.context.avm1.push(result);

        self.continue_if_base_clip_exists()
    }

    fn action_or(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 logical or
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.as_bool(self.swf_version()) || a.as_bool(self.swf_version());
        self.context.avm1.push(result.into()); // Diverges from spec: returns a boolean even in SWF 4
        Ok(FrameControl::Continue)
    }

    fn action_play(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.play(&mut self.context)
            } else {
                avm_warn!(self, "Play: Target is not a MovieClip");
            }
        } else {
            avm_warn!(self, "Play: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_prev_frame(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.prev_frame(&mut self.context);
            } else {
                avm_warn!(self, "PrevFrame: Target is not a MovieClip");
            }
        } else {
            avm_warn!(self, "PrevFrame: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_pop(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.context.avm1.pop();
        Ok(FrameControl::Continue)
    }

    fn action_push(&mut self, action: Push) -> Result<FrameControl<'gc>, Error<'gc>> {
        for value in action.values {
            use swf::avm1::types::Value as SwfValue;
            let value = match value {
                SwfValue::Undefined => Value::Undefined,
                SwfValue::Null => Value::Null,
                SwfValue::Bool(v) => v.into(),
                SwfValue::Int(v) => v.into(),
                SwfValue::Float(v) => v.into(),
                SwfValue::Double(v) => v.into(),
                SwfValue::Str(v) => {
                    AvmString::new_utf8(self.context.gc_context, v.to_str_lossy(self.encoding()))
                        .into()
                }
                SwfValue::Register(v) => self.current_register(v),
                SwfValue::ConstantPool(i) => {
                    if let Some(value) = self.constant_pool().read().get(i as usize) {
                        *value
                    } else {
                        avm_warn!(
                            self,
                            "ActionPush: Constant pool index {} out of range (len = {})",
                            i,
                            self.constant_pool().read().len()
                        );
                        Value::Undefined
                    }
                }
            };
            self.context.avm1.push(value);
        }
        Ok(FrameControl::Continue)
    }

    fn action_push_duplicate(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.context.avm1.pop();
        self.context.avm1.push(val);
        self.context.avm1.push(val);
        Ok(FrameControl::Continue)
    }

    fn action_random_number(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // The max value is clamped to the range [0, 2^31 - 1).
        let max = self.context.avm1.pop().coerce_to_f64(self)? as i32;
        let result = if max > 0 {
            self.context.rng.gen_range(0..max)
        } else {
            0
        };
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_remove_sprite(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let target = self.context.avm1.pop();
        let start_clip = self.target_clip_or_root();
        let target_clip = self.resolve_target_display_object(start_clip, target, true)?;

        if let Some(target_clip) = target_clip {
            crate::avm1::globals::remove_display_object(target_clip, self);
        } else {
            avm_warn!(self, "RemoveSprite: Source is not a display object");
        }
        Ok(FrameControl::Continue)
    }

    fn action_return(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let return_value = self.context.avm1.pop();

        Ok(FrameControl::Return(ReturnType::Explicit(return_value)))
    }

    fn action_set_member(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.context.avm1.pop();
        let name_val = self.context.avm1.pop();
        let name = name_val.coerce_to_string(self)?;

        let object = self.context.avm1.pop().coerce_to_object(self);
        object.set(name, value, self)?;

        Ok(FrameControl::Continue)
    }

    fn action_set_property(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.context.avm1.pop();
        let prop_index = self.context.avm1.pop().coerce_to_u32(self)? as usize;
        let path = self.context.avm1.pop();
        if let Some(target) = self.target_clip() {
            if let Some(clip) = self.resolve_target_display_object(target, path, true)? {
                let display_properties = self.context.avm1.display_properties();
                let props = display_properties.read();
                if let Some(property) = props.get_by_index(prop_index) {
                    property.set(self, clip, value)?;
                }
            } else {
                avm_warn!(self, "SetProperty: Invalid target");
            }
        } else {
            avm_warn!(self, "SetProperty: Invalid base clip");
        }
        Ok(FrameControl::Continue)
    }

    fn action_set_variable(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Flash 4-style variable
        let value = self.context.avm1.pop();
        let var_path_val = self.context.avm1.pop();
        let var_path = var_path_val.coerce_to_string(self)?;
        self.set_variable(var_path, value)?;
        Ok(FrameControl::Continue)
    }

    #[allow(clippy::float_cmp)]
    fn action_strict_equals(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // The same as normal equality but types must match
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = a == b;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_set_target(&mut self, action: SetTarget) -> Result<FrameControl<'gc>, Error<'gc>> {
        let target = WString::from_utf8_owned(action.target.to_string_lossy(self.encoding()));
        self.set_target(&target)
    }

    fn action_set_target_2(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let target = self.context.avm1.pop();

        let base_clip = self.base_clip();
        if base_clip.removed() {
            self.set_target_clip(None);
            return Ok(FrameControl::Continue);
        }

        match target {
            Value::String(target) => {
                return self.set_target(&target);
            }
            Value::Undefined => {
                // Reset.
                let base_clip = self.base_clip();
                self.set_target_clip(Some(base_clip));
            }
            Value::Object(o) => {
                if let Some(clip) = o.as_display_object() {
                    // MovieClips can be targeted directly.
                    self.set_target_clip(Some(clip));
                } else {
                    // Other objects get coerced to string.
                    let target = target.coerce_to_string(self)?;
                    return self.set_target(&target);
                }
            }
            _ => {
                let target = target.coerce_to_string(self)?;
                return self.set_target(&target);
            }
        };

        let scope = self.scope_cell();
        let clip_obj = self.target_clip_or_root().object().coerce_to_object(self);

        self.set_scope(Scope::new_target_scope(
            scope,
            clip_obj,
            self.context.gc_context,
        ));
        Ok(FrameControl::Continue)
    }

    fn action_stack_swap(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        self.context.avm1.push(a);
        self.context.avm1.push(b);
        Ok(FrameControl::Continue)
    }

    fn action_start_drag(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let target = self.context.avm1.pop();
        let start_clip = self.target_clip_or_root();
        let display_object = self.resolve_target_display_object(start_clip, target, true)?;
        if let Some(display_object) = display_object {
            let lock_center = self.context.avm1.pop();
            let constrain = self.context.avm1.pop().as_bool(self.swf_version());
            if constrain {
                let y2 = self.context.avm1.pop();
                let x2 = self.context.avm1.pop();
                let y1 = self.context.avm1.pop();
                let x1 = self.context.avm1.pop();
                start_drag(display_object, self, &[lock_center, x1, y1, x2, y2]);
            } else {
                start_drag(display_object, self, &[lock_center]);
            };
        } else {
            avm_warn!(self, "StartDrag: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_stop(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        if let Some(clip) = self.target_clip() {
            if let Some(clip) = clip.as_movie_clip() {
                clip.stop(&mut self.context);
            } else {
                avm_warn!(self, "Stop: Target is not a MovieClip");
            }
        } else {
            avm_warn!(self, "Stop: Invalid target");
        }
        Ok(FrameControl::Continue)
    }

    fn action_stop_sounds(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.context.stop_all_sounds();
        Ok(FrameControl::Continue)
    }

    fn action_store_register(
        &mut self,
        action: StoreRegister,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // The value must remain on the stack.
        let val = self.context.avm1.pop();
        self.context.avm1.push(val);
        self.set_current_register(action.register, val);
        Ok(FrameControl::Continue)
    }

    fn action_string_add(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWFv4 string concatenation
        // TODO(Herschel): Result with non-string operands?
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let s = AvmString::concat(
            self.context.gc_context,
            b.coerce_to_string(self)?,
            a.coerce_to_string(self)?,
        );
        self.context.avm1.push(s.into());
        Ok(FrameControl::Continue)
    }

    fn action_string_equals(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strcmp
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.coerce_to_string(self)? == a.coerce_to_string(self)?;
        self.context.avm1.push(result.into()); // Diverges from spec: returns a boolean even in SWF 4
        Ok(FrameControl::Continue)
    }

    fn action_string_extract(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // SWF4 substring function
        let len = self.context.avm1.pop().coerce_to_i32(self)?;
        let len = usize::try_from(len).ok();

        // Index is 1-based for this opcode.
        let start = self.context.avm1.pop().coerce_to_i32(self)?;
        let start = if start >= 1 { start as usize - 1 } else { 0 };

        let val = self.context.avm1.pop();
        let s = val.coerce_to_string(self)?;

        let end = len
            .and_then(|l| start.checked_add(l))
            .filter(|l| *l <= s.len())
            .unwrap_or_else(|| s.len());

        let result = &s[start.min(end)..end];
        self.context
            .avm1
            .push(AvmString::new(self.context.gc_context, result).into());
        Ok(FrameControl::Continue)
    }

    fn action_string_greater(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strcmp
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.coerce_to_string(self)?.gt(&a.coerce_to_string(self)?);
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_string_length(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strlen
        // In SWF6+, this is the same as String.length (returns number of UTF-16 code units).
        // TODO: In SWF5, this returns the byte length, even though the encoding is locale dependent.
        let val = self.context.avm1.pop().coerce_to_string(self)?;
        self.context.avm1.push(val.len().into());
        Ok(FrameControl::Continue)
    }

    fn action_string_less(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // AS1 strcmp
        let a = self.context.avm1.pop();
        let b = self.context.avm1.pop();
        let result = b.coerce_to_string(self)?.lt(&a.coerce_to_string(self)?);
        self.context.avm1.push(result.into()); // Diverges from spec: returns a boolean even in SWF 4
        Ok(FrameControl::Continue)
    }

    fn action_subtract(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let a = self.context.avm1.pop().coerce_to_f64(self)?;
        let b = self.context.avm1.pop().coerce_to_f64(self)?;
        let result = b - a;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_target_path(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Prints out the dot-path for the parameter.
        // Parameter must be a display object (not a string path).
        let param = self.context.avm1.pop().coerce_to_object(self);
        let result = if let Some(display_object) = param.as_display_object() {
            let path = display_object.path();
            AvmString::new(self.context.gc_context, path).into()
        } else {
            Value::Undefined
        };
        self.context.avm1.push(result);
        Ok(FrameControl::Continue)
    }

    fn action_throw(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.context.avm1.pop();
        avm_debug!(
            self.context.avm1,
            "Thrown exception: {}",
            value
                .coerce_to_string(self)
                .unwrap_or_else(|_| "undefined".into())
        );
        Err(Error::ThrownValue(value))
    }

    fn action_toggle_quality(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        use crate::display_object::StageQuality;
        // Toggle between `Low` and `High`/`Best` quality.
        // This op remembers whether the stage quality was `Best` or higher, so we have to maintain
        // the bitmap downsampling flag to ensure we toggle back to the proper quality.
        let use_bitmap_downsamping = self.context.stage.use_bitmap_downsampling();
        let new_quality = match self.context.stage.quality() {
            StageQuality::High | StageQuality::Best => StageQuality::Low,
            _ if use_bitmap_downsamping => StageQuality::Best,
            _ => StageQuality::High,
        };
        self.context
            .stage
            .set_quality(self.context.gc_context, new_quality);
        self.context
            .stage
            .set_use_bitmap_downsampling(self.context.gc_context, use_bitmap_downsamping);
        Ok(FrameControl::Continue)
    }

    fn action_to_integer(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.context.avm1.pop();
        let result = val.coerce_to_i32(self)?;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_to_number(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.context.avm1.pop();
        let result = val.coerce_to_f64(self)?;
        self.context.avm1.push(result.into());
        Ok(FrameControl::Continue)
    }

    fn action_to_string(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.context.avm1.pop();
        let string = val.coerce_to_string(self)?;
        self.context.avm1.push(string.into());
        Ok(FrameControl::Continue)
    }

    fn action_trace(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.context.avm1.pop();
        // trace always prints "undefined" even though SWF6 and below normally
        // coerce undefined to "".
        let out = if val == Value::Undefined {
            "undefined".into()
        } else {
            val.coerce_to_string(self)?
        };
        self.context.avm_trace(&out.to_utf8_lossy());
        Ok(FrameControl::Continue)
    }

    fn action_try(
        &mut self,
        action: &Try,
        parent_data: &SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let mut result = self.run_actions(parent_data.to_unbounded_subslice(action.try_body));

        if let Some((catch_vars, actions)) = &action.catch_body {
            if let Err(Error::ThrownValue(value)) = &result {
                let mut activation = Activation::from_action(
                    self.context.reborrow(),
                    self.id.child("[Catch]"),
                    self.swf_version,
                    self.scope,
                    self.constant_pool,
                    self.base_clip,
                    self.this,
                    self.callee,
                );

                match catch_vars {
                    CatchVar::Var(name) => {
                        let name = name.to_str_lossy(activation.encoding());
                        let name = AvmString::new_utf8(activation.context.gc_context, name);
                        activation.set_variable(name, value.to_owned())?
                    }
                    CatchVar::Register(id) => {
                        activation.set_current_register(*id, value.to_owned())
                    }
                }

                result = activation.run_actions(parent_data.to_unbounded_subslice(actions));
            }
        }

        if let Some(actions) = action.finally_body {
            if let ReturnType::Explicit(value) =
                self.run_actions(parent_data.to_unbounded_subslice(actions))?
            {
                return Ok(FrameControl::Return(ReturnType::Explicit(value)));
            }
        }

        match result? {
            ReturnType::Implicit => Ok(FrameControl::Continue),
            ReturnType::Explicit(value) => Ok(FrameControl::Return(ReturnType::Explicit(value))),
        }
    }

    fn action_type_of(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let type_of = self.context.avm1.pop().type_of();
        self.context.avm1.push(AvmString::from(type_of).into());
        Ok(FrameControl::Continue)
    }

    fn action_wait_for_frame(
        &mut self,
        action: WaitForFrame,
        r: &mut Reader<'_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Always true for now.
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            skip_actions(r, action.num_actions_to_skip);
        }
        Ok(FrameControl::Continue)
    }

    fn action_wait_for_frame_2(
        &mut self,
        action: WaitForFrame2,
        r: &mut Reader<'_>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // TODO(Herschel): Always true for now.
        let _frame_num = self.context.avm1.pop().coerce_to_f64(self)? as u16;
        let loaded = true;
        if !loaded {
            // Note that the offset is given in # of actions, NOT in bytes.
            // Read the actions and toss them away.
            skip_actions(r, action.num_actions_to_skip);
        }
        Ok(FrameControl::Continue)
    }

    fn action_with(
        &mut self,
        action: With,
        parent_data: &SwfSlice,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let code = parent_data.to_unbounded_subslice(action.actions);
        let value = self.context.avm1.pop();
        match value {
            // Undefined/null with is ignored.
            Value::Undefined | Value::Null => {
                // Mimic Flash's error output.
                self.context.avm_trace(
                    "Error: A 'with' action failed because the specified object did not exist.\n",
                );
                Ok(FrameControl::Continue)
            }

            value => {
                // Note that primitives get boxed at this point.
                let object = value.coerce_to_object(self);
                let with_scope =
                    Scope::new_with_scope(self.scope_cell(), object, self.context.gc_context);
                let mut new_activation = self.with_new_scope("[With]", with_scope);
                if let ReturnType::Explicit(value) = new_activation.run_actions(code)? {
                    Ok(FrameControl::Return(ReturnType::Explicit(value)))
                } else {
                    Ok(FrameControl::Continue)
                }
            }
        }
    }

    fn action_unknown(
        &mut self,
        unknown: swf::avm1::types::Unknown,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        avm_error!(self, "Unknown AVM1 opcode: {:?}", unknown);
        Ok(FrameControl::Continue)
    }

    /// Retrieve a given register value.
    ///
    /// If a given register does not exist, this function yields
    /// Value::Undefined, which is also a valid register value.
    pub fn current_register(&self, id: u8) -> Value<'gc> {
        if self.has_local_register(id) {
            self.local_register(id).unwrap_or(Value::Undefined)
        } else {
            self.context
                .avm1
                .get_register(id as usize)
                .cloned()
                .unwrap_or(Value::Undefined)
        }
    }

    /// Set a register to a given value.
    ///
    /// If a given register does not exist, this function does nothing.
    pub fn set_current_register(&mut self, id: u8, value: Value<'gc>) {
        if self.has_local_register(id) {
            self.set_local_register(id, value);
        } else if let Some(v) = self.context.avm1.get_register_mut(id as usize) {
            *v = value;
        }
    }

    /// Convert the enumerable properties of an object into a set of form values.
    ///
    /// This is necessary to support form submission from Flash via a couple of
    /// legacy methods, such as the `ActionGetURL2` opcode or `getURL` function.
    ///
    /// WARNING: This does not support user defined virtual properties!
    pub fn object_into_form_values(&mut self, object: Object<'gc>) -> IndexMap<String, String> {
        let mut form_values = IndexMap::new();
        let keys = object.get_keys(self);

        for k in keys {
            let v = object.get(k, self);

            //TODO: What happens if an error occurs inside a virtual property?
            form_values.insert(
                k.to_string(),
                v.ok()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_string(self)
                    .unwrap_or_else(|_| "undefined".into())
                    .to_string(),
            );
        }

        form_values
    }

    /// Construct a request for a fetch operation that may send object properties as form data in
    /// the request body or URL.
    pub fn object_into_request(
        &mut self,
        object: Object<'gc>,
        url: AvmString<'gc>,
        method: Option<NavigationMethod>,
    ) -> Request {
        match method {
            Some(method) => {
                let vars = self.object_into_form_values(object);
                let qstring = form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(vars.iter())
                    .finish();

                match method {
                    NavigationMethod::Get if !url.contains(b'?') => {
                        Request::get(format!("{}?{}", url, qstring))
                    }
                    NavigationMethod::Get => Request::get(format!("{}&{}", url, qstring)),
                    NavigationMethod::Post => Request::post(
                        url.to_utf8_lossy().into_owned(),
                        Some((
                            qstring.as_bytes().to_owned(),
                            "application/x-www-form-urlencoded".to_string(),
                        )),
                    ),
                }
            }
            None => Request::get(url.to_utf8_lossy().into_owned()),
        }
    }

    /// Convert the current locals pool into a set of form values.
    ///
    /// This is necessary to support form submission from Flash via a couple of
    /// legacy methods, such as the `ActionGetURL2` opcode or `getURL` function.
    ///
    /// WARNING: This does not support user defined virtual properties!
    pub fn locals_into_form_values(&mut self) -> IndexMap<String, String> {
        let scope = self.scope_cell();
        let locals = scope.read().locals_cell();
        self.object_into_form_values(locals)
    }

    /// Construct a request for a fetch operation that may send locals as form data in the request
    /// body or URL.
    pub fn locals_into_request(
        &mut self,
        url: AvmString<'gc>,
        method: Option<NavigationMethod>,
    ) -> Request {
        let scope = self.scope_cell();
        let locals = scope.read().locals_cell();
        self.object_into_request(locals, url, method)
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
    ///
    /// `allow_empty` will allow the empty string to resolve to the start movie clip.
    pub fn resolve_target_display_object(
        &mut self,
        start: DisplayObject<'gc>,
        target: Value<'gc>,
        allow_empty: bool,
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
        let path = target.coerce_to_string(self)?;

        if !allow_empty && path.is_empty() {
            return Ok(None);
        }

        let root = start.avm1_root();
        let start = start.object().coerce_to_object(self);
        Ok(self
            .resolve_target_path(root, start, &path, false)?
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
        root: DisplayObject<'gc>,
        start: Object<'gc>,
        mut path: &WStr,
        mut first_element: bool,
    ) -> Result<Option<Object<'gc>>, Error<'gc>> {
        // Empty path resolves immediately to start clip.
        if path.is_empty() {
            return Ok(Some(start));
        }

        // Starting / means an absolute path starting from root.
        // (`/bar` means `_root.bar`)
        let (mut object, mut is_slash_path) = if path.starts_with(b'/') {
            path = &path[1..];
            (root.object().coerce_to_object(self), true)
        } else {
            (start, false)
        };

        let case_sensitive = self.is_case_sensitive();

        // Iterate through each token in the path.
        while !path.is_empty() {
            // Skip any number of leading :
            // `foo`, `:foo`, and `:::foo` are all the same
            path = path.trim_start_matches(b':');

            let prefix = &path[..path.len().min(3)];
            let val = if prefix == b".." || prefix == b"../" || prefix == b"..:" {
                // Check for ..
                // SWF-4 style _parent
                if path.get(2) == Some(u16::from(b'/')) {
                    is_slash_path = true;
                }
                path = path.slice(3..).unwrap_or_default();
                if let Some(parent) = object.as_display_object().and_then(|o| o.avm1_parent()) {
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
                    match u8::try_from(path.at(pos)) {
                        Ok(b':') => break,
                        Ok(b'.') if !is_slash_path => break,
                        Ok(b'/') => {
                            is_slash_path = true;
                            break;
                        }
                        _ => (),
                    }
                    pos += 1;
                }

                // Slice out the identifier and step the cursor past the delimiter.
                let name = &path[..pos];
                path = path.slice(pos + 1..).unwrap_or_default();

                if first_element && name == b"this" {
                    self.this_cell()
                } else if first_element && name == b"_root" {
                    self.root_object()
                } else {
                    // Get the value from the object.
                    // Resolves display object instances first, then local variables.
                    // This is the opposite of general GetMember property access!
                    if let Some(child) = object
                        .as_display_object()
                        .and_then(|o| o.as_container())
                        .and_then(|o| o.child_by_name(name, case_sensitive))
                    {
                        child.object()
                    } else {
                        let name = AvmString::new(self.context.gc_context, name);
                        object.get(name, self).unwrap()
                    }
                }
            };

            // `this`/`_root` can only be the first element in the path.
            first_element = false;

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
        start: DisplayObject<'gc>,
        path: &'s WStr,
    ) -> Result<Option<(Object<'gc>, &'s WStr)>, Error<'gc>> {
        // Find the right-most : or . in the path.
        // If we have one, we must resolve as a target path.
        if let Some(separator) = path.rfind(b":.".as_ref()) {
            // We have a . or :, so this is a path to an object plus a variable name.
            // We resolve it directly on the targeted object.
            let (path, var_name) = (&path[..separator], &path[separator + 1..]);

            let mut current_scope = Some(self.scope_cell());
            while let Some(scope) = current_scope {
                let avm1_root = start.avm1_root();
                if let Some(object) =
                    self.resolve_target_path(avm1_root, *scope.read().locals(), path, true)?
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
    /// Finally, if none of the above applies, it is a normal variable name resolved via the
    /// scope chain.
    pub fn get_variable(&mut self, path: AvmString<'gc>) -> Result<CallableValue<'gc>, Error<'gc>> {
        // Resolve a variable path for a GetVariable action.
        let start = self.target_clip_or_root();

        // Find the right-most : or . in the path.
        // If we have one, we must resolve as a target path.
        if let Some(separator) = path.rfind(b":.".as_ref()) {
            // We have a . or :, so this is a path to an object plus a variable name.
            // We resolve it directly on the targeted object.
            let (path, var_name) = (&path[..separator], &path[separator + 1..]);

            let mut current_scope = Some(self.scope_cell());
            while let Some(scope) = current_scope {
                let avm1_root = start.avm1_root();
                if let Some(object) =
                    self.resolve_target_path(avm1_root, *scope.read().locals(), path, true)?
                {
                    let var_name = AvmString::new(self.context.gc_context, var_name);
                    if object.has_property(self, var_name) {
                        return Ok(CallableValue::Callable(object, object.get(var_name, self)?));
                    }
                }
                current_scope = scope.read().parent_cell();
            }

            return Ok(CallableValue::UnCallable(Value::Undefined));
        }

        // If it doesn't have a trailing variable, it can still be a slash path.
        if path.contains(b'/') {
            let mut current_scope = Some(self.scope_cell());
            while let Some(scope) = current_scope {
                let avm1_root = start.avm1_root();
                if let Some(object) =
                    self.resolve_target_path(avm1_root, *scope.read().locals(), &path, false)?
                {
                    return Ok(CallableValue::UnCallable(object.into()));
                }
                current_scope = scope.read().parent_cell();
            }
        }

        // Finally! It's a plain old variable name.
        // Resolve using scope chain, as normal.
        self.resolve(path)
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
    pub fn set_variable(
        &mut self,
        path: AvmString<'gc>,
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
        let separator = path.rfind(b":.".as_ref());

        if let Some(sep) = separator {
            // We have a . or :, so this is a path to an object plus a variable name.
            // We resolve it directly on the targeted object.
            let (path, var_name) = (&path[..sep], &path[sep + 1..]);

            let mut current_scope = Some(self.scope_cell());
            while let Some(scope) = current_scope {
                let avm1_root = start.avm1_root();
                if let Some(object) =
                    self.resolve_target_path(avm1_root, *scope.read().locals(), path, true)?
                {
                    let var_name = AvmString::new(self.context.gc_context, var_name);
                    object.set(var_name, value, self)?;
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
        let scope = self.scope_cell();
        scope.read().set(path, value, self)?;
        Ok(())
    }

    /// Resolve a level by ID.
    ///
    /// If the level does not exist, then it will be created and instantiated
    /// with a script object.
    pub fn resolve_level(&mut self, level_id: i32) -> DisplayObject<'gc> {
        if let Some(level) = self.context.stage.child_by_depth(level_id) {
            level
        } else {
            let level: DisplayObject<'_> =
                MovieClip::new(self.base_clip().movie().unwrap(), self.context.gc_context).into();

            level.set_depth(self.context.gc_context, level_id);
            level.set_default_root_name(&mut self.context);
            self.context
                .stage
                .replace_at_depth(&mut self.context, level, level_id);
            level.post_instantiation(&mut self.context, None, Instantiator::Movie, false);

            level
        }
    }

    /// The current target clip of the executing code.
    /// Actions that affect `root` after an invalid `tellTarget` will use this.
    ///
    /// The `root` is determined relative to the base clip that defined the
    pub fn target_clip_or_root(&self) -> DisplayObject<'gc> {
        self.target_clip()
            .unwrap_or_else(|| self.base_clip().avm1_root())
    }

    /// Obtain the value of `_root`.
    pub fn root_object(&self) -> Value<'gc> {
        self.base_clip().avm1_root().object()
    }

    /// Returns whether property keys should be case sensitive based on the current SWF version.
    pub fn is_case_sensitive(&self) -> bool {
        self.swf_version() > 6
    }

    /// Resolve a particular named local variable within this activation.
    ///
    /// Because scopes are object chains, the same rules for `Object::get`
    /// still apply here.
    pub fn resolve(&mut self, name: AvmString<'gc>) -> Result<CallableValue<'gc>, Error<'gc>> {
        if &name == b"this" {
            return Ok(CallableValue::UnCallable(self.this_cell()));
        }

        self.scope_cell().read().resolve(name, self)
    }

    /// Returns the suggested string encoding for actions.
    /// For SWF version 6 and higher, this is always UTF-8.
    /// For SWF version 5 and lower, this is locale-dependent,
    /// and we default to WINDOWS-1252.
    pub fn encoding(&self) -> &'static swf::Encoding {
        swf::SwfStr::encoding_for_version(self.swf_version)
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

    /// Whether this activation operates in a local scope.
    pub fn in_local_scope(&self) -> bool {
        let mut current_scope = Some(self.scope);
        while let Some(scope) = current_scope {
            match scope.read().class() {
                scope::ScopeClass::Local => {
                    return true;
                }
                scope::ScopeClass::Target => {
                    return false;
                }
                _ => (),
            };
            current_scope = scope.read().parent_cell();
        }
        false
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
        // The target should revert to `None` if the clip is removed.
        self.target_clip = value.filter(|clip| !clip.removed());
    }

    /// Define a local property on the activation.
    ///
    /// If this activation operates in a local scope and `name` is a path string, we will resolve
    /// as a target path and set the variable on the mentioned object to `value`.
    /// Otherwise, we will create or set the property specified by `name` to `value` on the local
    /// scope. This does not crawl the scope chain. Any properties deeper in the scope chain with
    /// the same name will be shadowed.
    pub fn define_local(
        &mut self,
        name: AvmString<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        if !self.in_local_scope() && name.find(b":.".as_ref()).is_some() {
            self.set_variable(name, value)
        } else {
            let scope = self.scope;
            let scope = scope.write(self.context.gc_context);
            scope.define_local(name, value, self)
        }
    }

    /// Create a local property on the activation.
    ///
    /// This inserts a value as a stored property on the local scope. If the property already
    /// exists, it will be forcefully overwritten. Used internally to initialize objects.
    pub fn force_define_local(&mut self, name: AvmString<'gc>, value: Value<'gc>) {
        self.scope
            .read()
            .force_define_local(name, value, self.context.gc_context)
    }

    /// Returns value of `this` as a reference.
    pub fn this_cell(&self) -> Value<'gc> {
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
    pub fn set_local_register(&mut self, id: u8, value: Value<'gc>) {
        if let Some(ref mut local_registers) = self.local_registers {
            if let Some(r) = local_registers.write(self.context.gc_context).get_mut(id) {
                *r = value;
            }
        }
    }

    pub fn constant_pool(&self) -> GcCell<'gc, Vec<Value<'gc>>> {
        self.constant_pool
    }

    pub fn set_constant_pool(&mut self, constant_pool: GcCell<'gc, Vec<Value<'gc>>>) {
        self.constant_pool = constant_pool;
    }

    /// Checks that the clip executing a script still exists.
    /// If the clip executing a script is removed during exectuion, return from this activation.
    /// Should be called after any action that could potentially destroy a clip (gotos, etc.)
    fn continue_if_base_clip_exists(&self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // The exception is `unload` clip event handlers, which currently are called when the clip
        // has already been removed. If this activation started with the base clip already removed,
        // this is an unload handler, so allow the code to run regardless.
        // (This may no longer be necessary once #1535 is fixed.)
        if !self.base_clip_unloaded && self.base_clip.removed() {
            Ok(FrameControl::Return(ReturnType::Explicit(Value::Undefined)))
        } else {
            Ok(FrameControl::Continue)
        }
    }

    fn set_target(&mut self, target: &WStr) -> Result<FrameControl<'gc>, Error<'gc>> {
        let base_clip = self.base_clip();
        let new_target_clip;
        let root = base_clip.avm1_root();
        let start = base_clip.object().coerce_to_object(self);
        if target.is_empty() {
            new_target_clip = Some(base_clip);
        } else if let Some(clip) = self
            .resolve_target_path(root, start, target, false)?
            .and_then(|o| o.as_display_object())
            .filter(|_| !self.base_clip.removed())
        // All properties invalid if base clip is removed.
        {
            new_target_clip = Some(clip);
        } else {
            avm_warn!(self, "SetTarget failed: {} not found", target);
            // TODO: Emulate AVM1 trace error message.
            let path = if base_clip.removed() {
                None
            } else {
                Some(base_clip.path())
            };
            let message = format!(
                "Target not found: Target=\"{}\" Base=\"{}\"",
                target,
                match &path {
                    Some(p) => p,
                    None => WStr::from_units(b"?"),
                }
            );
            self.context.avm_trace(&message);

            // When SetTarget has an invalid target, subsequent GetVariables act
            // as if they are targeting root, but subsequent Play/Stop/etc.
            // fail silently.
            new_target_clip = None;
        }

        self.set_target_clip(new_target_clip);

        let scope = self.scope_cell();
        let clip_obj = self.target_clip_or_root().object().coerce_to_object(self);

        self.set_scope(Scope::new_target_scope(
            scope,
            clip_obj,
            self.context.gc_context,
        ));
        Ok(FrameControl::Continue)
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
