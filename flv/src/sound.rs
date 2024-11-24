use crate::error::Error;
use crate::FlvReader;
use std::io::Seek;

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum SoundFormat {
    LinearPCMPlatformEndian = 0,
    Adpcm = 1,
    MP3 = 2,
    LinearPCMLittleEndian = 3,
    Nellymoser16kHz = 4,
    Nellymoser8kHz = 5,
    Nellymoser = 6,
    G711ALawPCM = 7,
    G711MuLawPCM = 8,
    Aac = 10,
    Speex = 11,
    MP38kHz = 14,
    DeviceSpecific = 15,
}

impl TryFrom<u8> for SoundFormat {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::LinearPCMPlatformEndian),
            1 => Ok(Self::Adpcm),
            2 => Ok(Self::MP3),
            3 => Ok(Self::LinearPCMLittleEndian),
            4 => Ok(Self::Nellymoser16kHz),
            5 => Ok(Self::Nellymoser8kHz),
            6 => Ok(Self::Nellymoser),
            7 => Ok(Self::G711ALawPCM),
            8 => Ok(Self::G711MuLawPCM),
            10 => Ok(Self::Aac),
            11 => Ok(Self::Speex),
            14 => Ok(Self::MP38kHz),
            15 => Ok(Self::DeviceSpecific),
            unk => Err(Error::UnknownAudioFormatType(unk)),
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum SoundRate {
    R5_500 = 0,
    R11_000 = 1,
    R22_000 = 2,
    R44_000 = 3,
}

impl TryFrom<u8> for SoundRate {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::R5_500),
            1 => Ok(Self::R11_000),
            2 => Ok(Self::R22_000),
            3 => Ok(Self::R44_000),
            unk => Err(Error::UnknownAudioRate(unk)), //probably unreachable
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum SoundSize {
    Bits8 = 0,
    Bits16 = 1,
}

impl TryFrom<u8> for SoundSize {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Bits8),
            1 => Ok(Self::Bits16),
            unk => Err(Error::UnknownAudioSampleSize(unk)), //probably unreachable
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum SoundType {
    Mono = 0,
    Stereo = 1,
}

impl TryFrom<u8> for SoundType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Mono),
            1 => Ok(Self::Stereo),
            unk => Err(Error::UnknownAudioChannelCount(unk)), //probably unreachable
        }
    }
}

/// NOTE: For the `Aac*` variants, the discriminator byte is kept in the payload.
///
/// This is `0x00` for `AacSequenceHeader` and `0x01` for `AacRaw`.
/// While it's redundant, it's useful for the substream decoder.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AudioDataType<'a> {
    Raw(&'a [u8]),
    AacSequenceHeader(&'a [u8]),
    AacRaw(&'a [u8]),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AudioData<'a> {
    pub format: SoundFormat,
    pub rate: SoundRate,
    pub size: SoundSize,
    pub sound_type: SoundType,
    pub data: AudioDataType<'a>,
}

impl<'a> AudioData<'a> {
    /// Parse an audio data structure.
    ///
    /// This does not parse the actual audio data itself, which is instead
    /// returned as an array that must be provided to your audio decoder.
    ///
    /// `data_size` is the size of the entire audio data structure, *including*
    /// the header. Errors are yielded if the `data_size` is too small for the
    /// audio data present in the tag. This should not be confused for
    /// `EndOfData` which indicates that we've read past the end of the whole
    /// data stream.
    pub fn parse(reader: &mut FlvReader<'a>, data_size: u32) -> Result<Self, Error> {
        let start = reader.stream_position().expect("current position") as usize;
        let format_spec = reader.read_u8()?;

        let format = SoundFormat::try_from(format_spec >> 4)?;
        let rate = SoundRate::try_from((format_spec >> 2) & 0x03)?;
        let size = SoundSize::try_from((format_spec >> 1) & 0x01)?;
        let sound_type = SoundType::try_from(format_spec & 0x01)?;

        let header_size = reader.stream_position().expect("current position") as usize - start;
        if (data_size as usize) < header_size {
            return Err(Error::ShortAudioBlock);
        }
        let data = reader.read(data_size as usize - header_size)?;

        let data = match format {
            SoundFormat::Aac => {
                let aac_packet_type = data.first().ok_or(Error::ShortAudioBlock)?;
                // NOTE: The first byte is kept in the payload.
                match aac_packet_type {
                    //TODO: The FLV spec says this is explained in ISO 14496-3.
                    0 => AudioDataType::AacSequenceHeader(data),
                    1 => AudioDataType::AacRaw(data),
                    unk => return Err(Error::UnknownAacPacketType(*unk)),
                }
            }
            _ => AudioDataType::Raw(data),
        };

        Ok(AudioData {
            format,
            rate,
            size,
            sound_type,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::reader::FlvReader;
    use crate::sound::{AudioData, AudioDataType, SoundFormat, SoundRate, SoundSize, SoundType};

    #[test]
    fn read_audiodata() {
        let data = [0xBF, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            AudioData::parse(&mut reader, data.len() as u32),
            Ok(AudioData {
                format: SoundFormat::Speex,
                rate: SoundRate::R44_000,
                size: SoundSize::Bits16,
                sound_type: SoundType::Stereo,
                data: AudioDataType::Raw(&[0x12, 0x34, 0x56, 0x78])
            })
        );
    }

    #[test]
    fn read_audiodata_invalid_len() {
        let data = [0xBF, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            AudioData::parse(&mut reader, 0),
            Err(Error::ShortAudioBlock)
        );
    }

    #[test]
    fn read_audiodata_short_len() {
        let data = [0xBF, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            AudioData::parse(&mut reader, 2),
            Ok(AudioData {
                format: SoundFormat::Speex,
                rate: SoundRate::R44_000,
                size: SoundSize::Bits16,
                sound_type: SoundType::Stereo,
                data: AudioDataType::Raw(&[0x12])
            })
        );
    }

    #[test]
    fn read_audiodata_aac() {
        let data = [0xAD, 0x01, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            AudioData::parse(&mut reader, data.len() as u32),
            Ok(AudioData {
                format: SoundFormat::Aac,
                rate: SoundRate::R44_000,
                size: SoundSize::Bits8,
                sound_type: SoundType::Stereo,
                data: AudioDataType::AacRaw(&[0x01, 0x12, 0x34, 0x56, 0x78])
            })
        );
    }

    #[test]
    fn read_audiodata_aac_invalid() {
        let data = [0xAD, 0x02, 0x12, 0x34, 0x56, 0x78];
        let mut reader = FlvReader::from_source(&data);

        assert_eq!(
            AudioData::parse(&mut reader, data.len() as u32),
            Err(Error::UnknownAacPacketType(2))
        );
    }
}
