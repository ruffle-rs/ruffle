use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub struct ATFTexture {
    pub width: u32,
    pub height: u32,
    pub cubemap: bool,
    pub format: ATFFormat,
    pub mip_count: u8,
    // A nested array of `[0..num_faces][0..mip_count]`, where each
    // entry is the texture data for that mip level and face.
    pub face_mip_data: Vec<Vec<Vec<u8>>>,
}

#[derive(FromPrimitive, Debug)]
pub enum ATFFormat {
    RGB888 = 0,
    RGBA8888 = 1,
    Compressed = 2,
    RawCompressed = 3,
    CompressedAlpha = 4,
    RawCompressedAlpha = 5,
    CompressedLossy = 0xc,
    CompressedLossyAlpha = 0xd,
}

impl ATFTexture {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        // Based on https://github.com/openfl/openfl/blob/develop/src/openfl/display3D/_internal/ATFReader.hx
        let bytes = &mut bytes;

        let mut string_bytes = [0; 3];
        bytes.read_exact(&mut string_bytes)?;

        if &string_bytes != b"ATF" {
            return Err(format!("Invalid ATF signature {string_bytes:?}").into());
        }

        let version;
        let _length;

        if bytes[3] == 0xFF {
            version = bytes[4];
            *bytes = &bytes[5..];
            _length = bytes.read_u32::<byteorder::LittleEndian>()?;
        } else {
            version = 0;
            _length = read_uint24(bytes)?;
        }

        let tdata = bytes.read_u8()?;
        let cubemap = (tdata >> 7) != 0;

        let format = ATFFormat::from_u8(tdata & 0x7f).ok_or_else(|| {
            format!(
                "Invalid ATF format {format} (version {version})",
                format = tdata & 0x7f,
                version = version
            )
        })?;
        let width = 1 << bytes.read_u8()?;
        let height = 1 << bytes.read_u8()?;

        let mip_count = bytes.read_u8()?;
        let num_faces = if cubemap { 6 } else { 1 };

        let mut face_mip_data = vec![vec![]; num_faces];

        #[allow(clippy::needless_range_loop)]
        for face in 0..num_faces {
            for _ in 0..mip_count {
                // All of the formats consist of a number of (u32_length, data[u32_length]) records.
                // For now, we just combine them into a single buffer to allow parsing to succeed.
                let num_records = match format {
                    ATFFormat::RGB888 | ATFFormat::RGBA8888 => 1,
                    ATFFormat::RawCompressed | ATFFormat::RawCompressedAlpha => 4,
                    ATFFormat::Compressed => 11,
                    ATFFormat::CompressedAlpha => {
                        return Err("CompressedAlpha not supported".into());
                    }
                    ATFFormat::CompressedLossy => 12,
                    ATFFormat::CompressedLossyAlpha => 17,
                };
                let mut all_data = vec![];
                for _ in 0..num_records {
                    let len = if version == 0 {
                        read_uint24(bytes)?
                    } else {
                        bytes.read_u32::<BigEndian>()?
                    };
                    let orig_len = all_data.len();
                    all_data.resize(orig_len + len as usize, 0);
                    bytes.read_exact(&mut all_data[orig_len..])?;
                }
                face_mip_data[face].push(all_data);
            }
        }
        Ok(ATFTexture {
            width,
            height,
            cubemap,
            format,
            mip_count,
            face_mip_data,
        })
    }
}

fn read_uint24<R: Read>(data: &mut R) -> Result<u32, Box<dyn std::error::Error>> {
    let ch1 = data.read_u8()? as u32;
    let ch2 = data.read_u8()? as u32;
    let ch3 = data.read_u8()? as u32;
    Ok(ch3 | (ch2 << 8) | (ch1 << 16))
}
