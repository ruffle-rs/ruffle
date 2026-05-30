use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::op::IntOp;

use std::cell::RefMut;

/// The largest method frame size supported by the int interpreter.
///
/// NOTE: This value must be smaller than `u32::BITS`, as that is the most bits
/// that `optimizer::utils::SmallBitSet` can store!
pub const MAX_INT_INTERPRETER_FRAME: usize = 16;

pub struct DomainMemoryError;

pub struct IntInterpreter<'a> {
    /// The frame, which stores both the locals and the stack.
    frame: [i32; MAX_INT_INTERPRETER_FRAME],

    /// The current stack pointer.
    stack_pointer: usize,

    /// The domain memory, pre-borrowed for speed.
    domain_memory: RefMut<'a, ByteArrayStorage>,
}

impl<'a> IntInterpreter<'a> {
    pub fn new(domain_memory: RefMut<'a, ByteArrayStorage>) -> Self {
        Self {
            frame: [0; MAX_INT_INTERPRETER_FRAME],
            stack_pointer: 0,
            domain_memory,
        }
    }

    pub fn push_stack(&mut self, value: i32) {
        self.frame[self.stack_pointer] = value;
        self.stack_pointer += 1;
    }

    pub fn peek_stack(&mut self) -> i32 {
        self.frame[self.stack_pointer - 1]
    }

    pub fn pop_stack(&mut self) -> i32 {
        self.stack_pointer -= 1;
        self.frame[self.stack_pointer]
    }

    pub fn frame_at(&mut self, index: u32) -> i32 {
        self.frame[index as usize]
    }

    pub fn set_frame_at(&mut self, index: u32, value: i32) {
        self.frame[index as usize] = value;
    }

    pub fn run(&mut self, opcodes: &[IntOp]) -> Result<(), DomainMemoryError> {
        let mut ip = 0;

        loop {
            let op = &opcodes[ip];
            ip += 1;

            match op {
                IntOp::Add => self.op_add(),
                IntOp::BitAnd => self.op_bitand(),
                IntOp::BitNot => self.op_bitnot(),
                IntOp::BitOr => self.op_bitor(),
                IntOp::BitXor => self.op_bitxor(),
                IntOp::DecLocal { index } => self.op_dec_local(*index),
                IntOp::Dup => self.op_dup(),
                IntOp::GetLocal { index } => self.op_get_local(*index),
                IntOp::IncLocal { index } => self.op_inc_local(*index),
                IntOp::Li32 => self.op_li32()?,
                IntOp::Li8 => self.op_li8()?,
                IntOp::Nop => {}
                IntOp::PushInt { value } => self.op_push_int(*value),
                IntOp::SetLocal { index } => self.op_set_local(*index),
                IntOp::Si32 => self.op_si32()?,
                IntOp::Si8 => self.op_si8()?,
                IntOp::StoreLocal { index } => self.op_store_local(*index),
                IntOp::Subtract => self.op_subtract(),

                IntOp::End => {
                    break;
                }
            }
        }

        Ok(())
    }

    fn op_add(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();
        self.push_stack(value1.wrapping_add(value2));
    }

    fn op_bitand(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();
        self.push_stack(value1 & value2);
    }

    fn op_bitnot(&mut self) {
        let value = self.pop_stack();
        self.push_stack(!value);
    }

    fn op_bitor(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();
        self.push_stack(value1 | value2);
    }

    fn op_bitxor(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();
        self.push_stack(value1 ^ value2);
    }

    fn op_dec_local(&mut self, index: u32) {
        let value = self.frame_at(index);
        self.set_frame_at(index, value.wrapping_sub(1));
    }

    fn op_dup(&mut self) {
        let value = self.peek_stack();
        self.push_stack(value);
    }

    fn op_get_local(&mut self, index: u32) {
        let value = self.frame_at(index);
        self.push_stack(value);
    }

    fn op_inc_local(&mut self, index: u32) {
        let value = self.frame_at(index);
        self.set_frame_at(index, value.wrapping_add(1));
    }

    fn op_li32(&mut self) -> Result<(), DomainMemoryError> {
        // See `Activation::op_li32` for an explanation
        let address = self.pop_stack() as usize;

        let dm = &self.domain_memory;

        if address > dm.len() - 4 {
            return Err(DomainMemoryError);
        }

        let val = dm.read_at(4, address).expect("Already checked");
        self.push_stack(i32::from_le_bytes(val.try_into().unwrap()));

        Ok(())
    }

    fn op_li8(&mut self) -> Result<(), DomainMemoryError> {
        // See `Activation::op_li8` for an explanation
        let address = self.pop_stack() as usize;

        let dm = &self.domain_memory;

        let val = dm.get(address);

        if let Some(val) = val {
            self.push_stack(val as i32);
        } else {
            return Err(DomainMemoryError);
        }

        Ok(())
    }

    fn op_push_int(&mut self, value: i32) {
        self.push_stack(value);
    }

    fn op_set_local(&mut self, index: u32) {
        let value = self.pop_stack();
        self.set_frame_at(index, value);
    }

    fn op_si32(&mut self) -> Result<(), DomainMemoryError> {
        // See `Activation::op_si32` for an explanation
        let address = self.pop_stack() as usize;

        let val = self.pop_stack();

        let dm = &mut self.domain_memory;

        if address > dm.len() - 4 {
            return Err(DomainMemoryError);
        }

        dm.write_at_nongrowing(&val.to_le_bytes(), address)
            .expect("Already checked");

        Ok(())
    }

    fn op_si8(&mut self) -> Result<(), DomainMemoryError> {
        // See `Activation::op_si8` for an explanation
        let address = self.pop_stack() as usize;

        let val = self.pop_stack() as i8;

        let dm = &mut self.domain_memory;

        if address >= dm.len() {
            return Err(DomainMemoryError);
        }

        dm.set_nongrowing(address, val as u8);

        Ok(())
    }

    fn op_store_local(&mut self, index: u32) {
        let value = self.peek_stack();
        self.set_frame_at(index, value);
    }

    fn op_subtract(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();
        self.push_stack(value1.wrapping_sub(value2));
    }
}
