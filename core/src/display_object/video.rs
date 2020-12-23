//! Video player display object

use crate::avm1::Object as Avm1Object;
use crate::backend::render::BitmapHandle;
use crate::backend::video::{EncodedFrame, VideoStreamHandle};
use crate::bounding_box::BoundingBox;
use crate::collect::CollectWrapper;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::types::{Degrees, Percent};
use crate::vminterface::Instantiator;
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

    /// The source of the video data (e.g. an external file, a SWF bitstream)
    source: GcCell<'gc, VideoSource>,

    /// The decoder stream that this video source is associated to.
    stream: Option<CollectWrapper<VideoStreamHandle>>,

    /// The last decoded frame in the video stream.
    decoded_frame: Option<CollectWrapper<BitmapHandle>>,
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
        frames: BTreeMap<u32, (usize, usize)>,
    },
}

impl<'gc> Video<'gc> {
    /// Construct a Video object that is tied to a SWF file's video stream.
    pub fn from_swf_tag(
        movie: Arc<SwfMovie>,
        streamdef: DefineVideoStream,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        let source = GcCell::allocate(
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
                source,
                stream: None,
                decoded_frame: None,
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
            .source
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
                    frames.insert(tag.frame_num.into(), (subslice.start, subslice.end));
                } else {
                    log::warn!("Invalid bitstream subslice on frame {}", tag.frame_num);
                }
            }
        }
    }

    /// Seek to a particular frame in the video stream.
    pub fn seek(self, context: &mut UpdateContext<'_, 'gc, '_>, frame_id: u32) {
        let read = self.0.read();
        let source = read.source;
        let stream = if let Some(stream) = &read.stream {
            stream
        } else {
            log::error!("Attempted to sync uninstantiated video stream!");
            return;
        };

        let res = match &*source.read() {
            VideoSource::SWF {
                movie,
                streamdef,
                frames,
            } => match frames.get(&frame_id) {
                Some((slice_start, slice_end)) => {
                    let encframe = EncodedFrame {
                        codec: streamdef.codec,
                        data: &movie.data()[*slice_start..*slice_end],
                        frame_id,
                    };
                    context
                        .video
                        .decode_video_stream_frame(stream.0, encframe, context.renderer)
                }
                None => Err(Box::from(format!(
                    "Attempted to seek to unknown frame {}",
                    frame_id
                ))),
            },
        };

        drop(read);

        match res {
            Ok(bitmapinfo) => {
                let bitmap = bitmapinfo.handle;

                self.0.write(context.gc_context).decoded_frame = Some(CollectWrapper(bitmap))
            }
            Err(e) => log::error!("Got error when seeking video: {}", e),
        }
    }
}

impl<'gc> TDisplayObject<'gc> for Video<'gc> {
    impl_display_object!(base);

    fn as_video(self) -> Option<Video<'gc>> {
        Some(self)
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _display_object: DisplayObject<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        let mut write = self.0.write(context.gc_context);

        let stream = match &*write.source.read() {
            VideoSource::SWF { streamdef, .. } => {
                let stream = context.video.register_video_stream(
                    streamdef.num_frames.into(),
                    (streamdef.width, streamdef.height),
                    streamdef.codec,
                    streamdef.deblocking,
                );
                if stream.is_err() {
                    log::error!(
                        "Got error when post-instantiating video: {}",
                        stream.unwrap_err()
                    );
                    return;
                }

                Some(CollectWrapper(stream.unwrap()))
            }
        };

        write.stream = stream;
        drop(write);

        if run_frame {
            self.run_frame(context);
        }
    }

    fn id(&self) -> CharacterId {
        match (*self.0.read().source.read()).borrow() {
            VideoSource::SWF { streamdef, .. } => streamdef.id,
        }
    }

    fn self_bounds(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::default();

        match (*self.0.read().source.read()).borrow() {
            VideoSource::SWF { streamdef, .. } => {
                bounding_box.set_width(Twips::from_pixels(streamdef.width as f64));
                bounding_box.set_height(Twips::from_pixels(streamdef.height as f64));
            }
        }

        bounding_box
    }

    fn render(&self, context: &mut RenderContext) {
        if !self.world_bounds().intersects(&context.view_bounds) {
            // Off-screen; culled
            return;
        }

        context.transform_stack.push(&*self.transform());

        if let Some(ref bitmap) = self.0.read().decoded_frame {
            context
                .renderer
                .render_bitmap(bitmap.0, context.transform_stack.transform(), false);
        }

        context.transform_stack.pop();
    }
}
