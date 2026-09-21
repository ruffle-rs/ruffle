use crate::backend::audio::decoders::{Decoder, Mp3Metadata, SeekableDecoder};
use std::io::{Cursor, Read};
use symphonia::{
    core::{
        codecs::{
            CodecParameters,
            audio::{AudioCodecParameters, AudioDecoder, AudioDecoderOptions},
        },
        errors,
        formats::{self, FormatReader, Track, TrackType},
        io,
        units::Timestamp,
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
    reader: SymphoniaMpaReader<'static>,
    decoder: Box<dyn AudioDecoder>,
    /// Interleaved samples of the last decoded MP3 frame.
    sample_buf: Vec<i16>,
    cur_sample: usize,
    sample_rate: u16,
    num_channels: u8,
    stream_ended: bool,
}

impl Mp3Decoder {
    // MP3 frames contain 1152 samples.
    const SAMPLE_BUFFER_DURATION: usize = 1152;

    pub fn new<R: 'static + Read + Send + Sync>(reader: R) -> Result<Self, Error> {
        let source = Box::new(io::ReadOnlySource::new(reader)) as Box<dyn io::MediaSource>;
        let source = io::MediaSourceStream::new(source, Default::default());
        let reader = SymphoniaMpaReader::try_new(source, Default::default())?;
        let (_, codec_params) = default_track(&reader)?;
        let codec_params = codec_params.clone();
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(&codec_params, &decoder_options())?;
        let sample_rate = codec_params.sample_rate.ok_or(Error::InvalidSampleRate)?;
        let channels = codec_params.channels.ok_or(Error::InvalidChannels)?;
        Ok(Mp3Decoder {
            reader,
            decoder,
            sample_buf: Vec::new(),
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
        let reader = SymphoniaMpaReader::try_new(source, Default::default())?;
        let (_, codec_params) = default_track(&reader)?;
        let codec_params = codec_params.clone();
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(&codec_params, &decoder_options())?;
        let sample_rate = codec_params.sample_rate.ok_or(Error::InvalidSampleRate)?;
        let channels = codec_params.channels.ok_or(Error::InvalidChannels)?;
        Ok(Mp3Decoder {
            reader,
            decoder,
            sample_buf: Vec::with_capacity(Self::SAMPLE_BUFFER_DURATION * channels.count()),
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
        while let Ok(Some(packet)) = self.reader.next_packet() {
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    decoded.copy_to_vec_interleaved(&mut self.sample_buf);
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

        let sample_buf = &self.sample_buf;
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
        // Symphonia timestamps start at `-delay`, so that the encoder delay is skipped in gapless
        // playback. We don't do gapless playback, so `frame` counts from the very first decoded
        // sample, and has to be shifted into Symphonia's timeline.
        let delay = default_track(&self.reader)
            .ok()
            .and_then(|(track, _)| track.delay)
            .unwrap_or(0);
        // Seek to the desired position,
        let seek_result = self.reader.seek(
            formats::SeekMode::Accurate,
            formats::SeekTo::Timestamp {
                track_id: 0,
                ts: Timestamp::from(i64::from(frame) - i64::from(delay)),
            },
        );
        self.sample_buf.clear();
        self.decoder.reset();
        self.cur_sample = 0;
        // Seeking past the end fails without moving the reader, so end the stream explicitly
        // instead of continuing to play from the previous position.
        self.stream_ended = matches!(
            seek_result,
            Err(errors::Error::SeekError(errors::SeekErrorKind::OutOfRange))
        );
        // Seeking isn't exact, so we may end up slightly before our desired position.
        // Pump samples until we get to the exact position.
        let samples_remaining = seek_result.map_or(0, |seek| {
            seek.required_ts
                .duration_from(seek.actual_ts)
                .map_or(0, |duration| duration.get())
        });
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

/// Returns the audio track of the given MP3, along with its codec parameters.
fn default_track<'a>(
    reader: &'a SymphoniaMpaReader<'_>,
) -> Result<(&'a Track, &'a AudioCodecParameters), Error> {
    let track = reader
        .default_track(TrackType::Audio)
        .ok_or(Error::NoDefaultTrack)?;
    let codec_params = track
        .codec_params
        .as_ref()
        .and_then(CodecParameters::audio)
        .ok_or(Error::NoDefaultTrack)?;
    Ok((track, codec_params))
}

/// Options for the Symphonia decoder.
fn decoder_options() -> AudioDecoderOptions {
    // Gapless playback would trim the encoder delay and padding from the decoded audio.
    // Flash doesn't do that, and it would shift the sample positions we seek to.
    AudioDecoderOptions::default().gapless(false)
}

/// Returns the sample rate and length of the given MP3.
pub fn mp3_metadata(data: &std::sync::Arc<[u8]>) -> Result<Mp3Metadata, Error> {
    let source =
        io::MediaSourceStream::new(Box::new(Cursor::new(data.clone())), Default::default());
    let reader = SymphoniaMpaReader::try_new(source, Default::default())?;
    let (track, codec_params) = default_track(&reader)?;
    let sample_rate = codec_params.sample_rate.ok_or(Error::InvalidSampleRate)? as u16;
    // Symphonia excludes the encoder delay and padding from the frame count, but we decode them.
    let num_sample_frames = track.num_frames.map_or(0, |num_frames| {
        num_frames + u64::from(track.delay.unwrap_or(0)) + u64::from(track.padding.unwrap_or(0))
    });
    Ok(Mp3Metadata {
        num_sample_frames: num_sample_frames as u32,
        sample_rate,
    })
}
