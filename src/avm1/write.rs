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
    
    pub fn write_action(&mut self, action: &Action) -> Result<()> {
        unimplemented!()
    }
}