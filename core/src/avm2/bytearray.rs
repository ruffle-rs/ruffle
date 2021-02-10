use gc_arena::Collect;
use std::mem;



#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ByteArrayStorage {
    /// Underlying ByteArray 
    bytes: Vec<u8>,

    // The current position to read/write from
    position: usize,

    /// This represents what endian to use while reading data
    endian: String,

    /// Boolean representing if the ByteArray is sharable. If false passing the ByteArray will create a brand new ByteArray, 
    /// but if it's true it will keep the same underlying data.
    shareable: bool,
}



impl ByteArrayStorage {
    /// Create a new ByteArrayStorage
    pub fn new() -> ByteArrayStorage {
        ByteArrayStorage {
            bytes: Vec::new(),
            position: 0,
            endian: "BIG_ENDIAN".to_string(),
            shareable: false
        }
    }


    /// Write a byte at next position in the bytearray
    pub fn write_byte(&mut self, byte: u8) {

        let bytes_len = self.bytes.len();
        // Allocate space for the byte
        self.position += 1;
        if self.position > bytes_len {
            self.bytes.resize(bytes_len + (self.position - bytes_len), 0);
        }
        mem::replace(&mut self.bytes[self.position - 1], byte);

    }

    /// Write bytes at next position in bytearray
    pub fn write_bytes(&mut self, other: Vec<u8>) {

        let bytes_len = self.bytes.len();
        let other_offset = self.position + other.len();
        // Allocate enough space for the new bytes
        if other_offset > bytes_len {
            self.bytes.resize(bytes_len + (other_offset - bytes_len), 0);
        }
        for (i, byte) in other.iter().enumerate(){
            mem::replace(&mut self.bytes[self.position + i], *byte);
        }
        // Set new position
        self.position += other_offset - bytes_len;

    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn reborrow(&self) -> ByteArrayStorage{
        ByteArrayStorage {
            bytes: self.bytes.clone(),
            position: self.position,
            endian: self.endian.clone(),
            shareable: self.shareable
        }
    }
}