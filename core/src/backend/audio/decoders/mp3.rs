use super::{Decoder, SeekableDecoder};
use std::io::{Cursor, Read};

#[cfg(feature = "minimp3")]
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
    #[allow(unknown_lints, clippy::branches_sharing_code)]
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
