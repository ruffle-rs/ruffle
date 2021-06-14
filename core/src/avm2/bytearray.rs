use crate::avm2::Error;
use flate2::read::*;
use flate2::Compression;
use gc_arena::Collect;
use std::convert::{TryFrom, TryInto};
use std::io;
use std::io::prelude::*;
use std::ops::Range;
use std::io::Read;
use std::cell::Cell;

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

    /// The current position to read/write from
    position: Cell<usize>,

    /// This represents what endian to use while reading/writing data.
    endian: Endian,
}

impl ByteArrayStorage {
    /// Create a new ByteArrayStorage
    pub fn new() -> ByteArrayStorage {
        ByteArrayStorage {
            bytes: Vec::new(),
            position: Cell::new(0),
            endian: Endian::Big,
        }
    }

    /// Write bytes at the next position in the ByteArray, growing if needed.
    #[inline]
    pub fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.write_at(buf, self.position.get())?;
        self.position.set(self.position.get() + buf.len());
        Ok(())
    }

    /// Reads any amount of bytes from the current position in the ByteArray
    #[inline]
    pub fn read_bytes(&mut self, amnt: usize) -> Result<&[u8], Error> {
        let bytes = self.read_at(amnt, self.position.get())?;
        self.position.set(self.position.get() + amnt);
        Ok(bytes)
    }

    /// Reads any amount of bytes at any offset in the ByteArray
    #[inline]
    pub fn read_at(&self, amnt: usize, offset: usize) -> Result<&[u8], Error> {
        let end: Result<_, Error> = self.position.get().checked_add(amnt).ok_or("RangeError: Cannot overflow usize".into());
        self.bytes.get(offset..end?).ok_or("RangeError: Reached EOF".into())
    }

    /// Write bytes at any offset in the ByteArray
    /// Will automatically grow the ByteArray to fit the new buffer
    pub fn write_at(&mut self, buf: &[u8], offset: usize) -> Result<(), Error> {
        match offset.checked_add(buf.len()) {
            Some(new_len) => {
                if self.bytes.len() < new_len {
                    self.bytes.resize(new_len, 0);
                }
                unsafe { self.write_at_unchecked(buf, offset) }
            }
            None => return Err("RangeError: The length of this ByteArray is too big".into())
        }
        Ok(())
    }

    /// Write bytes at any offset in the ByteArray
    /// Will return an error if the new buffer does not fit the ByteArray
    pub fn write_at_nongrowing(&mut self, buf: &[u8], offset: usize) -> Result<(), Error> {
        match offset.checked_add(buf.len()) {
            Some(new_len) => {
                if self.bytes.len() < new_len {
                    return Err("RangeError: The specified range is invalid".into());
                }
                unsafe { self.write_at_unchecked(buf, offset) }
            }
            None => return Err("RangeError: The length of this ByteArray is too big".into())
        }
        Ok(())
    }

    /// Writes a buffer into the ByteArray at any offset without bounds checking.
    /// Callers must garuntee that:
    /// 1. The length of the buffer + offset MUST be less than or equal to the length of the ByteArray
    /// 2. The length of the buffer + offset MUST not overflow usize
    #[inline]
    pub unsafe fn write_at_unchecked(&mut self, buf: &[u8], offset: usize) {
        std::ptr::copy_nonoverlapping(buf.as_ptr(), self.bytes.as_mut_ptr().add(offset), buf.len());
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
        // According to docs, this is where the bytearray should free resources
        self.bytes.shrink_to_fit();
        self.position.set(0);
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

    pub fn read_utf(&mut self) -> Result<String, Error> {
        let len = self.read_unsigned_short()?;
        let val = String::from_utf8_lossy(self.read_bytes(len.into())?);
        Ok(val.into_owned())
    }

    pub fn write_boolean(&mut self, val: bool) -> Result<(), Error> {
        self.write_bytes(&[val as u8; 1])
    }

    pub fn read_boolean(&mut self) -> Result<bool, Error> {
        Ok(self.read_bytes(1)?[0] != 0)
    }

    // Writes a UTF String into the buffer, with its length as a prefix
    pub fn write_utf(&mut self, utf_string: &str) -> Result<(), Error> {
        if let Ok(str_size) = u16::try_from(utf_string.len()) {
            self.write_unsigned_short(str_size)?;
            self.write_bytes(utf_string.as_bytes())
        } else {
            return Err("RangeError: UTF String length must fit into a short".into());
        }
    }

    pub fn get(&self, item: usize) -> Option<u8> {
        self.bytes.get(item).copied()
    }

    pub fn get_range(&self, item: Range<usize>) -> Option<&[u8]> {
        self.bytes.get(item)
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

    pub fn position(&self) -> usize {
        self.position.get()
    }

    pub fn set_position(&self, pos: usize) {
        self.position.set(pos);
    }

    pub fn add_position(&self, amnt: usize) {
        self.position.set(self.position.get() + amnt);
    }

    pub fn endian(&self) -> &Endian {
        &self.endian
    }

    pub fn set_endian(&mut self, new_endian: Endian) {
        self.endian = new_endian;
    }
}

impl std::ops::Deref for ByteArrayStorage {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl std::ops::DerefMut for ByteArrayStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl Write for ByteArrayStorage {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_bytes(buf).map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to write to ByteArrayStorage"))?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for ByteArrayStorage {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self.read_bytes(buf.len()).map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to read from ByteArray"))?;
        buf.copy_from_slice(bytes);
        Ok(buf.len())
    }
}

macro_rules! impl_write{
    ($($method_name:ident $data_type:ty ), *)
    =>
    {
        impl ByteArrayStorage {
            $( pub fn $method_name (&mut self, val: $data_type) -> Result<(), Error> { 
                let val_bytes = match self.endian {
                    Endian::Big => val.to_be_bytes(),
                    Endian::Little => val.to_le_bytes(),
                };
                self.write_bytes(&val_bytes)
             } )*
        }
    }
}

macro_rules! impl_read{
    ($($method_name:ident $size:expr; $data_type:ty ), *)
    =>
    {
        impl ByteArrayStorage {
            $( pub fn $method_name (&mut self) -> Result<$data_type, Error> { 
                Ok(match self.endian {
                    Endian::Big => <$data_type>::from_be_bytes(self.read_bytes($size)?.try_into().unwrap()),
                    Endian::Little => <$data_type>::from_le_bytes(self.read_bytes($size)?.try_into().unwrap())
                })
             } )*
        }
    }
}

impl_write!(write_float f32, write_double f64, write_int i32, write_unsigned_int u32, write_short i16, write_unsigned_short u16);
impl_read!(read_float 4; f32, read_double 8; f64, read_int 4; i32, read_unsigned_int 4; u32, read_short 2; i16, read_unsigned_short 2; u16, read_byte 1; i8, read_unsigned_byte 1; u8);

impl Default for ByteArrayStorage {
    fn default() -> Self {
        Self::new()
    }
}
