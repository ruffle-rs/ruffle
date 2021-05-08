//! Audio decoders.

mod adpcm;
#[cfg(any(feature = "minimp3", feature = "symphonia"))]
mod mp3;
mod nellymoser;
mod pcm;

pub use adpcm::AdpcmDecoder;
#[cfg(feature = "minimp3")]
pub use mp3::minimp3::Mp3Decoder;
#[cfg(all(feature = "symphonia", not(feature = "minimp3")))]
pub use mp3::symphonia::Mp3Decoder;
pub use nellymoser::NellymoserDecoder;
pub use pcm::PcmDecoder;

use crate::tag_utils::SwfSlice;
use std::io::{Cursor, Read};
use swf::{AudioCompression, SoundFormat, TagCode};

type Error = Box<dyn std::error::Error>;

/// An audio decoder. Can be used as an `Iterator` to return stero sample frames.
/// If the sound is mono, the sample is duplicated across both channels.
pub trait Decoder: Iterator<Item = [i16; 2]> {
    /// The number of channels of this audio decoder. Always 1 or 2.
    fn num_channels(&self) -> u8;

    /// The sample rate of this audio decoder.
    fn sample_rate(&self) -> u16;
}

/// Instantiate a decoder for the compression that the sound data uses.
pub fn make_decoder<R: 'static + Send + Read>(
    format: &SoundFormat,
    data: R,
) -> Result<Box<dyn Send + Decoder>, Error> {
    let decoder: Box<dyn Send + Decoder> = match format.compression {
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
        )),
        #[cfg(any(feature = "minimp3", feature = "symphonia"))]
        AudioCompression::Mp3 => Box::new(Mp3Decoder::new(
            if format.is_stereo { 2 } else { 1 },
            format.sample_rate.into(),
            data,
        )),
        AudioCompression::Nellymoser => {
            Box::new(NellymoserDecoder::new(data, format.sample_rate.into()))
        }
        _ => {
            let msg = format!(
                "make_decoder: Unhandled audio compression {:?}",
                format.compression
            );
            log::error!("{}", msg);
            return Err(msg.into());
        }
    };
    Ok(decoder)
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
    decoder: Box<dyn Decoder + Send>,
}

impl StandardStreamDecoder {
    /// Constructs a new `StandardStreamDecoder.
    /// `swf_data` should be the tag data of the MovieClip that contains the stream.
    fn new(format: &SoundFormat, swf_data: SwfSlice) -> Result<Self, Error> {
        // Create a tag reader to get the audio data from SoundStreamBlock tags.
        let tag_reader = StreamTagReader::new(format.compression, swf_data);
        // Wrap the tag reader in the decoder.
        let decoder = make_decoder(format, tag_reader)?;
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
    fn new(format: &SoundFormat, swf_data: SwfSlice) -> Self {
        let movie = swf_data.movie.clone();
        let mut tag_reader = StreamTagReader::new(format.compression, swf_data);
        let audio_data = tag_reader.next().unwrap_or_else(|| SwfSlice::empty(movie));
        let decoder = AdpcmDecoder::new(
            Cursor::new(audio_data),
            format.is_stereo,
            format.sample_rate,
        );
        Self {
            format: format.clone(),
            tag_reader,
            decoder,
        }
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
            );
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
    format: &swf::SoundFormat,
    swf_data: SwfSlice,
) -> Result<Box<dyn Decoder + Send>, Error> {
    let decoder: Box<dyn Decoder + Send> = if format.compression == AudioCompression::Adpcm {
        Box::new(AdpcmStreamDecoder::new(format, swf_data))
    } else {
        Box::new(StandardStreamDecoder::new(format, swf_data)?)
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
    swf_data: SwfSlice,
    pos: usize,
    current_frame: u16,
    current_audio_data: SwfSlice,
    compression: AudioCompression,
}

impl StreamTagReader {
    /// Builds a new `StreamTagReader` from the given SWF data.
    /// `swf_data` should be the tag data of a MovieClip.
    fn new(compression: AudioCompression, swf_data: SwfSlice) -> Self {
        let current_audio_data = SwfSlice::empty(swf_data.movie.clone());
        Self {
            swf_data,
            pos: 0,
            compression,
            current_frame: 1,
            current_audio_data,
        }
    }
}

impl Iterator for StreamTagReader {
    type Item = SwfSlice;

    fn next(&mut self) -> Option<Self::Item> {
        let current_frame = &mut self.current_frame;
        let audio_data = &mut self.current_audio_data;
        let compression = self.compression;
        let mut found = false;
        // MP3 stream blocks store seek samples and sample count in the first 4 bytes.
        // SWF19 p.184, p.188
        let skip_len = if compression == AudioCompression::Mp3 {
            4
        } else {
            0
        };

        let swf_data = &self.swf_data;
        let tag_callback = |reader: &mut swf::read::Reader<'_>, tag_code, tag_len| match tag_code {
            TagCode::ShowFrame => {
                *current_frame += 1;
                Ok(())
            }
            TagCode::SoundStreamBlock => {
                // TODO: Implement index ops on `SwfSlice`.
                //let pos = reader.get_ref().as_ptr() as usize - swf_data.as_ref().as_ptr() as usize;
                found = true;
                if tag_len >= skip_len {
                    *audio_data = swf_data
                        .to_subslice(&reader.get_ref()[skip_len..tag_len])
                        .unwrap()
                } else {
                    *audio_data = swf_data.to_subslice(&reader.get_ref()[..tag_len]).unwrap()
                };
                Ok(())
            }
            _ => Ok(()),
        };

        let version = swf_data.version();
        let mut reader = swf::read::Reader::new(&self.swf_data.as_ref()[self.pos..], version);
        let _ = crate::tag_utils::decode_tags(&mut reader, tag_callback, TagCode::SoundStreamBlock);
        self.pos = reader.get_ref().as_ptr() as usize - swf_data.as_ref().as_ptr() as usize;

        if found {
            Some(self.current_audio_data.clone())
        } else {
            None
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
