//! Video player display object

use crate::bounding_box::BoundingBox;
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::types::{Degrees, Percent};
use gc_arena::{Collect, Gc, GcCell};
use swf::CharacterId;

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
    static_data: Gc<'gc, VideoStatic>,
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct VideoStatic {
    movie: SwfMovie,
    id: CharacterId,
    width: Twips,
    height: Twips,
}

impl<'gc> TDisplayObject<'gc> for Video<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
    }

    fn self_bounds(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::default();

        bounding_box.set_width(self.0.read().static_data.width);
        bounding_box.set_height(self.0.read().static_data.height);

        bounding_box
    }
}
