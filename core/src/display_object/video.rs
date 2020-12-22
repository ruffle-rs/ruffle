//! Video player display object

use crate::bounding_box::BoundingBox;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::types::{Degrees, Percent};
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::{Borrow, BorrowMut};
use std::collections::BTreeMap;
use std::sync::Arc;
use swf::{CharacterId, DefineVideoStream, VideoFrame};

/// A Video display object is a high-level interface to a video player.
///
/// Video data may be embedded within a variety of container formats, including
/// a host SWF, or an externally-loaded FLV or F4V file. In the latter form,
/// video framerates are (supposedly) permitted to differ from the stage
/// framerate.
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Video<'gc>(GcCell<'gc, VideoData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct VideoData<'gc> {
    base: DisplayObjectBase<'gc>,
    source_stream: GcCell<'gc, VideoSource>,
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub enum VideoSource {
    SWF {
        /// The movie that defined this video stream.
        movie: Arc<SwfMovie>,

        /// The video stream definition.
        streamdef: DefineVideoStream,

        /// The H.263 bitstream indexed by video frame ID.
        ///
        /// Each frame consists of a start and end parameter which can be used
        /// to reconstruct a reference to the embedded bitstream.
        frames: BTreeMap<u16, (usize, usize)>,
    },
}

impl<'gc> Video<'gc> {
    /// Construct a Video object that is tied to a SWF file's video stream.
    pub fn from_swf_tag(
        movie: Arc<SwfMovie>,
        streamdef: DefineVideoStream,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        let source_stream = GcCell::allocate(
            mc,
            VideoSource::SWF {
                movie,
                streamdef,
                frames: BTreeMap::new(),
            },
        );

        Video(GcCell::allocate(
            mc,
            VideoData {
                base: Default::default(),
                source_stream,
            },
        ))
    }

    /// Preload frame data from an SWF.
    ///
    /// This function yields an error if this video player is not playing an
    /// embedded SWF video.
    pub fn preload_swf_frame(&mut self, tag: VideoFrame, context: &mut UpdateContext<'_, 'gc, '_>) {
        match (*self
            .0
            .write(context.gc_context)
            .source_stream
            .write(context.gc_context))
        .borrow_mut()
        {
            VideoSource::SWF {
                movie,
                streamdef: _streamdef,
                frames,
            } => {
                let subslice = SwfSlice::from(movie.clone()).to_unbounded_subslice(tag.data);

                if let Some(subslice) = subslice {
                    frames.insert(tag.frame_num, (subslice.start, subslice.end));
                } else {
                    log::warn!("Invalid bitstream subslice on frame {}", tag.frame_num);
                }
            }
        }
    }

    /// Seek to a particular frame in the video stream.
    pub fn seek(&self, _context: &mut UpdateContext<'_, 'gc, '_>, _frame: u16) {}
}

impl<'gc> TDisplayObject<'gc> for Video<'gc> {
    impl_display_object!(base);

    fn as_video(self) -> Option<Video<'gc>> {
        Some(self)
    }

    fn id(&self) -> CharacterId {
        match (*self.0.read().source_stream.read()).borrow() {
            VideoSource::SWF { streamdef, .. } => streamdef.id,
        }
    }

    fn self_bounds(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::default();

        match (*self.0.read().source_stream.read()).borrow() {
            VideoSource::SWF { streamdef, .. } => {
                bounding_box.set_width(Twips::from_pixels(streamdef.width as f64));
                bounding_box.set_height(Twips::from_pixels(streamdef.height as f64));
            }
        }

        bounding_box
    }
}
