use avm1::types::Action;
use avm1::opcode::OpCode;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Result};

pub struct ActionReader<R: Read> {
    inner: R,
    version: u8,
}

impl<R: Read> ActionReader<R> {
    pub fn new(inner: R, version: u8) -> ActionReader<R> {
        ActionReader { inner: inner, version: version }
    }

    pub fn read_action(&mut self) -> Result<Option<Action>> {
        unimplemented!()
    }
}