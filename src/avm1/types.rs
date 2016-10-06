#[derive(Debug,PartialEq)]
pub enum Action {
    NextFrame,
    Unknown { opcode: u8, data: Vec<u8> },
}