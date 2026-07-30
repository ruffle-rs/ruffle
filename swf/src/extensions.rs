use crate::error::Result;
use crate::string::SwfStr;
use std::io;

macro_rules! impl_read_little_endian {
    ( $($name:ident for $ty:ty;)* ) => {$(
        #[inline]
        fn $name(&mut self) -> Result<$ty> {
            self.read_array().map(<$ty>::from_le_bytes)
        }
    )*}
}

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

    fn seek_absolute(&mut self, data: &'a [u8], pos: usize) {
        let pos = pos.min(data.len());
        *self.as_mut_slice() = &data[pos..];
    }

    #[inline]
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        use std::io::Read as _;

        let mut buf = [0u8; N];
        self.as_mut_slice().read_exact(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
        let slice = self.as_mut_slice();
        if let Some((bytes, rest)) = slice.split_at_checked(len) {
            *slice = rest;
            Ok(bytes)
        } else {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough data for slice").into())
        }
    }

    impl_read_little_endian! {
        read_u8 for u8;
        read_u16 for u16;
        read_u32 for u32;
        read_u64 for u64;
        read_i8 for i8;
        read_i16 for i16;
        read_i32 for i32;
        read_f32 for f32;
        read_f64 for f64;
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
