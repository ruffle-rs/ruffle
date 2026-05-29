use crate::avm2::op::IntOp;

pub const MAX_INT_INTERPRETER_FRAME: usize = 16;

pub struct IntInterpreter {
    frame: [i32; MAX_INT_INTERPRETER_FRAME],
    stack_pointer: usize,
}

impl IntInterpreter {
    pub fn new() -> Self {
        Self {
            frame: [0; MAX_INT_INTERPRETER_FRAME],
            stack_pointer: 0,
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

    pub fn run(&mut self, opcodes: &[IntOp]) {
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
                IntOp::Nop => {}
                IntOp::PushInt { value } => self.op_push_int(*value),
                IntOp::SetLocal { index } => self.op_set_local(*index),
                IntOp::StoreLocal { index } => self.op_store_local(*index),
                IntOp::Subtract => self.op_subtract(),

                IntOp::End => {
                    break;
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

    fn op_get_local(&mut self, index: u32) {
        let value = self.frame_at(index);
        self.push_stack(value);
    }

    fn op_inc_local(&mut self, index: u32) {
        let value = self.frame_at(index);
        self.set_frame_at(index, value.wrapping_add(1));
    }

    fn op_push_int(&mut self, value: i32) {
        self.push_stack(value);
    }

    fn op_store_local(&mut self, index: u32) {
        let value = self.peek_stack();
        self.set_frame_at(index, value);
    }

    fn op_set_local(&mut self, index: u32) {
        let value = self.pop_stack();
        self.set_frame_at(index, value);
    }

    fn op_subtract(&mut self) {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();
        self.push_stack(value1.wrapping_sub(value2));
    }
}
