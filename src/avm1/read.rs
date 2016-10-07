use avm1::types::Action;
use avm1::opcode::OpCode;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Result};

pub struct Reader<R: Read> {
    inner: R,
    version: u8,
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

            Some(OpCode::NextFrame) => Action::NextFrame,
            Some(OpCode::Play) => Action::Play,
            Some(OpCode::PreviousFrame) => Action::PreviousFrame,
            Some(OpCode::Stop) => Action::Stop,
            Some(OpCode::StopSounds) => Action::StopSounds,
            Some(OpCode::ToggleQuality) => Action::ToggleQuality,

            _ => {
                let mut data = Vec::with_capacity(length);
                try!(action_reader.inner.read_to_end(&mut data));
                Action::Unknown { opcode: opcode, data: data }
            }
        };

        Ok(Some(action))
    }

    pub fn read_opcode_and_length(&mut self) -> Result<(u8, usize)> {
        let opcode = try!(self.inner.read_u8());
        let length = if opcode >= 0x80 {
            try!(self.inner.read_u16::<LittleEndian>()) as usize
        } else { 0 };
        Ok((opcode, length))
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