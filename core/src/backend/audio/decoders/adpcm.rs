use super::{Decoder, SeekableDecoder};
use bitstream_io::{BigEndian, BitRead, BitReader};
use std::io::{Cursor, Read};

pub struct AdpcmDecoder<R: Read> {
    inner: BitReader<R, BigEndian>,
    sample_rate: u16,
    is_stereo: bool,
    bits_per_sample: usize,
    sample_num: u16,
    left_sample: i16,
    left_step_index: i16,
    right_sample: i16,
    right_step_index: i16,
    decoder: fn(u16, i32) -> u16,
}

impl<R: Read> AdpcmDecoder<R> {
    const INDEX_TABLE: [&'static [i16]; 4] = [
        &[-1, 2],
        &[-1, -1, 2, 4],
        &[-1, -1, -1, -1, 2, 4, 6, 8],
        &[-1, -1, -1, -1, -1, -1, -1, -1, 1, 2, 4, 6, 8, 10, 13, 16],
    ];

    const STEP_TABLE: [u16; 89] = [
        7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60,
        66, 73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371,
        408, 449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878,
        2066, 2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845,
        8630, 9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086,
        29794, 32767,
    ];

    const SAMPLE_DELTA_CALCULATOR: [fn(u16, i32) -> u16; 4] = [
        // 2 bits
        |step: u16, magnitude: i32| {
            let mut delta = step >> 1;
            if magnitude & 1 != 0 {
                delta += step;
            };
            delta
        },
        // 3 bits
        |step: u16, magnitude: i32| {
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
        |step: u16, magnitude: i32| {
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
        |step: u16, magnitude: i32| {
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

    pub fn new(inner: R, is_stereo: bool, sample_rate: u16) -> Self {
        let mut reader = BitReader::new(inner);
        let bits_per_sample = reader.read::<u8>(2).unwrap_or_else(|e| {
            log::warn!("Invalid ADPCM stream: {}", e);
            0
        }) as usize
            + 2;

        Self {
            inner: reader,
            sample_rate,
            is_stereo,
            bits_per_sample,
            sample_num: 0,
            left_sample: 0,
            left_step_index: 0,
            right_sample: 0,
            right_step_index: 0,
            decoder: Self::SAMPLE_DELTA_CALCULATOR[bits_per_sample - 2],
        }
    }

    pub fn next_sample(&mut self) -> Result<(), std::io::Error> {
        if self.sample_num == 0 {
            // The initial sample values are NOT byte-aligned.
            self.left_sample = self.inner.read_signed(16)?;
            self.left_step_index = self.inner.read::<u16>(6)? as i16;
            if self.is_stereo {
                self.right_sample = self.inner.read_signed(16)?;
                self.right_step_index = self.inner.read::<u16>(6)? as i16;
            }
        }

        self.sample_num = (self.sample_num + 1) % 4095;

        let data = self.inner.read::<u32>(self.bits_per_sample as u32)? as i32;
        let left_step = Self::STEP_TABLE[self.left_step_index as usize];

        // (data + 0.5) * step / 2^(bits_per_sample - 2)
        // Data is sign-magnitude, NOT two's complement.
        // TODO(Herschel): Other implementations use some bit-tricks for this.
        let sign_mask = 1 << (self.bits_per_sample - 1);
        let magnitude = data & !sign_mask;
        let delta = (self.decoder)(left_step, magnitude);

        self.left_sample = if (data & sign_mask) != 0 {
            (self.left_sample as i32 - delta as i32).max(i16::MIN.into())
        } else {
            (self.left_sample as i32 + delta as i32).min(i16::MAX.into())
        } as i16;

        self.left_step_index += Self::INDEX_TABLE[self.bits_per_sample - 2][magnitude as usize];
        if self.left_step_index < 0 {
            self.left_step_index = 0;
        } else if self.left_step_index >= Self::STEP_TABLE.len() as i16 {
            self.left_step_index = Self::STEP_TABLE.len() as i16 - 1;
        }

        if self.is_stereo {
            let data = self.inner.read::<u32>(self.bits_per_sample as u32)? as i32;
            let right_step = Self::STEP_TABLE[self.right_step_index as usize];

            let sign_mask = 1 << (self.bits_per_sample - 1);
            let magnitude = data & !sign_mask;
            let delta = (self.decoder)(right_step, magnitude);

            self.right_sample = if (data & sign_mask) != 0 {
                (self.right_sample as i32 - delta as i32).max(i16::MIN.into())
            } else {
                (self.right_sample as i32 + delta as i32).min(i16::MAX.into())
            } as i16;

            self.right_step_index +=
                Self::INDEX_TABLE[self.bits_per_sample - 2][magnitude as usize];
            if self.right_step_index < 0 {
                self.right_step_index = 0;
            } else if self.right_step_index >= Self::STEP_TABLE.len() as i16 {
                self.right_step_index = Self::STEP_TABLE.len() as i16 - 1;
            }
        }

        Ok(())
    }
}

impl<R: Read> Iterator for AdpcmDecoder<R> {
    type Item = [i16; 2];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_sample().ok()?;
        if self.is_stereo {
            Some([self.left_sample, self.right_sample])
        } else {
            Some([self.left_sample, self.left_sample])
        }
    }
}

impl<R: std::io::Read> Decoder for AdpcmDecoder<R> {
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

impl<R: AsRef<[u8]> + Default> SeekableDecoder for AdpcmDecoder<Cursor<R>> {
    #[inline]
    fn reset(&mut self) {
        // TODO: This is funky.
        // I want to reset the `BitStream` and `Cursor` to their initial positions,
        // but have to work around the borrowing rules of Rust.
        let bit_stream = std::mem::replace(&mut self.inner, BitReader::new(Default::default()));
        let mut cursor = bit_stream.into_reader();
        cursor.set_position(0);
        *self = AdpcmDecoder::new(cursor, self.is_stereo, self.sample_rate());
    }
}
