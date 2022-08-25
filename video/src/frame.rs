use swf::VideoCodec;

/// An encoded video frame of some video codec.
pub struct EncodedFrame<'a> {
    /// The codec used to encode the frame.
    pub codec: VideoCodec,

    /// The raw bitstream data to funnel into the codec.
    pub data: &'a [u8],

    /// A caller-specified frame ID. Frame IDs must be consistent between
    /// subsequent uses of the same data stream.
    pub frame_id: u32,
}

impl<'a> EncodedFrame<'a> {
    /// Borrow this frame's data.
    pub fn data(&'a self) -> &'a [u8] {
        self.data
    }
}

/// A decoded frame of video in RGBA format.
pub struct DecodedFrame {
    pub width: u16,
    pub height: u16,
    pub rgba: Vec<u8>,
}

/// What dependencies a given video frame has on any previous frames.
#[derive(Copy, Clone, Debug)]
pub enum FrameDependency {
    /// This frame has no reference frames and can be seeked to at any time.
    None,

    /// This frame has some number of reference frames that prohibit any
    /// out-of-order decoding.
    ///
    /// The only legal way to decode a `Past` frame is to decode every prior
    /// frame from the last `None` frame. In the event that there is no prior
    /// `None` frame, then video decoding should start from the beginning.
    Past,
}

impl FrameDependency {
    /// Determine if this given frame is a keyframe.
    ///
    /// A keyframe is a frame that can be independently seeked to without
    /// decoding any prior or future frames.
    pub fn is_keyframe(self) -> bool {
        matches!(self, FrameDependency::None)
    }
}
