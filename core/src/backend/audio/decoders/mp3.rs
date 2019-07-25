#[cfg(feature = "minimp3")]
#[allow(dead_code)]
pub struct Mp3Decoder<R: std::io::Read> {
    decoder: minimp3::Decoder<R>,
    sample_rate: u32,
    num_channels: u16,
    cur_frame: minimp3::Frame,
    cur_sample: usize,
    num_samples: usize,
}

#[cfg(feature = "minimp3")]
impl<R: std::io::Read> Mp3Decoder<R> {
    pub fn new(num_channels: u16, sample_rate: u32, reader: R) -> Self {
        Mp3Decoder {
            decoder: minimp3::Decoder::new(reader),
            num_channels,
            sample_rate,
            cur_frame: unsafe { std::mem::zeroed::<minimp3::Frame>() },
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
impl<R: std::io::Read> Iterator for Mp3Decoder<R> {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.cur_sample >= self.num_samples {
            self.next_frame();
        }

        if self.num_samples > 0 {
            let sample = self.cur_frame.data[self.cur_sample];
            self.cur_sample += 1;
            Some(sample)
        } else {
            None
        }
    }
}

#[cfg(all(feature = "puremp3", not(feature = "minimp3")))]
pub struct Mp3Decoder<R: std::io::Read> {
    decoder: puremp3::Mp3Decoder<R>,
    sample_rate: u32,
    num_channels: u16,
    cur_frame: puremp3::Frame,
    cur_sample: usize,
    cur_channel: usize,
}

#[cfg(all(feature = "puremp3", not(feature = "minimp3")))]
impl<R: std::io::Read> Mp3Decoder<R> {
    pub fn new(num_channels: u16, sample_rate: u32, reader: R) -> Self {
        Mp3Decoder {
            decoder: puremp3::Mp3Decoder::new(reader),
            num_channels,
            sample_rate,
            cur_frame: unsafe { std::mem::zeroed::<puremp3::Frame>() },
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

impl<R: std::io::Read> super::Decoder for Mp3Decoder<R> {
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
impl<R: std::io::Read> Iterator for Mp3Decoder<R> {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.cur_sample >= self.cur_frame.num_samples {
            self.next_frame();
        }

        if self.cur_frame.num_samples > 0 {
            let sample = self.cur_frame.samples[self.cur_channel][self.cur_sample];
            self.cur_channel += 1;
            if self.cur_channel >= usize::from(self.num_channels) {
                self.cur_channel = 0;
                self.cur_sample += 1;
            }
            Some((sample * 32767.0) as i16)
        } else {
            None
        }
    }
}
