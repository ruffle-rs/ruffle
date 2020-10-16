//! Video player display object

use crate::bounding_box::BoundingBox;
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::types::{Degrees, Percent};
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::Borrow;
use std::sync::Arc;
use swf::{CharacterId, DefineVideoStream};

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
        movie: Arc<SwfMovie>,
        streamdef: DefineVideoStream,
    },
}

impl<'gc> Video<'gc> {
    /// Construct a Video object that is tied to a SWF file's video stream.
    pub fn from_swf_tag(
        movie: Arc<SwfMovie>,
        streamdef: DefineVideoStream,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        let source_stream = GcCell::allocate(mc, VideoSource::SWF { movie, streamdef });

        Video(GcCell::allocate(
            mc,
            VideoData {
                base: Default::default(),
                source_stream,
            },
        ))
    }
}

impl<'gc> TDisplayObject<'gc> for Video<'gc> {
    impl_display_object!(base);

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
