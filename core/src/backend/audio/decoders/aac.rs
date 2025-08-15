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
                        // This is an `AacSequenceHeader` chunk
                        0 => {
                            let mut codec_params = CodecParameters::new();
                            codec_params.for_codec(core::codecs::CODEC_TYPE_AAC);
                            codec_params.with_extra_data(chunk.data()[1..].into());

                            self.decoder =
                                SymphoniaAacDecoder::try_new(&codec_params, &Default::default())
                                    .unwrap();

                            let spec = *(self.decoder.last_decoded().spec());
                            self.sample_buf = audio::SampleBuffer::new(0, spec);
                        }
                        // This is an `AacRaw` chunk
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
