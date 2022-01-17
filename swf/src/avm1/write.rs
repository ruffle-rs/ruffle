use crate::avm1::opcode::OpCode;
use crate::avm1::types::*;
use crate::string::SwfStr;
use crate::write::SwfWriteExt;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Result, Write};

pub struct Writer<W: Write> {
    output: W,
    #[allow(dead_code)]
    version: u8,
}

impl<W: Write> SwfWriteExt for Writer<W> {
    #[inline]
    fn write_u8(&mut self, n: u8) -> Result<()> {
        self.output.write_u8(n)
    }

    #[inline]
    fn write_u16(&mut self, n: u16) -> Result<()> {
        self.output.write_u16::<LittleEndian>(n)
    }

    #[inline]
    fn write_u32(&mut self, n: u32) -> Result<()> {
        self.output.write_u32::<LittleEndian>(n)
    }

    #[inline]
    fn write_u64(&mut self, n: u64) -> Result<()> {
        self.output.write_u64::<LittleEndian>(n)
    }

    #[inline]
    fn write_i8(&mut self, n: i8) -> Result<()> {
        self.output.write_i8(n)
    }

    #[inline]
    fn write_i16(&mut self, n: i16) -> Result<()> {
        self.output.write_i16::<LittleEndian>(n)
    }

    #[inline]
    fn write_i32(&mut self, n: i32) -> Result<()> {
        self.output.write_i32::<LittleEndian>(n)
    }

    #[inline]
    fn write_f32(&mut self, n: f32) -> Result<()> {
        self.output.write_f32::<LittleEndian>(n)
    }

    #[inline]
    fn write_f64(&mut self, n: f64) -> Result<()> {
        self.output.write_f64::<LittleEndian>(n)
    }

    #[inline]
    fn write_string(&mut self, s: &'_ SwfStr) -> Result<()> {
        self.output.write_all(s.as_bytes())?;
        self.write_u8(0)
    }
}

impl<W: Write> Writer<W> {
    pub fn new(output: W, version: u8) -> Self {
        Self { output, version }
    }

    #[inline]
    fn write_f64_me(&mut self, n: f64) -> Result<()> {
        // Flash weirdly stores f64 as two LE 32-bit chunks.
        // First word is the hi-word, second word is the lo-word.
        self.write_u64(n.to_bits().rotate_left(32))
    }

    pub fn write_action(&mut self, action: &Action) -> Result<()> {
        match action {
            Action::Add => self.write_small_action(OpCode::Add),
            Action::Add2 => self.write_small_action(OpCode::Add2),
            Action::And => self.write_small_action(OpCode::And),
            Action::AsciiToChar => self.write_small_action(OpCode::AsciiToChar),
            Action::BitAnd => self.write_small_action(OpCode::BitAnd),
            Action::BitLShift => self.write_small_action(OpCode::BitLShift),
            Action::BitOr => self.write_small_action(OpCode::BitOr),
            Action::BitRShift => self.write_small_action(OpCode::BitRShift),
            Action::BitURShift => self.write_small_action(OpCode::BitURShift),
            Action::BitXor => self.write_small_action(OpCode::BitXor),
            Action::Call => self.write_small_action(OpCode::Call),
            Action::CallFunction => self.write_small_action(OpCode::CallFunction),
            Action::CallMethod => self.write_small_action(OpCode::CallMethod),
            Action::CastOp => self.write_small_action(OpCode::CastOp),
            Action::CharToAscii => self.write_small_action(OpCode::CharToAscii),
            Action::CloneSprite => self.write_small_action(OpCode::CloneSprite),
            Action::ConstantPool(action) => self.write_constant_pool(action),
            Action::Decrement => self.write_small_action(OpCode::Decrement),
            Action::DefineFunction(action) => self.write_define_function(action),
            Action::DefineFunction2(action) => self.write_define_function_2(action),
            Action::DefineLocal => self.write_small_action(OpCode::DefineLocal),
            Action::DefineLocal2 => self.write_small_action(OpCode::DefineLocal2),
            Action::Divide => self.write_small_action(OpCode::Divide),
            Action::Delete => self.write_small_action(OpCode::Delete),
            Action::Delete2 => self.write_small_action(OpCode::Delete2),
            Action::EndDrag => self.write_small_action(OpCode::EndDrag),
            Action::Enumerate => self.write_small_action(OpCode::Enumerate),
            Action::Enumerate2 => self.write_small_action(OpCode::Enumerate2),
            Action::Equals => self.write_small_action(OpCode::Equals),
            Action::Equals2 => self.write_small_action(OpCode::Equals2),
            Action::Extends => self.write_small_action(OpCode::Extends),
            Action::GetMember => self.write_small_action(OpCode::GetMember),
            Action::GetProperty => self.write_small_action(OpCode::GetProperty),
            Action::GetTime => self.write_small_action(OpCode::GetTime),
            Action::GetUrl(action) => self.write_get_url(action),
            Action::GetUrl2(action) => self.write_get_url_2(action),
            Action::GetVariable => self.write_small_action(OpCode::GetVariable),
            Action::GotoFrame(action) => self.write_goto_frame(action),
            Action::GotoFrame2(action) => self.write_goto_frame_2(action),
            Action::GotoLabel(action) => self.write_goto_label(action),
            Action::Greater => self.write_small_action(OpCode::Greater),
            Action::If(action) => self.write_if(action),
            Action::ImplementsOp => self.write_small_action(OpCode::ImplementsOp),
            Action::Increment => self.write_small_action(OpCode::Increment),
            Action::InitArray => self.write_small_action(OpCode::InitArray),
            Action::InitObject => self.write_small_action(OpCode::InitObject),
            Action::InstanceOf => self.write_small_action(OpCode::InstanceOf),
            Action::Jump(action) => self.write_jump(action),
            Action::Less => self.write_small_action(OpCode::Less),
            Action::Less2 => self.write_small_action(OpCode::Less2),
            Action::MBAsciiToChar => self.write_small_action(OpCode::MBAsciiToChar),
            Action::MBCharToAscii => self.write_small_action(OpCode::MBCharToAscii),
            Action::MBStringExtract => self.write_small_action(OpCode::MBStringExtract),
            Action::MBStringLength => self.write_small_action(OpCode::MBStringLength),
            Action::Modulo => self.write_small_action(OpCode::Modulo),
            Action::Multiply => self.write_small_action(OpCode::Multiply),
            Action::NewMethod => self.write_small_action(OpCode::NewMethod),
            Action::NewObject => self.write_small_action(OpCode::NewObject),
            Action::NextFrame => self.write_small_action(OpCode::NextFrame),
            Action::Not => self.write_small_action(OpCode::Not),
            Action::Or => self.write_small_action(OpCode::Or),
            Action::Play => self.write_small_action(OpCode::Play),
            Action::Pop => self.write_small_action(OpCode::Pop),
            Action::PreviousFrame => self.write_small_action(OpCode::PreviousFrame),
            Action::Push(action) => self.write_push(action),
            Action::PushDuplicate => self.write_small_action(OpCode::PushDuplicate),
            Action::RandomNumber => self.write_small_action(OpCode::RandomNumber),
            Action::RemoveSprite => self.write_small_action(OpCode::RemoveSprite),
            Action::Return => self.write_small_action(OpCode::Return),
            Action::SetMember => self.write_small_action(OpCode::SetMember),
            Action::SetProperty => self.write_small_action(OpCode::SetProperty),
            Action::SetTarget(action) => self.write_set_target(action),
            Action::SetTarget2 => self.write_small_action(OpCode::SetTarget2),
            Action::SetVariable => self.write_small_action(OpCode::SetVariable),
            Action::StackSwap => self.write_small_action(OpCode::StackSwap),
            Action::StartDrag => self.write_small_action(OpCode::StartDrag),
            Action::Stop => self.write_small_action(OpCode::Stop),
            Action::StopSounds => self.write_small_action(OpCode::StopSounds),
            Action::StoreRegister(action) => self.write_store_register(action),
            Action::StrictEquals => self.write_small_action(OpCode::StrictEquals),
            Action::StringAdd => self.write_small_action(OpCode::StringAdd),
            Action::StringEquals => self.write_small_action(OpCode::StringEquals),
            Action::StringExtract => self.write_small_action(OpCode::StringExtract),
            Action::StringGreater => self.write_small_action(OpCode::StringGreater),
            Action::StringLength => self.write_small_action(OpCode::StringLength),
            Action::StringLess => self.write_small_action(OpCode::StringLess),
            Action::Subtract => self.write_small_action(OpCode::Subtract),
            Action::TargetPath => self.write_small_action(OpCode::TargetPath),
            Action::Throw => self.write_small_action(OpCode::Throw),
            Action::ToggleQuality => self.write_small_action(OpCode::ToggleQuality),
            Action::ToInteger => self.write_small_action(OpCode::ToInteger),
            Action::ToNumber => self.write_small_action(OpCode::ToNumber),
            Action::ToString => self.write_small_action(OpCode::ToString),
            Action::Trace => self.write_small_action(OpCode::Trace),
            Action::Try(action) => self.write_try(action),
            Action::TypeOf => self.write_small_action(OpCode::TypeOf),
            Action::WaitForFrame(action) => self.write_wait_for_frame(action),
            Action::WaitForFrame2(action) => self.write_wait_for_frame_2(action),
            Action::With(action) => self.write_with(action),
            Action::Unknown(action) => self.write_unknown(action),
        }
    }

    pub fn write_action_header(&mut self, opcode: OpCode, length: usize) -> Result<()> {
        self.write_opcode_and_length(opcode as u8, length)
    }

    pub fn write_opcode_and_length(&mut self, opcode: u8, length: usize) -> Result<()> {
        self.write_u8(opcode)?;
        assert!(
            opcode >= 0x80 || length == 0,
            "Opcodes less than 0x80 must have length 0"
        );
        if opcode >= 0x80 {
            self.write_u16(length as u16)?;
        }
        Ok(())
    }

    /// Writes an action that has no payload.
    fn write_small_action(&mut self, opcode: OpCode) -> Result<()> {
        self.write_action_header(opcode, 0)
    }

    fn write_constant_pool(&mut self, action: &ConstantPool) -> Result<()> {
        let len = 2 + action.strings.iter().map(|c| c.len() + 1).sum::<usize>();
        self.write_action_header(OpCode::ConstantPool, len)?;
        self.write_u16(action.strings.len() as u16)?;
        for string in &action.strings {
            self.write_string(string)?;
        }
        Ok(())
    }

    fn write_define_function(&mut self, action: &DefineFunction) -> Result<()> {
        // 1 zero byte for string name, 1 zero byte per param, 2 bytes for # of params,
        // 2 bytes for code length
        let len = action.name.len()
            + 1
            + 2
            + action.params.iter().map(|p| p.len() + 1).sum::<usize>()
            + 2;
        self.write_action_header(OpCode::DefineFunction, len)?;
        self.write_string(action.name)?;
        self.write_u16(action.params.len() as u16)?;
        for param in &action.params {
            self.write_string(param)?;
        }
        self.write_u16(action.actions.len() as u16)?;
        self.output.write_all(action.actions)?;
        Ok(())
    }

    fn write_define_function_2(&mut self, action: &DefineFunction2) -> Result<()> {
        let len = action.name.len()
            + 1
            + 3
            + action
                .params
                .iter()
                .map(|p| p.name.len() + 2)
                .sum::<usize>()
            + 4;
        self.write_action_header(OpCode::DefineFunction2, len)?;
        self.write_string(action.name)?;
        self.write_u16(action.params.len() as u16)?;
        self.write_u8(action.register_count)?;
        self.write_u16(action.flags.bits())?;
        for param in &action.params {
            self.write_u8(if let Some(n) = param.register_index {
                n
            } else {
                0
            })?;
            self.write_string(param.name)?;
        }
        self.write_u16(action.actions.len() as u16)?;
        self.output.write_all(action.actions)?;
        Ok(())
    }

    fn write_get_url(&mut self, action: &GetUrl) -> Result<()> {
        self.write_action_header(OpCode::GetUrl, action.url.len() + action.target.len() + 2)?;
        self.write_string(action.url)?;
        self.write_string(action.target)?;
        Ok(())
    }

    fn write_get_url_2(&mut self, action: &GetUrl2) -> Result<()> {
        self.write_action_header(OpCode::GetUrl2, 1)?;
        let flags = (match action.send_vars_method {
            SendVarsMethod::None => 0,
            SendVarsMethod::Get => 1,
            SendVarsMethod::Post => 2,
        }) | if action.is_target_sprite {
            0b01_0000_00
        } else {
            0
        } | if action.is_load_vars { 0b10_0000_00 } else { 0 };
        self.write_u8(flags)?;
        Ok(())
    }

    fn write_goto_frame(&mut self, action: &GotoFrame) -> Result<()> {
        self.write_action_header(OpCode::GotoFrame, 2)?;
        self.write_u16(action.frame)?;
        Ok(())
    }

    fn write_goto_frame_2(&mut self, action: &GotoFrame2) -> Result<()> {
        if action.scene_offset != 0 {
            self.write_action_header(OpCode::GotoFrame2, 3)?;
            self.write_u8(if action.set_playing { 0b11 } else { 0b01 })?;
            self.write_u16(action.scene_offset)?;
        } else {
            self.write_action_header(OpCode::GotoFrame2, 1)?;
            self.write_u8(if action.set_playing { 0b10 } else { 0b00 })?;
        }
        Ok(())
    }

    fn write_goto_label(&mut self, action: &GotoLabel) -> Result<()> {
        self.write_action_header(OpCode::GotoLabel, action.label.len() + 1)?;
        self.write_string(action.label)?;
        Ok(())
    }

    fn write_if(&mut self, action: &If) -> Result<()> {
        self.write_action_header(OpCode::If, 2)?;
        self.write_i16(action.offset)?;
        Ok(())
    }

    fn write_jump(&mut self, action: &Jump) -> Result<()> {
        self.write_action_header(OpCode::Jump, 2)?;
        self.write_i16(action.offset)?;
        Ok(())
    }

    fn write_push(&mut self, action: &Push) -> Result<()> {
        let len = action
            .values
            .iter()
            .map(|v| match *v {
                Value::Str(string) => string.len() + 2,
                Value::Null | Value::Undefined => 1,
                Value::Register(_) | Value::Bool(_) => 2,
                Value::Double(_) => 9,
                Value::Float(_) | Value::Int(_) => 5,
                Value::ConstantPool(v) => {
                    if v < 256 {
                        2
                    } else {
                        3
                    }
                }
            })
            .sum();
        self.write_action_header(OpCode::Push, len)?;
        for value in &action.values {
            self.write_push_value(value)?;
        }
        Ok(())
    }

    fn write_push_value(&mut self, value: &Value) -> Result<()> {
        match *value {
            Value::Str(string) => {
                self.write_u8(0)?;
                self.write_string(string)?;
            }
            Value::Float(v) => {
                self.write_u8(1)?;
                self.write_f32(v)?;
            }
            Value::Null => {
                self.write_u8(2)?;
            }
            Value::Undefined => {
                self.write_u8(3)?;
            }
            Value::Register(v) => {
                self.write_u8(4)?;
                self.write_u8(v)?;
            }
            Value::Bool(v) => {
                self.write_u8(5)?;
                self.write_u8(v as u8)?;
            }
            Value::Double(v) => {
                self.write_u8(6)?;
                self.write_f64_me(v)?;
            }
            Value::Int(v) => {
                self.write_u8(7)?;
                self.write_i32(v)?;
            }
            Value::ConstantPool(v) => {
                if v < 256 {
                    self.write_u8(8)?;
                    self.write_u8(v as u8)?;
                } else {
                    self.write_u8(9)?;
                    self.write_u16(v)?;
                }
            }
        };
        Ok(())
    }

    fn write_set_target(&mut self, action: &SetTarget) -> Result<()> {
        self.write_action_header(OpCode::SetTarget, action.target.len() + 1)?;
        self.write_string(action.target)?;
        Ok(())
    }

    fn write_store_register(&mut self, action: &StoreRegister) -> Result<()> {
        self.write_action_header(OpCode::StoreRegister, 1)?;
        self.write_u8(action.register)?;
        Ok(())
    }

    fn write_try(&mut self, action: &Try) -> Result<()> {
        let len = 7 + if let Some((CatchVar::Var(name), _)) = action.catch_body {
            name.len() + 1
        } else {
            1
        };
        self.write_action_header(OpCode::Try, len)?;

        let mut flags = TryFlags::empty();
        flags.set(TryFlags::CATCH_BLOCK, action.catch_body.is_some());
        flags.set(TryFlags::FINALLY_BLOCK, action.finally_body.is_some());
        flags.set(
            TryFlags::CATCH_IN_REGISTER,
            matches!(action.catch_body, Some((CatchVar::Register(_), _))),
        );
        self.write_u8(flags.bits())?;

        let try_size = action.try_body.len();
        self.write_u16(try_size as u16)?;

        let catch_size = action
            .catch_body
            .as_ref()
            .map_or(0, |(_, catch_body)| catch_body.len());
        self.write_u16(catch_size as u16)?;

        let finally_size = action
            .finally_body
            .map_or(0, |finally_body| finally_body.len());
        self.write_u16(finally_size as u16)?;

        match action.catch_body {
            Some((CatchVar::Var(name), _)) => self.write_string(name)?,
            Some((CatchVar::Register(i), _)) => self.write_u8(i)?,
            None => self.write_u8(0)?,
        }

        self.output.write_all(action.try_body)?;

        if let Some((_, catch_body)) = action.catch_body {
            self.output.write_all(catch_body)?;
        }

        if let Some(finally_body) = action.finally_body {
            self.output.write_all(finally_body)?;
        }
        Ok(())
    }

    fn write_wait_for_frame(&mut self, action: &WaitForFrame) -> Result<()> {
        self.write_action_header(OpCode::WaitForFrame, 3)?;
        self.write_u16(action.frame)?;
        self.write_u8(action.num_actions_to_skip)?;
        Ok(())
    }

    fn write_wait_for_frame_2(&mut self, action: &WaitForFrame2) -> Result<()> {
        self.write_action_header(OpCode::WaitForFrame2, 1)?;
        self.write_u8(action.num_actions_to_skip)?;
        Ok(())
    }

    fn write_with(&mut self, action: &With) -> Result<()> {
        self.write_action_header(OpCode::With, 2)?;
        self.write_u16(action.actions.len() as u16)?;
        self.output.write_all(action.actions)?;
        Ok(())
    }

    fn write_unknown(&mut self, action: &Unknown) -> Result<()> {
        self.write_opcode_and_length(action.opcode, action.data.len())?;
        self.output.write_all(action.data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data;

    #[test]
    fn write_action() {
        for (swf_version, action, expected_bytes) in test_data::avm1_tests() {
            let mut written_bytes = Vec::new();
            Writer::new(&mut written_bytes, swf_version)
                .write_action(&action)
                .unwrap();
            assert_eq!(
                written_bytes, expected_bytes,
                "Error writing action.\nTag:\n{:?}\n\nWrote:\n{:?}\n\nExpected:\n{:?}",
                action, written_bytes, expected_bytes
            );
        }
    }
}
