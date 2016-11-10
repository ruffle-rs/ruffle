use avm1::types::{Action, Value};
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
            &Action::Divide => try!(self.write_action_header(OpCode::Divide, 0)),
            &Action::GetUrl { ref url, ref target } => {
                try!(self.write_action_header(OpCode::GetUrl, url.len() + target.len() + 2));
                try!(self.write_c_string(url));
                try!(self.write_c_string(target));
            },
            &Action::GotoFrame(frame) => {
                try!(self.write_action_header(OpCode::GotoFrame, 2));
                try!(self.write_u16(frame));
            },
            &Action::GotoLabel(ref label) => {
                try!(self.write_action_header(OpCode::GotoLabel, label.len() + 1));
                try!(self.write_c_string(label));
            },
            &Action::Multiply => try!(self.write_action_header(OpCode::Multiply, 0)),
            &Action::NextFrame => try!(self.write_action_header(OpCode::NextFrame, 0)),
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
            &Action::SetTarget(ref target) => {
                try!(self.write_action_header(OpCode::SetTarget, target.len() + 1));
                try!(self.write_c_string(target));
            },
            &Action::Stop => try!(self.write_action_header(OpCode::Stop, 0)),
            &Action::StopSounds => try!(self.write_action_header(OpCode::StopSounds, 0)),
            &Action::Subtract => try!(self.write_action_header(OpCode::Subtract, 0)),
            &Action::ToggleQuality => try!(self.write_action_header(OpCode::ToggleQuality, 0)),
            &Action::WaitForFrame { frame, num_actions_to_skip } => {
                try!(self.write_action_header(OpCode::WaitForFrame, 3));
                try!(self.write_u16(frame));
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