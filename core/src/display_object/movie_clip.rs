//! `MovieClip` display object and support code.
use crate::avm1::{Object as Avm1Object, StageObject, TObject as Avm1TObject, Value as Avm1Value};
use crate::avm2::object::LoaderInfoObject;
use crate::avm2::object::LoaderStream;
use crate::avm2::script::Script;
use crate::avm2::Activation as Avm2Activation;
use crate::avm2::{
    Avm2, ClassObject as Avm2ClassObject, Error as Avm2Error, Object as Avm2Object,
    QName as Avm2QName, StageObject as Avm2StageObject, TObject as Avm2TObject, Value as Avm2Value,
};
use crate::backend::audio::{AudioManager, SoundHandle, SoundInstanceHandle};
use crate::backend::navigator::Request;
use crate::backend::ui::MouseCursor;
use crate::frame_lifecycle::run_inner_goto_frame;
use bitflags::bitflags;

use crate::avm1::Avm1;
use crate::avm1::{Activation as Avm1Activation, ActivationIdentifier};
use crate::binary_data::BinaryData;
use crate::character::{Character, CompressedBitmap};
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::container::{dispatch_removed_event, ChildContainer};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{
    Avm1Button, Avm2Button, DisplayObjectBase, DisplayObjectPtr, EditText, Graphic, MorphShape,
    Text, Video,
};
use crate::drawing::Drawing;
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult};
use crate::font::{Font, FontType};
use crate::limits::ExecutionLimit;
use crate::loader::{self, ContentType};
use crate::loader::{LoadManager, Loader};
use crate::prelude::*;
use crate::streams::NetStream;
use crate::string::{AvmString, SwfStrExt as _, WStr, WString};
use crate::tag_utils::{self, ControlFlow, DecodeResult, Error, SwfMovie, SwfSlice, SwfStream};
use crate::vminterface::{AvmObject, Instantiator};
use core::fmt;
use gc_arena::{Collect, Gc, GcCell, GcWeakCell, Mutation};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::cmp::max;
use std::collections::HashMap;
use std::sync::Arc;
use swf::extensions::ReadSwfExt;
use swf::{ClipEventFlag, DefineBitsLossless, FrameLabelData, TagCode, UTF_8};

use super::interactive::Avm2MousePick;
use super::BitmapClass;

type FrameNumber = u16;

/// Indication of what frame `run_frame` should jump to next.
#[derive(PartialEq, Eq)]
enum NextFrame {
    /// Construct and run the next frame in the clip.
    Next,

    /// Jump to the first frame in the clip.
    First,

    /// Do not construct or run any frames.
    Same,
}

/// A movie clip is a display object with its own timeline that runs independently of the root timeline.
/// The SWF19 spec calls this "Sprite" and the SWF tag defines it is "DefineSprite".
/// However, in AVM2, Sprite is a separate display object, and MovieClip is a subclass of Sprite.
///
/// (SWF19 pp. 201-203)
///
/// # MovieClip States
/// A MovieClip can be in different states.
/// The state of a MovieClip consists of the values of all properties and the results of some getter
/// functions of the MovieClip.
/// Most of these states are inaccessible in AVM2.
///
/// The states a MovieClip can be in are the following:
///
/// ## Default State
/// This is the default state a MovieClip is in after it's created (in AVM1 with createEmptyMovieClip).
///
/// ## Initial Loading State
/// This state is entered when FP / Ruffle try to load the MovieClip. As soon as FP / Ruffle either
/// load the first frame of the SWF or realise the movie can't be loaded, a different state is
/// entered.
///
/// Therefore, if FP / Ruffle are too fast to determine whether the file exists or not, the state
/// can directly change after one frame from the default state to a different state.
///
/// The initial loading state is different, depending on whether the SWF file which is loading is
/// an online file or a local file.
///
/// ## Error State
/// This state is entered if no file could be loaded or if the loaded content is no valid supported
/// content.
///
/// ## Image State
/// This state is entered if an image has been loaded.
///
/// ## Success State
/// This state is entered if the first frame of a valid SWF file has been loaded.
///
/// ## Unloaded State
/// This state is entered on the next frame after the movie has been unloaded.
///
/// ## States in AVM2
/// In AVM2, only the success state is accessible to the ActionScript code. The Ruffle MovieClip
/// can still be in the default state, initial loading state and error state, however it is only
/// passed to the code (via the ActionScript Loader) after it has reached the success state. If
/// an image is loaded in AVM2, the ActionScript code doesn't get any MovieClip object, even if
/// the MovieClip exists in Ruffle and is in the image state.
///
/// The unloaded state can only be reached in AVM1 through the unloadMovie function.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct MovieClip<'gc>(GcCell<'gc, MovieClipData<'gc>>);

impl fmt::Debug for MovieClip<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MovieClip")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct MovieClipWeak<'gc>(GcWeakCell<'gc, MovieClipData<'gc>>);

impl<'gc> MovieClipWeak<'gc> {
    pub fn upgrade(self, mc: &Mutation<'gc>) -> Option<MovieClip<'gc>> {
        self.0.upgrade(mc).map(MovieClip)
    }

    pub fn as_ptr(self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct MovieClipData<'gc> {
    base: InteractiveObjectBase<'gc>,
    static_data: Gc<'gc, MovieClipStatic<'gc>>,
    tag_stream_pos: u64,
    current_frame: FrameNumber,
    #[collect(require_static)]
    audio_stream: Option<SoundInstanceHandle>,
    container: ChildContainer<'gc>,
    object: Option<AvmObject<'gc>>,
    #[collect(require_static)]
    clip_event_handlers: Vec<ClipEventHandler>,
    #[collect(require_static)]
    clip_event_flags: ClipEventFlag,
    frame_scripts: Vec<Option<Avm2Object<'gc>>>,
    #[collect(require_static)]
    flags: MovieClipFlags,
    #[collect(require_static)]
    drawing: Drawing,
    avm2_enabled: bool,

    /// Show a hand cursor when the clip is in button mode.
    avm2_use_hand_cursor: bool,

    /// A DisplayObject (doesn't need to be visible) to use for hit tests instead of this clip.
    hit_area: Option<DisplayObject<'gc>>,

    /// Force enable button mode, which causes all mouse-related events to
    /// trigger on this clip rather than any input-eligible children.
    button_mode: bool,
    last_queued_script_frame: Option<FrameNumber>,
    queued_script_frame: Option<FrameNumber>,
    queued_goto_frame: Option<FrameNumber>,
    drop_target: Option<DisplayObject<'gc>>,

    /// The tag stream start and stop positions for each frame in the clip.
    #[cfg(feature = "timeline_debug")]
    tag_frame_boundaries: HashMap<FrameNumber, (u64, u64)>,

    /// List of tags queued up for the current frame.
    #[collect(require_static)]
    queued_tags: HashMap<Depth, QueuedTagList>,

    /// Attached audio (AVM1)
    attached_audio: Option<NetStream<'gc>>,

    // If this movie was loaded from ImportAssets(2), this will be the parent movie.
    importer_movie: Option<Arc<SwfMovie>>,
}

impl<'gc> MovieClip<'gc> {
    pub fn new(movie: Arc<SwfMovie>, gc_context: &Mutation<'gc>) -> Self {
        MovieClip(GcCell::new(
            gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::new(
                    gc_context,
                    MovieClipStatic::empty(movie.clone(), gc_context),
                ),
                tag_stream_pos: 0,
                current_frame: 0,
                audio_stream: None,
                container: ChildContainer::new(movie),
                object: None,
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::empty(),
                drawing: Drawing::new(),
                avm2_enabled: true,
                avm2_use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,
                hit_area: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
                attached_audio: None,
                importer_movie: None,
            },
        ))
    }

    pub fn new_with_avm2(
        movie: Arc<SwfMovie>,
        this: Avm2Object<'gc>,
        class: Avm2ClassObject<'gc>,
        gc_context: &Mutation<'gc>,
    ) -> Self {
        let clip = MovieClip(GcCell::new(
            gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::new(
                    gc_context,
                    MovieClipStatic::empty(movie.clone(), gc_context),
                ),
                tag_stream_pos: 0,
                current_frame: 0,
                audio_stream: None,
                container: ChildContainer::new(movie),
                object: Some(this.into()),
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::empty(),
                drawing: Drawing::new(),
                avm2_enabled: true,
                avm2_use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,
                hit_area: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
                attached_audio: None,
                importer_movie: None,
            },
        ));
        clip.set_avm2_class(gc_context, Some(class));
        clip
    }

    /// Constructs a non-root movie
    pub fn new_with_data(
        gc_context: &Mutation<'gc>,
        id: CharacterId,
        swf: SwfSlice,
        num_frames: u16,
    ) -> Self {
        MovieClip(GcCell::new(
            gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::new(
                    gc_context,
                    MovieClipStatic::with_data(id, swf.clone(), num_frames, None, gc_context),
                ),
                tag_stream_pos: 0,
                current_frame: 0,
                audio_stream: None,
                container: ChildContainer::new(swf.movie),
                object: None,
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::PLAYING,
                drawing: Drawing::new(),
                avm2_enabled: true,
                avm2_use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,
                hit_area: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
                attached_audio: None,
                importer_movie: None,
            },
        ))
    }

    pub fn downgrade(self) -> MovieClipWeak<'gc> {
        MovieClipWeak(GcCell::downgrade(self.0))
    }

    pub fn new_import_assets(
        context: &mut UpdateContext<'gc>,
        movie: Arc<SwfMovie>,
        parent: Arc<SwfMovie>,
    ) -> Self {
        let num_frames = movie.num_frames();

        let loader_info = None;

        let mc = MovieClip(GcCell::new(
            context.gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::new(
                    context.gc_context,
                    MovieClipStatic::with_data(
                        0,
                        movie.clone().into(),
                        num_frames,
                        loader_info,
                        context.gc_context,
                    ),
                ),
                tag_stream_pos: 0,
                current_frame: 0,
                audio_stream: None,
                container: ChildContainer::new(movie.clone()),
                object: None,
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::PLAYING,
                drawing: Drawing::new(),
                avm2_enabled: true,
                avm2_use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,
                hit_area: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
                attached_audio: None,
                importer_movie: Some(parent.clone()),
            },
        ));

        mc
    }

    /// Construct a movie clip that represents the root movie
    /// for the entire `Player`.
    pub fn player_root_movie(
        activation: &mut Avm2Activation<'_, 'gc>,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let num_frames = movie.num_frames();

        let loader_info = if movie.is_action_script_3() {
            // The root movie doesn't have a `Loader`
            // We will replace this with a `LoaderStream::Swf` later in this function
            let loader_info =
                LoaderInfoObject::not_yet_loaded(activation, movie.clone(), None, None, false)
                    .expect("Failed to construct LoaderInfoObject");
            let loader_info_obj = loader_info.as_loader_info_object().unwrap();
            loader_info_obj.set_expose_content();
            loader_info_obj.set_content_type(ContentType::Swf);
            Some(loader_info)
        } else {
            None
        };

        let mc = MovieClip(GcCell::new(
            activation.context.gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::new(
                    activation.context.gc_context,
                    MovieClipStatic::with_data(
                        0,
                        movie.clone().into(),
                        num_frames,
                        loader_info,
                        activation.context.gc_context,
                    ),
                ),
                tag_stream_pos: 0,
                current_frame: 0,
                audio_stream: None,
                container: ChildContainer::new(movie.clone()),
                object: None,
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::PLAYING,
                drawing: Drawing::new(),
                avm2_enabled: true,
                avm2_use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,
                hit_area: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
                attached_audio: None,
                importer_movie: None,
            },
        ));

        if movie.is_action_script_3() {
            let mc_data = mc.0.read();
            let loader_info = mc_data
                .static_data
                .loader_info
                .as_ref()
                .unwrap()
                .as_loader_info_object()
                .unwrap();
            loader_info.set_loader_stream(
                LoaderStream::Swf(movie, mc.into()),
                activation.context.gc_context,
            );
        }
        mc.set_is_root(activation.context.gc_context, true);
        mc
    }

    /// Replace the current MovieClipData with a completely new SwfMovie.
    ///
    /// If no movie is provided, then the movie clip will be replaced with an
    /// empty movie of the same SWF version.
    ///
    /// Playback will start at position zero, any existing streamed audio will
    /// be terminated, and so on. Children and AVM data will NOT be kept across
    /// the load boundary.
    pub fn replace_with_movie(
        self,
        context: &mut UpdateContext<'gc>,
        movie: Option<Arc<SwfMovie>>,
        is_root: bool,
        loader_info: Option<LoaderInfoObject<'gc>>,
    ) {
        let mut mc = self.0.write(context.gc_context);
        let movie = movie.unwrap_or_else(|| Arc::new(SwfMovie::empty(mc.movie().version())));
        let total_frames = movie.num_frames();
        assert_eq!(
            mc.static_data.loader_info, None,
            "Called replace_movie on a clip with LoaderInfo set"
        );

        mc.base.base.reset_for_movie_load();
        mc.static_data = Gc::new(
            context.gc_context,
            MovieClipStatic::with_data(
                0,
                movie.clone().into(),
                total_frames,
                loader_info.map(|l| l.into()),
                context.gc_context,
            ),
        );
        mc.tag_stream_pos = 0;
        mc.flags = MovieClipFlags::PLAYING;
        mc.base.base.set_is_root(is_root);
        mc.current_frame = 0;
        mc.audio_stream = None;
        mc.container = ChildContainer::new(movie);
        drop(mc);
    }

    pub fn set_initialized(self, gc_context: &Mutation<'gc>) {
        self.0.write(gc_context).set_initialized(true);
    }

    /// Tries to fire events from our `LoaderInfo` object if we're ready - returns
    /// `true` if both `init` and `complete` have been fired
    pub fn try_fire_loaderinfo_events(self, context: &mut UpdateContext<'gc>) -> bool {
        if self.0.read().initialized() {
            if let Some(loader_info) = self
                .loader_info()
                .as_ref()
                .and_then(|o| o.as_loader_info_object())
            {
                return loader_info.fire_init_and_complete_events(context, 0, false);
            }
        }
        false
    }

    /// Preload a chunk of the movie.
    ///
    /// A "chunk" is an implementor-chosen number of tags that are parsed
    /// before this function returns. This function will only parse up to a
    /// certain number of tags, and then return. If this function returns false,
    /// then the preload didn't complete and further preloads should occur
    /// until this returns true.
    ///
    /// The chunked preload assumes that preloading is happening within the
    /// context of an event loop. As such, multiple chunks should be processed
    /// in between yielding to the underlying event loop, either through
    /// `await`, returning to the loop directly, or some other mechanism.
    pub fn preload(
        self,
        context: &mut UpdateContext<'gc>,
        chunk_limit: &mut ExecutionLimit,
    ) -> bool {
        {
            let read = self.0.read();
            if read.static_data.preload_progress.read().next_preload_chunk
                >= read.static_data.swf.len() as u64
            {
                return true;
            }
        }

        // TODO: Re-creating static data because preload step occurs after construction.
        // Should be able to hoist this up somewhere, or use MaybeUninit.
        let mut static_data = (*self.0.read().static_data).clone();
        let data = self.0.read().static_data.swf.clone();
        let (mut cur_frame, mut start_pos, next_preload_chunk, preload_symbol) = {
            let read = static_data.preload_progress.read();
            (
                read.cur_preload_frame,
                read.last_frame_start_pos,
                read.next_preload_chunk,
                read.cur_preload_symbol,
            )
        };
        let mut reader = data.read_from(next_preload_chunk);

        if let Some(cur_preload_symbol) = preload_symbol {
            match context
                .library
                .library_for_movie_mut(self.movie())
                .character_by_id(cur_preload_symbol)
            {
                Some(Character::MovieClip(mc)) => {
                    let sub_preload_done = mc.preload(context, chunk_limit);
                    if sub_preload_done {
                        static_data
                            .preload_progress
                            .write(context.gc_context)
                            .cur_preload_symbol = None;
                    }
                }
                Some(unk) => {
                    tracing::error!(
                        "Symbol {} changed to unexpected type {:?}",
                        cur_preload_symbol,
                        unk
                    );

                    static_data
                        .preload_progress
                        .write(context.gc_context)
                        .cur_preload_symbol = None;
                }
                None => {
                    tracing::error!(
                        "Symbol {} disappeared during preloading",
                        cur_preload_symbol
                    );

                    static_data
                        .preload_progress
                        .write(context.gc_context)
                        .cur_preload_symbol = None;
                }
            }
        }

        let mut end_tag_found = false;

        let sub_preload_done = static_data
            .preload_progress
            .read()
            .cur_preload_symbol
            .is_none();
        let tag_callback = |reader: &mut SwfStream<'_>, tag_code, tag_len| {
            match tag_code {
                TagCode::CsmTextSettings => self
                    .0
                    .write(context.gc_context)
                    .csm_text_settings(context, reader),
                TagCode::DefineBits => self
                    .0
                    .write(context.gc_context)
                    .define_bits(context, reader),
                TagCode::DefineBitsJpeg2 => self
                    .0
                    .write(context.gc_context)
                    .define_bits_jpeg_2(context, reader),
                TagCode::DefineBitsJpeg3 => self
                    .0
                    .write(context.gc_context)
                    .define_bits_jpeg_3_or_4(context, reader, 3),
                TagCode::DefineBitsJpeg4 => self
                    .0
                    .write(context.gc_context)
                    .define_bits_jpeg_3_or_4(context, reader, 4),
                TagCode::DefineBitsLossless => self
                    .0
                    .write(context.gc_context)
                    .define_bits_lossless(context, reader, 1),
                TagCode::DefineBitsLossless2 => self
                    .0
                    .write(context.gc_context)
                    .define_bits_lossless(context, reader, 2),
                TagCode::DefineButton => self
                    .0
                    .write(context.gc_context)
                    .define_button_1(context, reader),
                TagCode::DefineButton2 => self
                    .0
                    .write(context.gc_context)
                    .define_button_2(context, reader),
                TagCode::DefineButtonCxform => self
                    .0
                    .write(context.gc_context)
                    .define_button_cxform(context, reader),
                TagCode::DefineButtonSound => self
                    .0
                    .write(context.gc_context)
                    .define_button_sound(context, reader),
                TagCode::DefineEditText => self
                    .0
                    .write(context.gc_context)
                    .define_edit_text(context, reader),
                TagCode::DefineFont => self
                    .0
                    .write(context.gc_context)
                    .define_font_1(context, reader),
                TagCode::DefineFont2 => self
                    .0
                    .write(context.gc_context)
                    .define_font_2(context, reader),
                TagCode::DefineFont3 => self
                    .0
                    .write(context.gc_context)
                    .define_font_3(context, reader),
                TagCode::DefineFont4 => self
                    .0
                    .write(context.gc_context)
                    .define_font_4(context, reader),
                TagCode::DefineMorphShape => self
                    .0
                    .write(context.gc_context)
                    .define_morph_shape(context, reader, 1),
                TagCode::DefineMorphShape2 => self
                    .0
                    .write(context.gc_context)
                    .define_morph_shape(context, reader, 2),
                TagCode::DefineScalingGrid => self
                    .0
                    .write(context.gc_context)
                    .define_scaling_grid(context, reader),
                TagCode::DefineShape => self
                    .0
                    .write(context.gc_context)
                    .define_shape(context, reader, 1),
                TagCode::DefineShape2 => self
                    .0
                    .write(context.gc_context)
                    .define_shape(context, reader, 2),
                TagCode::DefineShape3 => self
                    .0
                    .write(context.gc_context)
                    .define_shape(context, reader, 3),
                TagCode::DefineShape4 => self
                    .0
                    .write(context.gc_context)
                    .define_shape(context, reader, 4),
                TagCode::DefineSound => self
                    .0
                    .write(context.gc_context)
                    .define_sound(context, reader),
                TagCode::DefineVideoStream => self
                    .0
                    .write(context.gc_context)
                    .define_video_stream(context, reader),
                TagCode::DefineSprite => {
                    return self.0.write(context.gc_context).define_sprite(
                        context,
                        reader,
                        tag_len,
                        chunk_limit,
                    )
                }
                TagCode::DefineText => self
                    .0
                    .write(context.gc_context)
                    .define_text(context, reader, 1),
                TagCode::DefineText2 => self
                    .0
                    .write(context.gc_context)
                    .define_text(context, reader, 2),
                TagCode::DoInitAction => self.do_init_action(context, reader, tag_len),
                TagCode::DefineSceneAndFrameLabelData => {
                    self.scene_and_frame_labels(reader, &mut static_data)
                }
                TagCode::ExportAssets => self
                    .0
                    .write(context.gc_context)
                    .export_assets(context, reader),
                TagCode::FrameLabel => self.0.write(context.gc_context).frame_label(
                    reader,
                    cur_frame,
                    &mut static_data,
                    context,
                ),
                TagCode::JpegTables => self
                    .0
                    .write(context.gc_context)
                    .jpeg_tables(context, reader),
                TagCode::ShowFrame => self.0.write(context.gc_context).show_frame(
                    reader,
                    tag_len,
                    &mut cur_frame,
                    &mut start_pos,
                ),
                TagCode::ScriptLimits => self
                    .0
                    .write(context.gc_context)
                    .script_limits(reader, context.avm1),
                TagCode::SoundStreamHead => {
                    self.0
                        .write(context.gc_context)
                        .sound_stream_head(reader, &mut static_data, 1)
                }
                TagCode::SoundStreamHead2 => {
                    self.0
                        .write(context.gc_context)
                        .sound_stream_head(reader, &mut static_data, 2)
                }
                TagCode::VideoFrame => self
                    .0
                    .write(context.gc_context)
                    .preload_video_frame(context, reader),
                TagCode::DefineBinaryData => self
                    .0
                    .write(context.gc_context)
                    .define_binary_data(context, reader),
                TagCode::ImportAssets => {
                    self.0
                        .write(context.gc_context)
                        .import_assets(context, reader, chunk_limit)
                }
                TagCode::ImportAssets2 => {
                    self.0
                        .write(context.gc_context)
                        .import_assets_2(context, reader, chunk_limit)
                }
                TagCode::DoAbc | TagCode::DoAbc2 => self.preload_bytecode_tag(
                    tag_code,
                    reader,
                    context,
                    cur_frame - 1,
                    &mut static_data,
                ),
                TagCode::SymbolClass => {
                    self.preload_symbol_class(reader, context, cur_frame - 1, &mut static_data)
                }
                TagCode::End => {
                    end_tag_found = true;
                    return Ok(ControlFlow::Exit);
                }
                _ => Ok(()),
            }?;

            // Each preloaded byte is treated as an operation.
            if chunk_limit.did_ops_breach_limit(context, tag_len) {
                return Ok(ControlFlow::Exit);
            }

            Ok(ControlFlow::Continue)
        };

        let result = if sub_preload_done {
            tag_utils::decode_tags(&mut reader, tag_callback)
        } else {
            Ok(true)
        };
        let is_finished = end_tag_found || result.is_err() || !result.unwrap_or_default();

        self.0
            .write(context.gc_context)
            .import_exports_of_importer(context);

        // These variables will be persisted to be picked back up in the next
        // chunk.
        {
            let mut write = static_data.preload_progress.write(context.gc_context);

            write.next_preload_chunk = if is_finished {
                // Flag the movie as fully preloaded when we hit the end of the
                // tag stream.
                u64::MAX
            } else {
                (reader.get_ref().as_ptr() as u64).saturating_sub(data.data().as_ptr() as u64)
            };
            write.cur_preload_frame = if is_finished {
                // Flag the movie as fully preloaded when we hit the end of the
                // tag stream.
                static_data.total_frames + 1
            } else {
                cur_frame
            };
            write.last_frame_start_pos = start_pos;
        }

        if is_finished {
            // End-of-clip should be treated as ShowFrame
            self.0
                .write(context.gc_context)
                .show_frame(&mut reader, 0, &mut cur_frame, &mut start_pos)
                .unwrap();
        }

        self.0.write(context.gc_context).static_data = Gc::new(context.gc_context, static_data);

        is_finished
    }

    #[inline]
    fn do_init_action(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'_>,
        tag_len: usize,
    ) -> Result<(), Error> {
        if self.movie().is_action_script_3() {
            tracing::warn!("DoInitAction tag in AVM2 movie");
            return Ok(());
        }

        let start = reader.as_slice();
        // Queue the init actions.
        // TODO: Init actions are supposed to be executed once, and it gives a
        // sprite ID... how does that work?
        let _sprite_id = reader.read_u16()?;
        let num_read = reader.pos(start);

        let slice = self
            .0
            .read()
            .static_data
            .swf
            .resize_to_reader(reader, tag_len - num_read);

        if !slice.is_empty() {
            Avm1::run_stack_frame_for_init_action(self.into(), slice, context);
        }

        Ok(())
    }

    #[inline]
    fn do_abc(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'_>,
    ) -> Result<Option<Script<'gc>>, Error> {
        if !context.swf.is_action_script_3() {
            tracing::warn!("DoABC tag with non-AVM2 root");
            return Ok(None);
        }

        let data = reader.read_slice_to_end();
        if !data.is_empty() {
            let movie = self.movie();
            let domain = context.library.library_for_movie_mut(movie).avm2_domain();

            // DoAbc tag seems to be equivalent to a DoAbc2 with Lazy flag set
            match Avm2::do_abc(
                context,
                data,
                None,
                swf::DoAbc2Flag::LAZY_INITIALIZE,
                domain,
                self.movie(),
            ) {
                Ok(res) => return Ok(res),
                Err(e) => {
                    tracing::warn!("Error loading ABC file: {e:?}");
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    #[inline]
    fn do_abc_2(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'_>,
    ) -> Result<Option<Script<'gc>>, Error> {
        if !context.swf.is_action_script_3() {
            tracing::warn!("DoABC2 tag with non-AVM2 root");
            return Ok(None);
        }

        let do_abc = reader.read_do_abc_2()?;
        if !do_abc.data.is_empty() {
            let movie = self.movie();
            let domain = context.library.library_for_movie_mut(movie).avm2_domain();
            let name = AvmString::new(context.gc_context, do_abc.name.decode(reader.encoding()));

            match Avm2::do_abc(
                context,
                do_abc.data,
                Some(name),
                do_abc.flags,
                domain,
                self.movie(),
            ) {
                Ok(res) => return Ok(res),
                Err(e) => {
                    tracing::warn!("Error loading ABC file: {e:?}");
                }
            }
        }

        Ok(None)
    }

    #[inline]
    fn scene_and_frame_labels(
        self,
        reader: &mut SwfStream<'_>,
        static_data: &mut MovieClipStatic<'gc>,
    ) -> Result<(), Error> {
        let mut sfl_data = reader.read_define_scene_and_frame_label_data()?;
        sfl_data
            .scenes
            .sort_unstable_by(|s1, s2| s1.frame_num.cmp(&s2.frame_num));

        for (i, FrameLabelData { frame_num, label }) in sfl_data.scenes.iter().enumerate() {
            let start = *frame_num as u16 + 1;
            let end = sfl_data
                .scenes
                .get(i + 1)
                .map(|fld| fld.frame_num as u16 + 1)
                .unwrap_or_else(|| static_data.total_frames + 1);

            let scene = Scene {
                name: label.decode(reader.encoding()).into_owned(),
                start,
                length: end - start,
            };
            static_data.scene_labels.push(scene.clone());
            if let std::collections::hash_map::Entry::Vacant(v) =
                static_data.scene_labels_map.entry(scene.name.clone())
            {
                v.insert(scene);
            } else {
                tracing::warn!("Movie clip {}: Duplicated scene label", self.id());
            }
        }

        for FrameLabelData { frame_num, label } in sfl_data.frame_labels {
            let label = label.decode(reader.encoding()).into_owned();
            static_data
                .frame_labels
                .push((frame_num as u16 + 1, label.clone()));
            if let std::collections::hash_map::Entry::Vacant(v) =
                static_data.frame_labels_map.entry(label)
            {
                v.insert(frame_num as u16 + 1);
            } else {
                tracing::warn!("Movie clip {}: Duplicated frame label", self.id());
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn playing(self) -> bool {
        self.0.read().playing()
    }

    pub fn programmatically_played(self) -> bool {
        self.0.read().programmatically_played()
    }

    pub fn drop_target(self) -> Option<DisplayObject<'gc>> {
        self.0.read().drop_target
    }

    pub fn set_drop_target(
        self,
        gc_context: &Mutation<'gc>,
        drop_target: Option<DisplayObject<'gc>>,
    ) {
        self.0.write(gc_context).drop_target = drop_target;
    }

    pub fn set_programmatically_played(self, mc: &Mutation<'gc>) {
        self.0.write(mc).set_programmatically_played()
    }

    pub fn next_frame(self, context: &mut UpdateContext<'gc>) {
        if self.current_frame() < self.total_frames() {
            self.goto_frame(context, self.current_frame() + 1, true);
        }
    }

    pub fn play(self, context: &mut UpdateContext<'gc>) {
        self.0.write(context.gc_context).play()
    }

    pub fn prev_frame(self, context: &mut UpdateContext<'gc>) {
        if self.current_frame() > 1 {
            self.goto_frame(context, self.current_frame() - 1, true);
        }
    }

    pub fn initialized(self) -> bool {
        self.0.read().initialized()
    }

    pub fn stop(self, context: &mut UpdateContext<'gc>) {
        self.0.write(context.gc_context).stop(context)
    }

    /// Does this clip have a unload handler
    pub fn has_unload_handler(&self) -> bool {
        self.0
            .read()
            .clip_event_handlers
            .iter()
            .any(|handler| handler.events.contains(ClipEventFlag::UNLOAD))
    }

    /// Queues up a goto to the specified frame.
    /// `frame` should be 1-based.
    ///
    /// This is treated as an 'explicit' goto: frame scripts and other frame
    /// lifecycle events will be retriggered.
    pub fn goto_frame(self, context: &mut UpdateContext<'gc>, frame: FrameNumber, stop: bool) {
        // Stop first, in case we need to kill and restart the stream sound.
        if stop {
            self.stop(context);
        } else {
            self.play(context);
        }

        // Clamp frame number in bounds.
        let frame = frame.max(1);

        // In AS3, no-op gotos have side effects that are visible to user code.
        // Hence, we have to run them anyway.
        if frame != self.current_frame() {
            if self
                .0
                .read()
                .flags
                .contains(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT)
            {
                // AVM2 does not allow a clip to see while it is executing a frame script.
                // The goto is instead queued and run once the frame script is completed.
                self.0.write(context.gc_context).queued_goto_frame = Some(frame);
            } else {
                self.run_goto(context, frame, false);
            }
        } else if self.movie().is_action_script_3() {
            // Pretend we actually did a goto, but don't do anything.
            run_inner_goto_frame(context, &[], self);
        }
    }

    pub fn current_frame(self) -> FrameNumber {
        self.0.read().current_frame()
    }

    /// Return the current scene.
    pub fn current_scene(self) -> Option<Scene> {
        let current_frame = self.0.read().current_frame();

        self.filter_scenes(
            |best,
             Scene {
                 name: _,
                 start,
                 length: _,
             }| {
                *start <= current_frame
                    && best
                        .map(
                            |Scene {
                                 name: _,
                                 start: best_start,
                                 length: _,
                             }| start >= best_start,
                        )
                        .unwrap_or(true)
            },
        )
    }

    /// Return the previous scene.
    pub fn previous_scene(self) -> Option<Scene> {
        let current_frame = self
            .current_scene()
            .map(
                |Scene {
                     name: _,
                     start,
                     length: _,
                 }| start,
            )
            .unwrap_or_else(|| self.current_frame());

        self.filter_scenes(
            |best,
             Scene {
                 name: _,
                 start,
                 length: _,
             }| {
                *start < current_frame
                    && best
                        .map(
                            |Scene {
                                 name: _,
                                 start: best_start,
                                 length: _,
                             }| start >= best_start,
                        )
                        .unwrap_or(true)
            },
        )
        .or(self.current_scene())
    }

    /// Return the next scene.
    pub fn next_scene(self) -> Option<Scene> {
        let current_frame = self.0.read().current_frame();

        self.filter_scenes(
            |best,
             Scene {
                 name: _,
                 start,
                 length: _,
             }| {
                *start > current_frame
                    && best
                        .map(
                            |Scene {
                                 name: _,
                                 start: best_start,
                                 length: _,
                             }| start <= best_start,
                        )
                        .unwrap_or(true)
            },
        )
        .or(self.current_scene())
    }

    /// Return all scenes in the movie.
    ///
    /// Scenes will be sorted in playback order.
    pub fn scenes(self) -> Vec<Scene> {
        let mut out: Vec<_> = self.0.read().static_data.scene_labels.clone();
        out.sort_unstable_by(|Scene { start: a, .. }, Scene { start: b, .. }| a.cmp(b));
        out
    }

    /// Scan through the list of scenes and yield the best one, if available,
    /// according to a given criterion function.
    fn filter_scenes<F>(self, mut cond: F) -> Option<Scene>
    where
        F: FnMut(Option<&Scene>, &Scene) -> bool,
    {
        let read = self.0.read();
        let mut best: Option<&Scene> = None;

        for scene in read.static_data.scene_labels.iter() {
            if cond(best, scene) {
                best = Some(scene);
            }
        }

        best.cloned()
    }

    /// Yield the current frame label as a tuple of string and frame number.
    pub fn current_label(self) -> Option<(WString, FrameNumber)> {
        let read = self.0.read();
        let current_frame = read.current_frame();
        let mut best: Option<(&WString, FrameNumber)> = None;

        for (frame, label) in read.static_data.frame_labels.iter() {
            if *frame > current_frame {
                continue;
            }

            if best.map(|v| *frame >= v.1).unwrap_or(true) {
                best = Some((label, *frame));
            }
        }

        best.map(|(s, fnum)| (s.clone(), fnum))
    }

    /// Yield a list of labels and frame-numbers in the current scene.
    ///
    /// Labels are returned sorted by frame number.
    pub fn labels_in_range(
        self,
        from: FrameNumber,
        to: FrameNumber,
    ) -> Vec<(WString, FrameNumber)> {
        let read = self.0.read();

        let mut values: Vec<(WString, FrameNumber)> = read
            .static_data
            .frame_labels
            .iter()
            .filter(|(frame, _label)| *frame >= from && *frame < to)
            .map(|(frame, label)| (label.clone(), *frame))
            .collect();

        values.sort_unstable_by(|(_, framea), (_, frameb)| framea.cmp(frameb));

        values
    }

    pub fn total_frames(self) -> FrameNumber {
        self.0.read().total_frames()
    }

    #[allow(dead_code)]
    pub fn has_frame_script(self, frame: FrameNumber) -> bool {
        self.0
            .read()
            .frame_scripts
            .get(frame as usize)
            .map(|v| v.is_some())
            .unwrap_or_default()
    }

    /// This sets the current preload frame of this MovieClipto a given number (resulting
    /// in the _framesloaded / framesLoaded property being the given number - 1).
    pub fn set_cur_preload_frame(self, gc_context: &Mutation<'gc>, cur_preload_frame: u16) {
        self.0
            .read()
            .static_data
            .preload_progress
            .write(gc_context)
            .cur_preload_frame = cur_preload_frame;
    }

    /// This sets the current frame of this MovieClip to a given number.
    pub fn set_current_frame(self, gc_context: &Mutation<'gc>, current_frame: FrameNumber) {
        self.0.write(gc_context).current_frame = current_frame;
    }

    pub fn frames_loaded(self) -> i32 {
        self.0.read().frames_loaded()
    }

    pub fn total_bytes(self) -> i32 {
        // For a loaded SWF, returns the uncompressed size of the SWF.
        // Otherwise, returns the size of the tag list in the clip's DefineSprite tag.
        if self.is_root() {
            self.movie().uncompressed_len()
        } else {
            self.tag_stream_len() as i32
        }
    }

    pub fn loaded_bytes(self) -> u32 {
        let read = self.0.read();
        let progress_read = read.static_data.preload_progress.read();
        if progress_read.next_preload_chunk == u64::MAX {
            // u64::MAX is a sentinel for load complete
            return max(self.total_bytes(), 0) as u32;
        }

        let swf_header_size = max(self.total_bytes(), 0) as u32 - self.tag_stream_len() as u32;

        swf_header_size + progress_read.next_preload_chunk as u32
    }

    /// Calculate the compressed total size of this movie clip's tag stream.
    ///
    /// For root movie clips, this is just the compressed size of the whole
    /// SWF. Other movie clips will have a compressed size calculated by taking
    /// the compression ratio of the whole SWF and scaling the uncompressed size
    /// down.
    pub fn compressed_total_bytes(self) -> u32 {
        let movie = self.movie();
        let compressed_movie_size = movie.as_ref().compressed_len();

        if self.is_root() {
            compressed_movie_size as u32
        } else {
            let uncompressed_movie_size = movie.data().len();
            let uncompressed_clip_size = self.tag_stream_len() as u32;

            (uncompressed_clip_size as f64 * compressed_movie_size as f64
                / uncompressed_movie_size as f64)
                .floor() as u32
        }
    }

    /// Calculate the compressed loaded size of this movie clip's tag stream.
    ///
    /// Since we only consider a byte loaded after it has been uncompressed and
    /// run through `preload`, we instead emulate this property by scaling the
    /// loaded bytes by the compression ratio of the SWF.
    pub fn compressed_loaded_bytes(self) -> u32 {
        (self.loaded_bytes() as f64 * self.compressed_total_bytes() as f64
            / self.total_bytes() as f64) as u32
    }

    pub fn set_avm2_class(self, gc_context: &Mutation<'gc>, constr: Option<Avm2ClassObject<'gc>>) {
        *self.0.read().static_data.avm2_class.write(gc_context) = constr;
    }

    pub fn frame_label_to_number(
        self,
        frame_label: &WStr,
        _context: &UpdateContext<'gc>,
    ) -> Option<FrameNumber> {
        // In AVM1, frame labels are case insensitive (ASCII).
        // They are case sensitive in AVM2.
        if self.movie().is_action_script_3() {
            self.0
                .read()
                .static_data
                .frame_labels_map
                .get(frame_label)
                .copied()
        } else {
            let label = frame_label.to_ascii_lowercase();
            self.0
                .read()
                .static_data
                .frame_labels_map
                .get(&label)
                .copied()
        }
    }

    pub fn scene_label_to_number(self, scene_label: &WStr) -> Option<FrameNumber> {
        // Never used in AVM1, so always be case sensitive.
        self.0
            .read()
            .static_data
            .scene_labels_map
            .get(&WString::from(scene_label))
            .map(|Scene { start, .. }| start)
            .copied()
    }

    pub fn frame_exists_within_scene(
        self,
        frame_label: &WStr,
        scene_label: &WStr,
        context: &UpdateContext<'gc>,
    ) -> bool {
        let scene = self.scene_label_to_number(scene_label);
        let frame = self.frame_label_to_number(frame_label, context);

        if scene.is_none() || frame.is_none() {
            return false;
        }

        let scene = scene.unwrap();
        let frame = frame.unwrap();

        if scene <= frame {
            let mut end = self.total_frames() + 1;
            for Scene {
                start: new_scene_start,
                ..
            } in self.0.read().static_data.scene_labels.iter()
            {
                if *new_scene_start < end && *new_scene_start > scene {
                    end = *new_scene_start;
                }
            }

            frame < end
        } else {
            false
        }
    }

    /// Gets the clip events for this MovieClip.
    pub fn clip_actions(&self) -> Ref<[ClipEventHandler]> {
        Ref::map(self.0.read(), |mc| mc.clip_event_handlers())
    }

    /// Sets the clip actions (a.k.a. clip events) for this MovieClip.
    /// Clip actions are created in the Flash IDE by using the `onEnterFrame`
    /// tag on a MovieClip instance.
    pub fn set_clip_event_handlers(
        self,
        gc_context: &Mutation<'gc>,
        event_handlers: Vec<ClipEventHandler>,
    ) {
        let mut mc = self.0.write(gc_context);
        mc.set_clip_event_handlers(event_handlers);
    }

    /// Returns an iterator of AVM1 `DoAction` blocks on the given frame number.
    /// Used by the AVM `Call` action.
    pub fn actions_on_frame(
        self,
        _context: &mut UpdateContext<'gc>,
        frame: FrameNumber,
    ) -> impl DoubleEndedIterator<Item = SwfSlice> {
        use swf::read::Reader;

        let mut actions: SmallVec<[SwfSlice; 2]> = SmallVec::new();

        // Iterate through this clip's tags, counting frames until we reach the target frame.
        if frame > 0 && frame <= self.total_frames() {
            let mut cur_frame = 1;
            let clip = self.0.read();
            let mut reader = clip.static_data.swf.read_from(0);
            while cur_frame <= frame && !reader.get_ref().is_empty() {
                let tag_callback = |reader: &mut Reader<'_>, tag_code, tag_len| {
                    match tag_code {
                        TagCode::ShowFrame => {
                            cur_frame += 1;
                            Ok(ControlFlow::Exit)
                        }
                        TagCode::DoAction if cur_frame == frame => {
                            // On the target frame, add any DoAction tags to the array.
                            let slice = clip.static_data.swf.resize_to_reader(reader, tag_len);
                            if !slice.is_empty() {
                                actions.push(slice);
                            }
                            Ok(ControlFlow::Continue)
                        }
                        _ => Ok(ControlFlow::Continue),
                    }
                };

                let _ = tag_utils::decode_tags(&mut reader, tag_callback);
            }
        }

        actions.into_iter()
    }

    /// Determine what the clip's next frame should be.
    fn determine_next_frame(self) -> NextFrame {
        if self.current_frame() < self.total_frames() {
            NextFrame::Next
        } else if self.total_frames() > 1 {
            NextFrame::First
        } else {
            NextFrame::Same
        }
    }

    fn run_frame_internal(
        self,
        context: &mut UpdateContext<'gc>,
        run_display_actions: bool,
        run_sounds: bool,
        is_action_script_3: bool,
    ) {
        let next_frame = self.determine_next_frame();
        match next_frame {
            NextFrame::Next => {
                let mut write = self.0.write(context.gc_context);
                if (write.current_frame + 1)
                    >= write.static_data.preload_progress.read().cur_preload_frame
                {
                    return;
                }

                // AS3 removals need to happen before frame advance (see below)
                if !is_action_script_3 {
                    write.current_frame += 1
                }
            }
            NextFrame::First => return self.run_goto(context, 1, true),
            NextFrame::Same => self.stop(context),
        }

        let mc = self.0.read();
        let tag_stream_start = mc.static_data.swf.as_ref().as_ptr() as u64;
        let data = mc.static_data.swf.clone();
        let mut reader = data.read_from(mc.tag_stream_pos);
        drop(mc);

        let tag_callback = |reader: &mut SwfStream<'_>, tag_code, tag_len| {
            match tag_code {
                TagCode::DoAction => self.do_action(context, reader, tag_len),
                TagCode::PlaceObject if run_display_actions && !is_action_script_3 => {
                    self.place_object(context, reader, 1)
                }
                TagCode::PlaceObject2 if run_display_actions && !is_action_script_3 => {
                    self.place_object(context, reader, 2)
                }
                TagCode::PlaceObject3 if run_display_actions && !is_action_script_3 => {
                    self.place_object(context, reader, 3)
                }
                TagCode::PlaceObject4 if run_display_actions && !is_action_script_3 => {
                    self.place_object(context, reader, 4)
                }
                TagCode::RemoveObject if run_display_actions && !is_action_script_3 => {
                    self.remove_object(context, reader, 1)
                }
                TagCode::RemoveObject2 if run_display_actions && !is_action_script_3 => {
                    self.remove_object(context, reader, 2)
                }
                TagCode::PlaceObject if run_display_actions && is_action_script_3 => {
                    self.queue_place_object(context, reader, 1)
                }
                TagCode::PlaceObject2 if run_display_actions && is_action_script_3 => {
                    self.queue_place_object(context, reader, 2)
                }
                TagCode::PlaceObject3 if run_display_actions && is_action_script_3 => {
                    self.queue_place_object(context, reader, 3)
                }
                TagCode::PlaceObject4 if run_display_actions && is_action_script_3 => {
                    self.queue_place_object(context, reader, 4)
                }
                TagCode::RemoveObject if run_display_actions && is_action_script_3 => {
                    self.queue_remove_object(context, reader, 1)
                }
                TagCode::RemoveObject2 if run_display_actions && is_action_script_3 => {
                    self.queue_remove_object(context, reader, 2)
                }
                TagCode::SetBackgroundColor => self.set_background_color(context, reader),
                TagCode::StartSound if run_sounds => self.start_sound_1(context, reader),
                TagCode::SoundStreamBlock if run_sounds => self.sound_stream_block(context, reader),
                TagCode::ShowFrame => return Ok(ControlFlow::Exit),
                _ => Ok(()),
            }?;

            Ok(ControlFlow::Continue)
        };
        let _ = tag_utils::decode_tags(&mut reader, tag_callback);
        if let Err(e) = self.run_abc_and_symbol_tags(context, self.0.read().current_frame) {
            tracing::error!("Error running abc/symbol in frame: {e:?}");
        }

        // On AS3, we deliberately run all removals before the frame number or
        // tag position updates. This ensures that code that runs gotos when a
        // display object is added or removed does not catch the movie clip in
        // an invalid state.
        let remove_actions = self.unqueue_removes(context);

        for (_, tag) in remove_actions {
            let mut reader = data.read_from(tag.tag_start);
            let version = match tag.tag_type {
                QueuedTagAction::Remove(v) => v,
                _ => unreachable!(),
            };

            if let Err(e) = self.remove_object(context, &mut reader, version) {
                tracing::error!("Error running queued tag: {:?}, got {}", tag.tag_type, e);
            }
        }

        // It is now safe to update the tag position and frame number.
        // TODO: Determine if explicit gotos override these or not.
        let mut write = self.0.write(context.gc_context);

        write.tag_stream_pos = reader.get_ref().as_ptr() as u64 - tag_stream_start;

        // Check if our audio track has finished playing.
        if let Some(audio_stream) = write.audio_stream {
            if !context.is_sound_playing(audio_stream) {
                write.audio_stream = None;
            }
        }

        if matches!(next_frame, NextFrame::Next) && is_action_script_3 {
            write.current_frame += 1;
        }

        write.queued_script_frame = Some(write.current_frame);
        if write.last_queued_script_frame != Some(write.current_frame) {
            // We explicitly clear this variable since AS3 may later GOTO back
            // to the already-ran frame. Since the frame number *has* changed
            // in the meantime, it should absolutely run again.
            write.last_queued_script_frame = None;
        }
    }

    /// Instantiate a given child object on the timeline at a given depth.
    fn instantiate_child(
        self,
        context: &mut UpdateContext<'gc>,
        id: CharacterId,
        depth: Depth,
        place_object: &swf::PlaceObject,
    ) -> Option<DisplayObject<'gc>> {
        let movie = self.movie();
        let library = context.library.library_for_movie_mut(movie.clone());
        match library.instantiate_by_id(id, context.gc_context) {
            Ok(child) => {
                // Remove previous child from children list,
                // and add new child onto front of the list.
                let prev_child = self.replace_at_depth(context, child, depth);
                {
                    // Set initial properties for child.
                    child.set_instantiated_by_timeline(context.gc_context, true);
                    child.set_depth(context.gc_context, depth);
                    child.set_parent(context, Some(self.into()));
                    child.set_place_frame(context.gc_context, self.current_frame());

                    // Apply PlaceObject parameters.
                    child.apply_place_object(context, place_object);
                    if let Some(name) = &place_object.name {
                        let encoding = swf::SwfStr::encoding_for_version(self.swf_version());
                        let name = AvmString::new(context.gc_context, name.decode(encoding));
                        child.set_name(context.gc_context, name);
                        child.set_has_explicit_name(context.gc_context, true);
                    }
                    if let Some(clip_depth) = place_object.clip_depth {
                        child.set_clip_depth(context.gc_context, clip_depth.into());
                    }
                    // Clip events only apply to movie clips.
                    if let (Some(clip_actions), Some(clip)) =
                        (&place_object.clip_actions, child.as_movie_clip())
                    {
                        // Convert from `swf::ClipAction` to Ruffle's `ClipEventHandler`.
                        clip.set_clip_event_handlers(
                            context.gc_context,
                            clip_actions
                                .iter()
                                .cloned()
                                .map(|a| ClipEventHandler::from_action_and_movie(a, movie.clone()))
                                .collect(),
                        );
                    }
                    // TODO: Missing PlaceObject property: amf_data

                    // Run first frame.
                    child.post_instantiation(context, None, Instantiator::Movie, false);
                    child.enter_frame(context);
                    // In AVM1, children are added in `run_frame` so this is necessary.
                    // In AVM2 we add them in `construct_frame` so calling this causes
                    // duplicate frames
                    if !movie.is_action_script_3() {
                        child.run_frame_avm1(context);
                    }
                }

                if let Some(prev_child) = prev_child {
                    dispatch_removed_event(prev_child, context);
                }

                Some(child)
            }
            Err(e) => {
                tracing::error!(
                    "Unable to instantiate display node id {}, reason being: {}",
                    id,
                    e
                );
                None
            }
        }
    }

    #[cfg(not(feature = "timeline_debug"))]
    fn assert_expected_tag_start(self) {}

    #[cfg(feature = "timeline_debug")]
    fn assert_expected_tag_start(self) {
        let read = self.0.read();

        assert_eq!(
            Some(read.tag_stream_pos),
            read.tag_frame_boundaries
                .get(&read.current_frame)
                .map(|(_start, end)| *end), // Yes, this is correct, at least for AVM1.
            "[{:?}] Gotos must start from the correct tag position for frame {}",
            read.base.base.name,
            read.current_frame
        );
    }

    #[cfg(not(feature = "timeline_debug"))]
    fn assert_expected_tag_end(self, _context: &mut UpdateContext<'gc>, _hit_target_frame: bool) {}

    #[cfg(feature = "timeline_debug")]
    fn assert_expected_tag_end(self, context: &mut UpdateContext<'gc>, hit_target_frame: bool) {
        // Gotos that do *not* hit their target frame will not update their tag
        // stream position, as they do not run the final frame's tags, and thus
        // cannot derive the end position of the clip anyway. This is not
        // observable to user code as any further timeline interaction would
        // trigger a rewind, so we ignore it here for now.
        if hit_target_frame {
            let read = self.0.read();

            assert_eq!(
                Some(read.tag_stream_pos),
                read.tag_frame_boundaries
                    .get(&read.current_frame)
                    .map(|(_start, end)| *end),
                "[{:?}] Gotos must end at the correct tag position for frame {}",
                read.base.base.name,
                read.current_frame
            );
        } else {
            // Of course, the target frame desync absolutely will break our
            // other asserts, so fix them up here.
            let mut write = self.0.write(context.gc_context);

            if let Some((_, end)) = write
                .tag_frame_boundaries
                .get(&write.current_frame)
                .cloned()
            {
                write.tag_stream_pos = end;
            }
        }
    }

    pub fn run_goto(
        mut self,
        context: &mut UpdateContext<'gc>,
        frame: FrameNumber,
        is_implicit: bool,
    ) {
        if cfg!(feature = "timeline_debug") {
            tracing::debug!(
                "[{}]: {} from frame {} to frame {}",
                self.name(),
                if is_implicit { "looping" } else { "goto" },
                self.current_frame(),
                frame
            );
            self.assert_expected_tag_start();
        }

        let frame_before_rewind = self.current_frame();
        self.base_mut(context.gc_context)
            .set_skip_next_enter_frame(false);

        // Flash gotos are tricky:
        // 1) Conceptually, a goto should act like the playhead is advancing forward or
        //    backward to a frame.
        // 2) However, MovieClip timelines are stored as deltas from frame to frame,
        //    so for rewinds, we must restart to frame 1 and play forward.
        // 3) Objects that would persist over the goto conceptually should not be
        //    destroyed and recreated; they should keep their properties.
        //    Particularly for rewinds, the object should persist if it was created
        //      *before* the frame we are going to. (DisplayObject::place_frame).
        // 4) We want to avoid creating objects just to destroy them if they aren't on
        //    the goto frame, so we should instead aggregate the deltas into a final list
        //    of commands, and THEN modify the children as necessary.

        // This map will maintain a map of depth -> placement commands.
        // TODO: Move this to UpdateContext to avoid allocations.
        let mut goto_commands: Vec<GotoPlaceObject<'_>> = vec![];

        self.0.write(context.gc_context).stop_audio_stream(context);

        let is_rewind = if frame <= self.current_frame() {
            // Because we can only step forward, we have to start at frame 1
            // when rewinding. We don't actually remove children yet because
            // otherwise AS3 can observe byproducts of the rewinding process.
            self.0.write(context.gc_context).tag_stream_pos = 0;
            self.0.write(context.gc_context).current_frame = 0;

            true
        } else {
            false
        };

        let from_frame = self.current_frame();

        // Explicit gotos in the middle of an AS3 loop cancel the loop's queued
        // tags. The rest of the goto machinery can handle the side effects of
        // a half-executed loop.
        let mut write = self.0.write(context.gc_context);
        if write.loop_queued() {
            write.queued_tags = HashMap::new();
        }

        if is_implicit {
            write.set_loop_queued();
        }
        drop(write);

        // Step through the intermediate frames, and aggregate the deltas of each frame.
        let mc = self.0.read();
        let tag_stream_start = mc.static_data.swf.as_ref().as_ptr() as u64;
        let mut frame_pos = mc.tag_stream_pos;
        let data = mc.static_data.swf.clone();
        let mut index = 0;

        // Sanity; let's make sure we don't seek way too far.
        let clamped_frame = frame.min(max(mc.frames_loaded(), 0) as FrameNumber);
        drop(mc);

        let mut removed_frame_scripts: Vec<DisplayObject<'gc>> = vec![];

        let mut reader = data.read_from(frame_pos);
        while self.current_frame() < clamped_frame && !reader.get_ref().is_empty() {
            self.0.write(context.gc_context).current_frame += 1;
            frame_pos = reader.get_ref().as_ptr() as u64 - tag_stream_start;

            let tag_callback = |reader: &mut _, tag_code, _tag_len| {
                match tag_code {
                    TagCode::PlaceObject => {
                        index += 1;
                        let mut mc = self.0.write(context.gc_context);
                        mc.goto_place_object(reader, 1, &mut goto_commands, is_rewind, index)
                    }
                    TagCode::PlaceObject2 => {
                        index += 1;
                        let mut mc = self.0.write(context.gc_context);
                        mc.goto_place_object(reader, 2, &mut goto_commands, is_rewind, index)
                    }
                    TagCode::PlaceObject3 => {
                        index += 1;
                        let mut mc = self.0.write(context.gc_context);
                        mc.goto_place_object(reader, 3, &mut goto_commands, is_rewind, index)
                    }
                    TagCode::PlaceObject4 => {
                        index += 1;
                        let mut mc = self.0.write(context.gc_context);
                        mc.goto_place_object(reader, 4, &mut goto_commands, is_rewind, index)
                    }
                    TagCode::RemoveObject => self.goto_remove_object(
                        reader,
                        1,
                        context,
                        &mut goto_commands,
                        is_rewind,
                        from_frame,
                        &mut removed_frame_scripts,
                    ),
                    TagCode::RemoveObject2 => self.goto_remove_object(
                        reader,
                        2,
                        context,
                        &mut goto_commands,
                        is_rewind,
                        from_frame,
                        &mut removed_frame_scripts,
                    ),
                    TagCode::ShowFrame => return Ok(ControlFlow::Exit),
                    _ => Ok(()),
                }?;

                Ok(ControlFlow::Continue)
            };
            let _ = tag_utils::decode_tags(&mut reader, tag_callback);
            if let Err(e) = self.run_abc_and_symbol_tags(context, self.current_frame() - 1) {
                tracing::error!("Error running abc/symbols in goto: {e:?}");
            }
        }
        let hit_target_frame = self.0.read().current_frame == frame;

        if is_rewind {
            // Remove all display objects that were created after the
            // destination frame.
            //
            // We do this after reading the clip timeline so that AS3 can't
            // observe side effects of the rewinding process.
            //
            // TODO: We want to do something like self.children.retain here,
            // but BTreeMap::retain does not exist.
            // TODO: Should AS3 children ignore GOTOs?
            let children: SmallVec<[_; 16]> = self
                .iter_render_list()
                .filter(|clip| clip.place_frame() > frame)
                .collect();
            for child in children {
                if !child.placed_by_script() {
                    self.remove_child(context, child);
                } else {
                    self.remove_child_from_depth_list(context, child);
                }
            }
        }

        // Run the list of goto commands to actually create and update the display objects.
        let run_goto_command = |clip: MovieClip<'gc>,
                                context: &mut UpdateContext<'gc>,
                                params: &GotoPlaceObject<'_>| {
            use swf::PlaceObjectAction;
            let child_entry = clip.child_by_depth(params.depth());
            if self.movie().is_action_script_3() && is_implicit && child_entry.is_none() {
                // Looping gotos do not run their PlaceObject commands at goto
                // time. They are instead held to frameConstructed like normal
                // playback.
                //
                // TODO: We can only queue *new* object placement, existing
                // objects still get updated too early.
                let mut write = self.0.write(context.gc_context);
                let new_tag = QueuedTag {
                    tag_type: QueuedTagAction::Place(params.version),
                    tag_start: params.tag_start,
                };
                let bucket = write
                    .queued_tags
                    .entry(params.place_object.depth as Depth)
                    .or_insert_with(|| QueuedTagList::None);

                bucket.queue_add(new_tag);

                return;
            }

            match (params.place_object.action, child_entry, is_rewind) {
                // Apply final delta to display parameters.
                // For rewinds, if an object was created before the final frame,
                // it will exist on the final frame as well. Re-use this object
                // instead of recreating.
                // If the ID is 0, we are modifying a previous child. Otherwise, we're replacing it.
                // If it's a rewind, we removed any dead children above, so we always
                // modify the previous child.
                (_, Some(prev_child), true) | (PlaceObjectAction::Modify, Some(prev_child), _) => {
                    prev_child.apply_place_object(context, &params.place_object);
                }
                (swf::PlaceObjectAction::Replace(id), Some(prev_child), _) => {
                    prev_child.replace_with(context, id);
                    prev_child.apply_place_object(context, &params.place_object);
                    prev_child.set_place_frame(context.gc_context, params.frame);
                }
                (PlaceObjectAction::Place(id), _, _)
                | (swf::PlaceObjectAction::Replace(id), _, _) => {
                    if let Some(child) =
                        clip.instantiate_child(context, id, params.depth(), &params.place_object)
                    {
                        // Set the place frame to the frame where the object *would* have been placed.
                        child.set_place_frame(context.gc_context, params.frame);
                    }
                }
                _ => {
                    tracing::error!(
                        "Unexpected PlaceObject during goto: {:?}",
                        params.place_object
                    )
                }
            }
        };

        // We have to be sure that queued actions are generated in the same order
        // as if the playhead had reached this frame normally.

        // First, sort the goto commands in the order of execution.
        // (Maybe it'd be better to keeps this list sorted as we create it?
        // Currently `swap_remove` calls futz with the order; but we could use `remove`).
        goto_commands.sort_by_key(|params| params.index);

        // Then, run frames for children that were created before this frame.
        goto_commands
            .iter()
            .filter(|params| params.frame < frame)
            .for_each(|goto| run_goto_command(self, context, goto));

        // Next, run the final frame for the parent clip.
        // Re-run the final frame without display tags (DoAction, etc.)
        // Note that this only happens if the frame exists and is loaded;
        // e.g. gotoAndStop(9999) displays the final frame, but actions don't run!
        if hit_target_frame {
            self.0.write(context.gc_context).current_frame -= 1;
            self.0.write(context.gc_context).tag_stream_pos = frame_pos;
            // If we changed frames, then trigger any sounds in our target frame.
            // However, if we executed a 'no-op goto' (start and end frames are the same),
            // then do *not* run sounds. Some SWFS (e.g. 'This is the only level too')
            // rely on this behavior.
            self.run_frame_internal(
                context,
                false,
                frame != frame_before_rewind,
                self.movie().is_action_script_3(),
            );
        } else {
            self.0.write(context.gc_context).current_frame = clamped_frame;
        }

        // Finally, run frames for children that are placed on this frame.
        goto_commands
            .iter()
            .filter(|params| params.frame >= frame)
            .for_each(|goto| run_goto_command(self, context, goto));

        // On AVM2, all explicit gotos act the same way as a normal new frame,
        // save for the lack of an enterFrame event. Since this must happen
        // before AS3 continues execution, this is effectively a "recursive
        // frame".
        //
        // Our queued place tags will now run at this time, too.
        if !is_implicit {
            run_inner_goto_frame(context, &removed_frame_scripts, self);
        }

        self.assert_expected_tag_end(context, hit_target_frame);
    }

    fn construct_as_avm1_object(
        self,
        context: &mut UpdateContext<'gc>,
        init_object: Option<Avm1Object<'gc>>,
        instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        //TODO: This will break horribly when AVM2 starts touching the display list
        if self.0.read().object.is_none() {
            let avm1_constructor = self.0.read().get_registered_avm1_constructor(context);

            // If we are running within the AVM, this must be an immediate action.
            // If we are not, then this must be queued to be ran first-thing
            if let Some(constructor) = avm1_constructor.filter(|_| instantiated_by.is_avm()) {
                let mut activation = Avm1Activation::from_nothing(
                    context,
                    ActivationIdentifier::root("[Construct]"),
                    self.into(),
                );

                if let Ok(prototype) = constructor
                    .get("prototype", &mut activation)
                    .map(|v| v.coerce_to_object(&mut activation))
                {
                    let object: Avm1Object<'gc> = StageObject::for_display_object(
                        activation.context.gc_context,
                        self.into(),
                        prototype,
                    )
                    .into();
                    self.0.write(activation.context.gc_context).object = Some(object.into());

                    if run_frame {
                        self.run_frame_avm1(activation.context);
                    }

                    if let Some(init_object) = init_object {
                        // AVM1 sets keys in reverse order (compared to enumeration order).
                        // This behavior is visible to setters, and some SWFs depend on it.
                        for key in init_object
                            .get_keys(&mut activation, false)
                            .into_iter()
                            .rev()
                        {
                            if let Ok(value) = init_object.get(key, &mut activation) {
                                let _ = object.set(key, value, &mut activation);
                            }
                        }
                    }
                    let _ = constructor.construct_on_existing(&mut activation, object, &[]);
                }

                return;
            }

            let object: Avm1Object<'gc> = StageObject::for_display_object(
                context.gc_context,
                self.into(),
                context.avm1.prototypes().movie_clip,
            )
            .into();
            self.0.write(context.gc_context).object = Some(object.into());

            if run_frame {
                self.run_frame_avm1(context);
            }

            if let Some(init_object) = init_object {
                let mut activation = Avm1Activation::from_nothing(
                    context,
                    ActivationIdentifier::root("[Init]"),
                    self.into(),
                );

                for key in init_object.get_keys(&mut activation, false) {
                    if let Ok(value) = init_object.get(key, &mut activation) {
                        let _ = object.set(key, value, &mut activation);
                    }
                }
            }

            let mut events = Vec::new();

            for event_handler in self
                .0
                .write(context.gc_context)
                .clip_event_handlers()
                .iter()
            {
                if event_handler.events.contains(ClipEventFlag::INITIALIZE) {
                    context.action_queue.queue_action(
                        self.into(),
                        ActionType::Initialize {
                            bytecode: event_handler.action_data.clone(),
                        },
                        false,
                    );
                }
                if event_handler.events.contains(ClipEventFlag::CONSTRUCT) {
                    events.push(event_handler.action_data.clone());
                }
            }

            context.action_queue.queue_action(
                self.into(),
                ActionType::Construct {
                    constructor: avm1_constructor,
                    events,
                },
                false,
            );
        } else if run_frame {
            self.run_frame_avm1(context);
        }

        // If this text field has a variable set, initialize text field binding.
        Avm1::run_with_stack_frame_for_display_object(self.into(), context, |activation| {
            self.bind_text_field_variables(activation);
        });
    }

    /// Allocate the AVM2 side of this object.
    ///
    /// This function does *not* call the constructor; it is intended that you
    /// will construct the object first before doing so. This function is
    /// intended to be called from `construct_frame`.
    #[inline(never)]
    fn allocate_as_avm2_object(
        self,
        context: &mut UpdateContext<'gc>,
        display_object: DisplayObject<'gc>,
    ) {
        let class_object = self
            .0
            .read()
            .static_data
            .avm2_class
            .read()
            .unwrap_or_else(|| context.avm2.classes().movieclip);

        let mut constr_thing = || {
            let mut activation = Avm2Activation::from_nothing(context);
            let object =
                Avm2StageObject::for_display_object(&mut activation, display_object, class_object)?
                    .into();

            Ok(object)
        };
        let result: Result<Avm2Object<'gc>, Avm2Error> = constr_thing();

        if let Ok(object) = result {
            self.set_object2(context, object);
        } else if let Err(e) = result {
            tracing::error!("Got {} when allocating AVM2 side of display object", e);
        }
    }

    /// Construct the AVM2 side of this object.
    ///
    /// This function does *not* allocate the object; it is intended that you
    /// will allocate the object first before doing so. This function is
    /// intended to be called from `post_instantiate`.
    #[inline(never)]
    fn construct_as_avm2_object(self, context: &mut UpdateContext<'gc>) {
        let class_object = self
            .0
            .read()
            .static_data
            .avm2_class
            .read()
            .unwrap_or_else(|| context.avm2.classes().movieclip);

        if let Avm2Value::Object(object) = self.object2() {
            let mut constr_thing = || {
                let mut activation = Avm2Activation::from_nothing(context);
                class_object.call_super_init(object.into(), &[], &mut activation)?;

                Ok(())
            };
            let result: Result<(), Avm2Error> = constr_thing();

            if let Err(e) = result {
                tracing::error!(
                    "Got \"{:?}\" when constructing AVM2 side of movie clip of type {}",
                    e,
                    class_object
                        .inner_class_definition()
                        .name()
                        .to_qualified_name(context.gc_context)
                );
            }
        }
    }

    pub fn register_frame_script(
        self,
        frame_id: FrameNumber,
        callable: Option<Avm2Object<'gc>>,
        context: &mut UpdateContext<'gc>,
    ) {
        let frame_scripts = &mut self.0.write(context.gc_context).frame_scripts;

        let index = frame_id as usize;
        if let Some(callable) = callable {
            if frame_scripts.len() <= index {
                frame_scripts.resize(index + 1, None);
            }
            frame_scripts[index] = Some(callable);
        } else if frame_scripts.len() > index {
            frame_scripts[index] = None;
        }
    }

    /// Handle a RemoveObject tag when running a goto action.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn goto_remove_object<'a>(
        mut self,
        reader: &mut SwfStream<'a>,
        version: u8,
        context: &mut UpdateContext<'gc>,
        goto_commands: &mut Vec<GotoPlaceObject<'a>>,
        is_rewind: bool,
        from_frame: FrameNumber,
        removed_frame_scripts: &mut Vec<DisplayObject<'gc>>,
    ) -> Result<(), Error> {
        let remove_object = if version == 1 {
            reader.read_remove_object_1()
        } else {
            reader.read_remove_object_2()
        }?;
        let depth: Depth = remove_object.depth.into();
        if let Some(i) = goto_commands.iter().position(|o| o.depth() == depth) {
            goto_commands.swap_remove(i);
        }
        if !is_rewind {
            // For fast-forwards, if this tag were to remove an object
            // that existed before the goto, then we can remove that child right away.
            // Don't do this for rewinds, because they conceptually
            // start from an empty display list, and we also want to examine
            // the old children to decide if they persist (place_frame <= goto_frame).
            //
            // We also have to reset the frame number as this emits AS3 events.
            let to_frame = self.current_frame();
            self.0.write(context.gc_context).current_frame = from_frame;

            let child = self.child_by_depth(depth);
            if let Some(child) = child {
                if !child.placed_by_script() {
                    self.remove_child(context, child);
                } else {
                    self.remove_child_from_depth_list(context, child);
                }

                removed_frame_scripts.push(child);
            }

            self.0.write(context.gc_context).current_frame = to_frame;
        }
        Ok(())
    }

    fn enabled(self, context: &mut UpdateContext<'gc>) -> bool {
        if !self.movie().is_action_script_3() {
            self.get_avm1_boolean_property(context, "enabled", |_| true)
        } else {
            self.avm2_enabled()
        }
    }

    pub fn avm2_enabled(self) -> bool {
        self.0.read().avm2_enabled
    }

    pub fn set_avm2_enabled(self, context: &mut UpdateContext<'gc>, enabled: bool) {
        self.0.write(context.gc_context).avm2_enabled = enabled;
    }

    fn use_hand_cursor(self, context: &mut UpdateContext<'gc>) -> bool {
        if !self.movie().is_action_script_3() {
            self.get_avm1_boolean_property(context, "useHandCursor", |_| true)
        } else {
            self.avm2_use_hand_cursor()
        }
    }

    pub fn avm2_use_hand_cursor(self) -> bool {
        self.0.read().avm2_use_hand_cursor
    }

    pub fn set_avm2_use_hand_cursor(self, context: &mut UpdateContext<'gc>, use_hand_cursor: bool) {
        self.0.write(context.gc_context).avm2_use_hand_cursor = use_hand_cursor;
    }

    pub fn hit_area(self) -> Option<DisplayObject<'gc>> {
        self.0.read().hit_area
    }

    pub fn set_hit_area(
        self,
        context: &mut UpdateContext<'gc>,
        hit_area: Option<DisplayObject<'gc>>,
    ) {
        self.0.write(context.gc_context).hit_area = hit_area;
    }

    pub fn tag_stream_len(&self) -> usize {
        self.0.read().tag_stream_len()
    }

    pub fn forced_button_mode(self) -> bool {
        self.0.read().button_mode
    }

    pub fn set_forced_button_mode(self, context: &mut UpdateContext<'gc>, button_mode: bool) {
        self.0.write(context.gc_context).button_mode = button_mode;
    }

    pub fn drawing(&self, gc_context: &Mutation<'gc>) -> RefMut<'_, Drawing> {
        // We're about to change graphics, so invalidate on the next frame
        self.invalidate_cached_bitmap(gc_context);
        RefMut::map(self.0.write(gc_context), |s| &mut s.drawing)
    }

    pub fn is_button_mode(&self, context: &mut UpdateContext<'gc>) -> bool {
        if self.forced_button_mode()
            || self
                .0
                .read()
                .clip_event_flags
                .intersects(ClipEvent::BUTTON_EVENT_FLAGS)
        {
            true
        } else {
            let object = self.object();
            if let Avm1Value::Object(object) = object {
                let mut activation = Avm1Activation::from_nothing(
                    context,
                    ActivationIdentifier::root("[Mouse Pick]"),
                    self.avm1_root(),
                );

                ClipEvent::BUTTON_EVENT_METHODS
                    .iter()
                    .copied()
                    .any(|handler| object.has_property(&mut activation, handler.into()))
            } else {
                false
            }
        }
    }

    /// Remove all `PlaceObject` tags off the internal tag queue.
    fn unqueue_adds(&self, context: &mut UpdateContext<'gc>) -> Vec<(Depth, QueuedTag)> {
        let mut write = self.0.write(context.gc_context);
        let mut unqueued: Vec<_> = write
            .queued_tags
            .iter_mut()
            .filter_map(|(d, b)| b.unqueue_add().map(|b| (*d, b)))
            .collect();

        unqueued.sort_by(|(_, t1), (_, t2)| t1.tag_start.cmp(&t2.tag_start));

        for (depth, _tag) in unqueued.iter() {
            if matches!(write.queued_tags.get(depth), Some(QueuedTagList::None)) {
                write.queued_tags.remove(depth);
            }
        }

        unqueued
    }

    /// Remove all `RemoveObject` tags off the internal tag queue.
    fn unqueue_removes(&self, context: &mut UpdateContext<'gc>) -> Vec<(Depth, QueuedTag)> {
        let mut write = self.0.write(context.gc_context);
        let mut unqueued: Vec<_> = write
            .queued_tags
            .iter_mut()
            .filter_map(|(d, b)| b.unqueue_remove().map(|b| (*d, b)))
            .collect();

        unqueued.sort_by(|(_, t1), (_, t2)| t1.tag_start.cmp(&t2.tag_start));

        for (depth, _tag) in unqueued.iter() {
            if matches!(write.queued_tags.get(depth), Some(QueuedTagList::None)) {
                write.queued_tags.remove(depth);
            }
        }

        unqueued
    }

    /// This unloads the MovieClip.
    ///
    /// This means that one frame after this method has been called, avm1_unload and
    /// transform_to_unloaded_state get called and the MovieClip enters the unloaded
    /// state in which some attributes have certain values.
    // TODO: Look at where avm1_unload gets called directly. Does Flash also execute these
    // calls with one frame delay? Does transform_to_unloaded_state need to get executed
    // after one frame if the target is a MovieClip? Test the behaviour and adapt the code
    // if necessary.
    pub fn avm1_unload_movie(&self, context: &mut UpdateContext<'gc>) {
        // TODO: In Flash player, the MovieClip properties change to the unloaded state
        // one frame after the unloadMovie command has been read, even if the MovieClip
        // is not a root MovieClip (see the movieclip_library_state_values test).
        // However, if avm1_unload and transform_to_unloaded_state are called with a one
        // frame delay when the MovieClip is not a root MovieClip, regressions appear.
        // Ruffle is probably replacing a MovieClip differently to Flash, therefore
        // introducing these regressions when trying to emulate that delay.

        if self.is_root() {
            let unloader = Loader::MovieUnloader {
                self_handle: None,
                target_clip: DisplayObject::MovieClip(*self),
            };
            let handle = context.load_manager.add_loader(unloader);

            let player = context
                .player
                .clone()
                .upgrade()
                .expect("Could not upgrade weak reference to player");
            let future = Box::pin(async move {
                player
                    .lock()
                    .unwrap()
                    .update(|uc| -> Result<(), loader::Error> {
                        let clip = match uc.load_manager.get_loader(handle) {
                            Some(Loader::MovieUnloader { target_clip, .. }) => *target_clip,
                            None => return Err(loader::Error::Cancelled),
                            _ => unreachable!(),
                        };
                        if let Some(mc) = clip.as_movie_clip() {
                            mc.avm1_unload(uc);
                            mc.transform_to_unloaded_state(uc);
                        }

                        uc.load_manager.remove_loader(handle);

                        Ok(())
                    })?;
                Ok(())
            });

            context.navigator.spawn_future(future);
        } else {
            self.avm1_unload(context);
            self.transform_to_unloaded_state(context);
        }
    }

    /// This makes the MovieClip enter the unloaded state in which some attributes have
    /// certain values.
    /// An unloaded state movie stub which provides the correct values is created and
    /// loaded.
    ///
    /// This happens if a MovieClip has been unloaded. The state is then changed one
    /// frame after the command to unload the MovieClip has been read.
    fn transform_to_unloaded_state(&self, context: &mut UpdateContext<'gc>) {
        let movie = if let Some(DisplayObject::MovieClip(parent_mc)) = self.parent() {
            let parent_movie = parent_mc.movie();
            let parent_version = parent_movie.version();
            let parent_url = parent_movie.url();
            let mut unloaded_movie = SwfMovie::empty(parent_version);
            unloaded_movie.set_url(parent_url.to_string());

            Some(Arc::new(unloaded_movie))
        } else {
            None
        };

        self.replace_with_movie(context, movie, self.is_root(), None);
    }

    pub fn attach_audio(self, context: &mut UpdateContext<'gc>, netstream: Option<NetStream<'gc>>) {
        let mut write = self.0.write(context.gc_context);
        if netstream != write.attached_audio {
            if let Some(old_netstream) = write.attached_audio {
                old_netstream.was_detached(context);
            }

            write.attached_audio = netstream;

            if let Some(netstream) = netstream {
                netstream.was_attached(context, self);
            }
        }
    }
}

impl<'gc> TDisplayObject<'gc> for MovieClip<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: &Mutation<'gc>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base.base)
    }

    fn instantiate(&self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(GcCell::new(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().id()
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.read().movie()
    }

    fn enter_frame(&self, context: &mut UpdateContext<'gc>) {
        let skip_frame = self.base().should_skip_next_enter_frame();
        //Child removals from looping gotos appear to resolve in reverse order.
        for child in self.iter_render_list().rev() {
            if skip_frame {
                // If we're skipping our current frame, then we want to skip it for our children
                // as well. This counts as the skipped frame for any children that already
                // has this set to true (e.g. a third-level grandchild doesn't skip three frames).
                // We'll still call 'enter_frame' on the child - it will recurse, propagating along
                // the flag, and then set its own flag back to 'false'.
                //
                // We do *not* propagate `skip_frame=false` down to children, since a normally
                // executing parent can add a child that should have its first frame skipped.

                // FIXME - does this propagate through non-movie-clip children (Loader/Button)?
                child
                    .base_mut(context.gc_context)
                    .set_skip_next_enter_frame(true);
            }
            child.enter_frame(context);
        }

        if skip_frame {
            self.base_mut(context.gc_context)
                .set_skip_next_enter_frame(false);
            return;
        }

        if self.movie().is_action_script_3() {
            let is_playing = self.playing();

            if is_playing {
                self.run_frame_internal(context, true, true, true);
            }

            // PlaceObject tags execute at this time.
            // Note that this is NOT when constructors run; that happens later
            // after tags have executed.
            let data = self.0.read().static_data.swf.clone();
            let place_actions = self.unqueue_adds(context);

            for (_, tag) in place_actions {
                let mut reader = data.read_from(tag.tag_start);
                let version = match tag.tag_type {
                    QueuedTagAction::Place(v) => v,
                    _ => unreachable!(),
                };

                if let Err(e) = self.place_object(context, &mut reader, version) {
                    tracing::error!("Error running queued tag: {:?}, got {}", tag.tag_type, e);
                }
            }
        }
    }

    /// Construct objects placed on this frame.
    fn construct_frame(&self, context: &mut UpdateContext<'gc>) {
        // AVM1 code expects to execute in line with timeline instructions, so
        // it's exempted from frame construction.
        if self.movie().is_action_script_3()
            && (self.frames_loaded() >= 1 || self.total_frames() == 0)
        {
            let is_load_frame = !self.0.read().initialized();
            let needs_construction = if matches!(self.object2(), Avm2Value::Null) {
                self.allocate_as_avm2_object(context, (*self).into());
                true
            } else {
                false
            };

            self.0.write(context.gc_context).unset_loop_queued();

            if needs_construction {
                self.construct_as_avm2_object(context);
                self.on_construction_complete(context);
                // If we're in the load frame and we were constructed by ActionScript,
                // then we want to wait for the DisplayObject constructor to run
                // 'construct_frame' on children. This is observable by ActionScript -
                // before calling super(), 'this.numChildren' will show a non-zero number
                // when we have children placed on the load frame, but 'this.getChildAt(0)'
                // will return 'null' since the children haven't had their AVM2 objects
                // constructed by `construct_frame` yet.
            } else if !(is_load_frame && self.placed_by_script()) {
                let running_construct_frame = self
                    .0
                    .read()
                    .flags
                    .contains(MovieClipFlags::RUNNING_CONSTRUCT_FRAME);
                // The supercall constructor for display objects is responsible
                // for triggering construct_frame on frame 1.
                for child in self.iter_render_list() {
                    if running_construct_frame && child.object2().as_object().is_none() {
                        continue;
                    }
                    child.construct_frame(context);
                }
            }
        }
    }

    fn run_frame_avm1(&self, context: &mut UpdateContext<'gc>) {
        if !self.movie().is_action_script_3() {
            // Run my load/enterFrame clip event.
            let is_load_frame = !self.0.read().flags.contains(MovieClipFlags::INITIALIZED);
            if is_load_frame {
                self.event_dispatch(context, ClipEvent::Load);
                self.0.write(context.gc_context).set_initialized(true);
            } else {
                self.event_dispatch(context, ClipEvent::EnterFrame);
            }

            // Run my SWF tags.
            // In AVM2, SWF tags are processed at enterFrame time.
            if self.playing() {
                self.run_frame_internal(context, true, true, false);
            }
        }
    }

    fn run_frame_scripts(self, context: &mut UpdateContext<'gc>) {
        let mut write = self.0.write(context.gc_context);
        let avm2_object = write.object.and_then(|o| o.as_avm2_object());

        if let Some(avm2_object) = avm2_object {
            if let Some(frame_id) = write.queued_script_frame {
                // If we are already executing frame scripts, then we shouldn't
                // run frame scripts recursively. This is because AVM2 can run
                // gotos, which will both queue and run frame scripts for the
                // whole movie again. If a goto is attempting to queue frame
                // scripts on us AGAIN, we should allow the current stack to
                // wind down before handling that.
                if !write
                    .flags
                    .contains(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT)
                {
                    let is_fresh_frame =
                        write.queued_script_frame != write.last_queued_script_frame;

                    if is_fresh_frame {
                        if let Some(Some(callable)) =
                            write.frame_scripts.get(frame_id as usize).cloned()
                        {
                            write.last_queued_script_frame = Some(frame_id);
                            write.queued_script_frame = None;
                            write
                                .flags
                                .insert(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT);

                            drop(write);

                            let movie = self.movie();
                            let domain = context
                                .library
                                .library_for_movie(movie)
                                .unwrap()
                                .avm2_domain();

                            if let Err(e) = Avm2::run_stack_frame_for_callable(
                                callable,
                                avm2_object.into(),
                                &[],
                                domain,
                                context,
                            ) {
                                tracing::error!(
                                    "Error occurred when running AVM2 frame script: {}",
                                    e
                                );
                            }
                            write = self.0.write(context.gc_context);

                            write
                                .flags
                                .remove(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT);
                        }
                    }
                }
            }
        }

        let goto_frame = write.queued_goto_frame.take();
        drop(write);
        if let Some(frame) = goto_frame {
            self.run_goto(context, frame, false);
        }

        if let Some(container) = self.as_container() {
            for child in container.iter_render_list() {
                child.run_frame_scripts(context);
            }
        }
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        self.0.read().drawing.render(context);
        self.render_children(context);
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        self.0.read().drawing.self_bounds().clone()
    }

    fn hit_test_shape(
        &self,
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        if options.contains(HitTestOptions::SKIP_INVISIBLE)
            && !self.visible()
            && self.maskee().is_none()
        {
            return false;
        }

        if options.contains(HitTestOptions::SKIP_MASK) && self.maskee().is_some() {
            return false;
        }

        if self.world_bounds().contains(point) {
            let Some(local_matrix) = self.global_to_local_matrix() else {
                return false;
            };
            if let Some(masker) = self.masker() {
                if !masker.hit_test_shape(context, point, HitTestOptions::SKIP_INVISIBLE) {
                    return false;
                }
            }

            let mut clip_depth = 0;

            for child in self.iter_render_list() {
                if child.clip_depth() > 0 {
                    if child.hit_test_shape(
                        context,
                        point,
                        HitTestOptions::SKIP_MASK | HitTestOptions::SKIP_INVISIBLE,
                    ) {
                        clip_depth = 0;
                    } else {
                        clip_depth = child.clip_depth();
                    }
                } else if child.depth() >= clip_depth
                    && child.hit_test_shape(context, point, options)
                {
                    return true;
                }
            }

            let point = local_matrix * point;
            if self.0.read().drawing.hit_test(point, &local_matrix) {
                return true;
            }
        }

        false
    }

    fn as_movie_clip(&self) -> Option<MovieClip<'gc>> {
        Some(*self)
    }

    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        Some(self.into())
    }

    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        Some(self.into())
    }

    fn as_drawing(&self, gc_context: &Mutation<'gc>) -> Option<RefMut<'_, Drawing>> {
        Some(self.drawing(gc_context))
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'gc>,
        init_object: Option<Avm1Object<'gc>>,
        instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if self
            .0
            .write(context.gc_context)
            .flags
            .contains(MovieClipFlags::POST_INSTANTIATED)
        {
            // Ensure that the same clip doesn't get post-instantiated twice.
            return;
        }
        self.0
            .write(context.gc_context)
            .flags
            .insert(MovieClipFlags::POST_INSTANTIATED);

        self.set_default_instance_name(context);

        if !self.movie().is_action_script_3() {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());

            self.construct_as_avm1_object(context, init_object, instantiated_by, run_frame);
        }
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

    fn set_object2(&self, context: &mut UpdateContext<'gc>, to: Avm2Object<'gc>) {
        self.0.write(context.gc_context).object = Some(to.into());
        if self.parent().is_none() {
            context.avm2.add_orphan_obj((*self).into());
        }
    }

    fn on_parent_removed(&self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() {
            context.avm2.add_orphan_obj((*self).into())
        }
    }

    fn avm1_unload(&self, context: &mut UpdateContext<'gc>) {
        for child in self.iter_render_list() {
            child.avm1_unload(context);
        }

        if let Some(node) = self.maskee() {
            node.set_masker(context.gc_context, None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc_context, None, true);
        }

        // Unregister any text field variable bindings.
        if let Avm1Value::Object(object) = self.object() {
            if let Some(stage_object) = object.as_stage_object() {
                stage_object.unregister_text_field_bindings(context);
            }
        }

        self.drop_focus(context);

        {
            let mut mc = self.0.write(context.gc_context);
            mc.stop_audio_stream(context);
        }

        context
            .audio_manager
            .stop_sounds_with_display_object(context.audio, (*self).into());

        // If this clip is currently pending removal, then it unload event will have already been dispatched
        if !self.avm1_pending_removal() {
            self.event_dispatch(context, ClipEvent::Unload);
        }

        self.set_avm1_removed(context.gc_context, true);
    }

    fn loader_info(&self) -> Option<Avm2Object<'gc>> {
        self.0.read().static_data.loader_info
    }

    fn allow_as_mask(&self) -> bool {
        !self.is_empty()
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for MovieClip<'gc> {
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>> {
        Ref::map(self.0.read(), |this| &this.container)
    }

    fn raw_container_mut(&self, gc_context: &Mutation<'gc>) -> RefMut<'_, ChildContainer<'gc>> {
        RefMut::map(self.0.write(gc_context), |this| &mut this.container)
    }

    fn is_tab_children_avm1(&self, context: &mut UpdateContext<'gc>) -> bool {
        self.get_avm1_boolean_property(context, "tabChildren", |_| true)
    }
}

impl<'gc> TInteractiveObject<'gc> for MovieClip<'gc> {
    fn raw_interactive(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn raw_interactive_mut(&self, mc: &Mutation<'gc>) -> RefMut<InteractiveObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        context: &mut UpdateContext<'gc>,
        event: ClipEvent,
    ) -> ClipEventResult {
        if event.is_button_event() {
            if !self.visible() && !matches!(event, ClipEvent::ReleaseOutside) {
                return ClipEventResult::NotHandled;
            }

            if !self.enabled(context) && !matches!(event, ClipEvent::KeyPress { .. }) {
                return ClipEventResult::NotHandled;
            }
        }

        ClipEventResult::Handled
    }

    fn event_dispatch(
        self,
        context: &mut UpdateContext<'gc>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        let frame_name = match event {
            ClipEvent::RollOut { .. } | ClipEvent::ReleaseOutside => Some(WStr::from_units(b"_up")),
            ClipEvent::RollOver { .. } | ClipEvent::Release { .. } | ClipEvent::DragOut { .. } => {
                Some(WStr::from_units(b"_over"))
            }
            ClipEvent::Press { .. } | ClipEvent::DragOver { .. } => {
                Some(WStr::from_units(b"_down"))
            }
            _ => None,
        };

        if let Some(frame_name) = frame_name {
            if let Some(frame_number) = self.frame_label_to_number(frame_name, context) {
                if self.is_button_mode(context) {
                    self.goto_frame(context, frame_number, true);
                }
            }
        }

        let mut handled = ClipEventResult::NotHandled;
        let read = self.0.read();
        if let Some(AvmObject::Avm1(object)) = read.object {
            let swf_version = read.movie().version();
            if swf_version >= 5 {
                if let Some(flag) = event.flag() {
                    for event_handler in read
                        .clip_event_handlers
                        .iter()
                        .filter(|handler| handler.events.contains(flag))
                    {
                        // KeyPress event must have matching key code.
                        if let ClipEvent::KeyPress { key_code } = event {
                            if key_code == event_handler.key_code {
                                // KeyPress events are consumed by a single instance.
                                handled = ClipEventResult::Handled;
                            } else {
                                continue;
                            }
                        }

                        context.action_queue.queue_action(
                            self.into(),
                            ActionType::Normal {
                                bytecode: event_handler.action_data.clone(),
                            },
                            event == ClipEvent::Unload,
                        );
                    }
                }

                // Queue ActionScript-defined event handlers after the SWF defined ones.
                // (e.g., clip.onEnterFrame = foo).
                if self.should_fire_event_handlers(context, event) {
                    if let Some(name) = event.method_name() {
                        context.action_queue.queue_action(
                            self.into(),
                            ActionType::Method {
                                object,
                                name,
                                args: vec![],
                            },
                            event == ClipEvent::Unload,
                        );
                    }
                }
            }
        }

        handled
    }

    fn mouse_pick_avm1(
        &self,
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        // Don't do anything if run in an AVM2 context.
        if self.movie().is_action_script_3() {
            return None;
        }

        if self.visible() {
            let this: InteractiveObject<'gc> = (*self).into();
            let local_matrix = self.global_to_local_matrix()?;

            if let Some(masker) = self.masker() {
                // FIXME - should this really use `SKIP_INVISIBLE`? Avm2 doesn't.
                if !masker.hit_test_shape(context, point, HitTestOptions::SKIP_INVISIBLE) {
                    return None;
                }
            }

            // In AVM2, mouse_enabled should only impact the ability to select the current clip
            // but it should still be possible to select any children where child.mouse_enabled() is
            // true.
            // InteractiveObject.mouseEnabled:
            // "Any children of this instance on the display list are not affected."
            if self.mouse_enabled() && self.world_bounds().contains(point) {
                // This MovieClip operates in "button mode" if it has a mouse handler,
                // either via on(..) or via property mc.onRelease, etc.
                let is_button_mode = self.is_button_mode(context);

                if is_button_mode {
                    let mut options = HitTestOptions::SKIP_INVISIBLE;
                    options.set(HitTestOptions::SKIP_MASK, self.maskee().is_none());
                    if self.hit_test_shape(context, point, options) {
                        return Some(this);
                    }
                }
            }

            // Maybe we could skip recursing down at all if !world_bounds.contains(point),
            // but a child button can have an invisible hit area outside the parent's bounds.
            let mut hit_depth = 0;
            let mut result = None;
            let mut options = HitTestOptions::SKIP_INVISIBLE;
            options.set(HitTestOptions::SKIP_MASK, self.maskee().is_none());
            // AVM2 allows movie clips to receive mouse events without explicitly enabling button mode.
            let check_non_interactive = !require_button_mode;

            for child in self.iter_render_list().rev() {
                if child.clip_depth() > 0 {
                    if result.is_some() && child.clip_depth() >= hit_depth {
                        if child.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK) {
                            return result;
                        } else {
                            result = None;
                        }
                    }
                } else if result.is_none() {
                    if let Some(child) = child.as_interactive() {
                        result = if !child.as_displayobject().movie().is_action_script_3() {
                            child.mouse_pick_avm1(context, point, require_button_mode)
                        } else {
                            let avm2_result =
                                child.mouse_pick_avm2(context, point, require_button_mode);
                            if let Avm2MousePick::Hit(result) = avm2_result {
                                Some(result)
                            } else {
                                None
                            }
                        }
                    } else if check_non_interactive
                        && self.mouse_enabled()
                        && child.hit_test_shape(context, point, options)
                    {
                        result = Some(this);
                    }

                    if result.is_some() {
                        hit_depth = child.depth();
                    }
                }
            }

            if result.is_some() {
                return result;
            }

            // Check drawing, because this selects the current clip, it must have mouse enabled
            if self.mouse_enabled() && check_non_interactive {
                let point = local_matrix * point;
                if self.0.read().drawing.hit_test(point, &local_matrix) {
                    return Some(this);
                }
            }
        }

        None
    }

    fn mouse_pick_avm2(
        &self,
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        // Don't do anything if run in an AVM1 context.
        if !self.movie().is_action_script_3() {
            return Avm2MousePick::Miss;
        }

        if self.visible() {
            let this: InteractiveObject<'gc> = (*self).into();
            let Some(local_matrix) = self.global_to_local_matrix() else {
                return Avm2MousePick::Miss;
            };

            if let Some(masker) = self.masker() {
                if !masker.hit_test_shape(context, point, HitTestOptions::empty()) {
                    return Avm2MousePick::Miss;
                }
            }

            if self.maskee().is_some() {
                // If we're masking another object, we can't be hit.
                return Avm2MousePick::Miss;
            }

            // Maybe we could skip recursing down at all if !world_bounds.contains(point),
            // but a child button can have an invisible hit area outside the parent's bounds.
            let mut options = HitTestOptions::SKIP_INVISIBLE;
            options.set(HitTestOptions::SKIP_MASK, self.maskee().is_none());

            let mut found_propagate = None;

            let mut clip_layers = self
                .iter_render_list()
                .flat_map(|child| {
                    if child.clip_depth() > 0 {
                        // Note - we intentionally use 'child.depth()' here insteado
                        // of the position in the render list - this matches Flash's
                        // behavior. The 'depth' value comes from the PlaceObject tag
                        Some((child, (child.depth() + 1)..=(child.clip_depth())))
                    } else {
                        None
                    }
                })
                .rev()
                .peekable();

            // Interactive children run first, followed by non-interactive children.
            // Depth is considered within each group.

            let interactive = self
                .iter_render_list()
                .rev()
                .filter(|child| child.as_interactive().is_some());
            let non_interactive = self
                .iter_render_list()
                .rev()
                .filter(|child| child.as_interactive().is_none());

            for child in interactive.into_iter().chain(non_interactive) {
                // Mask children are not clickable
                if child.clip_depth() > 0 || child.maskee().is_some() {
                    continue;
                }

                let mut res = if let Some(child) = child.as_interactive() {
                    if child.as_displayobject().movie().is_action_script_3() {
                        child.mouse_pick_avm2(context, point, require_button_mode)
                    } else {
                        let avm1_result =
                            child.mouse_pick_avm1(context, point, require_button_mode);
                        if let Some(result) = avm1_result {
                            Avm2MousePick::Hit(result)
                        } else {
                            Avm2MousePick::Miss
                        }
                    }
                } else if child.as_interactive().is_none()
                    && child.hit_test_shape(context, point, options)
                {
                    if self.mouse_enabled() {
                        Avm2MousePick::Hit(this)
                    } else {
                        Avm2MousePick::PropagateToParent
                    }
                } else {
                    Avm2MousePick::Miss
                };

                while let Some((clip, clip_range)) = clip_layers.peek() {
                    // This clip layer no longer applies to the remaining children (which all have lower depth values).
                    // This is a rare case where we actually use 'child.depth()' in AVM2 - child.depth()
                    // gets set from a PlaceObject tag, and may be *greater* than position in the render list.
                    if *clip_range.start() > child.depth() {
                        clip_layers.next();
                        continue;
                    }

                    if clip_range.contains(&child.depth()) {
                        // If the clip layer applies to the current child, check if the child is masked by the clip.
                        // If the position isn't within the mask region, then treat this as a miss unconditionally.
                        // We'll continue the outer loop over the children, as another child may be hit.
                        if !clip.hit_test_shape(context, point, options) {
                            res = Avm2MousePick::Miss;
                        }
                    }
                    break;
                }

                match res {
                    Avm2MousePick::Hit(_) => {
                        return res.combine_with_parent((*self).into());
                    }
                    Avm2MousePick::PropagateToParent => {
                        found_propagate = Some(res);
                    }
                    Avm2MousePick::Miss => {}
                }
            }

            // A 'propagated' event from a child seems to have lower 'priority' than anything else.
            if let Some(propagate) = found_propagate {
                return propagate.combine_with_parent((*self).into());
            }

            // Check drawing, because this selects the current clip, it must have mouse enabled
            if self.world_bounds().contains(point) {
                let point = local_matrix * point;

                if self.0.read().drawing.hit_test(point, &local_matrix) {
                    return if self.mouse_enabled() {
                        Avm2MousePick::Hit((*self).into())
                    } else {
                        Avm2MousePick::PropagateToParent
                    };
                }
            }
        }

        Avm2MousePick::Miss
    }

    fn mouse_cursor(self, context: &mut UpdateContext<'gc>) -> MouseCursor {
        if self.is_button_mode(context) && self.use_hand_cursor(context) && self.enabled(context) {
            MouseCursor::Hand
        } else {
            MouseCursor::Arrow
        }
    }

    fn is_focusable(&self, context: &mut UpdateContext<'gc>) -> bool {
        if self.is_root() {
            false
        } else if self.is_button_mode(context) {
            true
        } else {
            self.get_avm1_boolean_property(context, "focusEnabled", |_| false)
        }
    }

    fn is_highlightable(&self, context: &mut UpdateContext<'gc>) -> bool {
        // Root movie clips are not highlightable.
        // This applies only to AVM2, as in AVM1 they are also not focusable.
        !self.is_root() && self.is_highlight_enabled(context)
    }

    fn is_tabbable(&self, context: &mut UpdateContext<'gc>) -> bool {
        if self.is_root() {
            // Root movie clips are never tabbable.
            return false;
        }
        self.tab_enabled(context)
    }

    fn tab_enabled_default(&self, context: &mut UpdateContext<'gc>) -> bool {
        if self.is_button_mode(context) {
            return true;
        }

        let is_avm1 = !self.movie().is_action_script_3();
        if is_avm1 && self.tab_index().is_some() {
            return true;
        }

        false
    }
}

impl<'gc> MovieClipData<'gc> {
    fn id(&self) -> CharacterId {
        self.static_data.id
    }

    fn current_frame(&self) -> FrameNumber {
        self.current_frame
    }

    fn total_frames(&self) -> FrameNumber {
        self.static_data.total_frames
    }

    fn frames_loaded(&self) -> i32 {
        (self.static_data.preload_progress.read().cur_preload_frame) as i32 - 1
    }

    fn playing(&self) -> bool {
        self.flags.contains(MovieClipFlags::PLAYING)
    }

    fn set_playing(&mut self, value: bool) {
        self.flags.set(MovieClipFlags::PLAYING, value);
    }

    fn programmatically_played(&self) -> bool {
        self.flags.contains(MovieClipFlags::PROGRAMMATICALLY_PLAYED)
    }

    fn set_programmatically_played(&mut self) {
        self.flags |= MovieClipFlags::PROGRAMMATICALLY_PLAYED;
    }

    fn loop_queued(&self) -> bool {
        self.flags.contains(MovieClipFlags::LOOP_QUEUED)
    }

    fn set_loop_queued(&mut self) {
        self.flags |= MovieClipFlags::LOOP_QUEUED;
    }

    fn unset_loop_queued(&mut self) {
        self.flags.remove(MovieClipFlags::LOOP_QUEUED);
    }

    fn play(&mut self) {
        // Can only play clips with multiple frames.
        if self.total_frames() > 1 {
            self.set_playing(true);
        }
    }

    fn stop(&mut self, context: &mut UpdateContext<'gc>) {
        self.set_playing(false);
        self.stop_audio_stream(context);
    }

    fn tag_stream_len(&self) -> usize {
        self.static_data.swf.end - self.static_data.swf.start
    }

    /// Handles a PlaceObject tag when running a goto action.
    #[inline]
    fn goto_place_object<'a>(
        &mut self,
        reader: &mut SwfStream<'a>,
        version: u8,
        goto_commands: &mut Vec<GotoPlaceObject<'a>>,
        is_rewind: bool,
        index: usize,
    ) -> Result<(), Error> {
        let tag_start =
            reader.get_ref().as_ptr() as u64 - self.static_data.swf.as_ref().as_ptr() as u64;
        let place_object = if version == 1 {
            reader.read_place_object()
        } else {
            reader.read_place_object_2_or_3(version)
        }?;

        // We merge the deltas from this PlaceObject with the previous command.
        let depth: Depth = place_object.depth.into();
        let mut goto_place = GotoPlaceObject::new(
            self.current_frame(),
            place_object,
            is_rewind,
            index,
            tag_start,
            version,
        );
        if let Some(i) = goto_commands.iter().position(|o| o.depth() == depth) {
            goto_commands[i].merge(&mut goto_place);
        } else {
            goto_commands.push(goto_place);
        }

        Ok(())
    }

    pub fn clip_event_handlers(&self) -> &[ClipEventHandler] {
        &self.clip_event_handlers
    }

    pub fn set_clip_event_handlers(&mut self, event_handlers: Vec<ClipEventHandler>) {
        let mut all_event_flags = ClipEventFlag::empty();
        for handler in &event_handlers {
            all_event_flags |= handler.events;
        }
        self.clip_event_flags = all_event_flags;
        self.clip_event_handlers = event_handlers;
    }

    fn initialized(&self) -> bool {
        self.flags.contains(MovieClipFlags::INITIALIZED)
    }

    fn set_initialized(&mut self, value: bool) -> bool {
        let ret = self.flags.contains(MovieClipFlags::INITIALIZED);
        self.flags.set(MovieClipFlags::INITIALIZED, value);
        !ret
    }

    /// Stops the audio stream if one is playing.
    fn stop_audio_stream(&mut self, context: &mut UpdateContext<'gc>) {
        if let Some(audio_stream) = self.audio_stream.take() {
            context.stop_sound(audio_stream);
        }
    }

    /// Fetch the avm1 constructor associated with this MovieClip by `Object.registerClass`.
    /// Return `None` if this MovieClip isn't exported, or if no constructor is associated
    /// to its symbol name.
    fn get_registered_avm1_constructor(
        &self,
        context: &mut UpdateContext<'gc>,
    ) -> Option<Avm1Object<'gc>> {
        let symbol_name = self.static_data.exported_name.read();
        let symbol_name = symbol_name.as_ref()?;
        let constructor = context
            .avm1
            .get_registered_constructor(self.movie().version(), *symbol_name)?;
        Some((*constructor).into())
    }

    pub fn movie(&self) -> Arc<SwfMovie> {
        self.static_data.swf.movie.clone()
    }
}

// Preloading of definition tags
impl<'gc, 'a> MovieClipData<'gc> {
    #[inline]
    fn define_bits_lossless(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let define_bits_lossless = reader.read_define_bits_lossless(version)?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(
                define_bits_lossless.id,
                Character::Bitmap {
                    compressed: CompressedBitmap::Lossless(DefineBitsLossless {
                        id: define_bits_lossless.id,
                        format: define_bits_lossless.format,
                        width: define_bits_lossless.width,
                        height: define_bits_lossless.height,
                        version: define_bits_lossless.version,
                        data: Cow::Owned(define_bits_lossless.data.into_owned()),
                    }),
                    handle: RefCell::new(None),
                    avm2_bitmapdata_class: GcCell::new(context.gc_context, BitmapClass::NoSubclass),
                },
            );
        Ok(())
    }

    #[inline]
    fn define_scaling_grid(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let rect = reader.read_rectangle()?;
        let library = context.library.library_for_movie_mut(self.movie());
        if let Some(character) = library.character_by_id(id) {
            if let Character::MovieClip(clip) = character {
                clip.set_scaling_grid(context.gc_context, rect);
            } else {
                tracing::warn!("DefineScalingGrid for invalid ID {}", id);
            }
        }
        Ok(())
    }

    #[inline]
    fn define_morph_shape(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let movie = self.movie();
        let tag = reader.read_define_morph_shape(version)?;
        let id = tag.id;
        let morph_shape = MorphShape::from_swf_tag(context.gc_context, tag, movie.clone());
        context
            .library
            .library_for_movie_mut(movie)
            .register_character(id, Character::MorphShape(morph_shape));
        Ok(())
    }

    #[inline]
    fn define_shape(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let movie = self.movie();
        let swf_shape = reader.read_define_shape(version)?;
        let id = swf_shape.id;
        let graphic = Graphic::from_swf_tag(context, swf_shape, movie);
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(id, Character::Graphic(graphic));
        Ok(())
    }

    #[inline]
    fn sound_stream_head(
        &mut self,
        reader: &mut SwfStream<'a>,
        static_data: &mut MovieClipStatic,
        _version: u8,
    ) -> Result<(), Error> {
        static_data.audio_stream_info = Some(reader.read_sound_stream_head()?);
        Ok(())
    }

    fn csm_text_settings(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let settings = reader.read_csm_text_settings()?;
        let library = context.library.library_for_movie_mut(self.movie());
        match library.character_by_id(settings.id) {
            Some(Character::Text(text)) => {
                text.set_render_settings(context.gc_context, settings.into());
            }
            Some(Character::EditText(edit_text)) => {
                edit_text.set_render_settings(context.gc_context, settings.into());
            }
            Some(_) => {
                tracing::warn!(
                    "Tried to apply CSMTextSettings to non-text character ID {}",
                    settings.id
                );
            }
            None => {
                tracing::warn!(
                    "Tried to apply CSMTextSettings to unregistered character ID {}",
                    settings.id
                );
            }
        }
        Ok(())
    }

    #[inline]
    fn preload_video_frame(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream,
    ) -> Result<(), Error> {
        let vframe = reader.read_video_frame()?;
        let library = context.library.library_for_movie_mut(self.movie());
        match library.character_by_id(vframe.stream_id) {
            Some(Character::Video(mut v)) => {
                v.preload_swf_frame(vframe, context);

                Ok(())
            }
            _ => Err(Error::PreloadVideoIntoInvalidCharacter(vframe.stream_id)),
        }
    }

    #[inline]
    fn define_bits(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let jpeg_data = reader.read_slice_to_end();
        let jpeg_tables = context
            .library
            .library_for_movie_mut(self.movie())
            .jpeg_tables();
        let jpeg_data =
            ruffle_render::utils::glue_tables_to_jpeg(jpeg_data, jpeg_tables).into_owned();
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(&jpeg_data)?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(
                id,
                Character::Bitmap {
                    compressed: CompressedBitmap::Jpeg {
                        data: jpeg_data,
                        alpha: None,
                        width,
                        height,
                    },
                    handle: RefCell::new(None),
                    avm2_bitmapdata_class: GcCell::new(context.gc_context, BitmapClass::NoSubclass),
                },
            );
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_2(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let jpeg_data = reader.read_slice_to_end();
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(jpeg_data)?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(
                id,
                Character::Bitmap {
                    compressed: CompressedBitmap::Jpeg {
                        data: jpeg_data.to_vec(),
                        alpha: None,
                        width,
                        height,
                    },
                    handle: RefCell::new(None),
                    avm2_bitmapdata_class: GcCell::new(context.gc_context, BitmapClass::NoSubclass),
                },
            );
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_3_or_4(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let jpeg_len = reader.read_u32()? as usize;
        if version == 4 {
            let _deblocking = reader.read_u16()?;
        }
        let jpeg_data = reader.read_slice(jpeg_len)?;
        let alpha_data = reader.read_slice_to_end();
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(jpeg_data)?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(
                id,
                Character::Bitmap {
                    compressed: CompressedBitmap::Jpeg {
                        data: jpeg_data.to_owned(),
                        alpha: Some(alpha_data.to_owned()),
                        width,
                        height,
                    },
                    handle: RefCell::new(None),
                    avm2_bitmapdata_class: GcCell::new(context.gc_context, BitmapClass::NoSubclass),
                },
            );
        Ok(())
    }

    #[inline]
    fn define_button_1(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let swf_button = reader.read_define_button_1()?;

        self.define_button_any(context, swf_button)
    }

    #[inline]
    fn define_button_2(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let swf_button = reader.read_define_button_2()?;

        self.define_button_any(context, swf_button)
    }

    #[inline]
    fn define_button_any(
        &mut self,
        context: &mut UpdateContext<'gc>,
        swf_button: swf::Button<'a>,
    ) -> Result<(), Error> {
        let movie = self.movie();
        let button = if movie.is_action_script_3() {
            Character::Avm2Button(Avm2Button::from_swf_tag(
                &swf_button,
                &self.static_data.swf,
                context,
                true,
            ))
        } else {
            Character::Avm1Button(Avm1Button::from_swf_tag(
                &swf_button,
                &self.static_data.swf,
                context.gc_context,
            ))
        };
        let library = context.library.library_for_movie_mut(movie);
        library.register_character(swf_button.id, button);
        Ok(())
    }

    #[inline]
    fn define_button_cxform(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let button_colors = reader.read_define_button_cxform()?;
        match context
            .library
            .library_for_movie_mut(self.movie())
            .character_by_id(button_colors.id)
        {
            Some(Character::Avm1Button(button)) => {
                button.set_colors(&button_colors.color_transforms);
            }
            Some(_) => {
                tracing::warn!(
                    "DefineButtonCxform: Tried to apply on non-button ID {}",
                    button_colors.id
                );
            }
            None => {
                tracing::warn!(
                    "DefineButtonCxform: Character ID {} doesn't exist",
                    button_colors.id
                );
            }
        }
        Ok(())
    }

    #[inline]
    fn define_button_sound(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let button_sounds = reader.read_define_button_sound()?;
        match context
            .library
            .library_for_movie_mut(self.movie())
            .character_by_id(button_sounds.id)
        {
            Some(Character::Avm1Button(button)) => {
                button.set_sounds(button_sounds);
            }
            Some(Character::Avm2Button(button)) => {
                button.set_sounds(button_sounds);
            }
            Some(_) => {
                tracing::warn!(
                    "DefineButtonSound: Tried to apply on non-button ID {}",
                    button_sounds.id
                );
            }
            None => {
                tracing::warn!(
                    "DefineButtonSound: Character ID {} doesn't exist",
                    button_sounds.id
                );
            }
        }
        Ok(())
    }

    /// Defines a dynamic text field character.
    #[inline]
    fn define_edit_text(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let swf_edit_text = reader.read_define_edit_text()?;
        let edit_text = EditText::from_swf_tag(context, self.movie(), swf_edit_text);
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(edit_text.id(), Character::EditText(edit_text));
        Ok(())
    }

    #[inline]
    fn define_font_1(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_1()?;
        let glyphs = font
            .glyphs
            .into_iter()
            .map(|g| swf::Glyph {
                shape_records: g,
                code: 0,
                advance: 0,
                bounds: None,
            })
            .collect::<Vec<_>>();

        let font_id = font.id;
        let font = swf::Font {
            id: font.id,
            version: 0,
            name: "".into(),
            glyphs,
            language: swf::Language::Unknown,
            layout: None,
            flags: swf::FontFlag::empty(),
        };
        let font_object = Font::from_swf_tag(
            context.gc_context,
            context.renderer,
            font,
            reader.encoding(),
            FontType::Embedded,
        );
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(font_id, Character::Font(font_object));
        Ok(())
    }

    #[inline]
    fn define_font_2(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_2(2)?;
        let font_id = font.id;
        let font_object = Font::from_swf_tag(
            context.gc_context,
            context.renderer,
            font,
            reader.encoding(),
            FontType::Embedded,
        );
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(font_id, Character::Font(font_object));
        Ok(())
    }

    #[inline]
    fn define_font_3(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_2(3)?;
        let font_id = font.id;
        let font_object = Font::from_swf_tag(
            context.gc_context,
            context.renderer,
            font,
            reader.encoding(),
            FontType::Embedded,
        );
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(font_id, Character::Font(font_object));

        Ok(())
    }

    #[inline]
    fn define_font_4(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_4()?;
        let font_id = font.id;
        let font_object = Font::from_font4_tag(context.gc_context, font, reader.encoding())?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(font_id, Character::Font(font_object));
        Ok(())
    }

    #[inline]
    fn define_sound(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let sound = reader.read_define_sound()?;
        if let Ok(handle) = context.audio.register_sound(&sound) {
            context
                .library
                .library_for_movie_mut(self.movie())
                .register_character(sound.id, Character::Sound(handle));
        } else {
            tracing::error!(
                "MovieClip::define_sound: Unable to register sound ID {}",
                sound.id
            );
        }
        Ok(())
    }

    #[inline]
    fn define_video_stream(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream,
    ) -> Result<(), Error> {
        let streamdef = reader.read_define_video_stream()?;
        let id = streamdef.id;
        let video = Video::from_swf_tag(self.movie(), streamdef, context.gc_context);
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(id, Character::Video(video));
        Ok(())
    }

    fn define_sprite(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        tag_len: usize,
        chunk_limit: &mut ExecutionLimit,
    ) -> DecodeResult {
        let start = reader.as_slice();
        let id = reader.read_character_id()?;
        let num_frames = reader.read_u16()?;
        let num_read = reader.pos(start);

        let movie_clip = MovieClip::new_with_data(
            context.gc_context,
            id,
            self.static_data
                .swf
                .resize_to_reader(reader, tag_len - num_read),
            num_frames,
        );

        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(id, Character::MovieClip(movie_clip));

        self.static_data
            .preload_progress
            .write(context.gc_context)
            .cur_preload_symbol = Some(id);

        let should_exit = chunk_limit.did_ops_breach_limit(context, 4);
        if should_exit {
            return Ok(ControlFlow::Exit);
        }

        if movie_clip.preload(context, chunk_limit) {
            self.static_data
                .preload_progress
                .write(context.gc_context)
                .cur_preload_symbol = None;

            Ok(ControlFlow::Continue)
        } else {
            Ok(ControlFlow::Exit)
        }
    }

    #[inline]
    fn define_text(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let text = reader.read_define_text(version)?;
        let text_object = Text::from_swf_tag(context, self.movie(), &text);
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(text.id, Character::Text(text_object));
        Ok(())
    }

    #[inline]
    fn define_binary_data(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let tag_data = reader.read_define_binary_data()?;
        let binary_data = BinaryData::from_swf_tag(self.movie(), &tag_data);
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(tag_data.id, Character::BinaryData(binary_data));
        Ok(())
    }

    #[inline]
    fn get_exported_from_importer(
        &self,
        context: &mut UpdateContext<'gc>,
        importer_movie: Arc<SwfMovie>,
    ) -> HashMap<AvmString<'gc>, (CharacterId, Character<'gc>)> {
        let mut map: HashMap<AvmString<'gc>, (CharacterId, Character<'gc>)> = HashMap::new();
        let library = context.library.library_for_movie_mut(importer_movie);

        library.export_characters().iter().for_each(|(name, id)| {
            let character = library.character_by_id(*id).unwrap();
            map.insert(name, (*id, character.clone()));
        });
        map
    }

    #[inline]
    fn import_exports_of_importer(&mut self, context: &mut UpdateContext<'gc>) {
        if let Some(importer_movie) = self.importer_movie.as_ref() {
            let exported_from_importer =
                { self.get_exported_from_importer(context, importer_movie.clone()) };

            let self_library = context.library.library_for_movie_mut(self.movie().clone());

            exported_from_importer
                .iter()
                .for_each(|(name, (id, character))| {
                    let id = *id;
                    if self_library.character_by_id(id).is_none() {
                        self_library.register_character(id, character.clone());
                        self_library.register_export(id, *name);
                    }
                });
        }
    }

    #[inline]
    fn import_assets(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        chunk_limit: &mut ExecutionLimit,
    ) -> Result<(), Error> {
        let import_assets = reader.read_import_assets()?;
        self.import_assets_load(
            context,
            reader,
            import_assets.0,
            import_assets.1,
            chunk_limit,
        )
    }

    #[inline]
    fn import_assets_2(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        chunk_limit: &mut ExecutionLimit,
    ) -> Result<(), Error> {
        let import_assets = reader.read_import_assets_2()?;
        self.import_assets_load(
            context,
            reader,
            import_assets.0,
            import_assets.1,
            chunk_limit,
        )
    }

    #[inline]
    fn import_assets_load(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        url: &swf::SwfStr,
        exported_assets: Vec<swf::ExportedAsset>,
        _chunk_limit: &mut ExecutionLimit,
    ) -> Result<(), Error> {
        let library = context.library.library_for_movie_mut(self.movie());

        let asset_url = url.to_string_lossy(UTF_8);

        let request = Request::get(asset_url);

        for asset in exported_assets {
            let name = asset.name.decode(reader.encoding());
            let name = AvmString::new(context.gc_context, name);
            let id = asset.id;
            tracing::debug!("Importing asset: {} (ID: {})", name, id);

            library.register_import(name, id);
        }

        let player = context.player.clone();
        let fut = LoadManager::load_asset_movie(player, request, self.movie());

        context.navigator.spawn_future(fut);

        Ok(())
    }

    #[inline]
    fn script_limits(
        &mut self,
        reader: &mut SwfStream<'a>,
        avm: &mut Avm1<'gc>,
    ) -> Result<(), Error> {
        let max_recursion_depth = reader.read_u16()?;
        let _timeout_in_seconds = reader.read_u16()?;

        avm.set_max_recursion_depth(max_recursion_depth);

        Ok(())
    }

    #[inline]
    fn get_registered_character_by_id(
        &mut self,
        context: &mut UpdateContext<'gc>,
        id: CharacterId,
    ) -> Option<Character<'gc>> {
        let library_for_movie = context.library.library_for_movie(self.movie());

        if let Some(library) = library_for_movie {
            if let Some(character) = library.character_by_id(id) {
                return Some(character.clone());
            }
        }
        None
    }

    fn register_export(
        &mut self,
        context: &mut UpdateContext<'gc>,
        id: CharacterId,
        name: &AvmString<'gc>,
        movie: Arc<SwfMovie>,
    ) {
        let library = context.library.library_for_movie_mut(movie);
        library.register_export(id, *name);

        // TODO: do other types of Character need to know their exported name?
        if let Some(character) = library.character_by_id(id) {
            if let Character::MovieClip(movie_clip) = character {
                *movie_clip
                    .0
                    .read()
                    .static_data
                    .exported_name
                    .write(context.gc_context) = Some(*name);
            } else {
                // This is fairly common, don't log anything here
            }
        } else {
            tracing::warn!(
                "Can't register export {}: Character ID {} doesn't exist",
                name,
                id,
            );
        }
    }

    #[inline]
    fn export_assets(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let exports = reader.read_export_assets()?;
        for export in exports {
            let name = export.name.decode(reader.encoding());
            let name = AvmString::new(context.gc_context, name);

            if let Some(character) = self.get_registered_character_by_id(context, export.id) {
                self.register_export(context, export.id, &name, self.movie());
                tracing::debug!("register_export asset: {} (ID: {})", name, export.id);

                if self.importer_movie.is_some() {
                    let parent = self.importer_movie.as_ref().unwrap().clone();
                    let parent_library = context.library.library_for_movie_mut(parent.clone());

                    if let Some(id) = parent_library.character_id_by_import_name(name) {
                        parent_library.register_character(id, character);

                        self.register_export(context, id, &name, parent);
                        tracing::debug!(
                            "Registering parent asset: {} (Parent ID: {})(ID: {})",
                            name,
                            id,
                            export.id
                        );
                    }
                }
            } else {
                tracing::error!(
                    "Export asset: {} (ID: {}) not found in library",
                    name,
                    export.id
                );
            }
        }
        Ok(())
    }

    #[inline]
    fn frame_label(
        &mut self,
        reader: &mut SwfStream<'a>,
        cur_frame: FrameNumber,
        static_data: &mut MovieClipStatic<'gc>,
        _context: &UpdateContext<'gc>,
    ) -> Result<(), Error> {
        // This tag is ignored if scene labels exist.
        if !static_data.scene_labels.is_empty() {
            return Ok(());
        }

        let frame_label = reader.read_frame_label()?;
        let mut label = frame_label.label.decode(reader.encoding()).into_owned();

        // In AVM1, frame labels are case insensitive (ASCII), but in AVM2 they are case sensitive.
        if !self.movie().is_action_script_3() {
            label.make_ascii_lowercase();
        }

        static_data.frame_labels.push((cur_frame, label.clone()));
        if let std::collections::hash_map::Entry::Vacant(v) =
            static_data.frame_labels_map.entry(label)
        {
            v.insert(cur_frame);
        } else {
            tracing::warn!("Movie clip {}: Duplicated frame label", self.id());
        }
        Ok(())
    }

    #[inline]
    fn jpeg_tables(
        &mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let jpeg_data = reader.read_slice_to_end();
        context
            .library
            .library_for_movie_mut(self.movie())
            .set_jpeg_tables(jpeg_data);
        Ok(())
    }

    #[inline]
    #[cfg(not(feature = "timeline_debug"))]
    fn show_frame(
        &mut self,
        _reader: &mut SwfStream<'a>,
        _tag_len: usize,
        cur_frame: &mut FrameNumber,
        _start_pos: &mut u64,
    ) -> Result<(), Error> {
        *cur_frame += 1;
        Ok(())
    }

    #[inline]
    #[cfg(feature = "timeline_debug")]
    fn show_frame(
        &mut self,
        reader: &mut SwfStream<'a>,
        tag_len: usize,
        cur_frame: &mut FrameNumber,
        start_pos: &mut u64,
    ) -> Result<(), Error> {
        let tag_stream_start = self.static_data.swf.as_ref().as_ptr() as u64;
        let end_pos = reader.get_ref().as_ptr() as u64 - tag_stream_start;

        // We add tag_len because the reader position doesn't take it into
        // account. Strictly speaking ShowFrame should not have tag data, but
        // who *knows* what weird obfuscation hacks people have done with it.
        self.tag_frame_boundaries
            .insert(*cur_frame, (*start_pos, end_pos + tag_len as u64));

        *start_pos = end_pos;
        *cur_frame += 1;

        Ok(())
    }
}

// Control tags
impl<'gc, 'a> MovieClip<'gc> {
    #[inline]
    fn do_action(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        tag_len: usize,
    ) -> Result<(), Error> {
        if self.movie().is_action_script_3() {
            tracing::warn!("DoAction tag in AVM2 movie");
            return Ok(());
        }

        // Queue the actions.
        let slice = self
            .0
            .read()
            .static_data
            .swf
            .resize_to_reader(reader, tag_len);
        if !slice.is_empty() {
            context.action_queue.queue_action(
                self.into(),
                ActionType::Normal { bytecode: slice },
                false,
            );
        }
        Ok(())
    }

    /// Handles a DoAbc or DoAbc2 tag
    fn preload_bytecode_tag(
        self,
        tag_code: TagCode,
        reader: &mut SwfStream<'a>,
        context: &mut UpdateContext<'gc>,
        cur_frame: FrameNumber,
        static_data: &mut MovieClipStatic<'gc>,
    ) -> Result<(), Error> {
        let abc = match tag_code {
            TagCode::DoAbc | TagCode::DoAbc2 => static_data
                .swf
                .resize_to_reader(reader, reader.as_slice().len()),
            _ => unreachable!(),
        };
        // If we got an eager script (which happens for non-lazy DoAbc2 tags).
        // we store it for later. It will be run the first time we execute this frame
        // (for any instance of this MovieClip) in `run_eager_script_and_symbol`
        static_data
            .abc_tags
            .write(context.gc_context)
            .entry(cur_frame)
            .or_default()
            .push(AbcCodeAndTag { tag_code, abc });
        Ok(())
    }

    fn preload_symbol_class(
        self,
        reader: &mut SwfStream<'a>,
        context: &mut UpdateContext<'gc>,
        cur_frame: FrameNumber,
        static_data: &mut MovieClipStatic<'gc>,
    ) -> Result<(), Error> {
        let mut symbolclass_names = static_data.symbolclass_names.write(context.gc_context);
        let symbolclass_names = symbolclass_names.entry(cur_frame).or_default();
        let num_symbols = reader.read_u16()?;

        for _ in 0..num_symbols {
            let id = reader.read_u16()?;
            let class_name = AvmString::new(
                context.gc_context,
                reader.read_str()?.decode(reader.encoding()),
            );

            let name =
                Avm2QName::from_qualified_name(class_name, context.avm2.root_api_version, context);
            // Store the name and symbol with in the global data for this frame. The first time
            // we execute this frame (for any instance of this MovieClip), we will load the symbolclass
            // from `run_eager_script_and_symbol`
            symbolclass_names.push((name, id));
        }
        Ok(())
    }

    // Flash Player handles SymbolClass tags and eager (non-lazy) DoAbc2 tags in an unusual way:
    // During the first time that a given frame is executed:
    // 1. All Abc/DoAbc2 tags have their ABC files parsed and loaded. No script initializers are run yet.
    // 2. All SymbolClass tags are processed in order, triggering ClassObject loading (and the associated
    //    script initializer execution, if it hasn't already been run)
    // 3. All eager (non-lazy) DoAbc/DoAbc2 tags have their *final* script initializer executed.
    //
    // The relative order is preserved between SymbolClass tags and between DoAbc2 tags. However, all
    // of the SymbolClass tags in the frame will run before any of the 'eager' DoAbc2 tags have
    // their final script initializers run.
    //
    // We need to match this behavior exactly, in order for flascc/crossbridge games like 'minidash'
    // to work correctly.
    fn run_abc_and_symbol_tags(
        self,
        context: &mut UpdateContext<'gc>,
        current_frame: FrameNumber,
    ) -> Result<(), Error> {
        let read = self.0.read();
        let tags = read
            .static_data
            .abc_tags
            .write(context.gc_context)
            .remove(&current_frame);
        let mut eager_scripts = Vec::new();
        if let Some(tags) = tags {
            for AbcCodeAndTag { tag_code, abc } in tags {
                let mut reader = abc.read_from(0);
                let eager_script = match tag_code {
                    TagCode::DoAbc => self.do_abc(context, &mut reader)?,
                    TagCode::DoAbc2 => self.do_abc_2(context, &mut reader)?,
                    _ => unreachable!(),
                };
                if let Some(eager_script) = eager_script {
                    eager_scripts.push(eager_script);
                }
            }
        }

        if let Some(symbols) = read
            .static_data
            .symbolclass_names
            .write(context.gc_context)
            .remove(&current_frame)
        {
            let movie = self.movie();
            let mut activation = Avm2Activation::from_nothing(context);
            for (name, id) in symbols {
                let library = activation
                    .context
                    .library
                    .library_for_movie_mut(movie.clone());
                let domain = library.avm2_domain();
                let class_object = domain
                    .get_defined_value(&mut activation, name)
                    .and_then(|v| {
                        v.as_object()
                            .and_then(|o| o.as_class_object())
                            .ok_or_else(|| {
                                format!("Attempted to assign a non-class {name:?} in SymbolClass",)
                                    .into()
                            })
                    });

                match class_object {
                    Ok(class_object) => {
                        activation
                            .context
                            .library
                            .avm2_class_registry_mut()
                            .set_class_symbol(
                                class_object.inner_class_definition(),
                                movie.clone(),
                                id,
                            );

                        let library = activation
                            .context
                            .library
                            .library_for_movie_mut(movie.clone());

                        match library.character_by_id(id) {
                            Some(Character::MovieClip(mc)) => {
                                mc.set_avm2_class(activation.context.gc_context, Some(class_object))
                            }
                            Some(Character::Avm2Button(btn)) => {
                                btn.set_avm2_class(activation.context.gc_context, class_object)
                            }
                            Some(Character::BinaryData(_)) => {}
                            Some(Character::Font(_)) => {}
                            Some(Character::Sound(_)) => {}
                            Some(Character::Bitmap { .. }) => {
                                if let Some(bitmap_class) =
                                    BitmapClass::from_class_object(class_object, activation.context)
                                {
                                    // We need to re-fetch the library and character to satisfy the borrow checker
                                    let library = activation
                                        .context
                                        .library
                                        .library_for_movie_mut(movie.clone());

                                    let Some(Character::Bitmap {
                                        avm2_bitmapdata_class,
                                        ..
                                    }) = library.character_by_id(id)
                                    else {
                                        unreachable!();
                                    };
                                    *avm2_bitmapdata_class.write(activation.context.gc_context) =
                                        bitmap_class;
                                } else {
                                    tracing::error!("Associated class {:?} for symbol {} must extend flash.display.Bitmap or BitmapData, does neither", class_object.inner_class_definition().name(), self.id());
                                }
                            }
                            None => {
                                // Most SWFs use id 0 here, but some obfuscated SWFs can use other invalid IDs.
                                if self.0.read().static_data.avm2_class.read().is_none() {
                                    self.set_avm2_class(
                                        activation.context.gc_context,
                                        Some(class_object),
                                    );
                                }
                            }
                            _ => {
                                tracing::warn!(
                                    "Symbol class {name:?} cannot be assigned to character id {id}",
                                );
                            }
                        }
                    }
                    Err(e) => tracing::error!(
                        "Got AVM2 error {e:?} when attempting to assign symbol class {name:?}",
                    ),
                }
            }
        }
        for script in eager_scripts {
            if let Err(e) = script.globals(context) {
                tracing::error!("Error running eager script: {:?}", e);
            }
        }
        Ok(())
    }

    fn queue_place_object(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let mut write = self.0.write(context.gc_context);
        let tag_start =
            reader.get_ref().as_ptr() as u64 - write.static_data.swf.as_ref().as_ptr() as u64;
        let place_object = if version == 1 {
            reader.read_place_object()
        } else {
            reader.read_place_object_2_or_3(version)
        }?;

        let new_tag = QueuedTag {
            tag_type: QueuedTagAction::Place(version),
            tag_start,
        };
        let bucket = write
            .queued_tags
            .entry(place_object.depth as Depth)
            .or_insert_with(|| QueuedTagList::None);

        bucket.queue_add(new_tag);

        Ok(())
    }

    fn place_object(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let place_object = if version == 1 {
            reader.read_place_object()
        } else {
            reader.read_place_object_2_or_3(version)
        }?;
        use swf::PlaceObjectAction;
        match place_object.action {
            PlaceObjectAction::Place(id) => {
                self.instantiate_child(context, id, place_object.depth.into(), &place_object);
            }
            PlaceObjectAction::Replace(id) => {
                if let Some(child) = self.child_by_depth(place_object.depth.into()) {
                    child.replace_with(context, id);
                    child.apply_place_object(context, &place_object);
                    child.set_place_frame(context.gc_context, self.current_frame());
                }
            }
            PlaceObjectAction::Modify => {
                if let Some(child) = self.child_by_depth(place_object.depth.into()) {
                    child.apply_place_object(context, &place_object);
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn remove_object(
        mut self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let remove_object = if version == 1 {
            reader.read_remove_object_1()
        } else {
            reader.read_remove_object_2()
        }?;

        if let Some(child) = self.child_by_depth(remove_object.depth.into()) {
            if !child.placed_by_script() {
                self.remove_child(context, child);
            } else {
                self.remove_child_from_depth_list(context, child);
            }
        }

        Ok(())
    }

    #[inline]
    fn queue_remove_object(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let mut write = self.0.write(context.gc_context);
        let tag_start =
            reader.get_ref().as_ptr() as u64 - write.static_data.swf.as_ref().as_ptr() as u64;
        let remove_object = if version == 1 {
            reader.read_remove_object_1()
        } else {
            reader.read_remove_object_2()
        }?;

        let new_tag = QueuedTag {
            tag_type: QueuedTagAction::Remove(version),
            tag_start,
        };
        let bucket = write
            .queued_tags
            .entry(remove_object.depth as Depth)
            .or_insert_with(|| QueuedTagList::None);

        bucket.queue_remove(new_tag);

        Ok(())
    }

    #[inline]
    fn set_background_color(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        // Set background color if none set
        // bgcolor attribute on the HTML embed would override this
        // Also note that a loaded child SWF could change background color only
        // if parent SWF is missing SetBackgroundColor tag.
        let background_color = reader.read_rgb()?;
        if context.stage.background_color().is_none() {
            context
                .stage
                .set_background_color(context.gc_context, Some(background_color));
        }
        Ok(())
    }

    #[inline]
    fn sound_stream_block(
        self,
        context: &mut UpdateContext<'gc>,
        _reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let mc = self.0.read();
        if mc.playing() {
            if let (Some(stream_info), None) = (&mc.static_data.audio_stream_info, mc.audio_stream)
            {
                let slice = mc
                    .static_data
                    .swf
                    .to_start_and_end(mc.tag_stream_pos as usize, mc.tag_stream_len());
                let audio_stream =
                    context.start_stream(self, mc.current_frame(), slice, stream_info);
                drop(mc);
                self.0.write(context.gc_context).audio_stream = audio_stream;
            }
        }

        Ok(())
    }

    #[inline]
    fn start_sound_1(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let start_sound = reader.read_start_sound_1()?;
        AudioManager::perform_sound_event(
            self.into(),
            context,
            start_sound.id,
            &start_sound.sound_info,
        );
        Ok(())
    }

    pub fn set_constructing_frame(&self, val: bool, mc: &Mutation<'gc>) {
        self.0
            .write(mc)
            .flags
            .set(MovieClipFlags::RUNNING_CONSTRUCT_FRAME, val);
    }
}

#[derive(Clone)]
pub struct Scene {
    pub name: WString,
    pub start: FrameNumber,
    pub length: FrameNumber,
}

impl Default for Scene {
    fn default() -> Self {
        Scene {
            name: WString::default(),
            start: 1,
            length: u16::MAX,
        }
    }
}

/// The load progress for a given SWF or substream of that SWF.
#[derive(Clone, Collect)]
#[collect(require_static)]
struct PreloadProgress {
    /// The SWF offset to start the next preload chunk from.
    next_preload_chunk: u64,

    /// The current frame being preloaded.
    cur_preload_frame: u16,

    /// The SWF offset that the current frame started in.
    last_frame_start_pos: u64,

    /// The symbol we are currently asynchronously preloading.
    cur_preload_symbol: Option<CharacterId>,
}

impl Default for PreloadProgress {
    fn default() -> Self {
        Self {
            next_preload_chunk: 0,
            cur_preload_frame: 1,
            last_frame_start_pos: 0,
            cur_preload_symbol: None,
        }
    }
}

/// Static data shared between all instances of a movie clip.
#[allow(dead_code)]
#[derive(Clone, Collect)]
#[collect(no_drop)]
struct MovieClipStatic<'gc> {
    id: CharacterId,
    swf: SwfSlice,
    #[collect(require_static)]
    frame_labels: Vec<(FrameNumber, WString)>,
    #[collect(require_static)]
    frame_labels_map: HashMap<WString, FrameNumber>,
    #[collect(require_static)]
    scene_labels: Vec<Scene>,
    #[collect(require_static)]
    scene_labels_map: HashMap<WString, Scene>,
    #[collect(require_static)]
    audio_stream_info: Option<swf::SoundStreamHead>,
    #[collect(require_static)]
    audio_stream_handle: Option<SoundHandle>,
    total_frames: FrameNumber,
    /// The last known symbol name under which this movie clip was exported.
    /// Used for looking up constructors registered with `Object.registerClass`.
    exported_name: GcCell<'gc, Option<AvmString<'gc>>>,
    avm2_class: GcCell<'gc, Option<Avm2ClassObject<'gc>>>,
    /// Only set if this MovieClip is the root movie in an SWF
    /// (either the root SWF initially loaded by the player,
    /// or an SWF dynamically loaded by `Loader`)
    ///
    /// This is always `None` for the AVM1 root movie.
    /// However, it will be set for an AVM1 movie loaded from AVM2
    /// via `Loader`
    loader_info: Option<Avm2Object<'gc>>,

    /// Preload progress for the given clip's tag stream.
    preload_progress: GcCell<'gc, PreloadProgress>,

    // These two maps hold DoAbc/SymbolClass data that was loaded during preloading, but
    // hasn't yet been executed yet. The first time we encounter a frame, we will remove
    // the `Vec` from this map, and process it in `run_eager_script_and_symbol`
    abc_tags: GcCell<'gc, HashMap<FrameNumber, Vec<AbcCodeAndTag>>>,
    symbolclass_names: GcCell<'gc, HashMap<FrameNumber, Vec<(Avm2QName<'gc>, u16)>>>,
}

#[derive(Debug, Collect)]
#[collect(require_static)]
struct AbcCodeAndTag {
    tag_code: TagCode,
    abc: SwfSlice,
}

impl<'gc> MovieClipStatic<'gc> {
    fn empty(movie: Arc<SwfMovie>, gc_context: &Mutation<'gc>) -> Self {
        let s = Self::with_data(0, SwfSlice::empty(movie), 1, None, gc_context);

        s.preload_progress.write(gc_context).cur_preload_frame = s.total_frames + 1;

        s
    }

    fn with_data(
        id: CharacterId,
        swf: SwfSlice,
        total_frames: FrameNumber,
        loader_info: Option<Avm2Object<'gc>>,
        gc_context: &Mutation<'gc>,
    ) -> Self {
        Self {
            id,
            swf,
            total_frames,
            frame_labels: Vec::new(),
            frame_labels_map: HashMap::new(),
            scene_labels: Vec::new(),
            scene_labels_map: HashMap::new(),
            audio_stream_info: None,
            audio_stream_handle: None,
            exported_name: GcCell::new(gc_context, None),
            avm2_class: GcCell::new(gc_context, None),
            loader_info,
            preload_progress: GcCell::new(gc_context, Default::default()),
            abc_tags: GcCell::new(gc_context, Default::default()),
            symbolclass_names: GcCell::new(gc_context, Default::default()),
        }
    }
}

/// Stores the placement settings for display objects during a
/// goto command.
#[derive(Debug)]
struct GotoPlaceObject<'a> {
    /// The frame number that this character was first placed on.
    frame: FrameNumber,
    /// The display properties of the object.
    place_object: swf::PlaceObject<'a>,
    /// Increasing index of this place command, for sorting.
    index: usize,

    /// The location of the *first* SWF tag that created this command.
    ///
    /// NOTE: Only intended to be used in looping gotos, where tag merging is
    /// not possible and we want to add children after the goto completes.
    tag_start: u64,

    /// The version of the PlaceObject tag at `tag_start`.
    version: u8,
}

impl<'a> GotoPlaceObject<'a> {
    fn new(
        frame: FrameNumber,
        mut place_object: swf::PlaceObject<'a>,
        is_rewind: bool,
        index: usize,
        tag_start: u64,
        version: u8,
    ) -> Self {
        if is_rewind {
            if let swf::PlaceObjectAction::Place(_) = place_object.action {
                if place_object.matrix.is_none() {
                    place_object.matrix = Some(Default::default());
                }
                if place_object.color_transform.is_none() {
                    place_object.color_transform = Some(Default::default());
                }
                if place_object.ratio.is_none() {
                    place_object.ratio = Some(Default::default());
                }
                if place_object.blend_mode.is_none() {
                    place_object.blend_mode = Some(Default::default());
                }
                if place_object.is_bitmap_cached.is_none() {
                    place_object.is_bitmap_cached = Some(Default::default());
                }
                if place_object.background_color.is_none() {
                    place_object.background_color = Some(Color::from_rgba(0));
                }
                if place_object.filters.is_none() {
                    place_object.filters = Some(Default::default());
                }
                // Purposely omitted properties:
                // name, clip_depth, clip_actions, amf_data
                // These properties are only set on initial placement in `MovieClip::instantiate_child`
                // and can not be modified by subsequent PlaceObject tags.
                // Also, is_visible flag persists during rewind unlike all other properties.
            }
        }

        Self {
            frame,
            place_object,
            index,
            tag_start,
            version,
        }
    }

    #[inline]
    fn depth(&self) -> Depth {
        self.place_object.depth.into()
    }

    fn merge(&mut self, next: &mut GotoPlaceObject<'a>) {
        use swf::PlaceObjectAction;
        let cur_place = &mut self.place_object;
        let next_place = &mut next.place_object;
        match (cur_place.action, next_place.action) {
            (cur, PlaceObjectAction::Modify) => {
                cur_place.action = cur;
            }
            (_, new) => {
                cur_place.action = new;
                self.frame = next.frame;
            }
        };
        if next_place.matrix.is_some() {
            cur_place.matrix = next_place.matrix.take();
        }
        if next_place.color_transform.is_some() {
            cur_place.color_transform = next_place.color_transform.take();
        }
        if next_place.ratio.is_some() {
            cur_place.ratio = next_place.ratio.take();
        }
        if next_place.blend_mode.is_some() {
            cur_place.blend_mode = next_place.blend_mode.take();
        }
        if next_place.is_bitmap_cached.is_some() {
            cur_place.is_bitmap_cached = next_place.is_bitmap_cached.take();
        }
        if next_place.is_visible.is_some() {
            cur_place.is_visible = next_place.is_visible.take();
        }
        if next_place.background_color.is_some() {
            cur_place.background_color = next_place.background_color.take();
        }
        if next_place.filters.is_some() {
            cur_place.filters = next_place.filters.take();
        }
        // Purposely omitted properties:
        // name, clip_depth, clip_actions, amf_data
        // These properties are only set on initial placement in `MovieClip::instantiate_child`
        // and can not be modified by subsequent PlaceObject tags.
    }
}

/// A list of add/remove tags to process on a given depth this frame.
///
/// There are only a handful of valid tag configurations per depth: namely,
/// no tags, one removal, one add, or a removal followed by an add.
///
/// Any other configuration in the SWF tag stream is normalized to one of
/// these patterns.
#[derive(Default, Debug, Eq, PartialEq, Clone, Copy)]
pub enum QueuedTagList {
    #[default]
    None,
    Add(QueuedTag),
    Remove(QueuedTag),
    RemoveThenAdd(QueuedTag, QueuedTag),
}

impl QueuedTagList {
    fn queue_add(&mut self, add_tag: QueuedTag) {
        let new = match self {
            QueuedTagList::None => QueuedTagList::Add(add_tag),
            QueuedTagList::Add(existing) => {
                // Flash player traces "Warning: Failed to place object at depth 1.",
                // so let's log a warning too.
                tracing::warn!("Ignoring queued tag {add_tag:?} at same depth as {existing:?}");
                QueuedTagList::Add(*existing)
            }
            QueuedTagList::Remove(r) => QueuedTagList::RemoveThenAdd(*r, add_tag),
            QueuedTagList::RemoveThenAdd(r, _) => QueuedTagList::RemoveThenAdd(*r, add_tag),
        };

        *self = new;
    }

    fn queue_remove(&mut self, remove_tag: QueuedTag) {
        let new = match self {
            QueuedTagList::None => QueuedTagList::Remove(remove_tag),
            QueuedTagList::Add(_) => QueuedTagList::None,
            QueuedTagList::Remove(_) => QueuedTagList::Remove(remove_tag),
            QueuedTagList::RemoveThenAdd(r, _) => QueuedTagList::Remove(*r),
        };

        *self = new;
    }

    fn unqueue_add(&mut self) -> Option<QueuedTag> {
        let (new_queue, return_val) = match self {
            QueuedTagList::None => (QueuedTagList::None, None),
            QueuedTagList::Add(a) => (QueuedTagList::None, Some(*a)),
            QueuedTagList::Remove(r) => (QueuedTagList::Remove(*r), None),
            QueuedTagList::RemoveThenAdd(r, a) => (QueuedTagList::Remove(*r), Some(*a)),
        };

        *self = new_queue;

        return_val
    }

    fn unqueue_remove(&mut self) -> Option<QueuedTag> {
        let (new_queue, return_val) = match self {
            QueuedTagList::None => (QueuedTagList::None, None),
            QueuedTagList::Add(a) => (QueuedTagList::Add(*a), None),
            QueuedTagList::Remove(r) => (QueuedTagList::None, Some(*r)),
            QueuedTagList::RemoveThenAdd(r, a) => (QueuedTagList::Add(*a), Some(*r)),
        };

        *self = new_queue;

        return_val
    }
}

/// A single tag we encountered this frame that we intend to process on a queue.
///
/// No more than one queued action is allowed to be processed on-queue.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct QueuedTag {
    pub tag_type: QueuedTagAction,
    pub tag_start: u64,
}

/// The type of queued tag.
///
/// The u8 parameter is the tag version.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum QueuedTagAction {
    Place(u8),
    Remove(u8),
}

bitflags! {
    /// Boolean state flags used by `MovieClip`.
    #[derive(Clone, Copy)]
    struct MovieClipFlags: u8 {
        /// Whether this `MovieClip` has run its initial frame.
        const INITIALIZED             = 1 << 0;

        /// Whether this `MovieClip` is playing or stopped.
        const PLAYING                 = 1 << 1;

        /// Whether this `MovieClip` has been played as a result of an AS3 command.
        ///
        /// The AS3 `isPlaying` property is broken and yields false until you first
        /// call `play` to unbreak it. This flag tracks that bug.
        const PROGRAMMATICALLY_PLAYED = 1 << 2;

        /// Executing an AVM2 frame script.
        ///
        /// This causes any goto action to be queued and executed at the end of the script.
        const EXECUTING_AVM2_FRAME_SCRIPT = 1 << 3;

        /// Flag set when AVM2 loops to the next frame.
        ///
        /// Because AVM2 queues PlaceObject tags to run later, explicit gotos
        /// that happen while those tags run should cancel the loop.
        const LOOP_QUEUED = 1 << 4;

        const RUNNING_CONSTRUCT_FRAME = 1 << 5;

        /// Whether this `MovieClip` has been post-instantiated yet.
        const POST_INSTANTIATED = 1 << 5;
    }
}

/// Actions that are attached to a `MovieClip` event in
/// an `onClipEvent`/`on` handler.
#[derive(Debug, Clone)]
pub struct ClipEventHandler {
    /// The events that triggers this handler.
    events: ClipEventFlag,

    /// The key code used by the `onKeyPress` event.
    ///
    /// Only used if `events` contains `ClipEventFlag::KEY_PRESS`.
    key_code: ButtonKeyCode,

    /// The actions to run.
    action_data: SwfSlice,
}

impl ClipEventHandler {
    /// Build an event handler from a SWF movie and a parsed ClipAction.
    pub fn from_action_and_movie(other: swf::ClipAction<'_>, movie: Arc<SwfMovie>) -> Self {
        let key_code = if other.events.contains(ClipEventFlag::KEY_PRESS) {
            other
                .key_code
                .and_then(ButtonKeyCode::from_u8)
                .unwrap_or(ButtonKeyCode::Unknown)
        } else {
            ButtonKeyCode::Unknown
        };
        let action_data = SwfSlice::from(movie).to_unbounded_subslice(other.action_data);
        Self {
            events: other.events,
            key_code,
            action_data,
        }
    }
}
