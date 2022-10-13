//! Video player display object

use crate::avm1::{Object as Avm1Object, StageObject as Avm1StageObject};
use crate::avm2::{
    Activation as Avm2Activation, Object as Avm2Object, StageObject as Avm2StageObject,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::vminterface::{AvmObject, Instantiator};
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::bitmap::BitmapInfo;
use ruffle_render::bounding_box::BoundingBox;
use ruffle_render::commands::CommandHandler;
use ruffle_video::error::Error;
use ruffle_video::frame::EncodedFrame;
use ruffle_video::VideoStreamHandle;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefMut};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use swf::{CharacterId, DefineVideoStream, VideoFrame};

use super::StageQuality;

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
    #[collect(require_static)]
    decoded_frame: Option<(u32, BitmapInfo)>,

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
    Swf {
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
            VideoSource::Swf {
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
            VideoSource::Swf {
                movie,
                streamdef: _streamdef,
                frames,
            } => {
                let subslice = SwfSlice::from(movie.clone()).to_unbounded_subslice(tag.data);

                if frames.contains_key(&tag.frame_num.into()) {
                    log::warn!("Duplicate frame {}", tag.frame_num);
                }

                frames.insert(tag.frame_num.into(), (subslice.start, subslice.end));
            }
        }
    }

    /// Seek to a particular frame in the video stream.
    ///
    /// This function ensures that the given `frame_id` is valid by first
    /// wrapping it to the underlying video stream's boundaries, and then
    /// snapping it to the last independently seekable frame. Then, all frames
    /// from that keyframe up to the (wrapped) requested frame are decoded in
    /// order. This matches Flash Player behavior.
    pub fn seek(self, context: &mut UpdateContext<'_, 'gc, '_>, mut frame_id: u32) {
        let read = self.0.read();
        if let VideoStream::Uninstantiated(_) = &read.stream {
            drop(read);

            let mut write = self.0.write(context.gc_context);
            write.stream = VideoStream::Uninstantiated(frame_id);

            return;
        };

        let num_frames = match &*read.source.read() {
            VideoSource::Swf { streamdef, .. } => Some(streamdef.num_frames),
        };

        if let Some(num_frames) = num_frames {
            frame_id = if num_frames > 0 {
                frame_id % num_frames as u32
            } else {
                0
            }
        }

        let last_frame = read.decoded_frame.as_ref().map(|(lf, _)| *lf);

        if last_frame == Some(frame_id) {
            return; // we are already there, no-op
        }

        let is_ordered_seek = frame_id == 0 || Some(frame_id) == last_frame.map(|lf| lf + 1);

        // When seeking to a frame that is not right after the previously shown,
        // nor is it a keyframe, we have to first seek through all frames
        // starting with the preceding keyframe.
        let sweep_from = if is_ordered_seek {
            frame_id // no need to sweep
        } else {
            let prev_keyframe_id = read
                .keyframes
                .range(..=frame_id)
                .rev()
                .next()
                .copied()
                .unwrap_or(0);

            // Start sweeping from either the preceding keyframe, or continue from
            // where we last were if that is closer to where we want to be.
            if let Some(lf) = last_frame {
                if frame_id > lf {
                    // When seeking forward, there is a chance that continuing from
                    // the last frame gets us there faster (if the skip is small).
                    u32::max(prev_keyframe_id, lf + 1)
                } else {
                    prev_keyframe_id
                }
            } else {
                prev_keyframe_id
            }
        };

        drop(read);

        for fr in sweep_from..=frame_id {
            self.seek_internal(context, fr)
        }
    }

    /// Decode a single frame of video.
    ///
    /// This function makes no attempt to ensure that the proposed seek is
    /// valid, hence the fact that it's not `pub`. To do a seek that accounts
    /// for keyframes, see `Video.seek`.
    fn seek_internal(self, context: &mut UpdateContext<'_, 'gc, '_>, frame_id: u32) {
        let read = self.0.read();
        let source = read.source;
        let stream = if let VideoStream::Instantiated(stream) = &read.stream {
            stream
        } else {
            log::error!("Attempted to seek uninstantiated video stream.");
            return;
        };

        let res = match &*source.read() {
            VideoSource::Swf {
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
                }
                None => {
                    if let Some((_old_id, old_frame)) = read.decoded_frame {
                        Ok(old_frame)
                    } else {
                        Err(Error::SeekingBeforeDecoding(frame_id))
                    }
                }
            },
        };

        drop(read);

        match res {
            Ok(bitmap) => {
                self.0.write(context.gc_context).decoded_frame = Some((frame_id, bitmap));
            }
            Err(e) => log::error!("Got error when seeking to video frame {}: {}", frame_id, e),
        }
    }
}

impl<'gc> TDisplayObject<'gc> for Video<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn as_video(self) -> Option<Video<'gc>> {
        Some(self)
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if !context.is_action_script_3() {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());
        }

        let mut write = self.0.write(context.gc_context);

        let (stream, movie, keyframes) = match &*write.source.read() {
            VideoSource::Swf {
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

        if write.object.is_none() && !movie.is_action_script_3() {
            let object: Avm1Object<'_> = Avm1StageObject::for_display_object(
                context.gc_context,
                (*self).into(),
                context.avm1.prototypes().video,
            )
            .into();
            write.object = Some(object.into());
        }

        drop(write);

        self.seek(context, starting_seek);

        if run_frame {
            self.run_frame(context);
        }
    }

    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if context.is_action_script_3() && matches!(self.object2(), Avm2Value::Undefined) {
            let video_constr = context.avm2.classes().video;
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            match Avm2StageObject::for_display_object_childless(
                &mut activation,
                (*self).into(),
                video_constr,
            ) {
                Ok(object) => {
                    let object: Avm2Object<'gc> = object.into();
                    self.0.write(context.gc_context).object = Some(object.into())
                }
                Err(e) => log::error!("Got {} when constructing AVM2 side of video player", e),
            }
        }
    }

    fn id(&self) -> CharacterId {
        match (*self.0.read().source.read()).borrow() {
            VideoSource::Swf { streamdef, .. } => streamdef.id,
        }
    }

    fn self_bounds(&self) -> BoundingBox {
        let mut bounding_box = BoundingBox::default();

        match (*self.0.read().source.read()).borrow() {
            VideoSource::Swf { streamdef, .. } => {
                bounding_box.set_width(Twips::from_pixels(streamdef.width as f64));
                bounding_box.set_height(Twips::from_pixels(streamdef.height as f64));
            }
        }

        bounding_box
    }

    fn render(&self, context: &mut RenderContext) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        context.transform_stack.push(self.base().transform());

        let read = self.0.read();

        if let Some((_frame_id, ref bitmap)) = read.decoded_frame {
            let mut transform = context.transform_stack.transform().clone();
            let bounds = self.self_bounds();

            // The actual decoded frames might be different in size than the declared
            // bounds of the VideoStream tag, so a final scale adjustment has to be done.
            transform.matrix *= Matrix::scale(
                bounds.width().to_pixels() as f32 / bitmap.width as f32,
                bounds.height().to_pixels() as f32 / bitmap.height as f32,
            );

            let (smoothed_flag, num_frames, version) = match &*read.source.read() {
                VideoSource::Swf {
                    streamdef,
                    frames,
                    movie,
                } => (streamdef.is_smoothed, frames.len(), movie.version()),
            };

            let smoothing = match (context.stage.quality(), version) {
                (StageQuality::Low, _) => false,
                (_, 8..) => smoothed_flag,
                (StageQuality::Medium, _) => false,
                (StageQuality::High, _) => num_frames == 1,
                (_, _) => true,
            };

            context
                .commands
                .render_bitmap(bitmap.handle, &transform, smoothing);
        } else {
            log::warn!("Video has no decoded frame to render.");
        }

        context.transform_stack.pop();
    }

    fn set_object2(&mut self, mc: MutationContext<'gc, '_>, to: Avm2Object<'gc>) {
        self.0.write(mc).object = Some(to.into());
    }
}
