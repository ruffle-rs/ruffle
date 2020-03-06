//! ActionScript Virtual Machine 2 (AS3) support

use crate::avm2::activation::{Activation, Avm2ScriptEntry};
use crate::avm2::function::{Avm2ClassEntry, Avm2MethodEntry, FunctionObject};
use crate::avm2::globals::SystemPrototypes;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::scope::Scope;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::context::UpdateContext;
use crate::tag_utils::SwfSlice;
use gc_arena::{Collect, GcCell, MutationContext};
use std::io::Cursor;
use std::rc::Rc;
use swf::avm2::read::Reader;
use swf::avm2::types::{
    AbcFile, Class as AbcClass, Index, Method as AbcMethod, MethodBody, Multiname as AbcMultiname,
    Namespace as AbcNamespace, Op, Script as AbcScript,
};
use swf::read::SwfRead;

#[macro_export]
macro_rules! avm_debug {
    ($($arg:tt)*) => (
        #[cfg(feature = "avm_debug")]
        log::debug!($($arg)*)
    )
}

mod activation;
mod function;
mod globals;
mod names;
mod object;
mod property;
mod return_value;
mod scope;
mod script_object;
mod slot;
mod value;

/// Boxed error alias.
///
/// As AVM2 is a far stricter VM than AVM1, this may eventually be replaced
/// with a proper Avm2Error enum.
type Error = Box<dyn std::error::Error>;

/// The state of an AVM2 interpreter.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Avm2<'gc> {
    /// All activation records for the current interpreter.
    stack_frames: Vec<GcCell<'gc, Activation<'gc>>>,

    /// Values currently present on the operand stack.
    stack: Vec<Value<'gc>>,

    /// Global scope object.
    globals: Object<'gc>,

    /// System prototypes.
    system_prototypes: SystemPrototypes<'gc>,
}

impl<'gc> Avm2<'gc> {
    /// Construct a new AVM interpreter.
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        let (globals, system_prototypes) = globals::construct_global_scope(mc);

        Self {
            stack_frames: Vec::new(),
            stack: Vec::new(),
            globals,
            system_prototypes,
        }
    }

    /// Return the current set of system prototypes.
    pub fn prototypes(&self) -> &SystemPrototypes<'gc> {
        &self.system_prototypes
    }

    /// Load an ABC file embedded in a `SwfSlice`.
    ///
    /// The `SwfSlice` must resolve to the contents of an ABC file.
    pub fn load_abc(
        &mut self,
        abc: SwfSlice,
        _abc_name: &str,
        _lazy_init: bool,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut read = Reader::new(abc.as_ref());

        let abc_file = Rc::new(read.read()?);

        for i in (0..abc_file.scripts.len()).rev() {
            let entrypoint_script: Index<AbcScript> = Index::new(i as u32);
            let entrypoint: Result<Avm2ScriptEntry, Error> =
                Avm2ScriptEntry::from_script_index(abc_file.clone(), entrypoint_script.clone())
                    .ok_or_else(|| {
                        format!("Script method {} does not exist", entrypoint_script.0).into()
                    });
            let entrypoint = entrypoint?;
            let scope = Scope::push_scope(None, self.globals(), context.gc_context);

            for trait_entry in entrypoint.script().traits.iter() {
                self.globals().install_foreign_trait(
                    self,
                    context,
                    abc_file.clone(),
                    trait_entry,
                    Some(scope),
                    self.globals(),
                )?;
            }

            self.insert_stack_frame_for_script(context, entrypoint)?;
        }

        Ok(())
    }

    pub fn globals(&self) -> Object<'gc> {
        self.globals
    }

    /// Get the current stack frame (`Activation` object).
    pub fn current_stack_frame(&self) -> Option<GcCell<'gc, Activation<'gc>>> {
        self.stack_frames.last().copied()
    }

    /// Add a new stack frame to the stack, which can represent any particular
    /// operation you like that needs to execute AVM2 code.
    pub fn insert_stack_frame(&mut self, frame: GcCell<'gc, Activation<'gc>>) {
        self.stack_frames.push(frame)
    }

    /// Add a new stack frame for executing an entrypoint script.
    pub fn insert_stack_frame_for_script(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        script: Avm2ScriptEntry,
    ) -> Result<(), Error> {
        self.stack_frames.push(GcCell::allocate(
            context.gc_context,
            Activation::from_script(context, script, self.globals)?,
        ));

        Ok(())
    }

    /// Destroy the current stack frame (if there is one) with a return value.
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

                self.push(return_value);
            }
        }

        Ok(())
    }

    /// Destroy the current stack frame (if there is one) with an exception.
    ///
    /// TODO: This function should allow exception recovery at some point in
    /// the future.
    ///
    /// NOTE: This means that if you are starting a brand new AVM stack just to
    /// get it's return value, you won't get that value. Instead, retain a cell
    /// referencing the oldest activation frame and use that to retrieve the
    /// return value.
    fn unwind_stack_frame(&mut self) {
        if let Some(_frame) = self.current_stack_frame() {
            self.stack_frames.pop();
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
    pub fn with_current_reader_mut<F, R>(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        func: F,
    ) -> Result<R, Error>
    where
        F: FnOnce(
            &mut Self,
            &mut Reader<Cursor<&[u8]>>,
            &mut UpdateContext<'_, 'gc, '_>,
        ) -> Result<R, Error>,
    {
        let (abc, frame_cell, method_body_index, pc) = {
            let frame = self
                .current_stack_frame()
                .ok_or("No stack frame to read!")?;
            let mut frame_ref = frame.write(context.gc_context);
            frame_ref.lock()?;

            let method = frame_ref.method();
            let abc = method.abc.as_ref().clone();
            let _method_index = method.abc_method;
            let method_body_index = method.abc_method_body as usize;

            (abc, frame, method_body_index, frame_ref.pc())
        };

        let method_body: Result<&MethodBody, Error> =
            abc.method_bodies.get(method_body_index).ok_or_else(|| {
                "Attempting to execute a method that does not exist"
                    .to_string()
                    .into()
            });

        let cursor = Cursor::new(method_body?.code.as_ref());
        let mut read = Reader::new(cursor);
        read.get_inner().set_position(pc as u64);

        let r = func(self, &mut read, context);

        let mut frame_ref = frame_cell.write(context.gc_context);
        frame_ref.unlock_execution();
        frame_ref.set_pc(read.get_inner().position() as usize);

        r
    }

    /// Execute the AVM stack until it is exhausted.
    pub fn run_stack_till_empty(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        while !self.stack_frames.is_empty() {
            self.with_current_reader_mut(context, |this, r, context| {
                this.do_next_opcode(context, r)
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
                    this.do_next_opcode(context, r)
                })?;
            }

            Ok(())
        } else {
            Err("Attempted to run a frame not on the current interpreter stack".into())
        }
    }

    /// Push a value onto the operand stack.
    fn push(&mut self, value: impl Into<Value<'gc>>) {
        let value = value.into();
        avm_debug!("Stack push {}: {:?}", self.stack.len(), value);
        self.stack.push(value);
    }

    /// Retrieve the top-most value on the operand stack.
    #[allow(clippy::let_and_return)]
    fn pop(&mut self) -> Value<'gc> {
        let value = self.stack.pop().unwrap_or_else(|| {
            log::warn!("Avm1::pop: Stack underflow");
            Value::Undefined
        });

        avm_debug!("Stack pop {}: {:?}", self.stack.len(), value);

        value
    }

    fn pop_args(&mut self, arg_count: u32) -> Vec<Value<'gc>> {
        let mut args = Vec::with_capacity(arg_count as usize);
        args.resize(arg_count as usize, Value::Undefined);
        for arg in args.iter_mut().rev() {
            *arg = self.pop();
        }

        args
    }

    fn register_value(&self, index: u32) -> Result<Value<'gc>, Error> {
        self.current_stack_frame()
            .and_then(|sf| sf.read().local_register(index))
            .ok_or_else(|| format!("Out of bounds register read: {}", index).into())
    }

    fn set_register_value(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: u32,
        value: impl Into<Value<'gc>>,
    ) -> Result<(), Error> {
        match self.current_stack_frame().map(|sf| {
            sf.write(context.gc_context)
                .set_local_register(index, value, context.gc_context)
        }) {
            Some(true) => Ok(()),
            _ => Err(format!("Out of bounds register write: {}", index).into()),
        }
    }

    /// Retrieve the current constant pool for the currently executing function.
    fn current_abc(&self) -> Option<Rc<AbcFile>> {
        self.current_stack_frame()
            .map(|sf| sf.read().method().abc.clone())
    }

    /// Retrieve a int from the current constant pool.
    fn pool_int(&self, index: Index<i32>) -> Result<i32, Error> {
        value::abc_int(&self.current_abc().unwrap(), index)
    }

    /// Retrieve a int from the current constant pool.
    fn pool_uint(&self, index: Index<u32>) -> Result<u32, Error> {
        value::abc_uint(&self.current_abc().unwrap(), index)
    }

    /// Retrieve a double from the current constant pool.
    fn pool_double(&self, index: Index<f64>) -> Result<f64, Error> {
        value::abc_double(&self.current_abc().unwrap(), index)
    }

    /// Retrieve a string from the current constant pool.
    fn pool_string(&self, index: Index<String>) -> Result<String, Error> {
        value::abc_string(&self.current_abc().unwrap(), index)
    }

    /// Retrieve a namespace from the current constant pool.
    fn pool_namespace(&self, index: Index<AbcNamespace>) -> Result<Namespace, Error> {
        Namespace::from_abc_namespace(&self.current_abc().unwrap(), index)
    }

    /// Retrieve a multiname from the current constant pool.
    fn pool_multiname(&mut self, index: Index<AbcMultiname>) -> Result<Multiname, Error> {
        Multiname::from_abc_multiname(&self.current_abc().unwrap(), index, self)
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    fn pool_multiname_static(&mut self, index: Index<AbcMultiname>) -> Result<Multiname, Error> {
        Multiname::from_abc_multiname_static(&self.current_abc().unwrap(), index)
    }

    /// Retrieve a method entry from the current ABC file's method table.
    fn table_method(&mut self, index: Index<AbcMethod>) -> Result<Avm2MethodEntry, Error> {
        Avm2MethodEntry::from_method_index(self.current_abc().unwrap(), index.clone())
            .ok_or_else(|| format!("Method index {} does not exist", index.0).into())
    }

    /// Retrieve a class entry from the current ABC file's method table.
    fn table_class(&mut self, index: Index<AbcClass>) -> Result<Avm2ClassEntry, Error> {
        Avm2ClassEntry::from_class_index(self.current_abc().unwrap(), index.clone())
            .ok_or_else(|| format!("Class index {} does not exist", index.0).into())
    }

    /// Run a single action from a given action reader.
    pub fn do_next_opcode(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        if let Some(op) = reader.read_op()? {
            avm_debug!("Opcode: {:?}", op);

            let result = match op {
                Op::PushByte { value } => self.op_push_byte(value),
                Op::PushDouble { value } => self.op_push_double(value),
                Op::PushFalse => self.op_push_false(),
                Op::PushInt { value } => self.op_push_int(value),
                Op::PushNamespace { value } => self.op_push_namespace(value),
                Op::PushNaN => self.op_push_nan(),
                Op::PushNull => self.op_push_null(),
                Op::PushShort { value } => self.op_push_short(value),
                Op::PushString { value } => self.op_push_string(value),
                Op::PushTrue => self.op_push_true(),
                Op::PushUint { value } => self.op_push_uint(value),
                Op::PushUndefined => self.op_push_undefined(),
                Op::Dup => self.op_dup(),
                Op::GetLocal { index } => self.op_get_local(index),
                Op::SetLocal { index } => self.op_set_local(context, index),
                Op::Kill { index } => self.op_kill(context, index),
                Op::Call { num_args } => self.op_call(context, num_args),
                Op::CallMethod { index, num_args } => self.op_call_method(context, index, num_args),
                Op::CallProperty { index, num_args } => {
                    self.op_call_property(context, index, num_args)
                }
                Op::CallPropLex { index, num_args } => {
                    self.op_call_prop_lex(context, index, num_args)
                }
                Op::CallPropVoid { index, num_args } => {
                    self.op_call_prop_void(context, index, num_args)
                }
                Op::CallStatic { index, num_args } => self.op_call_static(context, index, num_args),
                Op::CallSuper { index, num_args } => self.op_call_super(context, index, num_args),
                Op::CallSuperVoid { index, num_args } => {
                    self.op_call_super_void(context, index, num_args)
                }
                Op::ReturnValue => self.op_return_value(context),
                Op::ReturnVoid => self.op_return_void(context),
                Op::GetProperty { index } => self.op_get_property(context, index),
                Op::SetProperty { index } => self.op_set_property(context, index),
                Op::InitProperty { index } => self.op_init_property(context, index),
                Op::DeleteProperty { index } => self.op_delete_property(context, index),
                Op::GetSuper { index } => self.op_get_super(context, index),
                Op::SetSuper { index } => self.op_set_super(context, index),
                Op::PushScope => self.op_push_scope(context),
                Op::PushWith => self.op_push_with(context),
                Op::PopScope => self.op_pop_scope(context),
                Op::GetScopeObject { index } => self.op_get_scope_object(index),
                Op::GetGlobalScope => self.op_get_global_scope(),
                Op::FindProperty { index } => self.op_find_property(context, index),
                Op::FindPropStrict { index } => self.op_find_prop_strict(context, index),
                Op::GetLex { index } => self.op_get_lex(context, index),
                Op::GetSlot { index } => self.op_get_slot(index),
                Op::SetSlot { index } => self.op_set_slot(context, index),
                Op::GetGlobalSlot { index } => self.op_get_global_slot(index),
                Op::SetGlobalSlot { index } => self.op_set_global_slot(context, index),
                Op::Construct { num_args } => self.op_construct(context, num_args),
                Op::ConstructProp { index, num_args } => {
                    self.op_construct_prop(context, index, num_args)
                }
                Op::ConstructSuper { num_args } => self.op_construct_super(context, num_args),
                Op::NewActivation => self.op_new_activation(context),
                Op::NewObject { num_args } => self.op_new_object(context, num_args),
                Op::NewFunction { index } => self.op_new_function(context, index),
                Op::NewClass { index } => self.op_new_class(context, index),
                Op::CoerceA => self.op_coerce_a(),
                Op::Jump { offset } => self.op_jump(offset, reader),
                Op::IfTrue { offset } => self.op_if_true(offset, reader),
                Op::IfFalse { offset } => self.op_if_false(offset, reader),
                Op::IfStrictEq { offset } => self.op_if_strict_eq(offset, reader),
                Op::IfStrictNe { offset } => self.op_if_strict_ne(offset, reader),
                Op::Label => Ok(()),
                Op::Debug {
                    is_local_register,
                    register_name,
                    register,
                } => self.op_debug(is_local_register, register_name, register),
                Op::DebugFile { file_name } => self.op_debug_file(file_name),
                Op::DebugLine { line_num } => self.op_debug_line(line_num),
                _ => self.unknown_op(op),
            };

            if let Err(ref e) = result {
                log::error!("AVM2 error: {}", e);
                self.unwind_stack_frame();
                return result;
            }
        }

        Ok(())
    }

    fn unknown_op(&mut self, op: swf::avm2::types::Op) -> Result<(), Error> {
        log::error!("Unknown AVM2 opcode: {:?}", op);
        Err("Unknown op".into())
    }

    fn op_push_byte(&mut self, value: u8) -> Result<(), Error> {
        self.push(value);
        Ok(())
    }

    fn op_push_double(&mut self, value: Index<f64>) -> Result<(), Error> {
        self.push(self.pool_double(value)?);
        Ok(())
    }

    fn op_push_false(&mut self) -> Result<(), Error> {
        self.push(false);
        Ok(())
    }

    fn op_push_int(&mut self, value: Index<i32>) -> Result<(), Error> {
        self.push(self.pool_int(value)?);
        Ok(())
    }

    fn op_push_namespace(&mut self, value: Index<AbcNamespace>) -> Result<(), Error> {
        self.push(self.pool_namespace(value)?);
        Ok(())
    }

    fn op_push_nan(&mut self) -> Result<(), Error> {
        self.push(std::f64::NAN);
        Ok(())
    }

    fn op_push_null(&mut self) -> Result<(), Error> {
        self.push(Value::Null);
        Ok(())
    }

    fn op_push_short(&mut self, value: u32) -> Result<(), Error> {
        self.push(value);
        Ok(())
    }

    fn op_push_string(&mut self, value: Index<String>) -> Result<(), Error> {
        self.push(self.pool_string(value)?);
        Ok(())
    }

    fn op_push_true(&mut self) -> Result<(), Error> {
        self.push(true);
        Ok(())
    }

    fn op_push_uint(&mut self, value: Index<u32>) -> Result<(), Error> {
        self.push(self.pool_uint(value)?);
        Ok(())
    }

    fn op_push_undefined(&mut self) -> Result<(), Error> {
        self.push(Value::Undefined);
        Ok(())
    }

    fn op_dup(&mut self) -> Result<(), Error> {
        self.push(self.stack.last().cloned().unwrap_or(Value::Undefined));

        Ok(())
    }

    fn op_get_local(&mut self, register_index: u32) -> Result<(), Error> {
        self.push(self.register_value(register_index)?);
        Ok(())
    }

    fn op_set_local(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        register_index: u32,
    ) -> Result<(), Error> {
        let value = self.pop();
        self.set_register_value(context, register_index, value)
    }

    fn op_kill(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        register_index: u32,
    ) -> Result<(), Error> {
        self.set_register_value(context, register_index, Value::Undefined)
    }

    fn op_call(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let receiver = self.pop().as_object().ok();
        let function = self.pop().as_object()?;
        let base_proto = receiver.and_then(|r| r.proto());

        function
            .call(receiver, &args, self, context, base_proto)?
            .push(self);

        Ok(())
    }

    fn op_call_method(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMethod>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let receiver = self.pop().as_object()?;
        let function: Result<Object<'gc>, Error> = receiver
            .get_method(index.0)
            .ok_or_else(|| format!("Object method {} does not exist", index.0).into());
        let base_proto = receiver.proto();

        function?
            .call(Some(receiver), &args, self, context, base_proto)?
            .push(self);

        Ok(())
    }

    fn op_call_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let multiname = self.pool_multiname(index)?;
        let receiver = self.pop().as_object()?;
        let name: Result<QName, Error> = receiver
            .resolve_multiname(&multiname)?
            .ok_or_else(|| format!("Could not find method {:?}", multiname.local_name()).into());
        let name = name?;
        let base_proto = receiver.get_base_proto(&name)?;
        let function = base_proto
            .unwrap_or(receiver)
            .get_property(receiver, &name, self, context)?
            .resolve(self, context)?
            .as_object()?;

        function
            .call(Some(receiver), &args, self, context, base_proto)?
            .push(self);

        Ok(())
    }

    fn op_call_prop_lex(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let multiname = self.pool_multiname(index)?;
        let mut receiver = self.pop().as_object()?;
        let name: Result<QName, Error> = receiver
            .resolve_multiname(&multiname)?
            .ok_or_else(|| format!("Could not find method {:?}", multiname.local_name()).into());
        let function = receiver
            .get_property(receiver, &name?, self, context)?
            .resolve(self, context)?
            .as_object()?;

        function.call(None, &args, self, context, None)?.push(self);

        Ok(())
    }

    fn op_call_prop_void(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let multiname = self.pool_multiname(index)?;
        let receiver = self.pop().as_object()?;
        let name: Result<QName, Error> = receiver
            .resolve_multiname(&multiname)?
            .ok_or_else(|| format!("Could not find method {:?}", multiname.local_name()).into());
        let name = name?;
        let base_proto = receiver.get_base_proto(&name)?;
        let function = base_proto
            .unwrap_or(receiver)
            .get_property(receiver, &name, self, context)?
            .resolve(self, context)?
            .as_object()?;

        function
            .call(Some(receiver), &args, self, context, base_proto)?
            .resolve(self, context)?;

        Ok(())
    }

    fn op_call_static(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMethod>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let receiver = self.pop().as_object()?;
        let method = self.table_method(index)?;
        let scope = self.current_stack_frame().unwrap().read().scope(); //TODO: Is this correct?
        let function = FunctionObject::from_abc_method(
            context.gc_context,
            method,
            scope,
            self.system_prototypes.function,
            None,
        );

        function
            .call(Some(receiver), &args, self, context, receiver.proto())?
            .push(self);

        Ok(())
    }

    fn op_call_super(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let multiname = self.pool_multiname(index)?;
        let receiver = self.pop().as_object()?;
        let name: Result<QName, Error> = receiver
            .resolve_multiname(&multiname)?
            .ok_or_else(|| format!("Could not find method {:?}", multiname.local_name()).into());
        let base_proto: Result<Object<'gc>, Error> = self
            .current_stack_frame()
            .unwrap()
            .read()
            .base_proto()
            .and_then(|bp| bp.proto())
            .ok_or_else(|| {
                "Attempted to call super method without a superclass."
                    .to_string()
                    .into()
            });
        let base_proto = base_proto?;
        let mut base = base_proto.construct(self, context, &[])?; //TODO: very hacky workaround

        let function = base
            .get_property(receiver, &name?, self, context)?
            .resolve(self, context)?
            .as_object()?;

        function
            .call(Some(receiver), &args, self, context, Some(base_proto))?
            .push(self);

        Ok(())
    }

    fn op_call_super_void(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let multiname = self.pool_multiname(index)?;
        let receiver = self.pop().as_object()?;
        let name: Result<QName, Error> = receiver
            .resolve_multiname(&multiname)?
            .ok_or_else(|| format!("Could not find method {:?}", multiname.local_name()).into());
        let base_proto: Result<Object<'gc>, Error> = self
            .current_stack_frame()
            .unwrap()
            .read()
            .base_proto()
            .and_then(|bp| bp.proto())
            .ok_or_else(|| {
                "Attempted to call super method without a superclass."
                    .to_string()
                    .into()
            });
        let base_proto = base_proto?;
        let mut base = base_proto.construct(self, context, &[])?; //TODO: very hacky workaround

        let function = base
            .get_property(receiver, &name?, self, context)?
            .resolve(self, context)?
            .as_object()?;

        function
            .call(Some(receiver), &args, self, context, Some(base_proto))?
            .resolve(self, context)?;

        Ok(())
    }

    fn op_return_value(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let return_value = self.pop();

        self.retire_stack_frame(context, return_value)
    }

    fn op_return_void(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        self.retire_stack_frame(context, Value::Undefined)
    }

    fn op_get_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let multiname = self.pool_multiname(index)?;
        let mut object = self.pop().as_object()?;

        let name: Result<QName, Error> = object.resolve_multiname(&multiname)?.ok_or_else(|| {
            format!("Could not resolve property {:?}", multiname.local_name()).into()
        });

        let value = object
            .get_property(object, &name?, self, context)?
            .resolve(self, context)?;
        self.push(value);

        Ok(())
    }

    fn op_set_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let value = self.pop();
        let multiname = self.pool_multiname(index)?;
        let mut object = self.pop().as_object()?;

        if let Some(name) = object.resolve_multiname(&multiname)? {
            object.set_property(object, &name, value, self, context)
        } else {
            //TODO: Non-dynamic objects should fail
            //TODO: This should only work if the public namespace is present
            let local_name: Result<&str, Error> = multiname
                .local_name()
                .ok_or_else(|| "Cannot set property using any name".into());
            let name = QName::dynamic_name(local_name?);
            object.set_property(object, &name, value, self, context)
        }
    }

    fn op_init_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let value = self.pop();
        let multiname = self.pool_multiname(index)?;
        let mut object = self.pop().as_object()?;

        if let Some(name) = object.resolve_multiname(&multiname)? {
            object.init_property(object, &name, value, self, context)
        } else {
            //TODO: Non-dynamic objects should fail
            //TODO: This should only work if the public namespace is present
            let local_name: Result<&str, Error> = multiname
                .local_name()
                .ok_or_else(|| "Cannot set property using any name".into());
            let name = QName::dynamic_name(local_name?);
            object.init_property(object, &name, value, self, context)
        }
    }

    fn op_delete_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let multiname = self.pool_multiname(index)?;
        let object = self.pop().as_object()?;

        if let Some(name) = object.resolve_multiname(&multiname)? {
            self.push(object.delete_property(context.gc_context, &name))
        } else {
            self.push(false)
        }

        Ok(())
    }

    fn op_get_super(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let multiname = self.pool_multiname(index)?;
        let object = self.pop().as_object()?;
        let base_proto: Result<Object<'gc>, Error> = self
            .current_stack_frame()
            .unwrap()
            .read()
            .base_proto()
            .and_then(|p| p.proto())
            .ok_or_else(|| "Attempted to get property on non-existent super object".into());
        let base_proto = base_proto?;
        let mut base = base_proto.construct(self, context, &[])?; //TODO: very hacky workaround

        let name: Result<QName, Error> = base.resolve_multiname(&multiname)?.ok_or_else(|| {
            format!(
                "Could not resolve {:?} as super property",
                multiname.local_name()
            )
            .into()
        });

        let value = base
            .get_property(object, &name?, self, context)?
            .resolve(self, context)?;

        self.push(value);

        Ok(())
    }

    fn op_set_super(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let value = self.pop();
        let multiname = self.pool_multiname(index)?;
        let object = self.pop().as_object()?;
        let base_proto: Result<Object<'gc>, Error> = self
            .current_stack_frame()
            .unwrap()
            .read()
            .base_proto()
            .and_then(|p| p.proto())
            .ok_or_else(|| "Attempted to get property on non-existent super object".into());
        let base_proto = base_proto?;
        let mut base = base_proto.construct(self, context, &[])?; //TODO: very hacky workaround

        let name: Result<QName, Error> = base.resolve_multiname(&multiname)?.ok_or_else(|| {
            format!(
                "Could not resolve {:?} as super property",
                multiname.local_name()
            )
            .into()
        });

        base.set_property(object, &name?, value, self, context)?;

        Ok(())
    }

    fn op_push_scope(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let object = self.pop().as_object()?;
        let activation = self.current_stack_frame().unwrap();
        let mut write = activation.write(context.gc_context);
        let scope_stack = write.scope();
        let new_scope = Scope::push_scope(scope_stack, object, context.gc_context);

        write.set_scope(Some(new_scope));

        Ok(())
    }

    fn op_push_with(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let object = self.pop().as_object()?;
        let activation = self.current_stack_frame().unwrap();
        let mut write = activation.write(context.gc_context);
        let scope_stack = write.scope();
        let new_scope = Scope::push_with(scope_stack, object, context.gc_context);

        write.set_scope(Some(new_scope));

        Ok(())
    }

    fn op_pop_scope(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        let activation = self.current_stack_frame().unwrap();
        let mut write = activation.write(context.gc_context);
        let scope_stack = write.scope();
        let new_scope = scope_stack.and_then(|s| s.read().pop_scope());

        write.set_scope(new_scope);

        Ok(())
    }

    fn op_get_scope_object(&mut self, mut index: u8) -> Result<(), Error> {
        let mut scope = self.current_stack_frame().unwrap().read().scope();

        while index > 0 {
            if let Some(child_scope) = scope {
                scope = child_scope.read().parent_cell();
            }

            index -= 1;
        }

        self.push(
            scope
                .map(|s| s.read().locals().clone().into())
                .unwrap_or(Value::Undefined),
        );

        Ok(())
    }

    fn op_get_global_scope(&mut self) -> Result<(), Error> {
        let mut scope = self.current_stack_frame().unwrap().read().scope();

        while let Some(this_scope) = scope {
            let parent = this_scope.read().parent_cell();
            if parent.is_none() {
                break;
            }

            scope = parent;
        }

        self.push(
            scope
                .map(|s| s.read().locals().clone().into())
                .unwrap_or(Value::Undefined),
        );

        Ok(())
    }

    fn op_find_property(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let multiname = self.pool_multiname(index)?;
        avm_debug!("Resolving {:?}", multiname);
        let result = if let Some(scope) = self.current_stack_frame().unwrap().read().scope() {
            scope.read().find(&multiname, self, context)?
        } else {
            None
        };

        self.push(result.map(|o| o.into()).unwrap_or(Value::Undefined));

        Ok(())
    }

    fn op_find_prop_strict(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let multiname = self.pool_multiname(index)?;
        avm_debug!("Resolving {:?}", multiname);
        let found: Result<Object<'gc>, Error> =
            if let Some(scope) = self.current_stack_frame().unwrap().read().scope() {
                scope.read().find(&multiname, self, context)?
            } else {
                None
            }
            .ok_or_else(|| format!("Property does not exist: {:?}", multiname.local_name()).into());
        let result: Value<'gc> = found?.into();

        self.push(result);

        Ok(())
    }

    fn op_get_lex(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
    ) -> Result<(), Error> {
        let multiname = self.pool_multiname_static(index)?;
        avm_debug!("Resolving {:?}", multiname);
        let found: Result<ReturnValue<'gc>, Error> =
            if let Some(scope) = self.current_stack_frame().unwrap().read().scope() {
                scope
                    .write(context.gc_context)
                    .resolve(&multiname, self, context)?
            } else {
                None
            }
            .ok_or_else(|| format!("Property does not exist: {:?}", multiname.local_name()).into());
        let result: Value<'gc> = found?.resolve(self, context)?;

        self.push(result);

        Ok(())
    }

    fn op_get_slot(&mut self, index: u32) -> Result<(), Error> {
        let object = self.pop().as_object()?;
        let value = object.get_slot(index)?;

        self.push(value);

        Ok(())
    }

    fn op_set_slot(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: u32,
    ) -> Result<(), Error> {
        let object = self.pop().as_object()?;
        let value = self.pop();

        object.set_slot(index, value, context.gc_context)
    }

    fn op_get_global_slot(&mut self, index: u32) -> Result<(), Error> {
        let value = self.globals.get_slot(index)?;

        self.push(value);

        Ok(())
    }

    fn op_set_global_slot(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: u32,
    ) -> Result<(), Error> {
        let value = self.pop();

        self.globals.set_slot(index, value, context.gc_context)
    }

    fn op_construct(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let mut ctor = self.pop().as_object()?;

        let proto = ctor
            .get_property(
                ctor,
                &QName::new(Namespace::public_namespace(), "prototype"),
                self,
                context,
            )?
            .resolve(self, context)?
            .as_object()?;

        let object = proto.construct(self, context, &args)?;
        ctor.call(Some(object), &args, self, context, object.proto())?
            .resolve(self, context)?;

        self.push(object);

        Ok(())
    }

    fn op_construct_prop(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let multiname = self.pool_multiname(index)?;
        let mut source = self.pop().as_object()?;

        let ctor_name: Result<QName, Error> =
            source.resolve_multiname(&multiname)?.ok_or_else(|| {
                format!("Could not resolve property {:?}", multiname.local_name()).into()
            });
        let mut ctor = source
            .get_property(source, &ctor_name?, self, context)?
            .resolve(self, context)?
            .as_object()?;
        let proto = ctor
            .get_property(
                ctor,
                &QName::new(Namespace::public_namespace(), "prototype"),
                self,
                context,
            )?
            .resolve(self, context)?
            .as_object()?;

        let object = proto.construct(self, context, &args)?;
        ctor.call(Some(object), &args, self, context, Some(proto))?
            .resolve(self, context)?;

        self.push(object);

        Ok(())
    }

    fn op_construct_super(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        arg_count: u32,
    ) -> Result<(), Error> {
        let args = self.pop_args(arg_count);
        let receiver = self.pop().as_object()?;
        let name = QName::new(Namespace::public_namespace(), "constructor");
        let base_proto: Result<Object<'gc>, Error> = self
            .current_stack_frame()
            .unwrap()
            .read()
            .base_proto()
            .and_then(|p| p.proto())
            .ok_or_else(|| {
                "Attempted to call super constructor without a superclass."
                    .to_string()
                    .into()
            });
        let mut base_proto = base_proto?;

        let function = base_proto
            .get_property(receiver, &name, self, context)?
            .resolve(self, context)?
            .as_object()?;

        function
            .call(Some(receiver), &args, self, context, Some(base_proto))?
            .resolve(self, context)?;

        Ok(())
    }

    fn op_new_activation(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) -> Result<(), Error> {
        self.push(ScriptObject::bare_object(context.gc_context));

        Ok(())
    }

    fn op_new_object(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        num_args: u32,
    ) -> Result<(), Error> {
        let mut object = ScriptObject::object(context.gc_context, self.system_prototypes.object);

        for _ in 0..num_args {
            let value = self.pop();
            let name = self.pop();

            object.set_property(
                object,
                &QName::new(Namespace::public_namespace(), name.as_string()?),
                value,
                self,
                context,
            )?;
        }

        self.push(object);

        Ok(())
    }

    fn op_new_function(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcMethod>,
    ) -> Result<(), Error> {
        let method_entry = self.table_method(index)?;
        let scope = self.current_stack_frame().unwrap().read().scope();

        let new_fn = FunctionObject::from_abc_method(
            context.gc_context,
            method_entry,
            scope,
            self.system_prototypes.function,
            None,
        );

        self.push(new_fn);

        Ok(())
    }

    fn op_new_class(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        index: Index<AbcClass>,
    ) -> Result<(), Error> {
        let base_class = self.pop().as_object()?;
        let class_entry = self.table_class(index)?;
        let scope = self.current_stack_frame().unwrap().read().scope();

        let (new_class, class_init) =
            FunctionObject::from_abc_class(self, context, class_entry, base_class, scope)?;

        class_init
            .call(Some(new_class), &[], self, context, None)?
            .resolve(self, context)?;

        self.push(new_class);

        Ok(())
    }

    fn op_coerce_a(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn op_jump(&mut self, offset: i32, reader: &mut Reader<Cursor<&[u8]>>) -> Result<(), Error> {
        reader.seek(offset as i64)?;

        Ok(())
    }

    fn op_if_true(&mut self, offset: i32, reader: &mut Reader<Cursor<&[u8]>>) -> Result<(), Error> {
        let value = self.pop().as_bool()?;

        if value {
            reader.seek(offset as i64)?;
        }

        Ok(())
    }

    fn op_if_false(
        &mut self,
        offset: i32,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        let value = self.pop().as_bool()?;

        if !value {
            reader.seek(offset as i64)?;
        }

        Ok(())
    }

    fn op_if_strict_eq(
        &mut self,
        offset: i32,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        let value2 = self.pop().as_bool()?;
        let value1 = self.pop().as_bool()?;

        if value1 == value2 {
            reader.seek(offset as i64)?;
        }

        Ok(())
    }

    fn op_if_strict_ne(
        &mut self,
        offset: i32,
        reader: &mut Reader<Cursor<&[u8]>>,
    ) -> Result<(), Error> {
        let value2 = self.pop().as_bool()?;
        let value1 = self.pop().as_bool()?;

        if value1 != value2 {
            reader.seek(offset as i64)?;
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn op_debug(
        &mut self,
        is_local_register: bool,
        register_name: Index<String>,
        register: u8,
    ) -> Result<(), Error> {
        if is_local_register {
            let register_name = self.pool_string(register_name)?;
            let value = self.register_value(register as u32)?;

            avm_debug!("Debug: {} = {:?}", register_name, value);
        } else {
            avm_debug!("Unknown debugging mode!");
        }

        Ok(())
    }

    #[allow(unused_variables)]
    fn op_debug_file(&mut self, file_name: Index<String>) -> Result<(), Error> {
        let file_name = self.pool_string(file_name)?;

        avm_debug!("File: {}", file_name);

        Ok(())
    }

    #[allow(unused_variables)]
    fn op_debug_line(&mut self, line_num: u32) -> Result<(), Error> {
        avm_debug!("Line: {}", line_num);

        Ok(())
    }
}
