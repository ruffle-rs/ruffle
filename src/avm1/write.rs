use avm1::types::{Action, SendVarsMethod, Value};
use avm1::opcode::OpCode;
use write::SwfWrite;
use std::io::{Result, Write};

pub struct Writer<W: Write> {
    inner: W,
    version: u8,
}

impl<W: Write> SwfWrite<W> for Writer<W> {
    fn get_inner(&mut self) -> &mut W {
        &mut self.inner
    }
}

impl<W: Write> Writer<W> {
    pub fn new(inner: W, version: u8) -> Writer<W> {
        Writer { inner: inner, version: version }
    }
    
    pub fn write_action_list(&mut self, actions: &Vec<Action>) -> Result<()> {
        for action in actions {
            try!(self.write_action(action));
        }
        try!(self.write_u8(0)); // End
        Ok(())
    }

    pub fn write_action(&mut self, action: &Action) -> Result<()> {
        match action {
            &Action::Add => try!(self.write_action_header(OpCode::Add, 0)),
            &Action::And => try!(self.write_action_header(OpCode::And, 0)),
            &Action::AsciiToChar => try!(self.write_action_header(OpCode::AsciiToChar, 0)),
            &Action::Call => try!(self.write_action_header(OpCode::Call, 0)),
            &Action::CharToAscii => try!(self.write_action_header(OpCode::CharToAscii, 0)),
            &Action::CloneSprite => try!(self.write_action_header(OpCode::CloneSprite, 0)),
            &Action::Divide => try!(self.write_action_header(OpCode::Divide, 0)),
            &Action::EndDrag => try!(self.write_action_header(OpCode::EndDrag, 0)),
            &Action::Equals => try!(self.write_action_header(OpCode::Equals, 0)),
            &Action::GetProperty => try!(self.write_action_header(OpCode::GetProperty, 0)),
            &Action::GetTime => try!(self.write_action_header(OpCode::GetTime, 0)),
            &Action::GetUrl { ref url, ref target } => {
                try!(self.write_action_header(OpCode::GetUrl, url.len() + target.len() + 2));
                try!(self.write_c_string(url));
                try!(self.write_c_string(target));
            },
            &Action::GetUrl2 { send_vars_method, is_target_sprite, is_load_vars } => {
                try!(self.write_action_header(OpCode::GetUrl2, 1));
                let flags =
                    (match send_vars_method {
                        SendVarsMethod::None => 0,
                        SendVarsMethod::Get => 1,
                        SendVarsMethod::Post => 2,
                    } << 6) |
                    if is_target_sprite { 0b10 } else { 0 } |
                    if is_load_vars { 0b1 } else { 0 };
                try!(self.write_u8(flags));
            },
            &Action::GetVariable => try!(self.write_action_header(OpCode::GetVariable, 0)),
            &Action::GotoFrame(frame) => {
                try!(self.write_action_header(OpCode::GotoFrame, 2));
                try!(self.write_u16(frame));
            },
            &Action::GotoFrame2 { set_playing, scene_offset } => {
                if scene_offset != 0 {
                    try!(self.write_action_header(OpCode::GotoFrame2, 3));
                    try!(self.write_u8(if set_playing { 0b11 } else { 0b01 }));
                    try!(self.write_u16(scene_offset));
                } else {
                    try!(self.write_action_header(OpCode::GotoFrame2, 1));
                    try!(self.write_u8(if set_playing { 0b10 } else { 0b00 }));
                }
            },
            &Action::GotoLabel(ref label) => {
                try!(self.write_action_header(OpCode::GotoLabel, label.len() + 1));
                try!(self.write_c_string(label));
            },
            &Action::If { offset } => {
                try!(self.write_action_header(OpCode::If, 2));
                try!(self.write_i16(offset));
            },
            &Action::Jump { offset } => {
                try!(self.write_action_header(OpCode::Jump, 2));
                try!(self.write_i16(offset));
            },
            &Action::Less => try!(self.write_action_header(OpCode::Less, 0)),
            &Action::MBAsciiToChar => try!(self.write_action_header(OpCode::MBAsciiToChar, 0)),
            &Action::MBCharToAscii => try!(self.write_action_header(OpCode::MBCharToAscii, 0)),
            &Action::MBStringExtract => try!(self.write_action_header(OpCode::MBStringExtract, 0)),
            &Action::MBStringLength => try!(self.write_action_header(OpCode::MBStringLength, 0)),
            &Action::Multiply => try!(self.write_action_header(OpCode::Multiply, 0)),
            &Action::NextFrame => try!(self.write_action_header(OpCode::NextFrame, 0)),
            &Action::Not => try!(self.write_action_header(OpCode::Not, 0)),
            &Action::Or => try!(self.write_action_header(OpCode::Or, 0)),
            &Action::Play => try!(self.write_action_header(OpCode::Play, 0)),
            &Action::Pop => try!(self.write_action_header(OpCode::Pop, 0)),
            &Action::PreviousFrame => try!(self.write_action_header(OpCode::PreviousFrame, 0)),
            &Action::Push(ref values) => {
                let len = values.iter().map(|v| {
                    match v {
                        &Value::Str(ref string) => string.len() + 2,
                        &Value::Float(_) => 5,
                        &Value::Null => 1,
                        &Value::Undefined => 1,
                        &Value::Register(_) => 2,
                        &Value::Bool(_) => 2,
                        &Value::Double(_) => 9,
                        &Value::Int(_) => 5,
                        &Value::ConstantPool(v) => if v < 256 { 2 } else { 3 },
                    }
                }).sum();
                try!(self.write_action_header(OpCode::Push, len));
                for value in values {
                    try!(self.write_push_value(value));
                }
            },
            &Action::RandomNumber => try!(self.write_action_header(OpCode::RandomNumber, 0)),
            &Action::RemoveSprite => try!(self.write_action_header(OpCode::RemoveSprite, 0)),
            &Action::SetProperty => try!(self.write_action_header(OpCode::SetProperty, 0)),
            &Action::SetTarget(ref target) => {
                try!(self.write_action_header(OpCode::SetTarget, target.len() + 1));
                try!(self.write_c_string(target));
            },
            &Action::SetTarget2 => try!(self.write_action_header(OpCode::SetTarget2, 0)),
            &Action::SetVariable => try!(self.write_action_header(OpCode::SetVariable, 0)),
            &Action::StartDrag => try!(self.write_action_header(OpCode::StartDrag, 0)),
            &Action::Stop => try!(self.write_action_header(OpCode::Stop, 0)),
            &Action::StopSounds => try!(self.write_action_header(OpCode::StopSounds, 0)),
            &Action::StringAdd => try!(self.write_action_header(OpCode::StringAdd, 0)),
            &Action::StringEquals => try!(self.write_action_header(OpCode::StringEquals, 0)),
            &Action::StringExtract => try!(self.write_action_header(OpCode::StringExtract, 0)),
            &Action::StringLength => try!(self.write_action_header(OpCode::StringLength, 0)),
            &Action::StringLess => try!(self.write_action_header(OpCode::StringLess, 0)),
            &Action::Subtract => try!(self.write_action_header(OpCode::Subtract, 0)),
            &Action::ToInteger => try!(self.write_action_header(OpCode::ToInteger, 0)),
            &Action::ToggleQuality => try!(self.write_action_header(OpCode::ToggleQuality, 0)),
            &Action::Trace => try!(self.write_action_header(OpCode::Trace, 0)),
            &Action::WaitForFrame { frame, num_actions_to_skip } => {
                try!(self.write_action_header(OpCode::WaitForFrame, 3));
                try!(self.write_u16(frame));
                try!(self.write_u8(num_actions_to_skip));
            },
            &Action::WaitForFrame2 { num_actions_to_skip } => {
                try!(self.write_action_header(OpCode::WaitForFrame2, 1));
                try!(self.write_u8(num_actions_to_skip));
            },
            &Action::Unknown { opcode, ref data } => {
                try!(self.write_opcode_and_length(opcode, data.len()));
                try!(self.inner.write_all(&data));
            }
        }

        Ok(())
    }

    pub fn write_action_header(&mut self, opcode: OpCode, length: usize) -> Result<()> {
        self.write_opcode_and_length(opcode as u8, length)
    }

    pub fn write_opcode_and_length(&mut self, opcode: u8, length: usize) -> Result<()> {
        try!(self.write_u8(opcode));
        assert!( opcode >= 0x80 || length == 0, "Opcodes less than 0x80 must have length 0" );
        if opcode >= 0x80 {
            try!(self.write_u16(length as u16));
        }
        Ok(())
    }

    fn write_push_value(&mut self, value: &Value) -> Result<()> {
        match value {
            &Value::Str(ref string) => {
                try!(self.write_u8(0));
                try!(self.write_c_string(string));
            },
            &Value::Float(v) => {
                try!(self.write_u8(1));
                try!(self.write_f32(v));
            },
            &Value::Null => {
                try!(self.write_u8(2));
            },
            &Value::Undefined => {
                try!(self.write_u8(3));
            },
            &Value::Register(v) => {
                try!(self.write_u8(4));
                try!(self.write_u8(v));
            },
            &Value::Bool(v) => {
                try!(self.write_u8(5));
                try!(self.write_u8(v as u8));
            },
            &Value::Double(v) => {
                try!(self.write_u8(6));
                try!(self.write_f64(v));
            },
            &Value::Int(v) => {
                try!(self.write_u8(7));
                try!(self.write_u32(v));
            },
            &Value::ConstantPool(v) => {
                if v < 256 {
                    try!(self.write_u8(8));
                    try!(self.write_u8(v as u8));
                } else {
                    try!(self.write_u8(9));
                    try!(self.write_u16(v));
                }
            },
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_data;

    #[test]
    fn write_action() {
        for (swf_version, action, expected_bytes) in test_data::avm1_tests() {
            let mut written_bytes = Vec::new();
            Writer::new(&mut written_bytes, swf_version).write_action(&action).unwrap();
            if written_bytes != expected_bytes {
                panic!(
                    "Error writing action.\nTag:\n{:?}\n\nWrote:\n{:?}\n\nExpected:\n{:?}",
                    action,
                    written_bytes,
                    expected_bytes
                );
            }
        }
    }
}