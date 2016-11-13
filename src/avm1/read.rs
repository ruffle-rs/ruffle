use avm1::types::{Action, SendVarsMethod, Value};
use avm1::opcode::OpCode;
use read::SwfRead;
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
        Reader { inner: inner, version: version }
    }

    pub fn read_action_list(&mut self) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        while let Some(action) = try!(self.read_action()) {
            actions.push(action);
        }
        Ok(actions)
    }

    pub fn read_action(&mut self) -> Result<Option<Action>> {
        let (opcode, length) = try!(self.read_opcode_and_length());

        let mut action_reader = Reader::new(self.inner.by_ref().take(length as u64), self.version);

        use num::FromPrimitive;
        let action = if let Some(op) = OpCode::from_u8(opcode) {
            match op {
                OpCode::End => return Ok(None),

                OpCode::Add => Action::Add,
                OpCode::And => Action::And,
                OpCode::AsciiToChar => Action::AsciiToChar,
                OpCode::Call => Action::Call,
                OpCode::CharToAscii => Action::CharToAscii,
                OpCode::CloneSprite => Action::CloneSprite,
                OpCode::Divide => Action::Divide,
                OpCode::EndDrag => Action::EndDrag,
                OpCode::Equals => Action::Equals,
                OpCode::GetProperty => Action::GetProperty,
                OpCode::GetTime => Action::GetTime,
                OpCode::GetUrl => Action::GetUrl {
                    url: try!(action_reader.read_c_string()),
                    target: try!(action_reader.read_c_string()),
                },
                OpCode::GetUrl2 => {
                    let flags = try!(action_reader.read_u8());
                    Action::GetUrl2 {
                        is_target_sprite: flags & 0b10 != 0,
                        is_load_vars: flags & 0b1 != 0,
                        send_vars_method: match flags >> 6 {
                            0 => SendVarsMethod::None,
                            1 => SendVarsMethod::Get,
                            2 => SendVarsMethod::Post,
                            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid HTTP method in ActionGetUrl2")),
                        }
                    }
                },
                OpCode::GetVariable => Action::GetVariable,
                OpCode::GotoFrame => {
                    let frame = try!(action_reader.read_u16());
                    Action::GotoFrame(frame)
                },
                OpCode::GotoFrame2 => {
                    let flags = try!(action_reader.read_u8());
                    Action::GotoFrame2 {
                        set_playing: flags & 0b1 != 0,
                        scene_offset: if flags & 0b10 != 0 {
                            try!(action_reader.read_u16())
                        } else { 0 },
                    }
                },
                OpCode::GotoLabel => Action::GotoLabel(try!(action_reader.read_c_string())),
                OpCode::If => Action::If { offset: try!(action_reader.read_i16()) },
                OpCode::Jump => Action::Jump { offset: try!(action_reader.read_i16()) },
                OpCode::Less => Action::Less,
                OpCode::MBAsciiToChar => Action::MBAsciiToChar,
                OpCode::MBCharToAscii => Action::MBCharToAscii,
                OpCode::MBStringExtract => Action::MBStringExtract,
                OpCode::MBStringLength => Action::MBStringLength,
                OpCode::Multiply => Action::Multiply,
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
                    };
                    Action::Push(values)
                },
                OpCode::RandomNumber => Action::RandomNumber,
                OpCode::RemoveSprite => Action::RemoveSprite,
                OpCode::SetProperty => Action::SetProperty,
                OpCode::SetTarget => Action::SetTarget(try!(action_reader.read_c_string())),
                OpCode::SetTarget2 => Action::SetTarget2,
                OpCode::SetVariable => Action::SetVariable,
                OpCode::StartDrag => Action::StartDrag,
                OpCode::Stop => Action::Stop,
                OpCode::StopSounds => Action::StopSounds,
                OpCode::StringAdd => Action::StringAdd,
                OpCode::StringEquals => Action::StringEquals,
                OpCode::StringExtract => Action::StringExtract,
                OpCode::StringLength => Action::StringLength,
                OpCode::StringLess => Action::StringLess,
                OpCode::Subtract => Action::Subtract,
                OpCode::ToInteger => Action::ToInteger,
                OpCode::ToggleQuality => Action::ToggleQuality,
                OpCode::Trace => Action::Trace,
                OpCode::WaitForFrame => Action::WaitForFrame {
                    frame: try!(action_reader.read_u16()),
                    num_actions_to_skip: try!(action_reader.read_u8()),
                },
                OpCode::WaitForFrame2 => Action::WaitForFrame2 {
                    num_actions_to_skip: try!(action_reader.read_u8()),
                },
                _ => action_reader.read_unknown_action(opcode, length)?
            }
        } else {
            action_reader.read_unknown_action(opcode, length)?
        };

        Ok(Some(action))
    }

    pub fn read_opcode_and_length(&mut self) -> Result<(u8, usize)> {
        let opcode = try!(self.read_u8());
        let length = if opcode >= 0x80 {
            try!(self.read_u16()) as usize
        } else { 0 };
        Ok((opcode, length))
    }

    fn read_unknown_action(&mut self, opcode: u8, length: usize) -> Result<Action> {
        let mut data = vec![0u8; length];
        self.inner.read_exact(&mut data)?;
        Ok(Action::Unknown { opcode: opcode, data: data })
    }

    fn read_push_value(&mut self) -> Result<Value> {
        let value = match try!(self.read_u8()) {
            0 => Value::Str(try!(self.read_c_string())),
            1 => Value::Float(try!(self.read_f32())),
            2 => Value::Null,
            3 => Value::Undefined,
            4 => Value::Register(try!(self.read_u8())),
            5 => Value::Bool(try!(self.read_u8()) != 0),
            6 => Value::Double(try!(self.read_f64())),
            7 => Value::Int(try!(self.read_u32())),
            8 => Value::ConstantPool(try!(self.read_u8()) as u16),
            9 => Value::ConstantPool(try!(self.read_u16())),
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid value type in ActionPush")),
        };
        Ok(value)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use test_data;

    #[test]
    fn read_action() {
        for (swf_version, expected_action, action_bytes) in test_data::avm1_tests() {
            let mut reader = Reader::new(&action_bytes[..], swf_version);
            let parsed_action = reader.read_action().unwrap().unwrap();
            if parsed_action != expected_action {
                // Failed, result doesn't match.
                panic!(
                    "Incorrectly parsed action.\nRead:\n{:?}\n\nExpected:\n{:?}",
                    parsed_action,
                    expected_action
                );
            }
        }
    }
}