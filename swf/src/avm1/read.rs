use crate::avm1::{opcode::OpCode, types::*};
use crate::error::{Error, Result};
use crate::extensions::ReadSwfExt;
use std::num::NonZeroU8;

pub struct Reader<'a> {
    input: &'a [u8],
    #[allow(dead_code)]
    version: u8,
}

impl<'a> ReadSwfExt<'a> for Reader<'a> {
    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut &'a [u8] {
        &mut self.input
    }

    #[inline(always)]
    fn as_slice(&self) -> &'a [u8] {
        self.input
    }
}

impl<'a> Reader<'a> {
    #[inline]
    pub const fn new(input: &'a [u8], version: u8) -> Self {
        Self { input, version }
    }

    #[inline]
    pub fn seek(&mut self, data: &'a [u8], jump_offset: i16) {
        ReadSwfExt::seek(self, data, jump_offset as isize)
    }

    #[inline]
    pub const fn get_ref(&self) -> &'a [u8] {
        self.input
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut &'a [u8] {
        &mut self.input
    }

    #[inline]
    fn read_f64_me(&mut self) -> Result<f64> {
        // Flash weirdly stores (some?) f64 as two LE 32-bit chunks.
        // First word is the hi-word, second word is the lo-word.
        Ok(f64::from_bits(self.read_u64()?.rotate_left(32)))
    }

    #[inline]
    pub fn read_action(&mut self) -> Result<Action<'a>> {
        let (opcode, mut length) = self.read_opcode_and_length()?;
        let start = self.input;

        let action = self.read_op(opcode, &mut length);

        if let Err(e) = action {
            return Err(Error::avm1_parse_error_with_source(opcode, e));
        }

        // Verify that we parsed the correct amount of data.
        let end_pos = (start.as_ptr() as usize + length) as *const u8;
        if self.input.as_ptr() != end_pos {
            // We incorrectly parsed this action.
            // Re-sync to the expected end of the action and throw an error.
            self.input = &start[length.min(start.len())..];
            log::warn!("Length mismatch in AVM1 action: {}", OpCode::format(opcode));
        }

        action
    }

    pub fn read_opcode_and_length(&mut self) -> Result<(u8, usize)> {
        let opcode = self.read_u8()?;
        let length = if opcode >= 0x80 {
            self.read_u16()?.into()
        } else {
            0
        };
        Ok((opcode, length))
    }

    /// Reads an action with the given opcode.
    /// `length` is an in-out parameter and will be modified in the case of instructions
    /// that contain sub-blocks of code, such as `DefineFunction`.
    /// The `length` passed in should be the length excluding any sub-blocks.
    /// The final `length` returned will be total length of the action, including sub-blocks.
    #[inline]
    fn read_op(&mut self, opcode: u8, length: &mut usize) -> Result<Action<'a>> {
        let action = if let Some(op) = OpCode::from_u8(opcode) {
            match op {
                OpCode::Add => Action::Add,
                OpCode::Add2 => Action::Add2,
                OpCode::And => Action::And,
                OpCode::AsciiToChar => Action::AsciiToChar,
                OpCode::BitAnd => Action::BitAnd,
                OpCode::BitLShift => Action::BitLShift,
                OpCode::BitOr => Action::BitOr,
                OpCode::BitRShift => Action::BitRShift,
                OpCode::BitURShift => Action::BitURShift,
                OpCode::BitXor => Action::BitXor,
                OpCode::Call => Action::Call,
                OpCode::CallFunction => Action::CallFunction,
                OpCode::CallMethod => Action::CallMethod,
                OpCode::CastOp => Action::CastOp,
                OpCode::CharToAscii => Action::CharToAscii,
                OpCode::CloneSprite => Action::CloneSprite,
                OpCode::ConstantPool => Action::ConstantPool(self.read_constant_pool()?),
                OpCode::Decrement => Action::Decrement,
                OpCode::DefineFunction => {
                    Action::DefineFunction(self.read_define_function(length)?)
                }
                OpCode::DefineFunction2 => {
                    Action::DefineFunction2(self.read_define_function_2(length)?)
                }
                OpCode::DefineLocal => Action::DefineLocal,
                OpCode::DefineLocal2 => Action::DefineLocal2,
                OpCode::Delete => Action::Delete,
                OpCode::Delete2 => Action::Delete2,
                OpCode::Divide => Action::Divide,
                OpCode::EndDrag => Action::EndDrag,
                OpCode::Enumerate => Action::Enumerate,
                OpCode::Enumerate2 => Action::Enumerate2,
                OpCode::End => Action::End,
                OpCode::Equals => Action::Equals,
                OpCode::Equals2 => Action::Equals2,
                OpCode::Extends => Action::Extends,
                OpCode::GetMember => Action::GetMember,
                OpCode::GetProperty => Action::GetProperty,
                OpCode::GetTime => Action::GetTime,
                OpCode::GetUrl => Action::GetUrl(self.read_get_url()?),
                OpCode::GetUrl2 => Action::GetUrl2(self.read_get_url_2()?),
                OpCode::GetVariable => Action::GetVariable,
                OpCode::GotoFrame => Action::GotoFrame(self.read_goto_frame()?),
                OpCode::GotoFrame2 => Action::GotoFrame2(self.read_goto_frame_2()?),
                OpCode::GotoLabel => Action::GotoLabel(self.read_goto_label()?),
                OpCode::Greater => Action::Greater,
                OpCode::If => Action::If(self.read_if()?),
                OpCode::ImplementsOp => Action::ImplementsOp,
                OpCode::Increment => Action::Increment,
                OpCode::InitArray => Action::InitArray,
                OpCode::InitObject => Action::InitObject,
                OpCode::InstanceOf => Action::InstanceOf,
                OpCode::Jump => Action::Jump(self.read_jump()?),
                OpCode::Less => Action::Less,
                OpCode::Less2 => Action::Less2,
                OpCode::MBAsciiToChar => Action::MBAsciiToChar,
                OpCode::MBCharToAscii => Action::MBCharToAscii,
                OpCode::MBStringExtract => Action::MBStringExtract,
                OpCode::MBStringLength => Action::MBStringLength,
                OpCode::Modulo => Action::Modulo,
                OpCode::Multiply => Action::Multiply,
                OpCode::NewMethod => Action::NewMethod,
                OpCode::NewObject => Action::NewObject,
                OpCode::NextFrame => Action::NextFrame,
                OpCode::Not => Action::Not,
                OpCode::Or => Action::Or,
                OpCode::Play => Action::Play,
                OpCode::Pop => Action::Pop,
                OpCode::PreviousFrame => Action::PreviousFrame,
                OpCode::Push => Action::Push(self.read_push(*length)?),
                OpCode::PushDuplicate => Action::PushDuplicate,
                OpCode::RandomNumber => Action::RandomNumber,
                OpCode::RemoveSprite => Action::RemoveSprite,
                OpCode::Return => Action::Return,
                OpCode::SetMember => Action::SetMember,
                OpCode::SetProperty => Action::SetProperty,
                OpCode::SetTarget => Action::SetTarget(self.read_set_target()?),
                OpCode::SetTarget2 => Action::SetTarget2,
                OpCode::SetVariable => Action::SetVariable,
                OpCode::StackSwap => Action::StackSwap,
                OpCode::StartDrag => Action::StartDrag,
                OpCode::Stop => Action::Stop,
                OpCode::StopSounds => Action::StopSounds,
                OpCode::StoreRegister => Action::StoreRegister(self.read_store_register()?),
                OpCode::StrictEquals => Action::StrictEquals,
                OpCode::StringAdd => Action::StringAdd,
                OpCode::StringEquals => Action::StringEquals,
                OpCode::StringExtract => Action::StringExtract,
                OpCode::StringGreater => Action::StringGreater,
                OpCode::StringLength => Action::StringLength,
                OpCode::StringLess => Action::StringLess,
                OpCode::Subtract => Action::Subtract,
                OpCode::TargetPath => Action::TargetPath,
                OpCode::Throw => Action::Throw,
                OpCode::ToggleQuality => Action::ToggleQuality,
                OpCode::ToInteger => Action::ToInteger,
                OpCode::ToNumber => Action::ToNumber,
                OpCode::ToString => Action::ToString,
                OpCode::Trace => Action::Trace,
                OpCode::Try => Action::Try(self.read_try(length)?),
                OpCode::TypeOf => Action::TypeOf,
                OpCode::WaitForFrame => Action::WaitForFrame(self.read_wait_for_frame()?),
                OpCode::WaitForFrame2 => Action::WaitForFrame2(self.read_wait_for_frame_2()?),
                OpCode::With => Action::With(self.read_with(length)?),
            }
        } else {
            Action::Unknown(self.read_unknown_action(opcode, *length)?)
        };

        Ok(action)
    }

    fn read_constant_pool(&mut self) -> Result<ConstantPool<'a>> {
        let count = self.read_u16()?;
        let mut strings = Vec::with_capacity(count as usize);
        for _ in 0..count {
            strings.push(self.read_str()?);
        }
        Ok(ConstantPool { strings })
    }

    fn read_define_function(&mut self, action_length: &mut usize) -> Result<DefineFunction<'a>> {
        let name = self.read_str()?;
        let num_params = self.read_u16()?;
        let mut params = Vec::with_capacity(num_params as usize);
        for _ in 0..num_params {
            params.push(self.read_str()?);
        }
        // code_length isn't included in the DefineFunction's action length.
        let code_length: usize = (self.read_u16()?).into();
        *action_length += code_length;
        Ok(DefineFunction {
            name,
            params,
            actions: self.read_slice(code_length)?,
        })
    }

    fn read_define_function_2(&mut self, action_length: &mut usize) -> Result<DefineFunction2<'a>> {
        let name = self.read_str()?;
        let num_params = self.read_u16()?;
        let register_count = self.read_u8()?;
        let flags = FunctionFlags::from_bits_truncate(self.read_u16()?);
        let mut params = Vec::with_capacity(num_params as usize);
        for _ in 0..num_params {
            params.push(FunctionParam {
                register_index: NonZeroU8::new(self.read_u8()?),
                name: self.read_str()?,
            });
        }
        // code_length isn't included in the DefineFunction's length.
        let code_length: usize = (self.read_u16()?).into();
        *action_length += code_length;
        Ok(DefineFunction2 {
            name,
            params,
            register_count,
            flags,
            actions: self.read_slice(code_length)?,
        })
    }

    fn read_get_url(&mut self) -> Result<GetUrl<'a>> {
        Ok(GetUrl {
            url: self.read_str()?,
            target: self.read_str()?,
        })
    }

    fn read_get_url_2(&mut self) -> Result<GetUrl2> {
        let mut flags = GetUrlFlags::from_bits_truncate(self.read_u8()?);
        if flags.contains(GetUrlFlags::METHOD_MASK) {
            // Flash treats an invalid method the same as "none".
            log::warn!("Invalid SendVarsMethod in GetUrl2, defaulting to None");
            flags.set(GetUrlFlags::METHOD_MASK, false);
        }
        Ok(GetUrl2(flags))
    }

    fn read_goto_frame(&mut self) -> Result<GotoFrame> {
        Ok(GotoFrame {
            frame: self.read_u16()?,
        })
    }

    fn read_goto_frame_2(&mut self) -> Result<GotoFrame2> {
        let flags = self.read_u8()?;
        Ok(GotoFrame2 {
            set_playing: flags & 0b1 != 0,
            scene_offset: if flags & 0b10 != 0 {
                self.read_u16()?
            } else {
                0
            },
        })
    }

    fn read_goto_label(&mut self) -> Result<GotoLabel<'a>> {
        Ok(GotoLabel {
            label: self.read_str()?,
        })
    }

    fn read_if(&mut self) -> Result<If> {
        Ok(If {
            offset: self.read_i16()?,
        })
    }

    fn read_jump(&mut self) -> Result<Jump> {
        Ok(Jump {
            offset: self.read_i16()?,
        })
    }

    fn read_push(&mut self, length: usize) -> Result<Push<'a>> {
        // TODO: Verify correct version for complex types.
        let end_pos = (self.input.as_ptr() as usize + length) as *const u8;
        let mut values = Vec::with_capacity(4);
        while self.input.as_ptr() < end_pos {
            let value = match self.read_u8()? {
                0 => Value::Str(self.read_str()?),
                1 => Value::Float(self.read_f32()?),
                2 => Value::Null,
                3 => Value::Undefined,
                4 => Value::Register(self.read_u8()?),
                5 => Value::Bool(self.read_u8()? != 0),
                6 => Value::Double(self.read_f64_me()?),
                7 => Value::Int(self.read_i32()?),
                8 => Value::ConstantPool(self.read_u8()?.into()),
                9 => Value::ConstantPool(self.read_u16()?),
                type_ => {
                    // Newest Flash Player exits on unknown value types. However, older versions
                    // (at least FP9) just ignore them and continue to the next value.
                    // We follow the lenient behavior in order to support more content (e.g. #8389
                    // doesn't work on newest Flash Player).
                    // TODO: Return an error if we ever add player version emulation.
                    log::warn!("Invalid value type in ActionPush: {}", type_);
                    continue;
                }
            };
            values.push(value);
        }
        Ok(Push { values })
    }

    fn read_set_target(&mut self) -> Result<SetTarget<'a>> {
        Ok(SetTarget {
            target: self.read_str()?,
        })
    }

    fn read_store_register(&mut self) -> Result<StoreRegister> {
        Ok(StoreRegister {
            register: self.read_u8()?,
        })
    }

    fn read_try(&mut self, length: &mut usize) -> Result<Try<'a>> {
        let flags = TryFlags::from_bits_truncate(self.read_u8()?);
        let try_size: usize = self.read_u16()?.into();
        let catch_size: usize = self.read_u16()?.into();
        let finally_size: usize = self.read_u16()?.into();
        *length += try_size + catch_size + finally_size;
        let catch_var = if flags.contains(TryFlags::CATCH_IN_REGISTER) {
            CatchVar::Register(self.read_u8()?)
        } else {
            CatchVar::Var(self.read_str()?)
        };
        let try_body = self.read_slice(try_size)?;
        let catch_body = self.read_slice(catch_size)?;
        let finally_body = self.read_slice(finally_size)?;
        Ok(Try {
            try_body,
            catch_body: if flags.contains(TryFlags::CATCH_BLOCK) {
                Some((catch_var, catch_body))
            } else {
                None
            },
            finally_body: if flags.contains(TryFlags::FINALLY_BLOCK) {
                Some(finally_body)
            } else {
                None
            },
        })
    }

    fn read_wait_for_frame(&mut self) -> Result<WaitForFrame> {
        Ok(WaitForFrame {
            frame: self.read_u16()?,
            num_actions_to_skip: self.read_u8()?,
        })
    }

    fn read_wait_for_frame_2(&mut self) -> Result<WaitForFrame2> {
        Ok(WaitForFrame2 {
            num_actions_to_skip: self.read_u8()?,
        })
    }

    fn read_with(&mut self, action_length: &mut usize) -> Result<With<'a>> {
        let code_length: usize = (self.read_u16()?).into();
        *action_length += code_length;
        Ok(With {
            actions: self.read_slice(code_length)?,
        })
    }

    fn read_unknown_action(&mut self, opcode: u8, length: usize) -> Result<Unknown<'a>> {
        Ok(Unknown {
            opcode,
            data: self.read_slice(length)?,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::string::{SwfStr, WINDOWS_1252};
    use crate::test_data;

    #[test]
    fn read_action() {
        for (swf_version, expected_action, action_bytes) in test_data::avm1_tests() {
            let mut reader = Reader::new(&action_bytes[..], swf_version);
            let parsed_action = reader.read_action().unwrap();
            assert_eq!(
                parsed_action, expected_action,
                "Incorrectly parsed action.\nRead:\n{:?}\n\nExpected:\n{:?}",
                parsed_action, expected_action
            );
        }
    }

    /// Ensure that we return an error on invalid data.
    #[test]
    fn read_parse_error() {
        let action_bytes = [0xff, 0xff, 0xff, 0x00, 0x00];
        let mut reader = Reader::new(&action_bytes[..], 5);
        match reader.read_action() {
            Err(crate::error::Error::Avm1ParseError { .. }) => (),
            result => {
                panic!("Expected Avm1ParseError, got {result:?}");
            }
        }
    }

    #[test]
    fn read_define_function() {
        // Ensure we read a function properly along with the function data.
        let action_bytes = vec![
            0x9b, 0x08, 0x00, 0x66, 0x6f, 0x6f, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x96, 0x06, 0x00,
            0x00, 0x74, 0x65, 0x73, 0x74, 0x00, 0x26, 0x00,
        ];
        let mut reader = Reader::new(&action_bytes[..], 5);
        let action = reader.read_action().unwrap();
        assert_eq!(
            action,
            Action::DefineFunction(DefineFunction {
                name: SwfStr::from_str_with_encoding("foo", WINDOWS_1252).unwrap(),
                params: vec![],
                actions: &[0x96, 0x06, 0x00, 0x00, 0x74, 0x65, 0x73, 0x74, 0x00, 0x26],
            })
        );

        if let Action::DefineFunction(DefineFunction { actions, .. }) = action {
            let mut reader = Reader::new(actions, 5);
            let action = reader.read_action().unwrap();
            assert_eq!(
                action,
                Action::Push(Push {
                    values: vec![Value::Str(
                        SwfStr::from_str_with_encoding("test", WINDOWS_1252).unwrap()
                    )]
                })
            );
        }
    }

    #[test]
    fn read_push() {
        // ActionPush doesn't provide an explicit # of values, but instead reads values
        // until the end of the action. Ensure we don't read extra values.
        let action_bytes = [
            OpCode::Push as u8,
            3,
            0,
            2,  // null
            10, // Invalid value type.
            3,  // undefined
            3,  // Extra byte at the end shouldn't be read.
        ];
        let mut reader = Reader::new(&action_bytes, 5);
        let action = reader.read_action().unwrap();
        assert_eq!(
            action,
            Action::Push(Push {
                values: vec![Value::Null, Value::Undefined]
            })
        );
    }

    #[test]
    fn read_length_mismatch() {
        let action_bytes = [
            OpCode::ConstantPool as u8,
            5,
            0,
            1,
            0,
            b'a',
            0,
            OpCode::Add as u8,
            OpCode::Subtract as u8,
        ];
        let mut reader = Reader::new(&action_bytes[..], 5);

        let action = reader.read_action().unwrap();
        assert_eq!(
            action,
            Action::ConstantPool(ConstantPool {
                strings: vec!["a".into()]
            })
        );

        let action = reader.read_action().unwrap();
        assert_eq!(action, Action::Subtract);
    }
}
