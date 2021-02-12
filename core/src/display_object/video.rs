//! Video player display object

use crate::avm1::{Object as Avm1Object, StageObject as Avm1StageObject};
use crate::avm2::{Object as Avm2Object, StageObject as Avm2StageObject};
use crate::backend::render::BitmapHandle;
use crate::backend::video::{EncodedFrame, VideoStreamHandle};
use crate::bounding_box::BoundingBox;
use crate::collect::CollectWrapper;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::types::{Degrees, Percent};
use crate::vminterface::{AvmObject, AvmType, Instantiator};
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::{Borrow, BorrowMut};
use std::collections::{BTreeMap, BTreeSet};
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
    stream: VideoStream,

    /// The last decoded frame in the video stream.
    decoded_frame: Option<(u32, CollectWrapper<BitmapHandle>)>,

    /// AVM representation of this video player.
    object: Option<AvmObject<'gc>>,

    /// List of frames which can be independently seeked to.
    ///
    /// Frames outside of this set must be decoded by playing each frame from
    /// the last keyframe in order. Any out-of-order seeking will be snapped to
    /// the prior keyframe. The first frame in the stream will always be
    /// treated as a keyframe regardless of it being flagged as one.
    keyframes: BTreeSet<u32>,
}

/// An optionally-instantiated video stream.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub enum VideoStream {
    /// An uninstantiated video stream.
    ///
    /// The stream index parameter is what frame we should seek to once the
    /// stream is instantiated.
    Uninstantiated(u32),

    /// An instantiated video stream.
    Instantiated(VideoStreamHandle),
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub enum VideoSource {
    /// A video bitstream embedded inside of a SWF movie.
    SWF {
        /// The movie that defined this video stream.
        movie: Arc<SwfMovie>,

        /// The video stream definition.
        streamdef: DefineVideoStream,

        /// The locations of each embedded sub-bitstream for each video frame.
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
                stream: VideoStream::Uninstantiated(0),
                decoded_frame: None,
                object: None,
                keyframes: BTreeSet::new(),
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

                if frames.contains_key(&tag.frame_num.into()) {
                    log::warn!("Duplicate frame {}", tag.frame_num);
                }

                if let Some(subslice) = subslice {
                    frames.insert(tag.frame_num.into(), (subslice.start, subslice.end));
                } else {
                    log::warn!("Invalid bitstream subslice on frame {}", tag.frame_num);
                }
            }
        }
    }

    /// Seek to a particular frame in the video stream.
    pub fn seek(self, context: &mut UpdateContext<'_, 'gc, '_>, mut frame_id: u32) {
        let read = self.0.read();
        let source = read.source;
        let stream = if let VideoStream::Instantiated(stream) = &read.stream {
            stream
        } else {
            drop(read);

            let mut write = self.0.write(context.gc_context);
            write.stream = VideoStream::Uninstantiated(frame_id);

            return;
        };
        let last_frame = read.decoded_frame.as_ref().map(|(lf, _)| *lf);
        let is_unordered_seek = frame_id != 0 && Some(frame_id) != last_frame.map(|lf| lf + 1);
        if is_unordered_seek {
            frame_id = read
                .keyframes
                .range(..=frame_id)
                .rev()
                .next()
                .copied()
                .unwrap_or(0);
        }

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
                        .decode_video_stream_frame(*stream, encframe, context.renderer)
                        .map(|bi| bi.handle)
                }
                None => {
                    if let Some((_old_id, old_frame)) = &read.decoded_frame {
                        Ok(old_frame.0)
                    } else {
                        Err(Box::from(format!(
                            "Attempted to seek to omitted frame {} without prior decoded frame",
                            frame_id
                        )))
                    }
                }
            },
        };

        drop(read);

        match res {
            Ok(bitmap) => {
                self.0.write(context.gc_context).decoded_frame =
                    Some((frame_id, CollectWrapper(bitmap)));
            }
            Err(e) => log::error!("Got error when seeking to video frame {}: {}", frame_id, e),
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
        display_object: DisplayObject<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        let mut write = self.0.write(context.gc_context);

        let (stream, movie, keyframes) = match &*write.source.read() {
            VideoSource::SWF {
                streamdef,
                movie,
                frames,
            } => {
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

                let stream = stream.unwrap();
                let mut keyframes = BTreeSet::new();

                for (frame_id, (frame_start, frame_end)) in frames {
                    let dep = context.video.preload_video_stream_frame(
                        stream,
                        EncodedFrame {
                            codec: streamdef.codec,
                            data: &movie.data()[*frame_start..*frame_end],
                            frame_id: *frame_id,
                        },
                    );

                    match dep {
                        Ok(d) if d.is_keyframe() => {
                            keyframes.insert(*frame_id);
                        }
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Got error when pre-loading video frame: {}", e);
                        }
                    }
                }

                (stream, movie.clone(), keyframes)
            }
        };

        let starting_seek = if let VideoStream::Uninstantiated(seek_to) = write.stream {
            seek_to
        } else {
            log::warn!("Reinstantiating already-instantiated video stream!");

            0
        };

        write.stream = VideoStream::Instantiated(stream);
        write.keyframes = keyframes;

        if write.object.is_none() {
            let library = context.library.library_for_movie_mut(movie);
            let vm_type = library.avm_type();
            if vm_type == AvmType::Avm2 {
                let object: Avm2Object<'_> = Avm2StageObject::for_display_object(
                    context.gc_context,
                    display_object,
                    context.avm2.prototypes().video,
                )
                .into();
                write.object = Some(object.into());
            } else if vm_type == AvmType::Avm1 {
                let object: Avm1Object<'_> = Avm1StageObject::for_display_object(
                    context.gc_context,
                    display_object,
                    Some(context.avm1.prototypes().video),
                )
                .into();
                write.object = Some(object.into());
            }
        }

        drop(write);

        self.seek(context, starting_seek);

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

        if let Some((_frame_id, ref bitmap)) = self.0.read().decoded_frame {
            context
                .renderer
                .render_bitmap(bitmap.0, context.transform_stack.transform(), false);
        } else {
            log::warn!("Video has no decoded frame to render.");
        }

        context.transform_stack.pop();
    }
}
