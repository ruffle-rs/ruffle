//! Video player display object

use crate::avm1::{NativeObject as Avm1NativeObject, Object as Avm1Object};
use crate::avm2::StageObject as Avm2StageObject;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{Avm1TextFieldBinding, DisplayObjectBase, RenderOptions};
use crate::prelude::*;
use crate::streams::NetStream;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::utils::HasPrefixField;
use crate::vminterface::{AvmObject, Instantiator};
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, Gc, Mutation};
use ruffle_render::bitmap::{BitmapInfo, PixelSnapping};
use ruffle_render::commands::CommandHandler;
use ruffle_render::quality::StageQuality;
use ruffle_video::error::Error;
use ruffle_video::frame::EncodedFrame;
use ruffle_video::VideoStreamHandle;
use std::cell::{Cell, Ref, RefCell, RefMut};
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
pub struct Video<'gc>(Gc<'gc, VideoData<'gc>>);

impl fmt::Debug for Video<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Video")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct VideoData<'gc> {
    base: DisplayObjectBase<'gc>,

    avm1_text_field_bindings: RefLock<Vec<Avm1TextFieldBinding<'gc>>>,

    /// The source of the video data (e.g. an external file, a SWF bitstream)
    source: Lock<VideoSource<'gc>>,

    /// The decoder stream that this video source is associated to.
    stream: Cell<VideoStream>,

    /// AVM representation of this video player.
    object: Lock<Option<AvmObject<'gc>>>,

    /// List of frames which can be independently seeked to.
    ///
    /// Frames outside of this set must be decoded by playing each frame from
    /// the last keyframe in order. Any out-of-order seeking will be snapped to
    /// the prior keyframe. The first frame in the stream will always be
    /// treated as a keyframe regardless of it being flagged as one.
    keyframes: RefCell<BTreeSet<u32>>,

    /// The movie whose tagstream or code created the Video object.
    movie: Arc<SwfMovie>,
    /// The last decoded frame in the video stream.
    ///
    /// NOTE: This is only used for SWF-source video streams.
    decoded_frame: RefCell<Option<(u32, BitmapInfo)>>,

    /// The self bounds for this movie.
    size: Cell<(i32, i32)>,
}

/// An optionally-instantiated video stream.
#[derive(Clone, Copy, Debug)]
pub enum VideoStream {
    /// An uninstantiated video stream.
    ///
    /// The stream index parameter is what frame we should seek to once the
    /// stream is instantiated.
    Uninstantiated(u32),

    /// An instantiated video stream.
    Instantiated(VideoStreamHandle),
}

#[derive(Clone, Copy, Debug, Collect)]
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
    Swf(Gc<'gc, SwfVideoSource>),
    /// An attached NetStream.
    NetStream {
        /// The stream the video is downloaded from.
        stream: NetStream<'gc>,
    },
    Unconnected,
}

#[derive(Debug, Collect)]
#[collect(require_static)]
pub struct SwfVideoSource {
    /// The video stream definition.
    streamdef: DefineVideoStream,

    /// The locations of each embedded sub-bitstream for each video frame.
    ///
    /// Each frame consists of a start and end parameter which can be used
    /// to reconstruct a reference to the embedded bitstream.
    frames: RefCell<BTreeMap<u32, (usize, usize)>>,
}

impl<'gc> Video<'gc> {
    /// Construct a Video object that is tied to a SWF file's video stream.
    pub fn from_swf_tag(
        movie: Arc<SwfMovie>,
        streamdef: DefineVideoStream,
        mc: &Mutation<'gc>,
    ) -> Self {
        let size = (streamdef.width.into(), streamdef.height.into());
        let source = Lock::new(VideoSource::Swf(Gc::new(
            mc,
            SwfVideoSource {
                streamdef,
                frames: RefCell::new(BTreeMap::new()),
            },
        )));

        Video(Gc::new(
            mc,
            VideoData {
                base: Default::default(),
                avm1_text_field_bindings: RefLock::new(Vec::new()),
                source,
                stream: Cell::new(VideoStream::Uninstantiated(0)),
                object: Lock::new(None),
                keyframes: RefCell::new(BTreeSet::new()),
                movie,
                size: Cell::new(size),
                decoded_frame: RefCell::new(None),
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
        Video(Gc::new(
            mc,
            VideoData {
                base: Default::default(),
                avm1_text_field_bindings: RefLock::new(Vec::new()),
                source: Lock::new(VideoSource::Unconnected),
                stream: Cell::new(VideoStream::Uninstantiated(0)),
                object: Lock::new(object),
                keyframes: RefCell::new(BTreeSet::new()),
                movie,
                size: Cell::new((width, height)),
                decoded_frame: RefCell::new(None),
            },
        ))
    }

    fn set_object(&self, context: &mut UpdateContext<'gc>, to: AvmObject<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), VideoData, object).set(Some(to));
    }

    fn set_source(&self, context: &mut UpdateContext<'gc>, to: VideoSource<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), VideoData, source).set(to);
    }

    pub fn set_size(self, width: i32, height: i32) {
        self.0.size.set((width, height));
    }

    /// Convert this Video into a NetStream sourced video.
    ///
    /// Existing video state related to the old video stream will be dropped.
    pub fn attach_netstream(self, context: &mut UpdateContext<'gc>, stream: NetStream<'gc>) {
        self.set_source(context, VideoSource::NetStream { stream });
        self.0.stream.set(VideoStream::Uninstantiated(0));
        self.0.keyframes.replace(BTreeSet::new());
    }

    /// Preload frame data from an SWF.
    ///
    /// This function yields an error if this video player is not playing an
    /// embedded SWF video.
    pub fn preload_swf_frame(self, tag: VideoFrame) {
        let movie = self.0.movie.clone();

        match self.0.source.get() {
            VideoSource::Swf(swf_source) => {
                let subslice = SwfSlice::from(movie).to_unbounded_subslice(tag.data);
                let mut frames = swf_source.frames.borrow_mut();

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
        if let VideoStream::Uninstantiated(_) = self.0.stream.get() {
            self.0.stream.set(VideoStream::Uninstantiated(frame_id));
            return;
        };

        let num_frames = match self.0.source.get() {
            VideoSource::Swf(swf_source) => swf_source.streamdef.num_frames as usize,
            VideoSource::NetStream { .. } => return,
            VideoSource::Unconnected { .. } => return,
        };

        frame_id = if num_frames > 0 {
            frame_id % num_frames as u32
        } else {
            0
        };

        let last_frame = self.0.decoded_frame.borrow().as_ref().map(|(lf, _)| *lf);

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
            let prev_keyframe_id = self
                .0
                .keyframes
                .borrow()
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
        let stream = if let VideoStream::Instantiated(stream) = self.0.stream.get() {
            stream
        } else {
            tracing::error!("Attempted to seek uninstantiated video stream.");
            return;
        };

        let res = match self.0.source.get() {
            VideoSource::Swf(swf_source) => match swf_source.frames.borrow().get(&frame_id) {
                Some((slice_start, slice_end)) => {
                    let encframe = EncodedFrame {
                        codec: swf_source.streamdef.codec,
                        data: &self.0.movie.data()[*slice_start..*slice_end],
                        frame_id,
                    };
                    context
                        .video
                        .decode_video_stream_frame(stream, encframe, context.renderer)
                }
                None => {
                    if let Some((_old_id, old_frame)) = self.0.decoded_frame.borrow().clone() {
                        Ok(old_frame)
                    } else {
                        Err(Error::SeekingBeforeDecoding(frame_id))
                    }
                }
            },
            VideoSource::NetStream { .. } => return,
            VideoSource::Unconnected { .. } => return,
        };

        match res {
            Ok(bitmap) => {
                self.0.decoded_frame.replace(Some((frame_id, bitmap)));
                self.invalidate_cached_bitmap();
                *context.needs_render = true;
            }
            Err(e) => tracing::error!("Got error when seeking to video frame {}: {}", frame_id, e),
        }
    }
}

impl<'gc> TDisplayObject<'gc> for Video<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(gc_context, self.0.as_ref().clone())).into()
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        let movie = self.0.movie.clone();

        let (stream, keyframes) = match self.0.source.get() {
            VideoSource::Swf(swf_source) => {
                let streamdef = &swf_source.streamdef;
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
                    if let Err(err) = stream {
                        tracing::error!("Got error when post-instantiating video: {err}",);
                        return;
                    }

                    let stream = stream.unwrap();
                    let mut keyframes = BTreeSet::new();

                    for (frame_id, (frame_start, frame_end)) in swf_source.frames.borrow().iter() {
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

        let starting_seek = if let VideoStream::Uninstantiated(seek_to) = self.0.stream.get() {
            seek_to
        } else {
            tracing::warn!("Reinstantiating already-instantiated video stream!");
            0
        };

        if let Some(stream) = stream {
            self.0.stream.set(VideoStream::Instantiated(stream));
        }
        self.0.keyframes.replace(keyframes);

        if self.0.object.get().is_none() && !movie.is_action_script_3() {
            let object = Avm1Object::new_with_native(
                &context.strings,
                Some(context.avm1.prototypes(self.swf_version()).video),
                Avm1NativeObject::Video(self),
            );
            self.set_object(context, object.into());
        }

        self.seek(context, starting_seek);
    }

    fn construct_frame(self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() && self.object2().is_none() {
            let video_constr = context.avm2.classes().video;
            let object =
                Avm2StageObject::for_display_object(context.gc(), self.into(), video_constr);
            // We don't need to call the initializer method, as AVM2 can't link
            // a custom class to a Video, and the initializer method for Video
            // itself only sets the size of the Video- the Video already has the
            // correct size at this point.

            self.set_object2(context, object);

            self.on_construction_complete(context);
        }
    }

    fn on_ratio_changed(self, context: &mut UpdateContext<'gc>, new_ratio: u16) {
        self.seek(context, new_ratio.into());
    }

    fn id(self) -> CharacterId {
        match self.0.source.get() {
            VideoSource::Swf(swf_source) => swf_source.streamdef.id,
            VideoSource::NetStream { .. } => 0,
            VideoSource::Unconnected { .. } => 0,
        }
    }

    fn self_bounds(self) -> Rectangle<Twips> {
        let (size_x, size_y) = self.0.size.get();
        Rectangle {
            x_min: Twips::ZERO,
            x_max: Twips::from_pixels_i32(size_x),
            y_min: Twips::ZERO,
            y_max: Twips::from_pixels_i32(size_y),
        }
    }

    fn render_with_options(self, context: &mut RenderContext<'_, 'gc>, options: RenderOptions) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        if options.apply_transform {
            let transform = self.base().transform(options.apply_matrix);
            context.transform_stack.push(&transform);
        }

        let mut transform = context.transform_stack.transform();
        let bounds = self.self_bounds();

        // TODO: smoothing flag should be a video property
        let (smoothed_flag, num_frames, version, decoded_frame, codec) = match self.0.source.get() {
            VideoSource::Swf(swf_source) => (
                swf_source.streamdef.is_smoothed,
                Some(swf_source.frames.borrow().len()),
                self.0.movie.version(),
                self.0.decoded_frame.borrow().clone().map(|df| df.1),
                Some(swf_source.streamdef.codec),
            ),
            VideoSource::NetStream { stream, .. } => (
                false,
                None,
                self.0.movie.version(),
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

        if options.apply_transform {
            context.transform_stack.pop();
        }
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie.clone()
    }

    fn object1(self) -> Option<Avm1Object<'gc>> {
        self.0.object.get().and_then(|o| o.as_avm1_object())
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.object.get().and_then(|o| o.as_avm2_object())
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        self.set_object(context, to.into());
    }

    fn avm1_text_field_bindings(&self) -> Option<Ref<'_, [Avm1TextFieldBinding<'gc>]>> {
        self.0
            .object
            .get()
            .and_then(|o| o.as_avm1_object())
            .map(|_| Ref::map(self.0.avm1_text_field_bindings.borrow(), |b| &b[..]))
    }

    fn avm1_text_field_bindings_mut(
        &self,
        mc: &Mutation<'gc>,
    ) -> Option<RefMut<'_, Vec<Avm1TextFieldBinding<'gc>>>> {
        self.0
            .object
            .get()
            .and_then(|o| o.as_avm1_object())
            .map(|_| {
                unlock!(Gc::write(mc, self.0), VideoData, avm1_text_field_bindings).borrow_mut()
            })
    }
}
