use crate::reader::FlvReader;
use bitflags::bitflags;

bitflags! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct TypeFlags: u8 {
        const HAS_AUDIO = 0b1000_0000;
        const HAS_VIDEO = 0b0010_0000;
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Header {
    pub version: u8,
    pub type_flags: TypeFlags,
    pub data_offset: u32,
}

impl Header {
    /// Parse an FLV header.
    ///
    /// If this yields `None`, then the given data stream is either not an FLV
    /// container or too short to parse.
    pub fn parse(reader: &mut FlvReader<'_>) -> Option<Self> {
        let signature = reader.read_u24()?;
        if signature != 0x464C56 {
            return None;
        }

        let version = reader.read_u8()?;
        let type_flags = TypeFlags::from_bits_retain(reader.read_u8()?);
        let data_offset = reader.read_u32()?;

        Some(Header {
            version,
            type_flags,
            data_offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::header::{Header, TypeFlags};
    use crate::reader::FlvReader;

    #[test]
    fn read_header() {
        let data = [0x46, 0x4C, 0x56, 0x01, 0xA0, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            Header::parse(&mut reader),
            Some(Header {
                version: 1,
                type_flags: TypeFlags::HAS_AUDIO | TypeFlags::HAS_VIDEO,
                data_offset: 0x12345678
            })
        );
    }
}
