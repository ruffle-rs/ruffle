//! ActionScript Virtual Machine 2 (AS3) support

use crate::avm2::activation::Activation;
use crate::avm2::value::Value;
use crate::context::UpdateContext;
use crate::tag_utils::SwfSlice;
use gc_arena::{Collect, GcCell};
use std::io::Cursor;
use std::rc::Rc;
use swf::avm2::read::Reader;
use swf::avm2::types::{AbcFile, Index, MethodBody, Op};
use swf::read::SwfRead;

mod activation;
mod function;
mod names;
mod object;
mod return_value;
mod script_object;
mod value;

macro_rules! avm_debug {
    ($($arg:tt)*) => (
        #[cfg(feature = "avm_debug")]
        log::debug!($($arg)*)
    )
}

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
}

impl<'gc> Avm2<'gc> {
    /// Construct a new AVM interpreter.
    pub fn new() -> Self {
        Self {
            stack_frames: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// Load an ABC file embedded in a `SwfSlice`.
    ///
    /// The `SwfSlice` must resolve to the contents of an ABC file.
    ///
    /// The `preload` flag indicates if the file is being encountered as part
    /// of a preloading operation. If false, then this file has actually been
    /// encountered as part of normal movie playback and it's final script
    /// should be executed.
    pub fn load_abc(
        &mut self,
        abc: SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
        preload: bool,
    ) -> Result<(), Error> {
        let mut read = Reader::new(abc.as_ref());

        let _abc_file = read.read()?;

        Ok(())
    }

    pub fn current_stack_frame(&self) -> Option<GcCell<'gc, Activation<'gc>>> {
        self.stack_frames.last().copied()
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
        let (frame_cell, action, pc) = {
            let frame = self
                .current_stack_frame()
                .ok_or("No stack frame to read!")?;
            let mut frame_ref = frame.write(context.gc_context);
            frame_ref.lock()?;

            (frame, frame_ref.action(), frame_ref.pc())
        };

        let abc = action.abc.as_ref();
        let method_index = action.abc_method;
        let method_body_index = action.abc_method_body as usize;
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

    /// Retrieve the current constant pool for the currently executing function.
    fn current_abc(&self) -> Option<Rc<AbcFile>> {
        self.current_stack_frame()
            .map(|sf| sf.read().action().abc.clone())
    }

    /// Retrieve a int from the current constant pool.
    fn pool_int(&self, index: Index<i32>) -> Result<i32, Error> {
        self.current_abc()
            .and_then(|abc| abc.constant_pool.ints.get(index.0 as usize).copied())
            .ok_or_else(|| format!("Unknown int constant {}", index.0).into())
    }

    /// Retrieve a int from the current constant pool.
    fn pool_uint(&self, index: Index<u32>) -> Result<u32, Error> {
        self.current_abc()
            .and_then(|abc| abc.constant_pool.uints.get(index.0 as usize).copied())
            .ok_or_else(|| format!("Unknown uint constant {}", index.0).into())
    }

    /// Retrieve a double from the current constant pool.
    fn pool_double(&self, index: Index<f64>) -> Result<f64, Error> {
        self.current_abc()
            .and_then(|abc| abc.constant_pool.doubles.get(index.0 as usize).copied())
            .ok_or_else(|| format!("Unknown double constant {}", index.0).into())
    }

    /// Retrieve a string from the current constant pool.
    fn pool_string(&self, index: Index<String>) -> Result<String, Error> {
        self.current_abc()
            .and_then(|abc| abc.constant_pool.strings.get(index.0 as usize).cloned())
            .ok_or_else(|| format!("Unknown string constant {}", index.0).into())
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
                Op::PushNaN => self.op_push_nan(),
                Op::PushNull => self.op_push_null(),
                Op::PushShort { value } => self.op_push_short(value),
                Op::PushString { value } => self.op_push_string(value),
                Op::PushTrue => self.op_push_true(),
                Op::PushUint { value } => self.op_push_uint(value),
                Op::PushUndefined => self.op_push_undefined(),
                _ => self.unknown_op(op),
            };

            if let Err(ref e) = result {
                log::error!("AVM2 error: {}", e);
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
}
