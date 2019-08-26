#![allow(clippy::unreadable_literal)]

use crate::avm1::opcode::OpCode;
use crate::avm1::types::*;
use crate::read::SwfRead;
use std::io::{Error, ErrorKind, Read, Result};

pub struct Reader<R: Read> {
    inner: R,
    version: u8,
}

impl<R: Read> SwfRead<R> for Reader<R> {
    fn get_inner(&mut self) -> &mut R {
        &mut self.inner
    }
}

impl<R: Read> Reader<R> {
    pub fn new(inner: R, version: u8) -> Reader<R> {
        Reader { inner, version }
    }

    pub fn read_action_list(&mut self) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        while let Some(action) = self.read_action()? {
            actions.push(action);
        }
        Ok(actions)
    }

    pub fn read_action(&mut self) -> Result<Option<Action>> {
        use num_traits::FromPrimitive;

        let (opcode, length) = self.read_opcode_and_length()?;

        let mut action_reader = Reader::new(self.inner.by_ref().take(length as u64), self.version);

        let action = if let Some(op) = OpCode::from_u8(opcode) {
            match op {
                OpCode::End => return Ok(None),

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
                OpCode::ConstantPool => {
                    let mut constants = vec![];
                    for _ in 0..action_reader.read_u16()? {
                        constants.push(action_reader.read_c_string()?);
                    }
                    Action::ConstantPool(constants)
                }
                OpCode::Decrement => Action::Decrement,
                OpCode::DefineFunction => action_reader.read_define_function()?,
                OpCode::DefineFunction2 => action_reader.read_define_function_2()?,
                OpCode::DefineLocal => Action::DefineLocal,
                OpCode::DefineLocal2 => Action::DefineLocal2,
                OpCode::Delete => Action::Delete,
                OpCode::Delete2 => Action::Delete2,
                OpCode::Divide => Action::Divide,
                OpCode::EndDrag => Action::EndDrag,
                OpCode::Enumerate => Action::Enumerate,
                OpCode::Enumerate2 => Action::Enumerate2,
                OpCode::Equals => Action::Equals,
                OpCode::Equals2 => Action::Equals2,
                OpCode::Extends => Action::Extends,
                OpCode::GetMember => Action::GetMember,
                OpCode::GetProperty => Action::GetProperty,
                OpCode::GetTime => Action::GetTime,
                OpCode::GetUrl => Action::GetUrl {
                    url: action_reader.read_c_string()?,
                    target: action_reader.read_c_string()?,
                },
                OpCode::GetUrl2 => {
                    let flags = action_reader.read_u8()?;
                    Action::GetUrl2 {
                        is_target_sprite: flags & 0b10 != 0,
                        is_load_vars: flags & 0b1 != 0,
                        send_vars_method: match flags >> 6 {
                            0 => SendVarsMethod::None,
                            1 => SendVarsMethod::Get,
                            2 => SendVarsMethod::Post,
                            _ => {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    "Invalid HTTP method in ActionGetUrl2",
                                ))
                            }
                        },
                    }
                }
                OpCode::GetVariable => Action::GetVariable,
                OpCode::GotoFrame => {
                    let frame = action_reader.read_u16()?;
                    Action::GotoFrame(frame)
                }
                OpCode::GotoFrame2 => {
                    let flags = action_reader.read_u8()?;
                    Action::GotoFrame2 {
                        set_playing: flags & 0b1 != 0,
                        scene_offset: if flags & 0b10 != 0 {
                            action_reader.read_u16()?
                        } else {
                            0
                        },
                    }
                }
                OpCode::GotoLabel => Action::GotoLabel(action_reader.read_c_string()?),
                OpCode::Greater => Action::Greater,
                OpCode::If => Action::If {
                    offset: action_reader.read_i16()?,
                },
                OpCode::ImplementsOp => Action::ImplementsOp,
                OpCode::Increment => Action::Increment,
                OpCode::InitArray => Action::InitArray,
                OpCode::InitObject => Action::InitObject,
                OpCode::InstanceOf => Action::InstanceOf,
                OpCode::Jump => Action::Jump {
                    offset: action_reader.read_i16()?,
                },
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
                // TODO: Verify correct version for complex types.
                OpCode::Push => {
                    let mut values = vec![];
                    while let Ok(value) = action_reader.read_push_value() {
                        values.push(value);
                    }
                    Action::Push(values)
                }
                OpCode::PushDuplicate => Action::PushDuplicate,
                OpCode::RandomNumber => Action::RandomNumber,
                OpCode::RemoveSprite => Action::RemoveSprite,
                OpCode::Return => Action::Return,
                OpCode::SetMember => Action::SetMember,
                OpCode::SetProperty => Action::SetProperty,
                OpCode::SetTarget => Action::SetTarget(action_reader.read_c_string()?),
                OpCode::SetTarget2 => Action::SetTarget2,
                OpCode::SetVariable => Action::SetVariable,
                OpCode::StackSwap => Action::StackSwap,
                OpCode::StartDrag => Action::StartDrag,
                OpCode::Stop => Action::Stop,
                OpCode::StopSounds => Action::StopSounds,
                OpCode::StoreRegister => Action::StoreRegister(action_reader.read_u8()?),
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
                OpCode::Try => action_reader.read_try()?,
                OpCode::TypeOf => Action::TypeOf,
                OpCode::WaitForFrame => Action::WaitForFrame {
                    frame: action_reader.read_u16()?,
                    num_actions_to_skip: action_reader.read_u8()?,
                },
                OpCode::With => {
                    let code_length = action_reader.read_u16()?;
                    let mut with_reader = Reader::new(
                        (&mut action_reader.inner as &mut dyn Read).take(code_length.into()),
                        self.version,
                    );
                    Action::With {
                        actions: with_reader.read_action_list()?,
                    }
                }
                OpCode::WaitForFrame2 => Action::WaitForFrame2 {
                    num_actions_to_skip: action_reader.read_u8()?,
                },
            }
        } else {
            action_reader.read_unknown_action(opcode, length)?
        };

        Ok(Some(action))
    }

    pub fn read_opcode_and_length(&mut self) -> Result<(u8, usize)> {
        let opcode = self.read_u8()?;
        let length = if opcode >= 0x80 {
            self.read_u16()? as usize
        } else {
            0
        };
        Ok((opcode, length))
    }

    fn read_unknown_action(&mut self, opcode: u8, length: usize) -> Result<Action> {
        let mut data = vec![0u8; length];
        self.inner.read_exact(&mut data)?;
        Ok(Action::Unknown { opcode, data })
    }

    fn read_push_value(&mut self) -> Result<Value> {
        let value = match self.read_u8()? {
            0 => Value::Str(self.read_c_string()?),
            1 => Value::Float(self.read_f32()?),
            2 => Value::Null,
            3 => Value::Undefined,
            4 => Value::Register(self.read_u8()?),
            5 => Value::Bool(self.read_u8()? != 0),
            6 => Value::Double(self.read_f64()?),
            7 => Value::Int(self.read_i32()?),
            8 => Value::ConstantPool(self.read_u8()?.into()),
            9 => Value::ConstantPool(self.read_u16()?),
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid value type in ActionPush",
                ))
            }
        };
        Ok(value)
    }

    fn read_define_function(&mut self) -> Result<Action> {
        let name = self.read_c_string()?;
        let num_params = self.read_u16()?;
        let mut params = Vec::with_capacity(num_params as usize);
        for _ in 0..num_params {
            params.push(self.read_c_string()?);
        }
        let code_length = self.read_u16()?;
        let mut fn_reader = Reader::new(
            (&mut self.inner as &mut dyn Read).take(code_length.into()),
            self.version,
        );
        Ok(Action::DefineFunction {
            name,
            params,
            actions: fn_reader.read_action_list()?,
        })
    }

    fn read_define_function_2(&mut self) -> Result<Action> {
        let name = self.read_c_string()?;
        let num_params = self.read_u16()?;
        let num_registers = self.read_u8()?; // Number of registers
        let flags = self.read_u16()?;
        let mut params = Vec::with_capacity(num_params as usize + num_registers as usize);
        for _ in 0..num_params {
            let register = self.read_u8()?;
            params.push(FunctionParam {
                name: self.read_c_string()?,
                register_index: if register == 0 { None } else { Some(register) },
            });
        }
        let code_length = self.read_u16()?;
        let mut fn_reader = Reader::new(
            (&mut self.inner as &mut dyn Read).take(code_length.into()),
            self.version,
        );
        Ok(Action::DefineFunction2(Function {
            name,
            params,
            preload_global: flags & 0b1_00000000 != 0,
            preload_parent: flags & 0b10000000 != 0,
            preload_root: flags & 0b1000000 != 0,
            suppress_super: flags & 0b100000 != 0,
            preload_super: flags & 0b10000 != 0,
            suppress_arguments: flags & 0b1000 != 0,
            preload_arguments: flags & 0b100 != 0,
            suppress_this: flags & 0b10 != 0,
            preload_this: flags & 0b1 != 0,
            actions: fn_reader.read_action_list()?,
        }))
    }

    fn read_try(&mut self) -> Result<Action> {
        let flags = self.read_u8()?;
        let try_length = self.read_u16()?;
        let catch_length = self.read_u16()?;
        let finally_length = self.read_u16()?;
        let catch_var = if flags & 0b100 != 0 {
            CatchVar::Var(self.read_c_string()?)
        } else {
            CatchVar::Register(self.read_u8()?)
        };
        let try_actions = {
            let mut fn_reader = Reader::new(
                (&mut self.inner as &mut dyn Read).take(try_length.into()),
                self.version,
            );
            fn_reader.read_action_list()?
        };
        let catch_actions = {
            let mut fn_reader = Reader::new(
                (&mut self.inner as &mut dyn Read).take(catch_length.into()),
                self.version,
            );
            fn_reader.read_action_list()?
        };
        let finally_actions = {
            let mut fn_reader = Reader::new(
                (&mut self.inner as &mut dyn Read).take(finally_length.into()),
                self.version,
            );
            fn_reader.read_action_list()?
        };
        Ok(Action::Try(TryBlock {
            try_actions,
            catch: if flags & 0b1 != 0 {
                Some((catch_var, catch_actions))
            } else {
                None
            },
            finally: if flags & 0b10 != 0 {
                Some(finally_actions)
            } else {
                None
            },
        }))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::test_data;

    #[test]
    fn read_action() {
        for (swf_version, expected_action, action_bytes) in test_data::avm1_tests() {
            let mut reader = Reader::new(&action_bytes[..], swf_version);
            let parsed_action = reader.read_action().unwrap().unwrap();
            if parsed_action != expected_action {
                // Failed, result doesn't match.
                panic!(
                    "Incorrectly parsed action.\nRead:\n{:?}\n\nExpected:\n{:?}",
                    parsed_action, expected_action
                );
            }
        }
    }
}
