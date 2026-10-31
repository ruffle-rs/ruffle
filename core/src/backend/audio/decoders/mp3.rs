use crate::backend::audio::decoders::{Decoder, Mp3Metadata, SeekableDecoder};
use std::io::{Cursor, Read};
use symphonia::{
    core::{
        self, audio, codecs, errors,
        formats::{self, FormatReader},
        io,
    },
    default::formats::MpaReader as SymphoniaMpaReader,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Couldn't decode MP3 frame: {0}")]
    FrameDecode(#[from] errors::Error),

    #[error("No default track")]
    NoDefaultTrack,

    #[error("Invalid sample rate")]
    InvalidSampleRate,

    #[error("Invalid channels")]
    InvalidChannels,
}

pub struct Mp3Decoder {
    reader: SymphoniaMpaReader,
    decoder: Box<dyn codecs::Decoder>,
    sample_buf: audio::SampleBuffer<i16>,
    cur_sample: usize,
    sample_rate: u16,
    num_channels: u8,
    stream_ended: bool,
}

impl Mp3Decoder {
    // MP3 frames contain 1152 samples.
    const SAMPLE_BUFFER_DURATION: u64 = 1152;

    pub fn new<R: 'static + Read + Send + Sync>(reader: R) -> Result<Self, Error> {
        let source = Box::new(io::ReadOnlySource::new(reader)) as Box<dyn io::MediaSource>;
        let source = io::MediaSourceStream::new(source, Default::default());
        let reader = SymphoniaMpaReader::try_new(source, &Default::default())?;
        let track = reader.default_track().ok_or(Error::NoDefaultTrack)?;
        let codec_params = track.codec_params.clone();
        let decoder = symphonia::default::get_codecs().make(&codec_params, &Default::default())?;
        let sample_rate = codec_params.sample_rate.ok_or(Error::InvalidSampleRate)?;
        let channels = codec_params.channels.ok_or(Error::InvalidChannels)?;
        Ok(Mp3Decoder {
            reader,
            decoder,
            sample_buf: audio::SampleBuffer::new(0, audio::SignalSpec::new(sample_rate, channels)),
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
        let reader = SymphoniaMpaReader::try_new(source, &Default::default())?;
        let track = reader.default_track().ok_or(Error::NoDefaultTrack)?;
        let codec_params = track.codec_params.clone();
        let decoder = symphonia::default::get_codecs().make(&codec_params, &Default::default())?;
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
                    if self.sample_buf.capacity() < decoded.capacity() {
                        // Ensure our buffer has enough space for the decoded samples.
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
            let samples: [i16; 2] = [sample_buf[self.cur_sample], sample_buf[self.cur_sample + 1]];
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
        self.seek_to_sample_frame(0);
    }

    #[inline]
    fn seek_to_sample_frame(&mut self, frame: u32) {
        // Seek to the desired position,
        let seek_result = self.reader.seek(
            formats::SeekMode::Accurate,
            formats::SeekTo::TimeStamp {
                track_id: 0,
                ts: frame.into(),
            },
        );
        self.sample_buf.clear();
        self.decoder.reset();
        self.cur_sample = 0;
        self.stream_ended = false;
        // Seeking isn't exact, so we may end up slightly before our desired position.
        // Pump samples until we get to the exact position.
        let samples_remaining =
            seek_result.map_or(0, |seek| seek.required_ts.saturating_sub(seek.actual_ts));
        for _ in 0..samples_remaining {
            self.next();
        }
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

/// Returns the sample rate and length of the given MP3.
pub fn mp3_metadata(data: &std::sync::Arc<[u8]>) -> Result<Mp3Metadata, Error> {
    let source =
        io::MediaSourceStream::new(Box::new(Cursor::new(data.clone())), Default::default());
    let mut reader = SymphoniaMpaReader::try_new(source, &Default::default())?;
    let track = reader.default_track().ok_or(Error::NoDefaultTrack)?;
    let n_frames = track.codec_params.n_frames;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or(Error::InvalidSampleRate)? as u16;
    // An info frame without a frame count leaves the length unknown: count
    // the frames, as Flash Player does.
    let num_sample_frames = match n_frames {
        Some(n_frames) => n_frames as u32,
        None => {
            let mut num_sample_frames = 0u64;
            while let Ok(packet) = reader.next_packet() {
                num_sample_frames += packet.dur;
            }
            num_sample_frames as u32
        }
    };
    // Where the reader starts reading audio: after the first frame if it
    // took that for an info frame.
    let source =
        io::MediaSourceStream::new(Box::new(Cursor::new(data.clone())), Default::default());
    let reader = SymphoniaMpaReader::try_new(source, &Default::default())?;
    let first_packet_pos = io::ReadBytes::pos(&Box::new(reader).into_inner());
    Ok(Mp3Metadata {
        num_sample_frames,
        sample_rate,
        info_frame_sample_frames: info_frame_sample_frames(data, first_packet_pos),
    })
}

/// Returns the number of sample frames in the first MP3 frame if it is an
/// encoder's info frame (Xing, Info or VBRI), or 0.
///
/// The reader skips an info frame when it recognizes one, which it tells by
/// `first_packet_pos`, where it starts reading audio. A frame it doesn't
/// recognize is decoded like any other.
fn info_frame_sample_frames(data: &[u8], first_packet_pos: u64) -> u32 {
    // Skip an ID3v2 tag: a 10-byte header, with a syncsafe size and a flag
    // for a 10-byte footer.
    let mut offset = 0;
    if let [b'I', b'D', b'3', _, _, flags, size @ ..] = data
        && let [a, b, c, d, ..] = *size
    {
        let size = (usize::from(a) << 21)
            | (usize::from(b) << 14)
            | (usize::from(c) << 7)
            | usize::from(d);
        offset = 10 + size + if flags & 0x10 != 0 { 10 } else { 0 };
    }

    let Some(&[0xFF, b1, ..]) = data.get(offset..offset + 4) else {
        return 0;
    };
    let is_mpeg1 = b1 & 0b0001_1000 == 0b0001_1000;
    let is_layer3 = b1 & 0b0000_0110 == 0b0000_0010;
    // The largest Layer III frame, at 320 kbps and 32 kHz with padding (or
    // 160 kbps and 8 kHz): skipping more than that skipped something else.
    const MAX_FRAME_LEN: u64 = 1441;
    let skipped = first_packet_pos.saturating_sub(offset as u64);
    if b1 & 0xE0 != 0xE0 || !is_layer3 || skipped == 0 || skipped > MAX_FRAME_LEN {
        return 0;
    }
    if is_mpeg1 { 1152 } else { 576 }
}
