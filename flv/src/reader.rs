use crate::error::Error as FlvError;
use std::io::{Error as IoError, ErrorKind, Result as IoResult, Seek, SeekFrom};

/// A reader that allows demuxing an FLV container.
pub struct FlvReader<'a> {
    source: &'a [u8],

    position: usize,
}

impl<'a> FlvReader<'a> {
    pub fn from_source(source: &'a [u8]) -> Self {
        FlvReader {
            source,
            position: 0,
        }
    }

    /// Reconstitute an FLV reader from its source parts.
    ///
    /// The seek position must point to identical data between breaking down an
    /// FLV reader and building it back up.
    pub fn from_parts(source: &'a [u8], position: usize) -> Self {
        FlvReader { source, position }
    }

    /// Break down an FLV reader into its source buffer and seek position.
    pub fn into_parts(self) -> (&'a [u8], usize) {
        (self.source, self.position)
    }

    /// Read a certain number of bytes from the buffer.
    ///
    /// This works like `Read`, but returns borrowed slices of the source
    /// buffer. The buffer position will be advanced so that repeated reads
    /// yield new data.
    ///
    /// If the requested number of bytes are not available, `EndOfData` is
    /// returned. This error should halt all parsing; callers should take care
    /// to return the reader to it's prior position and NOT return partial data
    /// (e.g. headers without body data) so that callers can retry later when
    /// more data has been downloaded.
    pub fn read(&mut self, count: usize) -> Result<&'a [u8], FlvError> {
        let start = self.position;
        let end = self
            .position
            .checked_add(count)
            .ok_or(FlvError::PointerTooBig)?;
        if end > self.source.len() {
            return Err(FlvError::EndOfData);
        }

        self.position = end;

        Ok(&self.source[start..end])
    }

    /// Read a certain number of bytes from the buffer without advancing the
    /// buffer position.
    ///
    /// If the requested number of bytes are not available, `EndOfData` is
    /// returned. This error should halt all parsing; callers should take care
    /// to return the reader to it's prior position and NOT return partial data
    /// (e.g. headers without body data) so that callers can retry later when
    /// more data has been downloaded.
    pub fn peek(&mut self, count: usize) -> Result<&'a [u8], FlvError> {
        let pos = self.position;
        let ret = self.read(count);

        self.position = pos;

        ret
    }

    pub fn read_u8(&mut self) -> Result<u8, FlvError> {
        Ok(self.read(1)?[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, FlvError> {
        Ok(u16::from_be_bytes(
            self.read(2)?.try_into().expect("two bytes"),
        ))
    }

    pub fn read_i16(&mut self) -> Result<i16, FlvError> {
        Ok(i16::from_be_bytes(
            self.read(2)?.try_into().expect("two bytes"),
        ))
    }

    pub fn read_u24(&mut self) -> Result<u32, FlvError> {
        let bytes = self.read(3)?;

        Ok(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]))
    }

    pub fn peek_u24(&mut self) -> Result<u32, FlvError> {
        let bytes = self.peek(3)?;

        Ok(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]))
    }

    pub fn read_u32(&mut self) -> Result<u32, FlvError> {
        Ok(u32::from_be_bytes(
            self.read(4)?.try_into().expect("four bytes"),
        ))
    }

    pub fn read_f64(&mut self) -> Result<f64, FlvError> {
        Ok(f64::from_be_bytes(
            self.read(8)?.try_into().expect("eight bytes"),
        ))
    }
}

impl<'a> Seek for FlvReader<'a> {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let newpos = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::Current(pos) => (self.position as i64 + pos)
                .try_into()
                .map_err(|e| IoError::new(ErrorKind::InvalidInput, e))?,
            SeekFrom::End(pos) => (self.source.len() as i64 - pos)
                .try_into()
                .map_err(|e| IoError::new(ErrorKind::InvalidInput, e))?,
        };

        self.position = newpos as usize;

        Ok(self.position as u64)
    }
}

#[cfg(test)]
#[allow(clippy::seek_from_current, clippy::seek_to_start_instead_of_rewind)]
mod tests {
    use crate::reader::FlvReader;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn valid_position_seek() {
        let data = vec![0; 4000];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(reader.seek(SeekFrom::Current(0)).unwrap(), 0);
        assert_eq!(reader.seek(SeekFrom::Current(4000)).unwrap(), 4000);
        assert_eq!(reader.seek(SeekFrom::Current(-2000)).unwrap(), 2000);
        assert_eq!(reader.seek(SeekFrom::Start(0)).unwrap(), 0);
        assert_eq!(reader.seek(SeekFrom::Start(4000)).unwrap(), 4000);
        assert_eq!(reader.seek(SeekFrom::End(0)).unwrap(), 4000);
        assert_eq!(reader.seek(SeekFrom::End(4000)).unwrap(), 0);
    }

    #[test]
    fn invalid_position_seek() {
        let data = vec![];
        let mut reader = FlvReader::from_parts(&data, 12000);

        assert_eq!(reader.seek(SeekFrom::Current(0)).unwrap(), 12000);
        assert_eq!(reader.seek(SeekFrom::Current(4000)).unwrap(), 16000);
        assert_eq!(reader.seek(SeekFrom::Current(-2000)).unwrap(), 14000);
        assert_eq!(reader.seek(SeekFrom::Start(0)).unwrap(), 0);
        assert_eq!(reader.seek(SeekFrom::Start(4000)).unwrap(), 4000);
        assert_eq!(reader.seek(SeekFrom::End(0)).unwrap(), 0);
        assert!(reader.seek(SeekFrom::End(4000)).is_err());
    }
}
