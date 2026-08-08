use ruffle_video::error::Error;
use ruffle_video::frame::{DecodedFrame, EncodedFrame, FrameDependency};

#[cfg(feature = "h263")]
pub mod h263;

#[cfg(feature = "vp6")]
pub mod vp6;

#[cfg(feature = "screenvideo")]
pub mod screen;

/// Trait for video decoders.
/// This should be implemented for each video codec.
pub trait VideoDecoder {
    /// Configure the decoder.
    fn configure_decoder(&mut self, _configuration_data: &[u8]) -> Result<(), Error> {
        Ok(())
    }

    /// Preload a frame.
    ///
    /// No decoding is intended to happen at this point in time. Instead, the
    /// video data should be inspected to determine inter-frame dependencies
    /// between this and any previous frames in the stream.
    ///
    /// Frames should be preloaded in the order that they are received.
    ///
    /// Any dependencies listed here are inherent to the video bitstream. The
    /// containing video stream is also permitted to introduce additional
    /// interframe dependencies.
    fn preload_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<FrameDependency, Error>;

    /// Decode a frame of a given video stream.
    ///
    /// This function is provided the external index of the frame, the codec
    /// used to decode the data, and what codec to decode it with. The codec
    /// provided here must match the one used to register the video stream.
    ///
    /// Frames may be decoded in any order that does not violate the frame
    /// dependencies declared by the output of `preload_video_stream_frame`.
    ///
    /// Either calls the provided `callback` with the decoded frame and returns
    /// `Ok(())`, or returns an error if the frame could not be decoded.
    ///
    /// Note: this uses a callback instead of a simple return value to give
    /// decoders full control over the lifetime of the decoded frame data.
    fn decode_frame_dyn(
        &mut self,
        encoded_frame: EncodedFrame<'_>,
        callback: &mut dyn FnMut(DecodedFrame<'_>),
    ) -> Result<(), Error>;
}

impl dyn VideoDecoder {
    /// Helper method to wrangle the dyn-compatible `decode_frame_dyn` API into
    /// something friendlier.
    pub fn decode_frame<R>(
        &mut self,
        encoded_frame: EncodedFrame<'_>,
        callback: impl FnOnce(DecodedFrame<'_>) -> Result<R, Error>,
    ) -> Result<R, Error> {
        let mut cb = Some(callback);
        let mut r = None;
        self.decode_frame_dyn(encoded_frame, &mut |decoded| {
            if let Some(cb) = cb.take() {
                r = Some(cb(decoded));
            }
        })?;
        r.unwrap_or(Err(Error::DecoderNoOutputFrame))
    }
}
