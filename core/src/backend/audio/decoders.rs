//! Audio decoders.

mod adpcm;
#[cfg(feature = "mp3")]
mod mp3;
#[cfg(feature = "nellymoser")]
mod nellymoser;
mod pcm;

pub use adpcm::AdpcmDecoder;
#[cfg(feature = "mp3")]
pub use mp3::{mp3_metadata, Mp3Decoder};
#[cfg(feature = "nellymoser")]
pub use nellymoser::NellymoserDecoder;
pub use pcm::PcmDecoder;

use crate::backend::audio::{SoundStreamInfo, SoundStreamWrapping};
use crate::buffer::{Slice, SliceCursor, Substream, SubstreamChunksIter};
use crate::tag_utils::{ControlFlow, SwfSlice};
use std::io::{Cursor, Read};
use swf::{AudioCompression, SoundFormat, TagCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "mp3")]
    #[error("Couldn't decode MP3: {0}")]
    InvalidMp3(#[from] mp3::Error),

    #[cfg(feature = "aac")]
    #[error("Couldn't decode AAC: {0}")]
    InvalidAac(#[from] symphonia::core::errors::Error),

    #[error("Couldn't decode ADPCM: {0}")]
    InvalidAdpcm(#[from] adpcm::Error),

    #[error("Unhandled compression {0:?}")]
    UnhandledCompression(AudioCompression),

    #[error("Too many sounds are playing")]
    TooManySounds,
}

/// An audio decoder. Can be used as an `Iterator` to return stero sample frames.
/// If the sound is mono, the sample is duplicated across both channels.
pub trait Decoder: Iterator<Item = [i16; 2]> + Send + Sync {
    /// The number of channels of this audio decoder. Always 1 or 2.
    fn num_channels(&self) -> u8;

    /// The sample rate of this audio decoder.
    fn sample_rate(&self) -> u16;
}

/// Instantiate a decoder for the compression that the sound data uses.
pub fn make_decoder<R: 'static + Read + Send + Sync>(
    format: &SoundFormat,
    data: R,
) -> Result<Box<dyn Decoder>, Error> {
    let decoder: Box<dyn Decoder> = match format.compression {
        AudioCompression::UncompressedUnknownEndian => {
            // Cross fingers that it's little endian.
            tracing::warn!("make_decoder: PCM sound is unknown endian; assuming little endian");
            Box::new(PcmDecoder::new(
                data,
                format.is_stereo,
                format.sample_rate,
                format.is_16_bit,
            ))
        }
        AudioCompression::Uncompressed => Box::new(PcmDecoder::new(
            data,
            format.is_stereo,
            format.sample_rate,
            format.is_16_bit,
        )),
        AudioCompression::Adpcm => Box::new(AdpcmDecoder::new(
            data,
            format.is_stereo,
            format.sample_rate,
        )?),
        #[cfg(feature = "mp3")]
        AudioCompression::Mp3 => Box::new(Mp3Decoder::new(data)?),
        #[cfg(feature = "nellymoser")]
        AudioCompression::Nellymoser => {
            Box::new(NellymoserDecoder::new(data, format.sample_rate.into()))
        }
        _ => return Err(Error::UnhandledCompression(format.compression)),
    };
    Ok(decoder)
}

impl<T: Decoder + ?Sized> Decoder for Box<T> {
    #[inline]
    fn num_channels(&self) -> u8 {
        self.as_ref().num_channels()
    }

    /// The sample rate of this audio decoder.
    fn sample_rate(&self) -> u16 {
        self.as_ref().sample_rate()
    }
}

/// Audio stream decoder.
///
/// A "stream" sound is a sound that has its data distributed across `SoundStreamBlock` tags,
/// one per each frame of a MovieClip. The sound is synced to the MovieClip's timeline, and will
/// stop/seek as the MovieClip stops/seeks.
///
/// In the Flash IDE, the is created by changing the "Sync" setting of the sound
/// to "Stream."
///
/// TODO: Add `current_frame`.
pub trait StreamDecoder: Decoder {}

/// The `StandardStreamDecoder` takes care of reading the audio data from `SoundStreamBlock` tags
/// and feeds it to the decoder.
struct StandardStreamDecoder {
    /// The underlying decoder. The decoder will get its data from a `StreamTagReader`.
    decoder: Box<dyn Decoder>,
}

impl StandardStreamDecoder {
    /// Constructs a new `StandardStreamDecoder.
    /// `swf_data` should be the tag data of the MovieClip that contains the stream.
    fn new(stream_info: &swf::SoundStreamHead, swf_data: SwfSlice) -> Result<Self, Error> {
        // Create a tag reader to get the audio data from SoundStreamBlock tags.
        let tag_reader = StreamTagReader::new(stream_info, swf_data);
        // Wrap the tag reader in the decoder.
        let decoder = make_decoder(&stream_info.stream_format, tag_reader)?;
        Ok(Self { decoder })
    }
}

impl Decoder for StandardStreamDecoder {
    fn num_channels(&self) -> u8 {
        self.decoder.num_channels()
    }
    fn sample_rate(&self) -> u16 {
        self.decoder.sample_rate()
    }
}

impl Iterator for StandardStreamDecoder {
    type Item = [i16; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
    }
}

/// ADPCM stream decoder.
///
/// Stream sounds encoded with ADPCM have an ADPCM header in each `SoundStreamBlock` tag, unlike
/// other compression formats that remain the same as if they were a single sound clip.
/// Therefore, we must recreate the decoder with each `SoundStreamBlock` to parse the additional
/// headers.
pub struct AdpcmStreamDecoder {
    format: SoundFormat,
    tag_reader: StreamTagReader,
    decoder: AdpcmDecoder<Cursor<SwfSlice>>,
}

impl AdpcmStreamDecoder {
    fn new(stream_info: &swf::SoundStreamHead, swf_data: SwfSlice) -> Result<Self, Error> {
        let movie = swf_data.movie.clone();
        let mut tag_reader = StreamTagReader::new(stream_info, swf_data);
        let audio_data = tag_reader.next().unwrap_or_else(|| SwfSlice::empty(movie));
        let decoder = AdpcmDecoder::new(
            Cursor::new(audio_data),
            stream_info.stream_format.is_stereo,
            stream_info.stream_format.sample_rate,
        )?;
        Ok(Self {
            format: stream_info.stream_format.clone(),
            tag_reader,
            decoder,
        })
    }
}

impl Decoder for AdpcmStreamDecoder {
    fn num_channels(&self) -> u8 {
        self.decoder.num_channels()
    }
    fn sample_rate(&self) -> u16 {
        self.decoder.sample_rate()
    }
}

impl Iterator for AdpcmStreamDecoder {
    type Item = [i16; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample_frame) = self.decoder.next() {
            // Return sample frames until the decoder has exhausted
            // the SoundStreamBlock tag.
            Some(sample_frame)
        } else if let Some(audio_data) = self.tag_reader.next() {
            // We've reached the end of the sound stream block tag, so
            // read the next one and recreate the decoder.
            // `AdpcmDecoder` read the ADPCM header when it is created.
            self.decoder = AdpcmDecoder::new(
                Cursor::new(audio_data),
                self.format.is_stereo,
                self.format.sample_rate,
            )
            .ok()?;
            self.decoder.next()
        } else {
            // No more SoundStreamBlock tags.
            None
        }
    }
}

/// Makes a `StreamDecoder` for the given stream. `swf_data` should be the MovieClip's tag data.
/// Generally this will return a `StandardStreamDecoder`, except for ADPCM streams.
pub fn make_stream_decoder(
    stream_info: &swf::SoundStreamHead,
    swf_data: SwfSlice,
) -> Result<Box<dyn Decoder + Send>, Error> {
    let decoder: Box<dyn Decoder + Send> =
        if stream_info.stream_format.compression == AudioCompression::Adpcm {
            Box::new(AdpcmStreamDecoder::new(stream_info, swf_data)?)
        } else {
            Box::new(StandardStreamDecoder::new(stream_info, swf_data)?)
        };
    Ok(decoder)
}

/// Adds seeking ability to decoders where the underline stream is `std::io::Seek`.
pub trait SeekableDecoder: Decoder {
    /// Resets the decoder to the beginning of the stream.
    fn reset(&mut self);

    /// Seeks to a specific sample frame.
    fn seek_to_sample_frame(&mut self, frame: u32) {
        // The default implementation simply resets the stream and steps through
        // until the desired position.
        // This will be slow for long sounds on heavy decoders.
        self.reset();
        for _ in 0..frame {
            self.next();
        }
    }
}

/// `StreamTagReader` reads through the SWF tag data of a `MovieClip`, extracting
/// audio data from the `SoundStreamBlock` tags. It can be used as an `Iterator` that
/// will return consecutive slices of the underlying audio data.
struct StreamTagReader {
    /// The tag data of the `MovieClip` that contains the streaming audio track.
    swf_data: SwfSlice,

    /// The audio playback position inside `swf_data`.
    pos: usize,

    /// The compressed audio data in the most recent `SoundStreamBlock` we've seen, returned by `Iterator::next`.
    current_audio_data: SwfSlice,

    /// The compression used by the audio data.
    compression: AudioCompression,

    /// The number of audio samples for use in future animation frames.
    ///
    /// Only used in MP3 encoding to properly handle gaps in the audio track.
    mp3_samples_buffered: i32,

    /// The ideal number of audio samples in each animation frame, i.e. the sample rate divided by frame rate.
    ///
    /// Only used in MP3 encoding to properly handle gaps in the audio track.
    mp3_samples_per_block: u16,
}

impl StreamTagReader {
    /// Builds a new `StreamTagReader` from the given SWF data.
    /// `swf_data` should be the tag data of a MovieClip.
    fn new(stream_info: &swf::SoundStreamHead, swf_data: SwfSlice) -> Self {
        let current_audio_data = SwfSlice::empty(swf_data.movie.clone());
        Self {
            swf_data,
            pos: 0,
            compression: stream_info.stream_format.compression,
            current_audio_data,
            mp3_samples_buffered: 0,
            mp3_samples_per_block: stream_info.num_samples_per_block,
        }
    }
}

impl Iterator for StreamTagReader {
    type Item = SwfSlice;

    fn next(&mut self) -> Option<Self::Item> {
        let audio_data = &mut self.current_audio_data;
        let compression = self.compression;
        let mut found = false;

        let swf_data = &self.swf_data;
        loop {
            let tag_callback =
                |reader: &mut swf::read::Reader<'_>, tag_code, tag_len| match tag_code {
                    TagCode::SoundStreamBlock if !found => {
                        found = true;
                        let mut audio_block = &reader.get_ref()[..tag_len];
                        // MP3 audio blocks start with a header indicating sample count + seek offset (SWF19 p.184).
                        if compression == AudioCompression::Mp3 && audio_block.len() >= 4 {
                            // MP3s deliver audio in frames of 576 samples, which means we may have SoundStreamBlocks with
                            // lots of extra samples, followed by a block with 0 samples. Worse, there may be frames without
                            // blocks at all despite SWF19 saying this shouldn't happen. This may or may not indicate a gap
                            // in the audio depending on the number of empty frames.
                            // Keep a tally of the # of samples we've seen compared to the number of samples that will be
                            // played in each timeline frame. Only stop an MP3 sound if we've exhausted all of the samples.
                            // RESEARCHME: How does Flash Player actually determine when there is an audio gap or not?
                            // If an MP3 audio track has gaps, Flash Player will often play it out of sync (too early).
                            // Seems closely related to `stream_info.num_samples_per_block`.
                            let num_samples = u16::from_le_bytes(
                                audio_block[..2].try_into().expect("2 bytes fit into a u16"),
                            );
                            self.mp3_samples_buffered += i32::from(num_samples);
                            audio_block = &audio_block[4..];
                        }
                        *audio_data = swf_data.to_subslice(audio_block);
                        Ok(ControlFlow::Continue)
                    }
                    TagCode::ShowFrame if compression == AudioCompression::Mp3 => {
                        self.mp3_samples_buffered -= i32::from(self.mp3_samples_per_block);
                        Ok(ControlFlow::Exit)
                    }
                    TagCode::ShowFrame => Ok(ControlFlow::Exit),
                    _ => Ok(ControlFlow::Continue),
                };

            let mut reader = self.swf_data.read_from(self.pos as u64);
            let _ = crate::tag_utils::decode_tags(&mut reader, tag_callback);
            self.pos = reader.get_ref().as_ptr() as usize - swf_data.as_ref().as_ptr() as usize;

            // If we hit a SoundStreamBlock within this frame, return it. Otherwise, the stream should end.
            // The exception is MP3 streaming sounds, which will continue to play even when a few frames
            // are missing SoundStreamBlock tags (see above).
            if found {
                break Some(self.current_audio_data.clone());
            } else if compression != AudioCompression::Mp3
                // FIXME: The next condition should logically end with `<= 0`.
                // It was changed as a quick HACK to fix #7524 by not detecting an
                // underrun too soon. We are still not yet sure what exactly to do here.
                || self.mp3_samples_buffered <= -(self.mp3_samples_per_block as i32)
                || reader.get_ref().is_empty()
            {
                break None;
            }
        }
    }
}

/// Returns an `Reader` that reads through SWF tags and returns slices of any
/// audio stream data for `SoundStreamBlock` tags.
impl Read for StreamTagReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while self.current_audio_data.as_ref().is_empty() {
            self.current_audio_data = if let Some(audio_data) = self.next() {
                audio_data
            } else {
                return Ok(0);
            }
        }

        let len = std::cmp::min(buf.len(), self.current_audio_data.as_ref().len());
        buf[..len].copy_from_slice(&self.current_audio_data.as_ref()[..len]);
        self.current_audio_data.start += len;
        Ok(len)
    }
}

#[derive(Debug)]
pub struct Mp3Metadata {
    pub sample_rate: u16,
    pub num_sample_frames: u32,
}

/// `SubstreamTagReader` reads through the data chunks of a `Substream` with
/// audio data in it. Data is assumed to have already been extracted from its
/// container.
///
/// MP3 data will be further joined into properly-sized chunks.
struct SubstreamTagReader {
    /// The tag data of the `MovieClip` that contains the streaming audio track.
    data_stream: SubstreamChunksIter,

    /// The compressed audio data in the most recent `SoundStreamBlock` we've seen, returned by `Iterator::next`.
    current_audio_data: Option<Slice>,

    /// The compression used by the audio data.
    compression: AudioCompression,

    /// The wrapping used by audio chunks in the stream.
    wrapping: SoundStreamWrapping,

    /// The number of audio samples for use in future animation frames.
    ///
    /// Only used in MP3 encoding to properly handle gaps in the audio track.
    mp3_samples_buffered: i32,

    /// The ideal number of audio samples in each animation frame, i.e. the sample rate divided by frame rate.
    ///
    /// Only used in MP3 encoding to properly handle gaps in the audio track.
    _mp3_samples_per_block: u16,
}

impl SubstreamTagReader {
    /// Builds a new `SubstreamTagReader` from the given `Substream`.
    ///
    /// `Substream` should reference audio data.
    fn new(stream_info: &SoundStreamInfo, data_stream: Substream) -> Self {
        Self {
            data_stream: data_stream.iter_chunks(),
            compression: stream_info.stream_format.compression,
            wrapping: stream_info.wrapping,
            current_audio_data: None,
            mp3_samples_buffered: 0,
            _mp3_samples_per_block: stream_info.num_samples_per_block,
        }
    }
}

impl Iterator for SubstreamTagReader {
    type Item = Slice;

    fn next(&mut self) -> Option<Self::Item> {
        let compression = self.compression;
        let next_chunk = self.data_stream.next()?;
        let data = next_chunk.data();

        let mut audio_block = &data[..];
        if compression == AudioCompression::Mp3
            && self.wrapping == SoundStreamWrapping::Swf
            && next_chunk.len() >= 4
        {
            // MP3s deliver audio in frames of 576 samples, which means we may have SoundStreamBlocks with
            // lots of extra samples, followed by a block with 0 samples. Worse, there may be frames without
            // blocks at all despite SWF19 saying this shouldn't happen. This may or may not indicate a gap
            // in the audio depending on the number of empty frames.
            // Keep a tally of the # of samples we've seen compared to the number of samples that will be
            // played in each timeline frame. Only stop an MP3 sound if we've exhausted all of the samples.
            // RESEARCHME: How does Flash Player actually determine when there is an audio gap or not?
            // If an MP3 audio track has gaps, Flash Player will often play it out of sync (too early).
            // Seems closely related to `stream_info.num_samples_per_block`.
            let num_samples =
                u16::from_le_bytes(audio_block[..2].try_into().expect("2 bytes fit into a u16"));
            self.mp3_samples_buffered += i32::from(num_samples);
            audio_block = &audio_block[4..];
        }
        self.current_audio_data = Some(next_chunk.to_subslice(audio_block));

        // TODO: per-frame sample tracking. `Substream` does not have a notion
        // of a 'frame'. However, SWF audio streams end if there's no audio
        // data in a frame, EXCEPT for MP3 which is allowed to straddle frames,
        // have empty frames, etc. Non-MP3 stops at the first chunk.
        self.current_audio_data.clone()
    }
}

/// Returns an `Reader` that reads through SWF tags and returns slices of any
/// audio stream data for `SoundStreamBlock` tags.
impl Read for SubstreamTagReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.current_audio_data.is_none() && self.next().is_none() {
            //next() fills current_audio_data
            return Ok(0);
        }

        let chunk = self.current_audio_data.as_mut().unwrap();
        let data = chunk.data();

        //At this point, current_audio_data should be full
        let len = std::cmp::min(buf.len(), data.len());
        buf[..len].copy_from_slice(&data[..len]);

        drop(data);
        *chunk = chunk.to_start_and_end(len, chunk.len());

        if chunk.is_empty() {
            self.current_audio_data = None;
        }
        Ok(len)
    }
}

/// Create a new decoder that reads data from a shared `Substream` instance.
///
/// This works similarly to `make_stream_decoder` but using the new buffer
/// infrastructure that will eventually replace SWF-specific streaming.
///
/// The substream is shared in order to allow appending additional data into
/// the stream.
pub fn make_substream_decoder(
    stream_info: &SoundStreamInfo,
    data_stream: Substream,
) -> Result<Box<dyn Decoder + Send>, Error> {
    let decoder: Box<dyn Decoder + Send> = match stream_info.stream_format.compression {
        #[cfg(feature = "aac")]
        AudioCompression::Aac => Box::new(aac::AacSubstreamDecoder::new(stream_info, data_stream)?),
        AudioCompression::Adpcm => Box::new(AdpcmSubstreamDecoder::new(stream_info, data_stream)?),
        _ => Box::new(StandardSubstreamDecoder::new(stream_info, data_stream)?),
    };
    Ok(decoder)
}

/// The `StandardSubstreamDecoder` takes care of reading the audio data from
/// the chunks of a `Substream` and feeding it to the decoder.
struct StandardSubstreamDecoder {
    /// The underlying decoder. The decoder will get its data from a `Substream`.
    decoder: Box<dyn Decoder>,
}

impl StandardSubstreamDecoder {
    /// Constructs a new `StandardSubstreamDecoder`.
    /// `swf_data` should be the tag data of the MovieClip that contains the stream.
    fn new(stream_info: &SoundStreamInfo, data_stream: Substream) -> Result<Self, Error> {
        // Create a tag reader to get the audio data from SoundStreamBlock tags.
        let tag_reader = SubstreamTagReader::new(stream_info, data_stream);
        // Wrap the tag reader in the decoder.
        let decoder = make_decoder(&stream_info.stream_format, tag_reader)?;
        Ok(Self { decoder })
    }
}

impl Decoder for StandardSubstreamDecoder {
    fn num_channels(&self) -> u8 {
        self.decoder.num_channels()
    }
    fn sample_rate(&self) -> u16 {
        self.decoder.sample_rate()
    }
}

impl Iterator for StandardSubstreamDecoder {
    type Item = [i16; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
    }
}

/// ADPCM substream decoder.
///
/// Stream sounds encoded with ADPCM have an ADPCM header in each `SoundStreamBlock` tag, unlike
/// other compression formats that remain the same as if they were a single sound clip.
/// Therefore, we must recreate the decoder with each `SoundStreamBlock` to parse the additional
/// headers.
pub struct AdpcmSubstreamDecoder {
    format: SoundFormat,
    tag_reader: SubstreamTagReader,
    decoder: AdpcmDecoder<SliceCursor>,
}

impl AdpcmSubstreamDecoder {
    fn new(stream_info: &SoundStreamInfo, data_stream: Substream) -> Result<Self, Error> {
        let empty_buffer = data_stream.buffer().to_empty_slice();
        let mut tag_reader = SubstreamTagReader::new(stream_info, data_stream);
        let audio_data = tag_reader.next().unwrap_or(empty_buffer);
        let decoder = AdpcmDecoder::new(
            audio_data.as_cursor(),
            stream_info.stream_format.is_stereo,
            stream_info.stream_format.sample_rate,
        )?;
        Ok(Self {
            format: stream_info.stream_format.clone(),
            tag_reader,
            decoder,
        })
    }
}

impl Decoder for AdpcmSubstreamDecoder {
    fn num_channels(&self) -> u8 {
        self.decoder.num_channels()
    }
    fn sample_rate(&self) -> u16 {
        self.decoder.sample_rate()
    }
}

impl Iterator for AdpcmSubstreamDecoder {
    type Item = [i16; 2];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample_frame) = self.decoder.next() {
            // Return sample frames until the decoder has exhausted
            // the SoundStreamBlock tag.
            Some(sample_frame)
        } else if let Some(audio_data) = self.tag_reader.next() {
            // We've reached the end of the sound stream block tag, so
            // read the next one and recreate the decoder.
            // `AdpcmDecoder` read the ADPCM header when it is created.
            self.decoder = AdpcmDecoder::new(
                audio_data.as_cursor(),
                self.format.is_stereo,
                self.format.sample_rate,
            )
            .ok()?;
            self.decoder.next()
        } else {
            // No more SoundStreamBlock tags.
            None
        }
    }
}

#[cfg(feature = "aac")]
mod aac {
    use super::{Decoder, Error, SoundStreamInfo, Substream, SubstreamTagReader};

    use symphonia::{
        core::{
            self, audio,
            codecs::{CodecParameters, Decoder as SymphoniaDecoder},
            errors,
            formats::Packet,
        },
        default::codecs::AacDecoder as SymphoniaAacDecoder,
    };

    /// Unlike MP3, AAC can only be in FLV, not SWF. Therefore, no need for an
    /// `AacStreamDecoder`, nor for our own `AacDecoder` type wrapping Symphonia's.
    pub struct AacSubstreamDecoder {
        tag_reader: SubstreamTagReader,
        decoder: SymphoniaAacDecoder,
        sample_buf: audio::SampleBuffer<i16>,
        cur_sample: usize,
        stream_ended: bool,
    }

    impl AacSubstreamDecoder {
        pub fn new(stream_info: &SoundStreamInfo, data_stream: Substream) -> Result<Self, Error> {
            let tag_reader = SubstreamTagReader::new(stream_info, data_stream.clone());
            let layout = if stream_info.stream_format.is_stereo {
                audio::Layout::Stereo
            } else {
                audio::Layout::Mono
            };
            let sample_rate = stream_info.stream_format.sample_rate.into();
            let mut codec_params = CodecParameters::new();
            codec_params
                .for_codec(core::codecs::CODEC_TYPE_AAC)
                .with_channel_layout(layout)
                .with_sample_rate(sample_rate);

            let decoder = SymphoniaAacDecoder::try_new(&codec_params, &Default::default())?;

            Ok(Self {
                tag_reader,
                decoder,
                sample_buf: audio::SampleBuffer::new(
                    0,
                    audio::SignalSpec::new(sample_rate, layout.into_channels()),
                ),
                cur_sample: 0,
                stream_ended: false,
            })
        }
    }

    impl Decoder for AacSubstreamDecoder {
        fn num_channels(&self) -> u8 {
            self.decoder.last_decoded().spec().channels.count() as u8
        }
        fn sample_rate(&self) -> u16 {
            self.decoder.last_decoded().spec().rate as u16
        }
    }

    impl Iterator for AacSubstreamDecoder {
        type Item = [i16; 2];

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.cur_sample >= self.sample_buf.len() {
                if !self.stream_ended {
                    self.stream_ended = true;
                    self.cur_sample = 0;
                    for chunk in Iterator::by_ref(&mut self.tag_reader) {
                        match chunk.data()[0] {
                            0 => {
                                let mut codec_params = CodecParameters::new();
                                codec_params.for_codec(core::codecs::CODEC_TYPE_AAC);
                                codec_params.with_extra_data(chunk.data()[1..].into());

                                self.decoder = SymphoniaAacDecoder::try_new(
                                    &codec_params,
                                    &Default::default(),
                                )
                                .unwrap();

                                let spec = *(self.decoder.last_decoded().spec());
                                self.sample_buf = audio::SampleBuffer::new(0, spec);
                            }
                            1 => {
                                let packet = Packet::new_from_slice(0, 0, 0, &chunk.data()[1..]);
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

                                        self.stream_ended = false;
                                        break;
                                    }
                                    // Decode errors are not fatal.
                                    Err(errors::Error::DecodeError(_)) => (),
                                    Err(_) => break,
                                }
                            }
                            x => {
                                tracing::error!("AacSubstreamDecoder: unknown chunk type {}", x);
                            }
                        }
                    }
                }

                if self.stream_ended {
                    return None;
                }
            }

            let sample_buf = self.sample_buf.samples();
            if self.num_channels() == 2 {
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
}
