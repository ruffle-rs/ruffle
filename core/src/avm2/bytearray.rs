use crate::avm2::Error;
use flate2::read::*;
use flate2::Compression;
use gc_arena::Collect;
use std::cmp;
use std::convert::{TryFrom, TryInto};
use std::io;
use std::io::prelude::*;

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ByteArrayStorage {
    /// Underlying ByteArray
    bytes: Vec<u8>,

    // The current position to read/write from
    position: usize,

    /// This represents what endian to use while reading data.
    endian: Endian,
}

impl ByteArrayStorage {
    /// Create a new ByteArrayStorage
    pub fn new() -> ByteArrayStorage {
        ByteArrayStorage {
            bytes: Vec::new(),
            position: 0,
            endian: Endian::Big,
        }
    }

    /// Write a byte at next position in the bytearray
    pub fn write_byte(&mut self, byte: u8) {
        let bytes_len = self.bytes.len();
        // Allocate space for the byte
        self.position += 1;
        if self.position > bytes_len {
            self.bytes.resize(self.position, 0);
        }
        self.bytes[self.position - 1] = byte;
    }

    /// Write bytes at next position in bytearray (This function is similar to whats in std::io::Cursor)
    pub fn write_bytes(&mut self, buf: &[u8]) {
        // Make sure the internal buffer is as least as big as where we
        // currently are
        let len = self.bytes.len();
        if len < self.position {
            // use `resize` so that the zero filling is as efficient as possible
            self.bytes.resize(self.position, 0);
        }
        // Figure out what bytes will be used to overwrite what's currently
        // there (left), and what will be appended on the end (right)
        {
            let space = self.bytes.len() - self.position;
            let (left, right) = buf.split_at(cmp::min(space, buf.len()));
            self.bytes[self.position..self.position + left.len()].copy_from_slice(left);
            self.bytes.extend_from_slice(right);
        }

        // Bump us forward
        self.position += buf.len();
    }

    // Write bytes at an offset, ignoring the current position
    pub fn write_bytes_at(&mut self, buf: &[u8], offset: usize) {
        // Make sure the internal buffer is as least as big as where we
        // currently are
        let len = self.bytes.len();
        if len < offset {
            // use `resize` so that the zero filling is as efficient as possible
            self.bytes.resize(offset, 0);
        }
        // Figure out what bytes will be used to overwrite what's currently
        // there (left), and what will be appended on the end (right)
        {
            let space = self.bytes.len() - offset;
            let (left, right) = buf.split_at(cmp::min(space, buf.len()));
            self.bytes[offset..offset + left.len()].copy_from_slice(left);
            self.bytes.extend_from_slice(right);
        }
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
        // According to docs, this is where the bytearray should free resources
        self.bytes.shrink_to_fit();
        self.position = 0;
    }

    // Returns the bytearray compressed with zlib
    pub fn zlib_compress(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut compresser = ZlibEncoder::new(&*self.bytes, Compression::fast());
        compresser.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    // Returns the bytearray compressed with deflate
    pub fn deflate_compress(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut compresser = DeflateEncoder::new(&*self.bytes, Compression::fast());
        compresser.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    // Returns the bytearray decompressed with zlib
    pub fn zlib_decompress(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut compresser = ZlibDecoder::new(&*self.bytes);
        compresser.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    // Returns the bytearray decompressed with deflate
    pub fn deflate_decompress(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut compresser = DeflateDecoder::new(&*self.bytes);
        compresser.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// Set a new length for the bytearray
    pub fn set_length(&mut self, new_len: usize) {
        self.bytes.resize(new_len, 0);
    }

    // Reads exactly an amount of data
    pub fn read_exact(&mut self, amnt: usize) -> Result<&[u8], Error> {
        if self.position + amnt > self.bytes.len() {
            return Err("EOFError: Reached EOF".into());
        }
        let val = Ok(&self.bytes[self.position..self.position + amnt]);
        self.position += amnt;
        val
    }

    pub fn read_utf(&mut self) -> Result<String, Error> {
        let len = self.read_unsigned_short()?;
        let val = String::from_utf8_lossy(self.read_exact(len as usize)?);
        Ok(val.into_owned())
    }

    // Reads a i16 from the buffer
    pub fn read_short(&mut self) -> Result<i16, Error> {
        Ok(match self.endian {
            Endian::Big => i16::from_be_bytes(self.read_exact(2)?.try_into().unwrap()),
            Endian::Little => i16::from_le_bytes(self.read_exact(2)?.try_into().unwrap()),
        })
    }

    // Reads a u16 from the buffer
    pub fn read_unsigned_short(&mut self) -> Result<u16, Error> {
        Ok(match self.endian {
            Endian::Big => u16::from_be_bytes(self.read_exact(2)?.try_into().unwrap()),
            Endian::Little => u16::from_le_bytes(self.read_exact(2)?.try_into().unwrap()),
        })
    }

    // Reads a f64 from the buffer
    pub fn read_double(&mut self) -> Result<f64, Error> {
        Ok(match self.endian {
            Endian::Big => f64::from_be_bytes(self.read_exact(8)?.try_into().unwrap()),
            Endian::Little => f64::from_le_bytes(self.read_exact(8)?.try_into().unwrap()),
        })
    }

    // Reads a f32 from the buffer
    pub fn read_float(&mut self) -> Result<f32, Error> {
        Ok(match self.endian {
            Endian::Big => f32::from_be_bytes(self.read_exact(4)?.try_into().unwrap()),
            Endian::Little => f32::from_le_bytes(self.read_exact(4)?.try_into().unwrap()),
        })
    }

    // Reads a i32 from the buffer
    pub fn read_int(&mut self) -> Result<i32, Error> {
        Ok(match self.endian {
            Endian::Big => i32::from_be_bytes(self.read_exact(4)?.try_into().unwrap()),
            Endian::Little => i32::from_le_bytes(self.read_exact(4)?.try_into().unwrap()),
        })
    }

    // Reads a u32 from the buffer
    pub fn read_unsigned_int(&mut self) -> Result<u32, Error> {
        Ok(match self.endian {
            Endian::Big => u32::from_be_bytes(self.read_exact(4)?.try_into().unwrap()),
            Endian::Little => u32::from_le_bytes(self.read_exact(4)?.try_into().unwrap()),
        })
    }

    // Reads byte from buffer, returns false if zero, otherwise true
    pub fn read_boolean(&mut self) -> Result<bool, Error> {
        Ok(*self.read_exact(1)?.first().unwrap() != 0)
    }

    // Reads a i8 from the buffer
    pub fn read_byte(&mut self) -> Result<i8, Error> {
        Ok(match self.endian {
            Endian::Big => i8::from_be_bytes(self.read_exact(1)?.try_into().unwrap()),
            Endian::Little => i8::from_le_bytes(self.read_exact(1)?.try_into().unwrap()),
        })
    }

    // Reads a u8 from the buffer
    pub fn read_unsigned_byte(&mut self) -> Result<u8, Error> {
        Ok(match self.endian {
            Endian::Big => u8::from_be_bytes(self.read_exact(1)?.try_into().unwrap()),
            Endian::Little => u8::from_le_bytes(self.read_exact(1)?.try_into().unwrap()),
        })
    }

    // Writes a f32 to the buffer
    pub fn write_float(&mut self, val: f32) {
        let float_bytes = match self.endian {
            Endian::Big => val.to_be_bytes(),
            Endian::Little => val.to_le_bytes(),
        };
        self.write_bytes(&float_bytes);
    }

    // Writes a f64 to the buffer
    pub fn write_double(&mut self, val: f64) {
        let double_bytes = match self.endian {
            Endian::Big => val.to_be_bytes(),
            Endian::Little => val.to_le_bytes(),
        };
        self.write_bytes(&double_bytes);
    }

    // Writes a 1 byte to the buffer, either 1 or 0
    pub fn write_boolean(&mut self, val: bool) {
        self.write_bytes(&[val as u8; 1]);
    }

    // Writes a i32 to the buffer
    pub fn write_int(&mut self, val: i32) {
        let int_bytes = match self.endian {
            Endian::Big => val.to_be_bytes(),
            Endian::Little => val.to_le_bytes(),
        };
        self.write_bytes(&int_bytes);
    }

    // Writes a u32 to the buffer
    pub fn write_unsigned_int(&mut self, val: u32) {
        let uint_bytes = match self.endian {
            Endian::Big => val.to_be_bytes(),
            Endian::Little => val.to_le_bytes(),
        };
        self.write_bytes(&uint_bytes);
    }

    // Writes a i16 to the buffer
    pub fn write_short(&mut self, val: i16) {
        let short_bytes = match self.endian {
            Endian::Big => val.to_be_bytes(),
            Endian::Little => val.to_le_bytes(),
        };
        self.write_bytes(&short_bytes);
    }

    // Writes a u16 to the buffer
    pub fn write_unsigned_short(&mut self, val: u16) {
        let ushort_bytes = match self.endian {
            Endian::Big => val.to_be_bytes(),
            Endian::Little => val.to_le_bytes(),
        };
        self.write_bytes(&ushort_bytes);
    }

    // Writes a UTF String into the buffer, with its length as a prefix
    pub fn write_utf(&mut self, utf_string: &str) -> Result<(), Error> {
        if let Ok(str_size) = u16::try_from(utf_string.len()) {
            self.write_unsigned_short(str_size);
            self.write_bytes(utf_string.as_bytes());
        } else {
            return Err("RangeError: UTF String length must fit into a short".into());
        }
        Ok(())
    }

    pub fn get(&self, item: usize) -> Option<u8> {
        self.bytes.get(item).copied()
    }

    pub fn set(&mut self, item: usize, value: u8) {
        if self.bytes.len() < (item + 1) {
            self.bytes.resize(item + 1, 0)
        }

        *self.bytes.get_mut(item).unwrap() = value;
    }

    pub fn delete(&mut self, item: usize) {
        if let Some(i) = self.bytes.get_mut(item) {
            *i = 0;
        }
    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }

    pub fn add_position(&mut self, amnt: usize) {
        self.position += amnt;
    }

    pub fn endian(&self) -> &Endian {
        &self.endian
    }

    pub fn set_endian(&mut self, new_endian: Endian) {
        self.endian = new_endian;
    }
}

impl Write for ByteArrayStorage {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_bytes(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
