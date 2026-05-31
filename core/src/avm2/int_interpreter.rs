use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::object::{Object, TObject};
use crate::avm2::op::IntOp;

use enum_map::{Enum, EnumMap};
use gc_arena::Mutation;
use num_traits::FromPrimitive;
use std::cell::RefMut;

/// The maximum number of items on the interpreter frame. This value should be
/// less than the number of bits that `avm2::optimizer::utils::SmallBitSet` can
/// store. NOTE(uqers): Synchronizing a large number of locals/stack from the
/// int interpreter to the normal interpreter can have high performance costs!
/// Increasing this may result in worse overall performance in some SWFs.
pub const MAX_INT_INTERPRETER_FRAME: usize = 36;

#[derive(Clone, Copy, Debug, Enum, FromPrimitive, PartialEq)]
pub enum ObjectType {
    TopOuterScope = 0,
    Receiver = 1,
}

pub struct DomainMemoryError;

pub struct IntInterpreter<'a, 'gc: 'a> {
    /// The frame, which stores both the locals and the stack.
    frame: [i32; MAX_INT_INTERPRETER_FRAME],

    /// The current stack pointer.
    stack_pointer: usize,

    /// Objects that code from the int interpreter can use.
    objects: EnumMap<ObjectType, Option<Object<'gc>>>,

    /// The domain memory, pre-borrowed for speed.
    domain_memory: RefMut<'a, ByteArrayStorage>,

    mc: &'gc Mutation<'gc>,
}

impl<'a, 'gc> IntInterpreter<'a, 'gc> {
    pub fn new(
        mc: &'gc Mutation<'gc>,
        domain_memory: RefMut<'a, ByteArrayStorage>,
        objects: EnumMap<ObjectType, Option<Object<'gc>>>,
    ) -> Self {
        Self {
            frame: [0; MAX_INT_INTERPRETER_FRAME],
            stack_pointer: 0,
            objects,
            domain_memory,
            mc,
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

    #[inline(never)]
    pub fn run(&mut self, opcodes: &[IntOp]) -> Result<(usize, usize), DomainMemoryError> {
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
                IntOp::Equals => self.op_equals(),
                IntOp::GetLocal { index } => self.op_get_local(*index),
                IntOp::GetSlot { index } => self.op_get_slot(*index),
                IntOp::GreaterEquals => self.op_greater_equals(),
                IntOp::GreaterThan => self.op_greater_than(),
                IntOp::IncLocal { index } => self.op_inc_local(*index),
                IntOp::LessEquals => self.op_less_equals(),
                IntOp::LessThan => self.op_less_than(),
                IntOp::Li16 => self.op_li16()?,
                IntOp::Li32 => self.op_li32()?,
                IntOp::Li8 => self.op_li8()?,
                IntOp::LShift => self.op_lshift(),
                IntOp::MultiplyNumbers => self.op_multiply_numbers(),
                IntOp::Nop => {}
                IntOp::Not => self.op_not(),
                IntOp::Pop => self.op_pop(),
                IntOp::PushInt { value } => self.op_push_int(*value),
                IntOp::PushObject { value } => self.op_push_int(*value as i32),
                IntOp::RShift => self.op_rshift(),
                IntOp::SetLocal { index } => self.op_set_local(*index),
                IntOp::SetSlot { index } => self.op_set_slot(*index),
                IntOp::Si16 => self.op_si16()?,
                IntOp::Si32 => self.op_si32()?,
                IntOp::Si8 => self.op_si8()?,
                IntOp::StoreLocal { index } => self.op_store_local(*index),
                IntOp::Subtract => self.op_subtract(),
                IntOp::Swap => self.op_swap(),
                IntOp::Sxi16 => self.op_sxi16(),
                IntOp::Sxi8 => self.op_sxi8(),
                IntOp::URShift => self.op_urshift(),

                IntOp::IfFalse { offset } => {
                    if self.pop_stack() == 0 {
                        ip = *offset as usize;
                    }
                }
                IntOp::IfFalseExternal {
                    offset,
                    final_stack_height,
                } => {
                    if self.pop_stack() == 0 {
                        // Jump back into the normal interpreter
                        return Ok((offset.get() as usize, *final_stack_height as usize));
                    }
                }
                IntOp::IfTrue { offset } => {
                    if self.pop_stack() != 0 {
                        ip = *offset as usize;
                    }
                }
                IntOp::IfTrueExternal {
                    offset,
                    final_stack_height,
                } => {
                    if self.pop_stack() != 0 {
                        // Jump back into the normal interpreter
                        return Ok((offset.get() as usize, *final_stack_height as usize));
                    }
                }
                IntOp::Jump { offset } => {
                    ip = *offset as usize;
                }
                IntOp::JumpExternal {
                    offset,
                    final_stack_height,
                } => {
                    // Jump back into the normal interpreter
                    return Ok((offset.get() as usize, *final_stack_height as usize));
                }
            }
        }
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

    fn op_equals(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1 == value2;

        self.push_stack(result as i32);
    }

    fn op_get_local(&mut self, index: u32) {
        let value = self.frame_at(index);
        self.push_stack(value);
    }

    fn op_get_slot(&mut self, index: u32) {
        let object_type = self.pop_stack();
        let object_type = ObjectType::from_i32(object_type).expect("Is a valid object type");

        let object =
            self.objects[object_type].expect("Guaranteed by int interpreter promotion analysis");

        let value = object.get_slot(index as usize);

        self.push_stack(value.as_i32());
    }

    fn op_greater_equals(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1 >= value2;

        self.push_stack(result as i32);
    }

    fn op_greater_than(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1 > value2;

        self.push_stack(result as i32);
    }

    fn op_inc_local(&mut self, index: u32) {
        let value = self.frame_at(index);
        self.set_frame_at(index, value.wrapping_add(1));
    }

    fn op_less_equals(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1 <= value2;

        self.push_stack(result as i32);
    }

    fn op_less_than(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1 < value2;

        self.push_stack(result as i32);
    }

    fn op_li16(&mut self) -> Result<(), DomainMemoryError> {
        // See `Activation::op_li16` for an explanation
        let address = self.pop_stack() as usize;

        let dm = &self.domain_memory;

        if address > dm.len() - 2 {
            return Err(DomainMemoryError);
        }

        let val = dm.read_at(2, address).expect("Already checked");
        self.push_stack(u16::from_le_bytes(val.try_into().unwrap()) as i32);

        Ok(())
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

    fn op_lshift(&mut self) {
        let value2 = self.pop_stack() as u32;
        let value1 = self.pop_stack();

        self.push_stack(value1 << (value2 & 0x1F));
    }

    fn op_multiply_numbers(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = if let Some(result) = value1.checked_mul(value2) {
            result
        } else {
            crate::ecma_conversions::f64_to_wrapping_i32(value1 as f64 * value2 as f64)
        };

        self.push_stack(result);
    }

    fn op_not(&mut self) {
        let value = self.pop_stack();

        if value == 0 {
            self.push_stack(1);
        } else {
            self.push_stack(0);
        }
    }

    fn op_pop(&mut self) {
        self.pop_stack();
    }

    fn op_push_int(&mut self, value: i32) {
        self.push_stack(value);
    }

    fn op_rshift(&mut self) {
        let value2 = self.pop_stack() as u32;
        let value1 = self.pop_stack();

        self.push_stack(value1 >> (value2 & 0x1F));
    }

    fn op_set_local(&mut self, index: u32) {
        let value = self.pop_stack();
        self.set_frame_at(index, value);
    }

    fn op_set_slot(&mut self, index: u32) {
        let value = self.pop_stack();

        let object_type = self.pop_stack();
        let object_type = ObjectType::from_i32(object_type).expect("Is a valid object type");

        let object =
            self.objects[object_type].expect("Guaranteed by int interpreter promotion analysis");

        object.set_slot_no_coerce(index as usize, value.into(), self.mc);
    }

    fn op_si16(&mut self) -> Result<(), DomainMemoryError> {
        // See `Activation::op_si16` for an explanation
        let address = self.pop_stack() as usize;

        let val = self.pop_stack() as i16;

        let dm = &mut self.domain_memory;

        if address > dm.len() - 2 {
            return Err(DomainMemoryError);
        }

        dm.write_at_nongrowing(&val.to_le_bytes(), address)
            .expect("Already checked");

        Ok(())
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

    fn op_swap(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        self.push_stack(value2);
        self.push_stack(value1);
    }

    fn op_sxi16(&mut self) {
        let val = self.pop_stack();

        let val = (val.wrapping_shl(15).wrapping_shr(15) & 0xFFFF) as i16 as i32;

        self.push_stack(val);
    }

    fn op_sxi8(&mut self) {
        let val = self.pop_stack();

        let val = (val.wrapping_shl(23).wrapping_shr(23) & 0xFF) as i8 as i32;

        self.push_stack(val);
    }

    fn op_urshift(&mut self) {
        let value2 = self.pop_stack() as u32;
        let value1 = self.pop_stack() as u32;

        self.push_stack((value1 >> (value2 & 0x1F)) as i32);
    }
}
