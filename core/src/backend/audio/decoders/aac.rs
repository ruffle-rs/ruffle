use super::{Decoder, Error, SoundStreamInfo, Substream, SubstreamTagReader};

use symphonia::{
    core::{
        audio::layouts,
        codecs::audio::{AudioCodecParameters, AudioDecoder, well_known::CODEC_ID_AAC},
        errors,
        packet::PacketRef,
        units::{Duration, Timestamp},
    },
    default::codecs::AacDecoder as SymphoniaAacDecoder,
};

/// Decodes AAC audio out of an FLV `Substream`.
///
/// Unlike MP3, AAC can only be in FLV, not SWF. Therefore, no need for an
/// `AacStreamDecoder`, nor for our own `AacDecoder` type wrapping Symphonia's.
///
/// The substream holds only raw AAC access units: the container layer (see
/// `NetStream::flv_audio_tag`) demuxes the `AudioSpecificConfig` out-of-band and
/// hands it to us via [`SoundStreamInfo::extra_data`], so no FLV-specific packet
/// framing reaches this decoder.
pub struct AacSubstreamDecoder {
    tag_reader: SubstreamTagReader,
    decoder: SymphoniaAacDecoder,
    /// Interleaved samples of the last decoded AAC access unit.
    sample_buf: Vec<i16>,
    cur_sample: usize,
    stream_ended: bool,
}

impl AacSubstreamDecoder {
    pub fn new(stream_info: &SoundStreamInfo, data_stream: Substream) -> Result<Self, Error> {
        let tag_reader = SubstreamTagReader::new(stream_info, data_stream);
        let channels = if stream_info.stream_format.is_stereo {
            layouts::CHANNEL_LAYOUT_STEREO
        } else {
            layouts::CHANNEL_LAYOUT_MONO
        };
        let sample_rate = stream_info.stream_format.sample_rate.into();

        let mut codec_params = AudioCodecParameters::new();
        codec_params
            .for_codec(CODEC_ID_AAC)
            .with_channels(channels)
            .with_sample_rate(sample_rate);
        // The `AudioSpecificConfig`, if the container provided one ahead of the
        // audio data (for FLV, this is the AAC sequence header). It refines the
        // channel layout and sample rate guessed above.
        if let Some(extra_data) = &stream_info.extra_data {
            codec_params.with_extra_data(extra_data.clone());
        }

        let decoder = SymphoniaAacDecoder::try_new(&codec_params, &Default::default())?;

        Ok(Self {
            tag_reader,
            decoder,
            sample_buf: Vec::new(),
            cur_sample: 0,
            stream_ended: false,
        })
    }
}

impl Decoder for AacSubstreamDecoder {
    fn num_channels(&self) -> u8 {
        self.decoder.last_decoded().spec().channels().count() as u8
    }
    fn sample_rate(&self) -> u16 {
        self.decoder.last_decoded().spec().rate() as u16
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
                    let data = chunk.data();
                    let packet = PacketRef::new(0, Timestamp::ZERO, Duration::ZERO, &data);
                    match self.decoder.decode_ref(&packet) {
                        Ok(decoded) => {
                            decoded.copy_to_vec_interleaved(&mut self.sample_buf);

                            self.stream_ended = false;
                            break;
                        }
                        // Decode errors are not fatal.
                        Err(errors::Error::DecodeError(_)) => (),
                        Err(_) => break,
                    }
                }
            }

            if self.stream_ended {
                return None;
            }
        }

        let sample_buf = &self.sample_buf;
        if self.num_channels() == 2 {
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
