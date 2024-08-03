//! Video player display object

use crate::avm1::{Object as Avm1Object, StageObject as Avm1StageObject, Value as Avm1Value};
use crate::avm2::{
    Activation as Avm2Activation, Object as Avm2Object, StageObject as Avm2StageObject,
    Value as Avm2Value,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr};
use crate::prelude::*;
use crate::streams::NetStream;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::vminterface::{AvmObject, Instantiator};
use core::fmt;
use gc_arena::{Collect, GcCell, Mutation};
use ruffle_render::bitmap::{BitmapInfo, PixelSnapping};
use ruffle_render::commands::CommandHandler;
use ruffle_render::quality::StageQuality;
use ruffle_video::error::Error;
use ruffle_video::frame::EncodedFrame;
use ruffle_video::VideoStreamHandle;
use std::borrow::BorrowMut;
use std::cell::{Ref, RefMut};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use swf::{DefineVideoStream, VideoCodec, VideoFrame};

/// A Video display object is a high-level interface to a video player.
///
/// Video data may be embedded within a variety of container formats, including
/// a host SWF, or an externally-loaded FLV or F4V file. In the latter form,
/// video framerates are (supposedly) permitted to differ from the stage
/// framerate.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Video<'gc>(GcCell<'gc, VideoData<'gc>>);

impl fmt::Debug for Video<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Video")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct VideoData<'gc> {
    base: DisplayObjectBase<'gc>,

    /// The source of the video data (e.g. an external file, a SWF bitstream)
    source: GcCell<'gc, VideoSource<'gc>>,

    /// The decoder stream that this video source is associated to.
    #[collect(require_static)]
    stream: VideoStream,

    /// AVM representation of this video player.
    object: Option<AvmObject<'gc>>,

    /// List of frames which can be independently seeked to.
    ///
    /// Frames outside of this set must be decoded by playing each frame from
    /// the last keyframe in order. Any out-of-order seeking will be snapped to
    /// the prior keyframe. The first frame in the stream will always be
    /// treated as a keyframe regardless of it being flagged as one.
    keyframes: BTreeSet<u32>,

    /// The movie whose tagstream or code created the Video object.
    movie: Arc<SwfMovie>,

    /// The self bounds for this movie.
    size: (i32, i32),

    /// The last decoded frame in the video stream.
    ///
    /// NOTE: This is only used for SWF-source video streams.
    #[collect(require_static)]
    decoded_frame: Option<(u32, BitmapInfo)>,
}

/// An optionally-instantiated video stream.
#[derive(Clone, Debug)]
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
#[collect(no_drop)]
pub enum VideoSource<'gc> {
    /// A video bitstream embedded inside of a SWF movie.
    ///
    /// NOTE: Fields within this enum will be shared across all instances of a
    /// particular character. If you need to mutate the video source, consider
    /// reallocating a new source for your specific video instead.
    ///
    /// This warning does not apply to `NetStream` or `Unconnected` videos,
    /// which are never aliased.
    Swf {
        /// The video stream definition.
        #[collect(require_static)]
        streamdef: DefineVideoStream,

        /// The locations of each embedded sub-bitstream for each video frame.
        ///
        /// Each frame consists of a start and end parameter which can be used
        /// to reconstruct a reference to the embedded bitstream.
        frames: BTreeMap<u32, (usize, usize)>,
    },
    /// An attached NetStream.
    NetStream {
        /// The stream the video is downloaded from.
        stream: NetStream<'gc>,
    },
    Unconnected,
}

impl<'gc> Video<'gc> {
    /// Construct a Video object that is tied to a SWF file's video stream.
    pub fn from_swf_tag(
        movie: Arc<SwfMovie>,
        streamdef: DefineVideoStream,
        mc: &Mutation<'gc>,
    ) -> Self {
        let size = (streamdef.width.into(), streamdef.height.into());
        let source = GcCell::new(
            mc,
            VideoSource::Swf {
                streamdef,
                frames: BTreeMap::new(),
            },
        );

        Video(GcCell::new(
            mc,
            VideoData {
                base: Default::default(),
                source,
                stream: VideoStream::Uninstantiated(0),
                object: None,
                keyframes: BTreeSet::new(),
                movie,
                size,
                decoded_frame: None,
            },
        ))
    }

    pub fn new(
        mc: &Mutation<'gc>,
        movie: Arc<SwfMovie>,
        width: i32,
        height: i32,
        object: Option<AvmObject<'gc>>,
    ) -> Self {
        let source = GcCell::new(mc, VideoSource::Unconnected);

        Video(GcCell::new(
            mc,
            VideoData {
                base: Default::default(),
                source,
                stream: VideoStream::Uninstantiated(0),
                object,
                keyframes: BTreeSet::new(),
                movie,
                size: (width, height),
                decoded_frame: None,
            },
        ))
    }

    pub fn set_size(self, mc: &Mutation<'gc>, width: i32, height: i32) {
        self.0.write(mc).size = (width, height);
    }

    /// Convert this Video into a NetStream sourced video.
    ///
    /// Existing video state related to the old video stream will be dropped.
    pub fn attach_netstream(self, context: &mut UpdateContext<'gc>, stream: NetStream<'gc>) {
        let mut video = self.0.write(context.gc_context);

        video.source = GcCell::new(context.gc_context, VideoSource::NetStream { stream });
        video.stream = VideoStream::Uninstantiated(0);
        video.keyframes = BTreeSet::new();
    }

    /// Preload frame data from an SWF.
    ///
    /// This function yields an error if this video player is not playing an
    /// embedded SWF video.
    pub fn preload_swf_frame(&mut self, tag: VideoFrame, context: &mut UpdateContext<'gc>) {
        let movie = self.0.read().movie.clone();

        match (*self
            .0
            .write(context.gc_context)
            .source
            .write(context.gc_context))
        .borrow_mut()
        {
            VideoSource::Swf { frames, .. } => {
                let subslice = SwfSlice::from(movie).to_unbounded_subslice(tag.data);

                if frames.contains_key(&tag.frame_num.into()) {
                    tracing::warn!("Duplicate frame {}", tag.frame_num);
                }

                frames.insert(tag.frame_num.into(), (subslice.start, subslice.end));
            }
            VideoSource::NetStream { .. } => {}
            VideoSource::Unconnected { .. } => {}
        }
    }

    /// Seek to a particular frame in the video stream.
    ///
    /// This function ensures that the given `frame_id` is valid by first
    /// wrapping it to the underlying video stream's boundaries, and then
    /// snapping it to the last independently seekable frame. Then, all frames
    /// from that keyframe up to the (wrapped) requested frame are decoded in
    /// order. This matches Flash Player behavior.
    ///
    /// `seek` is only called when processing `PlaceObject` tags involving this
    /// Video. It is a no-op for Videos that are connected to a `NetStream`.
    pub fn seek(self, context: &mut UpdateContext<'gc>, mut frame_id: u32) {
        let read = self.0.read();
        if let VideoStream::Uninstantiated(_) = &read.stream {
            drop(read);

            let mut write = self.0.write(context.gc_context);
            write.stream = VideoStream::Uninstantiated(frame_id);

            return;
        };

        let num_frames = match &*read.source.read() {
            VideoSource::Swf { streamdef, .. } => streamdef.num_frames as usize,
            VideoSource::NetStream { .. } => return,
            VideoSource::Unconnected { .. } => return,
        };

        frame_id = if num_frames > 0 {
            frame_id % num_frames as u32
        } else {
            0
        };

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
                .next_back()
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
    fn seek_internal(self, context: &mut UpdateContext<'gc>, frame_id: u32) {
        let read = self.0.read();
        let source = read.source;
        let stream = if let VideoStream::Instantiated(stream) = &read.stream {
            stream
        } else {
            tracing::error!("Attempted to seek uninstantiated video stream.");
            return;
        };

        let res = match &*source.read() {
            VideoSource::Swf { streamdef, frames } => match frames.get(&frame_id) {
                Some((slice_start, slice_end)) => {
                    let encframe = EncodedFrame {
                        codec: streamdef.codec,
                        data: &read.movie.data()[*slice_start..*slice_end],
                        frame_id,
                    };
                    context
                        .video
                        .decode_video_stream_frame(*stream, encframe, context.renderer)
                }
                None => {
                    if let Some((_old_id, old_frame)) = read.decoded_frame.clone() {
                        Ok(old_frame)
                    } else {
                        Err(Error::SeekingBeforeDecoding(frame_id))
                    }
                }
            },
            VideoSource::NetStream { .. } => return,
            VideoSource::Unconnected { .. } => return,
        };

        drop(read);

        match res {
            Ok(bitmap) => {
                self.0.write(context.gc_context).decoded_frame = Some((frame_id, bitmap));
                self.invalidate_cached_bitmap(context.gc_context);
                *context.needs_render = true;
            }
            Err(e) => tracing::error!("Got error when seeking to video frame {}: {}", frame_id, e),
        }
    }
}

impl<'gc> TDisplayObject<'gc> for Video<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn base_mut<'a>(&'a self, mc: &Mutation<'gc>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn instantiate(&self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(GcCell::new(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn as_video(self) -> Option<Video<'gc>> {
        Some(self)
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if !self.movie().is_action_script_3() {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());
        }

        let mut write = self.0.write(context.gc_context);
        let movie = write.movie.clone();

        let (stream, keyframes) = match &*write.source.read() {
            VideoSource::Swf {
                streamdef, frames, ..
            } => {
                if streamdef.codec == VideoCodec::None {
                    // No codec means no frames.
                    (None, BTreeSet::new())
                } else {
                    let stream = context.video.register_video_stream(
                        streamdef.num_frames.into(),
                        (streamdef.width, streamdef.height),
                        streamdef.codec,
                        streamdef.deblocking,
                    );
                    if stream.is_err() {
                        tracing::error!(
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
                                tracing::error!("Got error when pre-loading video frame: {}", e);
                            }
                        }
                    }

                    (Some(stream), keyframes)
                }
            }
            VideoSource::NetStream { .. } => return,
            VideoSource::Unconnected { .. } => return,
        };

        let starting_seek = if let VideoStream::Uninstantiated(seek_to) = write.stream {
            seek_to
        } else {
            tracing::warn!("Reinstantiating already-instantiated video stream!");

            0
        };

        if let Some(stream) = stream {
            write.stream = VideoStream::Instantiated(stream);
        }
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

        if !self.movie().is_action_script_3() && run_frame {
            self.run_frame_avm1(context);
        }
    }

    fn construct_frame(&self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() && matches!(self.object2(), Avm2Value::Null) {
            let video_constr = context.avm2.classes().video;
            let mut activation = Avm2Activation::from_nothing(context);
            let size = self.0.read().size;
            match Avm2StageObject::for_display_object_childless_with_args(
                &mut activation,
                (*self).into(),
                video_constr,
                &[size.0.into(), size.1.into()],
            ) {
                Ok(object) => {
                    let object: Avm2Object<'gc> = object.into();
                    self.0.write(context.gc_context).object = Some(object.into())
                }
                Err(e) => tracing::error!("Got {} when constructing AVM2 side of video player", e),
            }

            self.on_construction_complete(context);
        }
    }

    fn id(&self) -> CharacterId {
        match &*self.0.read().source.read() {
            VideoSource::Swf { streamdef, .. } => streamdef.id,
            VideoSource::NetStream { .. } => 0,
            VideoSource::Unconnected { .. } => 0,
        }
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        let read = self.0.read();

        Rectangle {
            x_min: Twips::ZERO,
            x_max: Twips::from_pixels_i32(read.size.0),
            y_min: Twips::ZERO,
            y_max: Twips::from_pixels_i32(read.size.1),
        }
    }

    fn render(&self, context: &mut RenderContext) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        context.transform_stack.push(self.base().transform());

        let read = self.0.read();

        let mut transform = context.transform_stack.transform();
        let bounds = self.self_bounds();

        // TODO: smoothing flag should be a video property
        let (smoothed_flag, num_frames, version, decoded_frame, codec) = match &*read.source.read()
        {
            VideoSource::Swf { streamdef, frames } => (
                streamdef.is_smoothed,
                Some(frames.len()),
                read.movie.version(),
                read.decoded_frame.clone().map(|df| df.1),
                Some(streamdef.codec),
            ),
            VideoSource::NetStream { stream, .. } => (
                false,
                None,
                read.movie.version(),
                stream.last_decoded_bitmap(),
                None,
            ),
            VideoSource::Unconnected { .. } => return context.transform_stack.pop(),
        };

        let smoothing = match (context.stage.quality(), version) {
            (StageQuality::Low, _) => false,
            (_, 8..) => smoothed_flag,
            (StageQuality::Medium, _) => false,
            (StageQuality::High, _) => num_frames == Some(1),
            (_, _) => true,
        };

        if let Some(bitmap) = decoded_frame {
            // The actual decoded frames might be different in size than the declared
            // bounds of the VideoStream tag, so a final scale adjustment has to be done.
            transform.matrix *= Matrix::scale(
                bounds.width().to_pixels() as f32 / bitmap.width as f32,
                bounds.height().to_pixels() as f32 / bitmap.height as f32,
            );

            context.commands.render_bitmap(
                bitmap.handle,
                transform,
                smoothing,
                PixelSnapping::Never,
            );
        } else if codec != Some(VideoCodec::None) {
            tracing::warn!("Video has no decoded frame to render.");
        }

        context.transform_stack.pop();
    }

    fn set_object2(&self, context: &mut UpdateContext<'gc>, to: Avm2Object<'gc>) {
        self.0.write(context.gc_context).object = Some(to.into());
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.read().movie.clone()
    }

    fn object(&self) -> Avm1Value<'gc> {
        self.0
            .read()
            .object
            .and_then(|o| o.as_avm1_object())
            .map(Avm1Value::from)
            .unwrap_or(Avm1Value::Undefined)
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .object
            .and_then(|o| o.as_avm2_object())
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Null)
    }
}
