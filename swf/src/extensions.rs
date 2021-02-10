use crate::byteorder::{LittleEndian, ReadBytesExt};
use crate::error::Result;
use crate::string::SwfStr;
use std::io::{self, Read};

pub trait ReadSwfExt<'a> {
    fn as_mut_slice(&mut self) -> &mut &'a [u8];

    // TODO: Make this fallible?
    fn seek(&mut self, data: &'a [u8], relative_offset: isize) {
        let mut pos = self.as_mut_slice().as_ptr() as usize - data.as_ptr() as usize;
        pos = (pos as isize + relative_offset) as usize;
        pos = pos.min(data.len());
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
    fn read_fixed8(&mut self) -> Result<f32> {
        ReadSwfExt::read_i16(self).map(|n| f32::from(n) / 256f32)
    }

    #[inline]
    fn read_fixed16(&mut self) -> Result<f64> {
        ReadSwfExt::read_i32(self).map(|n| f64::from(n) / 65536f64)
    }

    #[inline]
    fn read_encoded_u32(&mut self) -> Result<u32> {
        let mut val = 0u32;
        for i in 0..5 {
            let byte = ReadSwfExt::read_u8(self)?;
            val |= u32::from(byte & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                break;
            }
        }
        Ok(val)
    }

    #[inline]
    fn read_f64_me(&mut self) -> Result<f64> {
        // Flash weirdly stores (some?) f64 as two LE 32-bit chunks.
        // First word is the hi-word, second word is the lo-word.
        let mut num = [0u8; 8];
        self.as_mut_slice().read_exact(&mut num)?;
        num.swap(0, 4);
        num.swap(1, 5);
        num.swap(2, 6);
        num.swap(3, 7);
        Ok(ReadBytesExt::read_f64::<LittleEndian>(&mut &num[..])?)
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
    fn read_str_with_len(&mut self, len: usize) -> Result<&'a SwfStr> {
        let bytes = &self.read_slice(len)?;
        // TODO: Maybe just strip the possible trailing null char instead of looping here.
        Ok(SwfStr::from_bytes_null_terminated(bytes).unwrap_or_else(|| SwfStr::from_bytes(bytes)))
    }
}
