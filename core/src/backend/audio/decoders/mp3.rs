use super::{Decoder, SeekableDecoder};
use std::io::{Cursor, Read};

#[cfg(feature = "minimp3")]
#[allow(dead_code)]
pub struct Mp3Decoder<R: Read> {
    decoder: minimp3::Decoder<R>,
    sample_rate: u32,
    num_channels: u16,
    cur_frame: minimp3::Frame,
    cur_sample: usize,
    num_samples: usize,
}

#[cfg(feature = "minimp3")]
impl<R: Read> Mp3Decoder<R> {
    pub fn new(num_channels: u16, sample_rate: u32, reader: R) -> Self {
        Mp3Decoder {
            decoder: minimp3::Decoder::new(reader),
            num_channels,
            sample_rate,
            cur_frame: minimp3::Frame {
                data: vec![],
                sample_rate: sample_rate as i32,
                channels: num_channels.into(),
                layer: 3,
                bitrate: 128,
            },
            cur_sample: 0,
            num_samples: 0,
        }
    }

    fn next_frame(&mut self) {
        if let Ok(frame) = self.decoder.next_frame() {
            self.num_samples = frame.data.len();
            self.cur_frame = frame;
        } else {
            self.num_samples = 0;
        }
        self.cur_sample = 0;
    }
}

#[cfg(feature = "minimp3")]
impl<R: Read> Iterator for Mp3Decoder<R> {
    type Item = [i16; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_sample >= self.num_samples {
            self.next_frame();
        }

        if self.num_samples > 0 {
            if self.num_channels == 2 {
                let left = self.cur_frame.data[self.cur_sample];
                let right = self.cur_frame.data[self.cur_sample + 1];
                self.cur_sample += 2;
                Some([left, right])
            } else {
                let sample = self.cur_frame.data[self.cur_sample];
                self.cur_sample += 1;
                Some([sample, sample])
            }
        } else {
            None
        }
    }
}

#[cfg(feature = "minimp3")]
impl<R: AsRef<[u8]> + Default> SeekableDecoder for Mp3Decoder<Cursor<R>> {
    #[inline]
    fn reset(&mut self) {
        // TODO: This is funky.
        // I want to reset the `BitStream` and `Cursor` to their initial positions,
        // but have to work around the borrowing rules of Rust.
        let mut cursor = std::mem::take(self.decoder.reader_mut());
        cursor.set_position(0);
        *self = Mp3Decoder::new(self.num_channels, self.sample_rate, cursor);
    }
}

#[cfg(all(feature = "puremp3", not(feature = "minimp3")))]
pub struct Mp3Decoder<R: Read> {
    decoder: puremp3::Mp3Decoder<R>,
    sample_rate: u32,
    num_channels: u16,
    cur_frame: puremp3::Frame,
    cur_sample: usize,
    cur_channel: usize,
}

#[cfg(all(feature = "puremp3", not(feature = "minimp3")))]
impl<R: Read> Mp3Decoder<R> {
    pub fn new(num_channels: u16, sample_rate: u32, reader: R) -> Self {
        Mp3Decoder {
            decoder: puremp3::Mp3Decoder::new(reader),
            num_channels,
            sample_rate,
            cur_frame: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            cur_sample: 0,
            cur_channel: 0,
        }
    }

    fn next_frame(&mut self) {
        if let Ok(frame) = self.decoder.next_frame() {
            self.cur_frame = frame;
        } else {
            self.cur_frame.num_samples = 0;
        }
        self.cur_sample = 0;
        self.cur_channel = 0;
    }
}

impl<R: Read> Decoder for Mp3Decoder<R> {
    #[inline]
    fn num_channels(&self) -> u8 {
        self.num_channels as u8
    }

    #[inline]
    fn sample_rate(&self) -> u16 {
        self.sample_rate as u16
    }
}

#[cfg(all(feature = "puremp3", not(feature = "minimp3")))]
impl<R: Read> Iterator for Mp3Decoder<R> {
    type Item = [i16; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_sample >= self.cur_frame.num_samples {
            self.next_frame();
        }

        if self.cur_frame.num_samples > 0 {
            let (left, right) = if self.num_channels == 1 {
                (
                    self.cur_frame.samples[0][self.cur_sample],
                    self.cur_frame.samples[0][self.cur_sample],
                )
            } else {
                (
                    self.cur_frame.samples[0][self.cur_sample],
                    self.cur_frame.samples[1][self.cur_sample],
                )
            };
            self.cur_sample += 1;
            Some([(left * 32767.0) as i16, (right * 32767.0) as i16])
        } else {
            None
        }
    }
}

#[cfg(all(feature = "puremp3", not(feature = "minimp3")))]
impl<R: AsRef<[u8]> + Default> SeekableDecoder for Mp3Decoder<Cursor<R>> {
    #[inline]
    fn reset(&mut self) {
        // TODO: This is funky.
        // I want to reset the `BitStream` and `Cursor` to their initial positions,
        // but have to work around the borrowing rules of Rust.
        let mut cursor = std::mem::replace(self.decoder.get_mut(), Default::default());
        cursor.set_position(0);
        *self = Mp3Decoder::new(self.num_channels, self.sample_rate, cursor);
    }
}
