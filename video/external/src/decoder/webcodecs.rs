use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;

use crate::decoder::VideoDecoder;

use ruffle_render::bitmap::BitmapFormat;
use ruffle_video::error::Error;
use ruffle_video::frame::{DecodedFrame, EncodedFrame, FrameDependency};

use js_sys::Uint8Array;
use tracing::{debug, error, trace, warn};
use tracing_subscriber::{Registry, layer::Layered};
use tracing_wasm::WASMLayer;
use wasm_bindgen::prelude::*;
use web_sys::{
    DomException, EncodedVideoChunk, EncodedVideoChunkInit, EncodedVideoChunkType,
    VideoDecoder as WebVideoDecoder, VideoDecoderConfig, VideoDecoderInit, VideoFrame,
    VideoPixelFormat,
};

// Abbreviations used:
//  - NAL: Network Abstraction Layer
//  - NALU: NAL unit
//  - VCL: Video Coding Layer
//  - SPS: Sequence Parameter Set
//  - PPS: Picture Parameter Set
//  - IDR: Instantaneous Decoding Refresh
//  - SEI: Supplemental enhancement information

// NALU type 5 means IDR frame - basically a keyframe.
const NALU_TYPE_IDR: u8 = 5;

fn js_error_to_decoder_error(js_error: JsValue) -> Error {
    Error::DecoderError(
        js_error
            .dyn_ref::<js_sys::Error>()
            .unwrap()
            .message()
            .as_string()
            .unwrap()
            .into(),
    )
}

pub struct H264Decoder {
    /// How many bytes are used to store the length of the NALU (1, 2, 3, or 4).
    length_size: u8,

    /// The WebCodecs decoder object.
    decoder: WebVideoDecoder,

    /// The decoder output callback writes this, and the decode_frame method reads it.
    ///
    /// This in itself results in one frame of delay (because we can't block decode_frame
    /// until the callback is invoked), but it shouldn't matter in practice.
    last_frame: Rc<RefCell<LastFrame>>,

    // Simply keeping these objects alive, as they are used by the JS side.
    // See: https://rustwasm.github.io/wasm-bindgen/examples/closures.html
    #[expect(dead_code)]
    output_callback: Closure<dyn Fn(VideoFrame)>,
    #[expect(dead_code)]
    error_callback: Closure<dyn Fn(DomException)>,
}

struct LastFrame {
    status: Result<(), Error>,
    // This is kept separate from `status` so that it can be reused between frames.
    data: DecodedFrame<'static>,
}

impl H264Decoder {
    /// `extradata` should hold "AVCC (MP4) format" decoder configuration, including PPS and SPS.
    /// Make sure it has any start code emulation prevention "three bytes" removed.
    ///
    /// The log_subscriber is needed so that we have proper logging from within the callbacks.
    pub fn new(log_subscriber: Arc<Layered<WASMLayer, Registry>>) -> Result<Self, Error> {
        let last_frame = Rc::new(RefCell::new(LastFrame {
            status: Err(Error::DecoderNoOutputFrame),
            data: DecodedFrame::empty(BitmapFormat::Rgb),
        }));
        let lf = last_frame.clone();

        let log_subscriber_for_output = log_subscriber.clone();
        let output = move |output: &VideoFrame| {
            let _subscriber = tracing::subscriber::set_default(log_subscriber_for_output.clone());
            let visible_rect = output.visible_rect().unwrap();
            let (width, height) = (visible_rect.width() as u32, visible_rect.height() as u32);

            type FormatProcessor<'a> = (BitmapFormat, &'a dyn Fn(&mut Vec<u8>));

            let src_format = output.format().unwrap();
            let processor: Result<FormatProcessor<'_>, _> = match src_format {
                VideoPixelFormat::I420 => Ok((BitmapFormat::Yuv420p, &|_data| {
                    // nothing to do
                })),
                VideoPixelFormat::Bgrx => Ok((BitmapFormat::Rgba, &|data| {
                    for [b, _g, r, x] in data.as_chunks_mut::<4>().0 {
                        std::mem::swap(b, r);
                        *x = 0xff;
                    }
                })),
                VideoPixelFormat::Nv12 => Ok((BitmapFormat::Yuv420p, &|data| {
                    let luma_len = width as usize * height as usize;
                    let chroma_len = width.div_ceil(2) as usize * height.div_ceil(2) as usize;
                    assert_eq!(luma_len + 2 * chroma_len, data.len());

                    // Need some scratch space to deinterlace chroma pairs
                    let original_len = data.len();
                    data.extend_from_within(luma_len..);

                    let (dst, chroma_pairs) = data.split_at_mut(original_len);
                    let (u_dst, v_dst) = dst[luma_len..].split_at_mut(chroma_len);

                    use std::iter::zip;
                    for ((u, v), uv) in zip(zip(u_dst, v_dst), chroma_pairs.as_chunks::<2>().0) {
                        *u = uv[0];
                        *v = uv[1];
                    }

                    // Clear scratch space
                    data.truncate(original_len);
                })),
                other_format => Err(Error::DecoderError(
                    format!("Unsupported pixel format: {other_format:?}").into(),
                )),
            };

            let mut frame = last_frame.borrow_mut();
            frame.status = processor.map(|(dst_format, process_pixels)| {
                let size_in_bytes = dst_format.length_for_size(width as usize, height as usize);

                // Recycle the last frame's buffer.
                let mut data = mem::replace(&mut frame.data, DecodedFrame::empty(dst_format))
                    .into_buf()
                    .into_owned();
                data.clear();
                data.reserve_exact(size_in_bytes);

                data.resize(size_in_bytes, 0);
                let _ = output.copy_to_with_u8_slice(&mut data);
                process_pixels(&mut data);

                frame.data = DecodedFrame::new(width, height, dst_format, data);
            });

            output.close();
        };

        let log_subscriber_for_error = log_subscriber.clone();
        let error = move |error: &DomException| {
            let _subscriber = tracing::subscriber::set_default(log_subscriber_for_error.clone());
            error!("WebCodecs error: {:}", error.message());
        };

        let output_callback = Closure::new(move |frame| output(&frame));
        let error_callback = Closure::new(move |exception| error(&exception));

        let decoder = WebVideoDecoder::new(&VideoDecoderInit::new(
            error_callback.as_ref().unchecked_ref(),
            output_callback.as_ref().unchecked_ref(),
        ))
        .map_err(js_error_to_decoder_error)?;

        Ok(Self {
            length_size: 0,
            decoder,
            output_callback,
            error_callback,
            last_frame: lf,
        })
    }
}

/// Provides an iterator for individual consecutive NALUs in a byte stream,
/// also providing the type of each NALU for easier usage.
fn iter_nalus(data: &[u8], length_size: usize) -> impl Iterator<Item = (u8, &[u8])> {
    trace!(
        "iter_nalus on a {} long chunk with length_size {}",
        data.len(),
        length_size
    );

    let mut rest = data;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }

        if rest.len() < length_size {
            warn!("Not enough data to read NALU length");
            return None;
        }

        // Extracting and skipping over the NALU length.
        let mut encoded_len = 0;
        for b in rest.iter().take(length_size) {
            encoded_len = (encoded_len << 8) | *b as usize;
        }
        trace!("encoded_len: {}", encoded_len);

        if rest.len() < length_size + encoded_len {
            warn!("Not enough data to read NALU");
            return None;
        }

        // Extracting and skipping over the NALU type and data.
        let nalu_type = rest[length_size] & 0b0001_1111;
        let nalu;
        (nalu, rest) = rest.split_at(length_size + encoded_len);

        trace!("nalu_type: {}", nalu_type);
        trace!("rest len: {}", rest.len());
        Some((nalu_type, nalu))
    })
}

impl VideoDecoder for H264Decoder {
    fn configure_decoder(&mut self, configuration_data: &[u8]) -> Result<(), Error> {
        // extradata[0]: configuration version, always 1
        // extradata[1]: profile
        // extradata[2]: compatibility
        // extradata[3]: level
        // extradata[4]: 6 reserved bits | NALU length size - 1

        // The codec string is the profile, compatibility, and level bytes as hex.

        if configuration_data.len() < 5 {
            return Err(Error::DecoderError(
                "Invalid configuration data for H264 decoder".into(),
            ));
        }
        if configuration_data[0] != 1 {
            return Err(Error::DecoderError(
                "Invalid configuration version for H264 decoder".into(),
            ));
        }

        self.length_size = (configuration_data[4] & 0b0000_0011) + 1;

        trace!("length_size: {}", self.length_size);

        let codec_string = format!(
            "avc1.{:02x}{:02x}{:02x}",
            configuration_data[1], configuration_data[2], configuration_data[3]
        );
        let config = VideoDecoderConfig::new(&codec_string);
        trace!("decoder state: {:?}", self.decoder.state());
        trace!("configuring decoder with: {:?}", &configuration_data[1..4]);

        let data = Uint8Array::from(configuration_data);
        config.set_description(&data);
        config.set_optimize_for_latency(true);
        self.decoder
            .configure(&config)
            .map_err(js_error_to_decoder_error)?;

        trace!("decoder state: {:?}", self.decoder.state());
        Ok(())
    }

    fn preload_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<FrameDependency, Error> {
        debug!("preloading frame {}", encoded_frame.frame_id);

        for (nalu_type, _nalu) in iter_nalus(encoded_frame.data, self.length_size as usize) {
            // "After the decoding of an IDR picture all following coded pictures in decoding order can
            // be decoded without inter prediction from any picture decoded prior to the IDR picture."
            if nalu_type == NALU_TYPE_IDR {
                trace!("is key");
                return Ok(FrameDependency::None);
            }
        }

        trace!("is not key");
        Ok(FrameDependency::Past)
    }

    fn decode_frame_dyn(
        &mut self,
        encoded_frame: EncodedFrame<'_>,
        callback: &mut dyn FnMut(DecodedFrame<'_>),
    ) -> Result<(), Error> {
        debug!("decoding frame {}", encoded_frame.frame_id);
        trace!("decoder state: {:?}", self.decoder.state());
        trace!("queue size: {}", self.decoder.decode_queue_size());

        let mut frame_type = EncodedVideoChunkType::Delta;
        for (nalu_type, _nalu) in iter_nalus(encoded_frame.data, self.length_size as usize) {
            if nalu_type == NALU_TYPE_IDR {
                frame_type = EncodedVideoChunkType::Key;
            }
        }
        trace!("frame type: {:?}", frame_type);

        // The timestamp doesn't matter for us.
        let init = EncodedVideoChunkInit::new(&Uint8Array::from(encoded_frame.data), 0, frame_type);
        let chunk = EncodedVideoChunk::new(&init).unwrap();

        self.decoder
            .decode(&chunk)
            .map_err(js_error_to_decoder_error)?;
        trace!("decoder state: {:?}", self.decoder.state());

        let mut frame = self.last_frame.borrow_mut();
        mem::replace(&mut frame.status, Err(Error::DecoderNoOutputFrame))
            .map(|()| callback(frame.data.reborrow()))
    }
}
