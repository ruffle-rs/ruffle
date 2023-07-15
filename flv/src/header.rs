use crate::error::Error;
use crate::reader::FlvReader;
use bitflags::bitflags;
use std::io::{Seek, SeekFrom};

bitflags! {
    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    pub struct TypeFlags: u8 {
        const HAS_AUDIO = 0b0000_0001;
        const HAS_VIDEO = 0b0000_0100;
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Header {
    pub version: u8,
    pub type_flags: TypeFlags,
    pub data_offset: u32,
}

impl Header {
    /// Parse an FLV header.
    ///
    /// The header must, at a minimum, contain the FLV magic, version number,
    /// valid type flags, and a valid offset into the data. The reader will
    /// seek to the start of the data tags if successful or retain it's prior
    /// position otherwise.
    pub fn parse(reader: &mut FlvReader<'_>) -> Result<Self, Error> {
        let old_position = reader.stream_position()?;

        let ret = (|| {
            let signature = reader.read_u24()?;
            if signature != 0x464C56 {
                return Err(Error::WrongMagic);
            }

            let version = reader.read_u8()?;
            let type_flags = TypeFlags::from_bits_retain(reader.read_u8()?);
            let data_offset = reader.read_u32()?;

            Ok(Header {
                version,
                type_flags,
                data_offset,
            })
        })();

        match ret {
            Ok(ret) => {
                reader.seek(SeekFrom::Start(ret.data_offset as u64))?;
                Ok(ret)
            }
            Err(e) => {
                reader.seek(SeekFrom::Start(old_position))?;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::header::{Header, TypeFlags};
    use crate::reader::FlvReader;

    #[test]
    fn read_header() {
        let data = [0x46, 0x4C, 0x56, 0x01, 0x05, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Header::parse(&mut reader),
            Ok(Header {
                version: 1,
                type_flags: TypeFlags::HAS_AUDIO | TypeFlags::HAS_VIDEO,
                data_offset: 0x12345678
            })
        );
    }
}
