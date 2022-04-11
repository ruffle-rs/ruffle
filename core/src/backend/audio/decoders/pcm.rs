use super::{Decoder, SeekableDecoder};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Decoder for PCM audio data in a Flash file.
/// Flash exports this when you use the "Raw" compression setting.
/// 8-bit unsigned or 16-bit signed PCM.
pub struct PcmDecoder<R: Read> {
    inner: R,
    sample_rate: u16,
    is_stereo: bool,
    is_16_bit: bool,
}

impl<R: Read> PcmDecoder<R> {
    pub fn new(inner: R, is_stereo: bool, sample_rate: u16, is_16_bit: bool) -> Self {
        Self {
            inner,
            is_stereo,
            sample_rate,
            is_16_bit,
        }
    }

    #[inline]
    fn read_sample(&mut self) -> Option<i16> {
        let sample = if self.is_16_bit {
            self.inner.read_i16::<LittleEndian>().ok()?
        } else {
            (i16::from(self.inner.read_u8().ok()?) - 127) * 128
        };
        Some(sample)
    }
}

impl<R: Read> Iterator for PcmDecoder<R> {
    type Item = [i16; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let left = self.read_sample()?;
        let right = if self.is_stereo {
            self.read_sample()?
        } else {
            left
        };
        Some([left, right])
    }
}

impl<R: Read + Send + Sync> Decoder for PcmDecoder<R> {
    #[inline]
    fn num_channels(&self) -> u8 {
        if self.is_stereo {
            2
        } else {
            1
        }
    }

    #[inline]
    fn sample_rate(&self) -> u16 {
        self.sample_rate
    }
}

impl<R: AsRef<[u8]> + Send + Sync> SeekableDecoder for PcmDecoder<Cursor<R>> {
    #[inline]
    fn reset(&mut self) {
        self.inner.set_position(0);
    }

    #[inline]
    fn seek_to_sample_frame(&mut self, frame: u32) {
        let pos = u64::from(frame) * u64::from(self.num_channels()) * 2;
        self.inner.set_position(pos);
    }
}
