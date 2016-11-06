use avm1::types::Action;
use avm1::opcode::OpCode;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Result, Write};

pub struct Writer<W: Write> {
    inner: W,
    version: u8,
}

impl<W: Write> Writer<W> {
    pub fn new(inner: W, version: u8) -> Writer<W> {
        Writer { inner: inner, version: version }
    }
    
    pub fn write_action_list(&mut self, actions: &Vec<Action>) -> Result<()> {
        for action in actions {
            try!(self.write_action(action));
        }
        try!(self.inner.write_u8(0)); // End
        Ok(())
    }

    pub fn write_action(&mut self, action: &Action) -> Result<()> {
        match action {
            &Action::GetUrl { ref url, ref target } => {
                try!(self.write_action_header(OpCode::GetUrl, url.len() + target.len() + 2));
                try!(self.write_c_string(url));
                try!(self.write_c_string(target));
            },
            &Action::GotoFrame(frame) => {
                try!(self.write_action_header(OpCode::GotoFrame, 2));
                try!(self.inner.write_u16::<LittleEndian>(frame));
            },
            &Action::NextFrame => try!(self.write_action_header(OpCode::NextFrame, 0)),
            &Action::Play => try!(self.write_action_header(OpCode::Play, 0)),
            &Action::PreviousFrame => try!(self.write_action_header(OpCode::PreviousFrame, 0)),
            &Action::Stop => try!(self.write_action_header(OpCode::Stop, 0)),
            &Action::StopSounds => try!(self.write_action_header(OpCode::StopSounds, 0)),
            &Action::ToggleQuality => try!(self.write_action_header(OpCode::ToggleQuality, 0)),
            &Action::WaitForFrame { frame, num_actions_to_skip } => {
                try!(self.write_action_header(OpCode::WaitForFrame, 3));
                try!(self.inner.write_u16::<LittleEndian>(frame));
                try!(self.inner.write_u8(num_actions_to_skip));
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
        try!(self.inner.write_u8(opcode));
        assert!( opcode >= 0x80 || length == 0, "Opcodes less than 0x80 must have length 0" );
        if opcode >= 0x80 {
            try!(self.inner.write_u16::<LittleEndian>(length as u16));
        }
        Ok(())
    }

    fn write_c_string(&mut self, s: &str) -> Result<()> {
        try!(self.inner.write_all(s.as_bytes()));
        self.inner.write_u8(0)
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