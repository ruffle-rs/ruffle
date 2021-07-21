//! Audio decoders.

mod adpcm;
#[cfg(any(feature = "minimp3", feature = "symphonia"))]
mod mp3;
#[cfg(feature = "nellymoser")]
mod nellymoser;
mod pcm;

pub use adpcm::AdpcmDecoder;
#[cfg(feature = "minimp3")]
pub use mp3::minimp3::{mp3_metadata, Mp3Decoder};
#[cfg(all(feature = "symphonia", not(feature = "minimp3")))]
pub use mp3::symphonia::{mp3_metadata, Mp3Decoder};
#[cfg(feature = "nellymoser")]
pub use nellymoser::NellymoserDecoder;
pub use pcm::PcmDecoder;

use crate::tag_utils::SwfSlice;
use std::io::{Cursor, Read};
use swf::{AudioCompression, SoundFormat, TagCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "minimp3")]
    #[error("Couldn't decode MP3 using minimp3")]
    InvalidMp3(#[from] mp3::minimp3::Error),

    #[cfg(all(feature = "symphonia", not(feature = "minimp3")))]
    #[error("Couldn't decode MP3 using symphonia")]
    InvalidMp3(#[from] mp3::symphonia::Error),

    #[error("Couldn't decode ADPCM")]
    InvalidAdpcm(#[from] adpcm::Error),

    #[error("Unhandled compression {0:?}")]
    UnhandledCompression(AudioCompression),
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
            log::warn!("make_decoder: PCM sound is unknown endian; assuming little endian");
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
        #[cfg(any(feature = "minimp3", feature = "symphonia"))]
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
                            let num_samples =
                                u16::from_le_bytes(audio_block[..2].try_into().unwrap());
                            self.mp3_samples_buffered += i32::from(num_samples);
                            audio_block = &audio_block[4..];
                        }
                        *audio_data = swf_data.to_subslice(audio_block);
                        Ok(())
                    }
                    TagCode::ShowFrame if compression == AudioCompression::Mp3 => {
                        self.mp3_samples_buffered -= i32::from(self.mp3_samples_per_block);
                        Ok(())
                    }
                    _ => Ok(()),
                };

            let mut reader = self.swf_data.read_from(self.pos as u64);
            let _ =
                crate::tag_utils::decode_tags(&mut reader, tag_callback, TagCode::ShowFrame, None);
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
