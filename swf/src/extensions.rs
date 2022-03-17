use crate::error::Result;
use crate::string::SwfStr;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io;

pub trait ReadSwfExt<'a> {
    fn as_mut_slice(&mut self) -> &mut &'a [u8];

    fn as_slice(&self) -> &'a [u8];

    fn pos(&self, data: &[u8]) -> usize {
        self.as_slice().as_ptr() as usize - data.as_ptr() as usize
    }

    // TODO: Make this fallible?
    fn seek(&mut self, data: &'a [u8], relative_offset: isize) {
        let pos = self.pos(data);
        let pos = (pos as isize + relative_offset) as usize;
        let pos = pos.min(data.len());
        *self.as_mut_slice() = &data[pos..];
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        Ok(ReadBytesExt::read_u8(self.as_mut_slice())?)
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16> {
        Ok(ReadBytesExt::read_u16::<LittleEndian>(self.as_mut_slice())?)
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32> {
        Ok(ReadBytesExt::read_u32::<LittleEndian>(self.as_mut_slice())?)
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64> {
        Ok(ReadBytesExt::read_u64::<LittleEndian>(self.as_mut_slice())?)
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8> {
        Ok(ReadBytesExt::read_i8(self.as_mut_slice())?)
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16> {
        Ok(ReadBytesExt::read_i16::<LittleEndian>(self.as_mut_slice())?)
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32> {
        Ok(ReadBytesExt::read_i32::<LittleEndian>(self.as_mut_slice())?)
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32> {
        Ok(ReadBytesExt::read_f32::<LittleEndian>(self.as_mut_slice())?)
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64> {
        Ok(ReadBytesExt::read_f64::<LittleEndian>(self.as_mut_slice())?)
    }

    #[inline]
    fn read_encoded_u32(&mut self) -> Result<u32> {
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

    fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
        let slice = self.as_mut_slice();
        if slice.len() >= len {
            let new_slice = &slice[..len];
            *slice = &slice[len..];
            Ok(new_slice)
        } else {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data for slice").into())
        }
    }

    fn read_slice_to_end(&mut self) -> &'a [u8] {
        let slice = self.as_mut_slice();
        let res = &slice[..];
        *slice = &[];
        res
    }

    #[inline]
    fn read_str(&mut self) -> Result<&'a SwfStr> {
        let slice = self.as_mut_slice();
        let s = SwfStr::from_bytes_null_terminated(slice).ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data for string")
        })?;
        *slice = &slice[s.len() + 1..];
        Ok(s)
    }

    #[inline]
    fn read_str_with_len(&mut self) -> Result<&'a SwfStr> {
        let len = self.read_u8()?;
        let bytes = &self.read_slice(len.into())?;
        // TODO: Maybe just strip the possible trailing null char instead of looping here.
        Ok(SwfStr::from_bytes_null_terminated(bytes).unwrap_or_else(|| SwfStr::from_bytes(bytes)))
    }
}
