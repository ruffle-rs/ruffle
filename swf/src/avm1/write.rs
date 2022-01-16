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
            Action::Add => self.write_action_header(OpCode::Add, 0)?,
            Action::Add2 => self.write_action_header(OpCode::Add2, 0)?,
            Action::And => self.write_action_header(OpCode::And, 0)?,
            Action::AsciiToChar => self.write_action_header(OpCode::AsciiToChar, 0)?,
            Action::BitAnd => self.write_action_header(OpCode::BitAnd, 0)?,
            Action::BitLShift => self.write_action_header(OpCode::BitLShift, 0)?,
            Action::BitOr => self.write_action_header(OpCode::BitOr, 0)?,
            Action::BitRShift => self.write_action_header(OpCode::BitRShift, 0)?,
            Action::BitURShift => self.write_action_header(OpCode::BitURShift, 0)?,
            Action::BitXor => self.write_action_header(OpCode::BitXor, 0)?,
            Action::Call => self.write_action_header(OpCode::Call, 0)?,
            Action::CallFunction => self.write_action_header(OpCode::CallFunction, 0)?,
            Action::CallMethod => self.write_action_header(OpCode::CallMethod, 0)?,
            Action::CastOp => self.write_action_header(OpCode::CastOp, 0)?,
            Action::CharToAscii => self.write_action_header(OpCode::CharToAscii, 0)?,
            Action::CloneSprite => self.write_action_header(OpCode::CloneSprite, 0)?,
            Action::ConstantPool(action) => self.write_constant_pool(action)?,
            Action::Decrement => self.write_action_header(OpCode::Decrement, 0)?,
            Action::DefineFunction(action) => self.write_define_function(action)?,
            Action::DefineFunction2(action) => self.write_define_function_2(action)?,
            Action::DefineLocal => self.write_action_header(OpCode::DefineLocal, 0)?,
            Action::DefineLocal2 => self.write_action_header(OpCode::DefineLocal2, 0)?,
            Action::Divide => self.write_action_header(OpCode::Divide, 0)?,
            Action::Delete => self.write_action_header(OpCode::Delete, 0)?,
            Action::Delete2 => self.write_action_header(OpCode::Delete2, 0)?,
            Action::EndDrag => self.write_action_header(OpCode::EndDrag, 0)?,
            Action::Enumerate => self.write_action_header(OpCode::Enumerate, 0)?,
            Action::Enumerate2 => self.write_action_header(OpCode::Enumerate2, 0)?,
            Action::Equals => self.write_action_header(OpCode::Equals, 0)?,
            Action::Equals2 => self.write_action_header(OpCode::Equals2, 0)?,
            Action::Extends => self.write_action_header(OpCode::Extends, 0)?,
            Action::GetMember => self.write_action_header(OpCode::GetMember, 0)?,
            Action::GetProperty => self.write_action_header(OpCode::GetProperty, 0)?,
            Action::GetTime => self.write_action_header(OpCode::GetTime, 0)?,
            Action::GetUrl(action) => self.write_get_url(action)?,
            Action::GetUrl2(action) => self.write_get_url_2(action)?,
            Action::GetVariable => self.write_action_header(OpCode::GetVariable, 0)?,
            Action::GotoFrame(action) => self.write_goto_frame(action)?,
            Action::GotoFrame2(action) => self.write_goto_frame_2(action)?,
            Action::GotoLabel(action) => self.write_goto_label(action)?,
            Action::Greater => self.write_action_header(OpCode::Greater, 0)?,
            Action::If(action) => self.write_if(action)?,
            Action::ImplementsOp => self.write_action_header(OpCode::ImplementsOp, 0)?,
            Action::Increment => self.write_action_header(OpCode::Increment, 0)?,
            Action::InitArray => self.write_action_header(OpCode::InitArray, 0)?,
            Action::InitObject => self.write_action_header(OpCode::InitObject, 0)?,
            Action::InstanceOf => self.write_action_header(OpCode::InstanceOf, 0)?,
            Action::Jump(action) => self.write_jump(action)?,
            Action::Less => self.write_action_header(OpCode::Less, 0)?,
            Action::Less2 => self.write_action_header(OpCode::Less2, 0)?,
            Action::MBAsciiToChar => self.write_action_header(OpCode::MBAsciiToChar, 0)?,
            Action::MBCharToAscii => self.write_action_header(OpCode::MBCharToAscii, 0)?,
            Action::MBStringExtract => self.write_action_header(OpCode::MBStringExtract, 0)?,
            Action::MBStringLength => self.write_action_header(OpCode::MBStringLength, 0)?,
            Action::Modulo => self.write_action_header(OpCode::Modulo, 0)?,
            Action::Multiply => self.write_action_header(OpCode::Multiply, 0)?,
            Action::NewMethod => self.write_action_header(OpCode::NewMethod, 0)?,
            Action::NewObject => self.write_action_header(OpCode::NewObject, 0)?,
            Action::NextFrame => self.write_action_header(OpCode::NextFrame, 0)?,
            Action::Not => self.write_action_header(OpCode::Not, 0)?,
            Action::Or => self.write_action_header(OpCode::Or, 0)?,
            Action::Play => self.write_action_header(OpCode::Play, 0)?,
            Action::Pop => self.write_action_header(OpCode::Pop, 0)?,
            Action::PreviousFrame => self.write_action_header(OpCode::PreviousFrame, 0)?,
            Action::Push(action) => self.write_push(action)?,
            Action::PushDuplicate => self.write_action_header(OpCode::PushDuplicate, 0)?,
            Action::RandomNumber => self.write_action_header(OpCode::RandomNumber, 0)?,
            Action::RemoveSprite => self.write_action_header(OpCode::RemoveSprite, 0)?,
            Action::Return => self.write_action_header(OpCode::Return, 0)?,
            Action::SetMember => self.write_action_header(OpCode::SetMember, 0)?,
            Action::SetProperty => self.write_action_header(OpCode::SetProperty, 0)?,
            Action::SetTarget(action) => self.write_set_target(action)?,
            Action::SetTarget2 => self.write_action_header(OpCode::SetTarget2, 0)?,
            Action::SetVariable => self.write_action_header(OpCode::SetVariable, 0)?,
            Action::StackSwap => self.write_action_header(OpCode::StackSwap, 0)?,
            Action::StartDrag => self.write_action_header(OpCode::StartDrag, 0)?,
            Action::Stop => self.write_action_header(OpCode::Stop, 0)?,
            Action::StopSounds => self.write_action_header(OpCode::StopSounds, 0)?,
            Action::StoreRegister(action) => self.write_store_register(action)?,
            Action::StrictEquals => self.write_action_header(OpCode::StrictEquals, 0)?,
            Action::StringAdd => self.write_action_header(OpCode::StringAdd, 0)?,
            Action::StringEquals => self.write_action_header(OpCode::StringEquals, 0)?,
            Action::StringExtract => self.write_action_header(OpCode::StringExtract, 0)?,
            Action::StringGreater => self.write_action_header(OpCode::StringGreater, 0)?,
            Action::StringLength => self.write_action_header(OpCode::StringLength, 0)?,
            Action::StringLess => self.write_action_header(OpCode::StringLess, 0)?,
            Action::Subtract => self.write_action_header(OpCode::Subtract, 0)?,
            Action::TargetPath => self.write_action_header(OpCode::TargetPath, 0)?,
            Action::Throw => self.write_action_header(OpCode::Throw, 0)?,
            Action::ToggleQuality => self.write_action_header(OpCode::ToggleQuality, 0)?,
            Action::ToInteger => self.write_action_header(OpCode::ToInteger, 0)?,
            Action::ToNumber => self.write_action_header(OpCode::ToNumber, 0)?,
            Action::ToString => self.write_action_header(OpCode::ToString, 0)?,
            Action::Trace => self.write_action_header(OpCode::Trace, 0)?,
            Action::Try(action) => self.write_try(action)?,
            Action::TypeOf => self.write_action_header(OpCode::TypeOf, 0)?,
            Action::WaitForFrame(action) => self.write_wait_for_frame(action)?,
            Action::WaitForFrame2(action) => self.write_wait_for_frame_2(action)?,
            Action::With(action) => self.write_with(action)?,
            Action::Unknown(action) => self.write_unknown(action)?,
        }

        Ok(())
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
