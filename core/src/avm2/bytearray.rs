use crate::avm2::Error;
use crate::string::{FromWStr, WStr};
use flate2::read::*;
use flate2::Compression;
use gc_arena::Collect;
use std::cell::Cell;
use std::cmp;
use std::fmt::{self, Display, Formatter};
use std::io::prelude::*;
use std::io::{self, Read, SeekFrom};

#[derive(Clone, Collect, Debug, Copy, PartialEq, Eq)]
#[collect(no_drop)]
pub enum Endian {
    Big,
    Little,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    Zlib,
    Deflate,
    Lzma,
}

impl Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match *self {
            CompressionAlgorithm::Zlib => "zlib",
            CompressionAlgorithm::Deflate => "deflate",
            CompressionAlgorithm::Lzma => "lzma",
        };
        f.write_str(s)
    }
}

impl FromWStr for CompressionAlgorithm {
    type Err = Error;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s == b"zlib" {
            Ok(CompressionAlgorithm::Zlib)
        } else if s == b"deflate" {
            Ok(CompressionAlgorithm::Deflate)
        } else if s == b"lzma" {
            Ok(CompressionAlgorithm::Lzma)
        } else {
            Err("Unknown compression algorithm".into())
        }
    }
}

#[derive(Clone, Collect, Debug, Copy, PartialEq, Eq)]
#[collect(no_drop)]
pub enum ObjectEncoding {
    Amf0 = 0,
    Amf3 = 3,
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

    /// The encoding used when serializing/deserializing using readObject/writeObject
    object_encoding: ObjectEncoding,
}

impl ByteArrayStorage {
    /// Create a new ByteArrayStorage
    pub fn new() -> ByteArrayStorage {
        ByteArrayStorage {
            bytes: Vec::new(),
            position: Cell::new(0),
            endian: Endian::Big,
            object_encoding: ObjectEncoding::Amf3,
        }
    }

    /// Create a new ByteArrayStorage using an already existing vector
    pub fn from_vec(bytes: Vec<u8>) -> ByteArrayStorage {
        ByteArrayStorage {
            bytes,
            position: Cell::new(0),
            endian: Endian::Big,
            object_encoding: ObjectEncoding::Amf3,
        }
    }

    /// Write bytes at the next position in the ByteArray, growing if needed.
    #[inline]
    pub fn write_bytes<'gc>(&mut self, buf: &[u8]) -> Result<(), Error<'gc>> {
        self.write_at(buf, self.position.get())?;
        self.position.set(self.position.get() + buf.len());
        Ok(())
    }

    #[inline]
    pub fn write_bytes_within<'gc>(&mut self, start: usize, amnt: usize) -> Result<(), Error<'gc>> {
        self.write_at_within(start, amnt, self.position.get())?;
        self.position.set(self.position.get() + amnt);
        Ok(())
    }

    /// Reads any amount of bytes from the current position in the ByteArray
    #[inline]
    pub fn read_bytes<'gc>(&self, amnt: usize) -> Result<&[u8], Error<'gc>> {
        let bytes = self.read_at(amnt, self.position.get())?;
        self.position.set(self.position.get() + amnt);
        Ok(bytes)
    }

    /// Reads any amount of bytes at any offset in the ByteArray
    #[inline]
    pub fn read_at<'gc>(&self, amnt: usize, offset: usize) -> Result<&[u8], Error<'gc>> {
        self.bytes
            .get(offset..)
            .and_then(|bytes| bytes.get(..amnt))
            .ok_or_else(|| "EOFError: Reached EOF".into())
    }

    /// Write bytes at any offset in the ByteArray
    /// Will automatically grow the ByteArray to fit the new buffer
    pub fn write_at<'gc>(&mut self, buf: &[u8], offset: usize) -> Result<(), Error<'gc>> {
        let new_len = offset
            .checked_add(buf.len())
            .ok_or("RangeError: Cannot overflow usize")?;
        if self.len() < new_len {
            self.set_length(new_len);
        }
        self.bytes
            .get_mut(offset..new_len)
            .expect("ByteArray write out of bounds")
            .copy_from_slice(buf);
        Ok(())
    }

    /// Write bytes at any offset in the ByteArray
    /// Will return an error if the new buffer does not fit the ByteArray
    pub fn write_at_nongrowing<'gc>(
        &mut self,
        buf: &[u8],
        offset: usize,
    ) -> Result<(), Error<'gc>> {
        self.bytes
            .get_mut(offset..)
            .and_then(|bytes| bytes.get_mut(..buf.len()))
            .ok_or("RangeError: The specified range is invalid")?
            .copy_from_slice(buf);
        Ok(())
    }

    /// Write bytes at any offset in the ByteArray from within the current ByteArray using a memmove.
    /// Will automatically grow the ByteArray to fit the new buffer
    pub fn write_at_within<'gc>(
        &mut self,
        start: usize,
        amnt: usize,
        offset: usize,
    ) -> Result<(), Error<'gc>> {
        // First verify that reading from `start` to `amnt` is valid
        let end = start
            .checked_add(amnt)
            .filter(|result| *result <= self.len())
            .ok_or("RangeError: Reached EOF")?;

        // Second we resize our underlying buffer to ensure that writing `amnt` from `offset` is valid.
        let new_len = offset
            .checked_add(amnt)
            .ok_or("RangeError: Cannot overflow usize")?;
        if self.len() < new_len {
            self.set_length(new_len);
        }

        self.bytes.copy_within(start..end, offset);
        Ok(())
    }

    /// Compress the ByteArray into a temporary buffer
    pub fn compress<'gc>(
        &mut self,
        algorithm: CompressionAlgorithm,
    ) -> Result<Vec<u8>, Error<'gc>> {
        let mut buffer = Vec::new();
        match algorithm {
            CompressionAlgorithm::Zlib => {
                let mut compresser = ZlibEncoder::new(&*self.bytes, Compression::fast());
                compresser.read_to_end(&mut buffer)?;
            }
            CompressionAlgorithm::Deflate => {
                let mut compresser = DeflateEncoder::new(&*self.bytes, Compression::fast());
                compresser.read_to_end(&mut buffer)?;
            }
            #[cfg(feature = "lzma")]
            CompressionAlgorithm::Lzma => lzma_rs::lzma_compress(&mut &*self.bytes, &mut buffer)?,
            #[cfg(not(feature = "lzma"))]
            CompressionAlgorithm::Lzma => {
                return Err("Ruffle was not compiled with LZMA support".into())
            }
        }
        Ok(buffer)
    }

    /// Decompress the ByteArray into a temporary buffer
    pub fn decompress<'gc>(
        &mut self,
        algorithm: CompressionAlgorithm,
    ) -> Result<Vec<u8>, Error<'gc>> {
        let mut buffer = Vec::new();
        match algorithm {
            CompressionAlgorithm::Zlib => {
                let mut compresser = ZlibDecoder::new(&*self.bytes);
                compresser.read_to_end(&mut buffer)?;
            }
            CompressionAlgorithm::Deflate => {
                let mut compresser = DeflateDecoder::new(&*self.bytes);
                compresser.read_to_end(&mut buffer)?;
            }
            #[cfg(feature = "lzma")]
            CompressionAlgorithm::Lzma => lzma_rs::lzma_decompress(&mut &*self.bytes, &mut buffer)?,
            #[cfg(not(feature = "lzma"))]
            CompressionAlgorithm::Lzma => {
                return Err("Ruffle was not compiled with LZMA support".into())
            }
        }
        Ok(buffer)
    }

    pub fn read_utf<'gc>(&self) -> Result<&[u8], Error<'gc>> {
        let len = self.read_unsigned_short()?;
        let val = self.read_bytes(len.into())?;
        Ok(val)
    }

    pub fn write_boolean<'gc>(&mut self, val: bool) -> Result<(), Error<'gc>> {
        self.write_bytes(&[val as u8; 1])
    }

    pub fn read_boolean<'gc>(&self) -> Result<bool, Error<'gc>> {
        Ok(self.read_bytes(1)? != [0])
    }

    // Writes a UTF String into the buffer, with its length as a prefix
    pub fn write_utf<'gc>(&mut self, utf_string: &str) -> Result<(), Error<'gc>> {
        if let Ok(str_size) = u16::try_from(utf_string.len()) {
            self.write_unsigned_short(str_size)?;
            self.write_bytes(utf_string.as_bytes())
        } else {
            Err("RangeError: UTF String length must fit into a short".into())
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.bytes.clear();
        self.position.set(0)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.bytes.shrink_to_fit()
    }

    #[inline]
    pub fn set_length(&mut self, new_len: usize) {
        self.bytes.resize(new_len, 0);
    }

    pub fn get(&self, pos: usize) -> Option<u8> {
        self.bytes.get(pos).copied()
    }

    pub fn set(&mut self, item: usize, value: u8) {
        if self.len() < (item + 1) {
            self.bytes.resize(item + 1, 0)
        }

        *self.bytes.get_mut(item).unwrap() = value;
    }

    pub fn delete(&mut self, item: usize) {
        if let Some(i) = self.bytes.get_mut(item) {
            *i = 0;
        }
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.position.get()
    }

    #[inline]
    pub fn set_position(&self, pos: usize) {
        self.position.set(pos);
    }

    #[inline]
    pub fn endian(&self) -> Endian {
        self.endian
    }

    #[inline]
    pub fn set_endian(&mut self, new_endian: Endian) {
        self.endian = new_endian;
    }

    #[inline]
    pub fn object_encoding(&self) -> ObjectEncoding {
        self.object_encoding
    }

    #[inline]
    pub fn set_object_encoding(&mut self, new_object_encoding: ObjectEncoding) {
        self.object_encoding = new_object_encoding;
    }

    #[inline]
    pub fn bytes_available(&self) -> usize {
        self.len().saturating_sub(self.position.get())
    }
}

impl Write for ByteArrayStorage {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_bytes(buf).map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Failed to write to ByteArrayStorage")
        })?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for ByteArrayStorage {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self
            .read_bytes(cmp::min(buf.len(), self.bytes_available()))
            .map_err(|_| {
                io::Error::new(io::ErrorKind::Other, "Failed to read from ByteArrayStorage")
            })?;
        buf[..bytes.len()].copy_from_slice(bytes);
        Ok(bytes.len())
    }
}

impl Seek for ByteArrayStorage {
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.position.set(n as usize);
                return Ok(n);
            }
            SeekFrom::End(n) => (self.len(), n),
            SeekFrom::Current(n) => (self.position.get(), n),
        };

        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as usize)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as usize)
        };

        match new_pos {
            Some(n) => {
                self.position.set(n);
                Ok(n as u64)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }
}

macro_rules! impl_write{
    ($($method_name:ident $data_type:ty ), *)
    =>
    {
        impl ByteArrayStorage {
            $( pub fn $method_name<'gc> (&mut self, val: $data_type) -> Result<(), Error<'gc>> {
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
            $( pub fn $method_name<'gc> (&self) -> Result<$data_type, Error<'gc>> {
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
