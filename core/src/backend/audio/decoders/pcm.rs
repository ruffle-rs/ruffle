use super::{Decoder, SeekableDecoder};
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
        PcmDecoder {
            inner,
            is_stereo,
            sample_rate,
            is_16_bit,
        }
    }
}

impl<R: Read> Iterator for PcmDecoder<R> {
    type Item = [i16; 2];
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_stereo {
            if self.is_16_bit {
                let mut left = [0u8; 2];
                let mut right = [0u8; 2];
                self.inner.read_exact(&mut left).ok()?;
                self.inner.read_exact(&mut right).ok()?;
                let left = i16::from_le_bytes(left);
                let right = i16::from_le_bytes(right);
                Some([left, right])
            } else {
                let mut bytes = [0u8];
                self.inner.read_exact(&mut bytes).ok()?;
                let sample = (i16::from(bytes[0]) - 127) * 128;
                Some([sample, sample])
            }
        } else if self.is_16_bit {
            let mut bytes = [0u8; 2];
            self.inner.read_exact(&mut bytes).ok()?;
            let sample = i16::from_le_bytes(bytes);
            Some([sample, sample])
        } else {
            let mut bytes = [0u8; 2];
            self.inner.read_exact(&mut bytes).ok()?;
            let left = (i16::from(bytes[0]) - 127) * 128;
            let right = (i16::from(bytes[1]) - 127) * 128;
            Some([left, right])
        }
    }
}

impl<R: Read> Decoder for PcmDecoder<R> {
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

impl<R: AsRef<[u8]>> SeekableDecoder for PcmDecoder<Cursor<R>> {
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
