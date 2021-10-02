use crate::{error::Result, string::SwfStr};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io;

pub struct ReaderBase<'a> {
    base: &'a [u8],
    curr: &'a [u8],
}

impl<'a> ReaderBase<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self {
            base: data,
            curr: data,
        }
    }

    /// Returns a reference to the underlying `Reader`.
    #[inline]
    pub const fn get_ref(&self) -> &'a [u8] {
        self.curr
    }

    /// Returns a mutable reference to the underlying `Reader`.
    ///
    /// Reading from this reference is not recommended.
    #[inline]
    pub fn get_mut(&mut self) -> &mut &'a [u8] {
        &mut self.curr
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.curr.as_ptr() as usize - self.base.as_ptr() as usize
    }

    // TODO: Make this fallible?
    #[inline]
    pub fn seek(&mut self, offset: isize) {
        let position = self.position();
        let position = (position as isize + offset) as usize;
        let position = position.min(self.base.len());
        self.curr = &self.base[position..];
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(ReadBytesExt::read_u8(&mut self.curr)?)
    }

    #[inline]
    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(ReadBytesExt::read_u16::<LittleEndian>(&mut self.curr)?)
    }

    #[inline]
    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(ReadBytesExt::read_u32::<LittleEndian>(&mut self.curr)?)
    }

    #[inline]
    pub fn read_u64(&mut self) -> Result<u64> {
        Ok(ReadBytesExt::read_u64::<LittleEndian>(&mut self.curr)?)
    }

    #[inline]
    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(ReadBytesExt::read_i8(&mut self.curr)?)
    }

    #[inline]
    pub fn read_i16(&mut self) -> Result<i16> {
        Ok(ReadBytesExt::read_i16::<LittleEndian>(&mut self.curr)?)
    }

    #[inline]
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(ReadBytesExt::read_i32::<LittleEndian>(&mut self.curr)?)
    }

    #[inline]
    pub fn read_f32(&mut self) -> Result<f32> {
        Ok(ReadBytesExt::read_f32::<LittleEndian>(&mut self.curr)?)
    }

    #[inline]
    pub fn read_f64(&mut self) -> Result<f64> {
        Ok(ReadBytesExt::read_f64::<LittleEndian>(&mut self.curr)?)
    }

    #[inline]
    pub fn read_encoded_u32(&mut self) -> Result<u32> {
        let mut val: u32 = 0;
        for i in (0..35).step_by(7) {
            let byte = self.read_u8()? as u32;
            val |= (byte & 0b0111_1111) << i;
            if byte & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(val)
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.curr.len() >= len {
            let new_slice = &self.curr[..len];
            self.curr = &self.curr[len..];
            Ok(new_slice)
        } else {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data for slice").into())
        }
    }

    pub fn read_slice_to_end(&mut self) -> &'a [u8] {
        let res = &self.curr;
        self.curr = &[];
        res
    }

    #[inline]
    pub fn read_str(&mut self) -> Result<&'a SwfStr> {
        let s = SwfStr::from_bytes_null_terminated(self.curr).ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data for string")
        })?;
        self.curr = &self.curr[s.len() + 1..];
        Ok(s)
    }

    #[inline]
    pub fn read_str_with_len(&mut self, len: usize) -> Result<&'a SwfStr> {
        let bytes = &self.read_slice(len)?;
        // TODO: Maybe just strip the possible trailing null char instead of looping here.
        Ok(SwfStr::from_bytes_null_terminated(bytes).unwrap_or_else(|| SwfStr::from_bytes(bytes)))
    }
}
