use super::{Decoder, SeekableDecoder};
use bitstream_io::{BigEndian, BitRead, BitReader};
use std::io::{Cursor, Read};

const INDEX_TABLE: [&[i16]; 4] = [
    &[-1, 2],
    &[-1, -1, 2, 4],
    &[-1, -1, -1, -1, 2, 4, 6, 8],
    &[-1, -1, -1, -1, -1, -1, -1, -1, 1, 2, 4, 6, 8, 10, 13, 16],
];

const STEP_TABLE: [u16; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

const SAMPLE_DELTA_CALCULATOR: [fn(u16, u32) -> u16; 4] = [
    // 2 bits
    |step: u16, magnitude: u32| {
        let mut delta = step >> 1;
        if magnitude & 1 != 0 {
            delta += step;
        };
        delta
    },
    // 3 bits
    |step: u16, magnitude: u32| {
        let mut delta = step >> 2;
        if magnitude & 1 != 0 {
            delta += step >> 1;
        }
        if magnitude & 2 != 0 {
            delta += step;
        }
        delta
    },
    // 4 bits
    |step: u16, magnitude: u32| {
        let mut delta = step >> 3;
        if magnitude & 1 != 0 {
            delta += step >> 2;
        }
        if magnitude & 2 != 0 {
            delta += step >> 1;
        }
        if magnitude & 4 != 0 {
            delta += step;
        }
        delta
    },
    // 5 bits
    |step: u16, magnitude: u32| {
        let mut delta = step >> 4;
        if magnitude & 1 != 0 {
            delta += step >> 3;
        }
        if magnitude & 2 != 0 {
            delta += step >> 2;
        }
        if magnitude & 4 != 0 {
            delta += step >> 1;
        }
        if magnitude & 8 != 0 {
            delta += step;
        }
        delta
    },
];

#[derive(Clone, Default)]
struct Channel {
    sample: i16,
    step_index: i16,
}

pub struct AdpcmDecoder<R: Read> {
    inner: BitReader<R, BigEndian>,
    sample_rate: u16,
    bits_per_sample: usize,
    sample_num: u16,
    channels: Vec<Channel>,
    decoder: fn(u16, u32) -> u16,
}

impl<R: Read> AdpcmDecoder<R> {
    pub fn new(inner: R, is_stereo: bool, sample_rate: u16) -> Self {
        let mut reader = BitReader::new(inner);
        let bits_per_sample = reader.read::<u8>(2).unwrap_or_else(|e| {
            log::warn!("Invalid ADPCM stream: {}", e);
            0
        }) as usize
            + 2;

        let num_channels = if is_stereo { 2 } else { 1 };

        Self {
            inner: reader,
            sample_rate,
            bits_per_sample,
            sample_num: 0,
            channels: vec![Default::default(); num_channels],
            decoder: SAMPLE_DELTA_CALCULATOR[bits_per_sample - 2],
        }
    }
}

impl<R: Read> Iterator for AdpcmDecoder<R> {
    type Item = [i16; 2];

    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_num == 0 {
            // The initial sample values are NOT byte-aligned.
            for channel in &mut self.channels {
                channel.sample = self.inner.read_signed(16).ok()?;
                channel.step_index = self.inner.read::<u16>(6).ok()? as i16;
            }
        }

        self.sample_num = (self.sample_num + 1) % 4095;

        for channel in &mut self.channels {
            let step = STEP_TABLE[channel.step_index as usize];

            // `data` is sign-magnitude, NOT two's complement.
            let data = self.inner.read::<u32>(self.bits_per_sample as u32).ok()?;
            let sign_mask = 1 << (self.bits_per_sample - 1);
            let magnitude = data & !sign_mask;

            // (data + 0.5) * step / 2^(bits_per_sample - 2)
            let delta = (self.decoder)(step, magnitude);

            channel.sample = if (data & sign_mask) != 0 {
                (channel.sample as i32 - delta as i32).max(i16::MIN.into())
            } else {
                (channel.sample as i32 + delta as i32).min(i16::MAX.into())
            } as i16;

            channel.step_index += INDEX_TABLE[self.bits_per_sample - 2][magnitude as usize];
            channel.step_index = channel.step_index.clamp(0, STEP_TABLE.len() as i16 - 1);
        }

        let left = self.channels[0].sample;
        let right = self.channels.get(1).map_or(left, |c| c.sample);
        Some([left, right])
    }
}

impl<R: std::io::Read> Decoder for AdpcmDecoder<R> {
    #[inline]
    fn num_channels(&self) -> u8 {
        self.channels.len() as u8
    }

    #[inline]
    fn sample_rate(&self) -> u16 {
        self.sample_rate
    }
}

impl<R: AsRef<[u8]> + Default> SeekableDecoder for AdpcmDecoder<Cursor<R>> {
    #[inline]
    fn reset(&mut self) {
        // TODO: This is funky.
        // I want to reset the `BitStream` and `Cursor` to their initial positions,
        // but have to work around the borrowing rules of Rust.
        let bit_stream = std::mem::replace(&mut self.inner, BitReader::new(Default::default()));
        let mut cursor = bit_stream.into_reader();
        cursor.set_position(0);
        *self = AdpcmDecoder::new(cursor, self.num_channels() == 2, self.sample_rate());
    }
}
