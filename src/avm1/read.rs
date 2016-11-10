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
        let action = match OpCode::from_u8(opcode) {
            Some(OpCode::End) => return Ok(None),

            Some(OpCode::Add) => Action::Add,
            Some(OpCode::And) => Action::And,
            Some(OpCode::AsciiToChar) => Action::AsciiToChar,
            Some(OpCode::Call) => Action::Call,
            Some(OpCode::CharToAscii) => Action::CharToAscii,
            Some(OpCode::CloneSprite) => Action::CloneSprite,
            Some(OpCode::Divide) => Action::Divide,
            Some(OpCode::EndDrag) => Action::EndDrag,
            Some(OpCode::Equals) => Action::Equals,
            Some(OpCode::GetProperty) => Action::GetProperty,
            Some(OpCode::GetTime) => Action::GetTime,
            Some(OpCode::GetUrl) => Action::GetUrl {
                url: try!(action_reader.read_c_string()),
                target: try!(action_reader.read_c_string()),
            },
            Some(OpCode::GetUrl2) => {
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
            Some(OpCode::GetVariable) => Action::GetVariable,
            Some(OpCode::GotoFrame) => {
                let frame = try!(action_reader.read_u16());
                Action::GotoFrame(frame)
            },
            Some(OpCode::GotoFrame2) => {
                let flags = try!(action_reader.read_u8());
                Action::GotoFrame2 {
                    set_playing: flags & 0b1 != 0,
                    scene_offset: if flags & 0b10 != 0 {
                        try!(action_reader.read_u16())
                    } else { 0 },
                }
            },
            Some(OpCode::GotoLabel) => Action::GotoLabel(try!(action_reader.read_c_string())),
            Some(OpCode::If) => Action::If { offset: try!(action_reader.read_i16()) },
            Some(OpCode::Jump) => Action::Jump { offset: try!(action_reader.read_i16()) },
            Some(OpCode::Less) => Action::Less,
            Some(OpCode::MBAsciiToChar) => Action::MBAsciiToChar,
            Some(OpCode::MBCharToAscii) => Action::MBCharToAscii,
            Some(OpCode::MBStringExtract) => Action::MBStringExtract,
            Some(OpCode::MBStringLength) => Action::MBStringLength,
            Some(OpCode::Multiply) => Action::Multiply,
            Some(OpCode::NextFrame) => Action::NextFrame,
            Some(OpCode::Not) => Action::Not,
            Some(OpCode::Or) => Action::Or,
            Some(OpCode::Play) => Action::Play,
            Some(OpCode::Pop) => Action::Pop,
            Some(OpCode::PreviousFrame) => Action::PreviousFrame,
            // TODO: Verify correct version for complex types.
            Some(OpCode::Push) => {
                let mut values = vec![];
                while let Ok(value) = action_reader.read_push_value() {
                    values.push(value);
                };
                Action::Push(values)
            },
            Some(OpCode::RandomNumber) => Action::RandomNumber,
            Some(OpCode::RemoveSprite) => Action::RemoveSprite,
            Some(OpCode::SetProperty) => Action::SetProperty,
            Some(OpCode::SetTarget) => Action::SetTarget(try!(action_reader.read_c_string())),
            Some(OpCode::SetTarget2) => Action::SetTarget2,
            Some(OpCode::SetVariable) => Action::SetVariable,
            Some(OpCode::StartDrag) => Action::StartDrag,
            Some(OpCode::Stop) => Action::Stop,
            Some(OpCode::StopSounds) => Action::StopSounds,
            Some(OpCode::StringAdd) => Action::StringAdd,
            Some(OpCode::StringEquals) => Action::StringEquals,
            Some(OpCode::StringExtract) => Action::StringExtract,
            Some(OpCode::StringLength) => Action::StringLength,
            Some(OpCode::StringLess) => Action::StringLess,
            Some(OpCode::Subtract) => Action::Subtract,
            Some(OpCode::ToInteger) => Action::ToInteger,
            Some(OpCode::ToggleQuality) => Action::ToggleQuality,
            Some(OpCode::Trace) => Action::Trace,
            Some(OpCode::WaitForFrame) => Action::WaitForFrame {
                frame: try!(action_reader.read_u16()),
                num_actions_to_skip: try!(action_reader.read_u8()),
            },
            Some(OpCode::WaitForFrame2) => Action::WaitForFrame2 {
                num_actions_to_skip: try!(action_reader.read_u8()),
            },
            _ => {
                let mut data = Vec::with_capacity(length);
                try!(action_reader.inner.read_to_end(&mut data));
                Action::Unknown { opcode: opcode, data: data }
            }
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