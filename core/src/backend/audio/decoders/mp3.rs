#[cfg(feature = "minimp3")]
pub mod minimp3 {
    use crate::backend::audio::decoders::{Decoder, SeekableDecoder};
    use std::io::{Cursor, Read};

    pub struct Mp3Decoder<R: Read> {
        decoder: minimp3::Decoder<R>,
        sample_rate: u32,
        num_channels: u16,
        cur_frame: minimp3::Frame,
        cur_sample: usize,
        num_samples: usize,
    }

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
}

#[cfg(feature = "symphonia")]
#[allow(dead_code)]
pub mod symphonia {
    use crate::backend::audio::decoders::{Decoder, SeekableDecoder};
    use std::io::{Cursor, Read};
    use symphonia::{
        core::{
            self, audio, codecs, errors,
            formats::{self, FormatReader},
            io,
        },
        default::formats::Mp3Reader as SymphoniaMp3Reader,
    };

    pub struct Mp3Decoder {
        reader: SymphoniaMp3Reader,
        decoder: Box<dyn codecs::Decoder>,
        codec_params: codecs::CodecParameters,
        sample_buf: audio::SampleBuffer<i16>,
        cur_sample: usize,
        sample_rate: u16,
        num_channels: u8,
        stream_ended: bool,
    }

    impl Mp3Decoder {
        pub fn new<R: 'static + Read + Send>(
            num_channels: u16,
            sample_rate: u32,
            reader: R,
        ) -> Self {
            let source = Box::new(io::ReadOnlySource::new(reader)) as Box<dyn io::MediaSource>;
            let source = io::MediaSourceStream::new(source, Default::default());
            let reader = SymphoniaMp3Reader::try_new(source, &Default::default()).unwrap();
            let track = reader.default_track().unwrap();
            let codec_params = track.codec_params.clone();
            let decoder = symphonia::default::get_codecs()
                .make(&codec_params, &Default::default())
                .unwrap();
            let sample_rate = codec_params
                .sample_rate
                .map_or(sample_rate as u16, |n| n as u16);
            let num_channels = codec_params
                .channels
                .map_or(num_channels as u8, |n| n.count() as u8);
            Mp3Decoder {
                reader,
                decoder,
                codec_params,
                sample_buf: audio::SampleBuffer::new(
                    0,
                    audio::SignalSpec::new(0, Default::default()),
                ),
                cur_sample: 0,
                num_channels,
                sample_rate,
                stream_ended: false,
            }
        }

        pub fn new_seekable<R: 'static + AsRef<[u8]> + Send>(
            num_channels: u16,
            sample_rate: u32,
            reader: Cursor<R>,
        ) -> Self {
            let source = Box::new(reader) as Box<dyn io::MediaSource>;
            let source = io::MediaSourceStream::new(source, Default::default());
            let reader = SymphoniaMp3Reader::try_new(source, &Default::default()).unwrap();
            let track = reader.default_track().unwrap();
            let codec_params = track.codec_params.clone();
            let decoder = symphonia::default::get_codecs()
                .make(&codec_params, &Default::default())
                .unwrap();
            let sample_rate = codec_params
                .sample_rate
                .map_or(sample_rate as u16, |n| n as u16);
            let num_channels = codec_params
                .channels
                .map_or(num_channels as u8, |n| n.count() as u8);
            Mp3Decoder {
                reader,
                decoder,
                codec_params,
                sample_buf: audio::SampleBuffer::new(
                    0,
                    audio::SignalSpec::new(0, Default::default()),
                ),
                cur_sample: 0,
                num_channels,
                sample_rate,
                stream_ended: false,
            }
        }

        fn next_frame(&mut self) {
            if self.stream_ended {
                return;
            }

            self.cur_sample = 0;
            while let Ok(packet) = self.reader.next_packet() {
                match self.decoder.decode(&packet) {
                    Ok(decoded) => {
                        if self.sample_buf.len() == 0 {
                            self.sample_buf = audio::SampleBuffer::new(
                                decoded.capacity() as core::units::Duration,
                                *decoded.spec(),
                            );
                        }
                        self.sample_buf.copy_interleaved_ref(decoded);
                        return;
                    }
                    // Decode errors are not fatal.
                    Err(errors::Error::DecodeError(_)) => (),
                    Err(_) => break,
                }
            }
            // EOF reached.
            self.decoder.close();
            self.stream_ended = true;
        }
    }

    impl Iterator for Mp3Decoder {
        type Item = [i16; 2];

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.cur_sample >= self.sample_buf.len() {
                self.next_frame();
                if self.stream_ended {
                    return None;
                }
            }

            let sample_buf = self.sample_buf.samples();
            if self.num_channels == 2 {
                let samples: [i16; 2] =
                    [sample_buf[self.cur_sample], sample_buf[self.cur_sample + 1]];
                self.cur_sample += 2;
                Some(samples)
            } else {
                let sample = sample_buf[self.cur_sample];
                self.cur_sample += 1;
                Some([sample, sample])
            }
        }
    }

    impl SeekableDecoder for Mp3Decoder {
        #[inline]
        fn reset(&mut self) {
            let _ = self.reader.seek(
                formats::SeekMode::Accurate,
                formats::SeekTo::TimeStamp { track_id: 0, ts: 0 },
            );
            self.cur_sample = self.sample_buf.len();
        }
    }

    impl Decoder for Mp3Decoder {
        #[inline]
        fn num_channels(&self) -> u8 {
            self.num_channels
        }

        #[inline]
        fn sample_rate(&self) -> u16 {
            self.sample_rate
        }
    }
}
