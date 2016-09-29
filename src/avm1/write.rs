use avm1::types::Action;
use avm1::opcode::OpCode;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Result, Write};

pub struct ActionWriter<W: Write> {
    inner: W,
    version: u8,
}

impl<W: Write> ActionWriter<W> {
    pub fn new(inner: W, version: u8) -> ActionWriter<W> {
        ActionWriter { inner: inner, version: version }
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
            &Action::NextFrame => try!(self.write_action_header(OpCode::NextFrame, 0)),

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
        try!(self.inner.write_u16::<LittleEndian>(length as u16));
        Ok(())
    }
}