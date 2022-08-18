#[cfg(feature = "minimp3")]
pub mod minimp3 {
    use crate::backend::audio::decoders::{Decoder, SeekableDecoder};
    use std::io::{Cursor, Read};
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("Couldn't decode MP3 frame")]
        FrameDecode(#[from] minimp3::Error),

        #[error("Invalid sample rate")]
        InvalidSampleRate,

        #[error("Invalid channels")]
        InvalidChannels,
    }

    pub struct Mp3Decoder<R: Read> {
        decoder: minimp3::Decoder<R>,
        frame: minimp3::Frame,
        sample_rate: u16,
        num_channels: u8,
        cur_sample: usize,
    }

    impl<R: Read> Mp3Decoder<R> {
        pub fn new(reader: R) -> Result<Self, Error> {
            let mut decoder = minimp3::Decoder::new(reader);
            let frame = decoder.next_frame()?;
            let sample_rate = frame
                .sample_rate
                .try_into()
                .map_err(|_| Error::InvalidSampleRate)?;
            let num_channels = frame
                .channels
                .try_into()
                .map_err(|_| Error::InvalidChannels)?;
            Ok(Mp3Decoder {
                decoder,
                frame,
                sample_rate,
                num_channels,
                cur_sample: 0,
            })
        }

        fn next_frame(&mut self) {
            if let Ok(frame) = self.decoder.next_frame() {
                self.frame = frame;
            } else {
                self.frame.data.clear();
            }
            self.cur_sample = 0;
        }
    }

    impl<R: Read + Send + Sync> Iterator for Mp3Decoder<R> {
        type Item = [i16; 2];

        #[inline]
        #[allow(clippy::branches_sharing_code)]
        fn next(&mut self) -> Option<Self::Item> {
            if self.cur_sample >= self.frame.data.len() {
                self.next_frame();
            }

            if !self.frame.data.is_empty() {
                if self.num_channels() == 2 {
                    let left = self.frame.data[self.cur_sample];
                    let right = self.frame.data[self.cur_sample + 1];
                    self.cur_sample += 2;
                    Some([left, right])
                } else {
                    let sample = self.frame.data[self.cur_sample];
                    self.cur_sample += 1;
                    Some([sample, sample])
                }
            } else {
                None
            }
        }
    }

    impl<R: AsRef<[u8]> + Default + Send + Sync> SeekableDecoder for Mp3Decoder<Cursor<R>> {
        #[inline]
        fn reset(&mut self) {
            // TODO: This is funky.
            // I want to reset the `BitStream` and `Cursor` to their initial positions,
            // but have to work around the borrowing rules of Rust.
            let mut cursor = std::mem::take(self.decoder.reader_mut());
            cursor.set_position(0);
            if let Ok(decoder) = Mp3Decoder::new(cursor) {
                *self = decoder;
            }
        }
    }

    impl<R: Read + Send + Sync> Decoder for Mp3Decoder<R> {
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
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("Couldn't decode MP3 frame")]
        FrameDecode(#[from] errors::Error),

        #[error("No default track")]
        NoDefaultTrack,

        #[error("Invalid sample rate")]
        InvalidSampleRate,

        #[error("Invalid channels")]
        InvalidChannels,
    }

    pub struct Mp3Decoder {
        reader: SymphoniaMp3Reader,
        decoder: Box<dyn codecs::Decoder>,
        sample_buf: audio::SampleBuffer<i16>,
        cur_sample: usize,
        sample_rate: u16,
        num_channels: u8,
        stream_ended: bool,
    }

    impl Mp3Decoder {
        const SAMPLE_BUFFER_DURATION: u64 = 4096;

        pub fn new<R: 'static + Read + Send + Sync>(reader: R) -> Result<Self, Error> {
            let source = Box::new(io::ReadOnlySource::new(reader)) as Box<dyn io::MediaSource>;
            let source = io::MediaSourceStream::new(source, Default::default());
            let reader = SymphoniaMp3Reader::try_new(source, &Default::default())?;
            let track = reader.default_track().ok_or(Error::NoDefaultTrack)?;
            let codec_params = track.codec_params.clone();
            let decoder =
                symphonia::default::get_codecs().make(&codec_params, &Default::default())?;
            let sample_rate = codec_params.sample_rate.ok_or(Error::InvalidSampleRate)?;
            let channels = codec_params.channels.ok_or(Error::InvalidChannels)?;
            Ok(Mp3Decoder {
                reader,
                decoder,
                sample_buf: audio::SampleBuffer::new(
                    Self::SAMPLE_BUFFER_DURATION,
                    audio::SignalSpec::new(sample_rate, channels),
                ),
                cur_sample: 0,
                num_channels: channels
                    .count()
                    .try_into()
                    .map_err(|_| Error::InvalidChannels)?,
                sample_rate: sample_rate.try_into().map_err(|_| Error::InvalidChannels)?,
                stream_ended: false,
            })
        }

        pub fn new_seekable<R: 'static + AsRef<[u8]> + Send + Sync>(
            reader: Cursor<R>,
        ) -> Result<Self, Error> {
            let source = Box::new(reader) as Box<dyn io::MediaSource>;
            let source = io::MediaSourceStream::new(source, Default::default());
            let reader = SymphoniaMp3Reader::try_new(source, &Default::default()).unwrap();
            let track = reader.default_track().ok_or(Error::NoDefaultTrack)?;
            let codec_params = track.codec_params.clone();
            let decoder =
                symphonia::default::get_codecs().make(&codec_params, &Default::default())?;
            let sample_rate = codec_params.sample_rate.ok_or(Error::InvalidSampleRate)?;
            let channels = codec_params.channels.ok_or(Error::InvalidChannels)?;
            Ok(Mp3Decoder {
                reader,
                decoder,
                sample_buf: audio::SampleBuffer::new(
                    Self::SAMPLE_BUFFER_DURATION,
                    audio::SignalSpec::new(sample_rate, channels),
                ),
                cur_sample: 0,
                num_channels: channels.count() as u8,
                sample_rate: sample_rate as u16,
                stream_ended: false,
            })
        }

        fn next_frame(&mut self) {
            if self.stream_ended {
                return;
            }

            self.cur_sample = 0;
            while let Ok(packet) = self.reader.next_packet() {
                match self.decoder.decode(&packet) {
                    Ok(decoded) => {
                        if self.sample_buf.is_empty() {
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
            self.sample_buf.clear();
            self.decoder.reset();
            self.cur_sample = 0;
            self.stream_ended = false;
            self.next_frame();
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
