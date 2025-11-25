//! `MovieClip` display object and support code.
use crate::avm1::globals::AVM_DEPTH_BIAS;
use crate::avm1::Avm1;
use crate::avm1::{Activation as Avm1Activation, ActivationIdentifier};
use crate::avm1::{NativeObject as Avm1NativeObject, Object as Avm1Object};
use crate::avm2::object::LoaderStream;
use crate::avm2::script::Script;
use crate::avm2::Activation as Avm2Activation;
use crate::avm2::{
    Avm2, ClassObject as Avm2ClassObject, FunctionArgs as Avm2FunctionArgs, LoaderInfoObject,
    Object as Avm2Object, StageObject as Avm2StageObject,
};
use crate::backend::audio::{AudioManager, SoundInstanceHandle};
use crate::backend::navigator::Request;
use crate::backend::ui::MouseCursor;
use crate::binary_data::BinaryData;
use crate::character::{BitmapCharacter, Character, CompressedBitmap};
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::container::{dispatch_removed_event, ChildContainer};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{
    Avm1Button, Avm1TextFieldBinding, Avm2Button, DisplayObjectBase, DisplayObjectPtr, EditText,
    Graphic, MorphShape, Text, Video,
};
use crate::drawing::Drawing;
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult};
use crate::font::{Font, FontType};
use crate::frame_lifecycle::{run_inner_goto_frame, FramePhase};
use crate::library::MovieLibrary;
use crate::limits::ExecutionLimit;
use crate::loader::LoadManager;
use crate::loader::{self, ContentType};
use crate::prelude::*;
use crate::streams::NetStream;
use crate::string::{AvmString, SwfStrExt as _, WStr, WString};
use crate::tag_utils::{self, ControlFlow, Error, SwfMovie, SwfSlice, SwfStream};
use crate::vminterface::Instantiator;
use bitflags::bitflags;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, DynamicRoot, Gc, GcWeak, Mutation, Rootable};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use ruffle_render::perspective_projection::PerspectiveProjection;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::{Cell, OnceCell, Ref, RefCell, RefMut};
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
pub struct MovieClip<'gc>(Gc<'gc, MovieClipData<'gc>>);

impl fmt::Debug for MovieClip<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MovieClip")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct MovieClipWeak<'gc>(GcWeak<'gc, MovieClipData<'gc>>);

impl<'gc> MovieClipWeak<'gc> {
    pub fn upgrade(self, mc: &Mutation<'gc>) -> Option<MovieClip<'gc>> {
        self.0.upgrade(mc).map(MovieClip)
    }

    pub fn as_ptr(self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }
}

#[derive(Clone)]
pub struct MovieClipHandle(DynamicRoot<Rootable![MovieClipData<'_>]>);

impl MovieClipHandle {
    pub fn stash<'gc>(uc: &UpdateContext<'gc>, this: MovieClip<'gc>) -> Self {
        Self(uc.dynamic_root.stash(uc.gc(), this.0))
    }

    pub fn fetch<'gc>(&self, uc: &UpdateContext<'gc>) -> MovieClip<'gc> {
        MovieClip(uc.dynamic_root.fetch(&self.0))
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct MovieClipData<'gc> {
    base: InteractiveObjectBase<'gc>,
    cell: RefLock<MovieClipDataMut<'gc>>,
    shared: Lock<Gc<'gc, MovieClipShared<'gc>>>,

    tag_stream_pos: Cell<u64>,

    // Unlike most other DisplayObjects, a MovieClip can have an AVM1
    // side and an AVM2 side simultaneously.
    object1: Lock<Option<Avm1Object<'gc>>>,
    object2: Lock<Option<Avm2StageObject<'gc>>>,

    drop_target: Lock<Option<DisplayObject<'gc>>>,

    /// A DisplayObject (doesn't need to be visible) to use for hit tests instead of this clip.
    hit_area: Lock<Option<DisplayObject<'gc>>>,

    /// List of tags queued up for the current frame.
    queued_tags: RefCell<HashMap<Depth, QueuedTagList>>,

    /// Attached audio (AVM1)
    attached_audio: Lock<Option<NetStream<'gc>>>,

    /// The next MovieClip in the AVM1 execution list.
    ///
    /// `None` in an AVM2 movie.
    next_avm1_clip: Lock<Option<MovieClip<'gc>>>,

    audio_stream: Cell<Option<SoundInstanceHandle>>,

    #[collect(require_static)]
    clip_event_handlers: OnceCell<Box<[ClipEventHandler]>>,
    /// This is lazily allocated on demand, to make `MovieClipData` smaller in the common case.
    #[collect(require_static)]
    drawing: OnceCell<Box<RefCell<Drawing>>>,

    last_queued_script_frame: Cell<Option<FrameNumber>>,
    queued_script_frame: Cell<FrameNumber>,
    queued_goto_frame: Cell<Option<FrameNumber>>,

    current_frame: Cell<FrameNumber>,

    flags: Cell<MovieClipFlags>,
    clip_event_flags: Cell<ClipEventFlag>,

    has_pending_script: Cell<bool>,

    /// Force enable button mode, which causes all mouse-related events to
    /// trigger on this clip rather than any input-eligible children.
    button_mode: Cell<bool>,

    avm2_enabled: Cell<bool>,

    /// Show a hand cursor when the clip is in button mode.
    avm2_use_hand_cursor: Cell<bool>,
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
struct MovieClipDataMut<'gc> {
    container: ChildContainer<'gc>,
    frame_scripts: Vec<Option<Avm2Object<'gc>>>,
    avm1_text_field_bindings: Vec<Avm1TextFieldBinding<'gc>>,
}

impl<'gc> MovieClipData<'gc> {
    fn new(shared: MovieClipShared<'gc>, mc: &Mutation<'gc>) -> Self {
        let movie = shared.movie();
        Self {
            base: Default::default(),
            cell: RefLock::new(MovieClipDataMut {
                container: ChildContainer::new(&movie),
                frame_scripts: Vec::new(),
                avm1_text_field_bindings: Vec::new(),
            }),
            shared: Lock::new(Gc::new(mc, shared)),
            tag_stream_pos: Cell::new(0),
            current_frame: Cell::new(0),
            audio_stream: Cell::new(None),
            object1: Lock::new(None),
            object2: Lock::new(None),
            clip_event_handlers: OnceCell::new(),
            clip_event_flags: Cell::new(ClipEventFlag::empty()),
            flags: Cell::new(MovieClipFlags::empty()),
            drawing: OnceCell::new(),
            avm2_enabled: Cell::new(true),
            avm2_use_hand_cursor: Cell::new(true),
            button_mode: Cell::new(false),
            last_queued_script_frame: Cell::new(None),
            queued_script_frame: Cell::new(0),
            has_pending_script: Cell::new(false),
            queued_goto_frame: Cell::new(None),
            drop_target: Lock::new(None),
            queued_tags: Default::default(),
            hit_area: Lock::new(None),
            attached_audio: Lock::new(None),
            next_avm1_clip: Lock::new(None),
        }
    }

    #[inline(always)]
    fn shared_cell(&self) -> Ref<'gc, MovieClipSharedMut> {
        Gc::as_ref(self.shared.get()).cell.borrow()
    }
}

impl<'gc> MovieClip<'gc> {
    pub fn downgrade(self) -> MovieClipWeak<'gc> {
        MovieClipWeak(Gc::downgrade(self.0))
    }

    pub fn new(movie: Arc<SwfMovie>, mc: &Mutation<'gc>) -> Self {
        let shared = MovieClipShared::empty(movie);
        MovieClip(Gc::new(mc, MovieClipData::new(shared, mc)))
    }

    pub fn new_with_avm2(
        movie: Arc<SwfMovie>,
        this: Avm2StageObject<'gc>,
        class: Avm2ClassObject<'gc>,
        mc: &Mutation<'gc>,
    ) -> Self {
        let mut shared = MovieClipShared::empty(movie);
        *shared.avm2_class.get_mut() = Some(class);
        let mut data = MovieClipData::new(shared, mc);
        data.object2 = Lock::new(Some(this));
        MovieClip(Gc::new(mc, data))
    }

    /// Constructs a non-root movie
    pub fn new_with_data(
        mc: &Mutation<'gc>,
        id: CharacterId,
        swf: SwfSlice,
        num_frames: u16,
    ) -> Self {
        let shared = MovieClipShared::with_data(id, swf, num_frames, None, None);
        let data = MovieClipData::new(shared, mc);
        data.flags.set(MovieClipFlags::PLAYING);
        MovieClip(Gc::new(mc, data))
    }

    pub fn new_import_assets(
        context: &mut UpdateContext<'gc>,
        movie: Arc<SwfMovie>,
        parent: MovieClip<'gc>,
    ) -> Self {
        let num_frames = movie.num_frames();
        let loader_info = None;
        let shared =
            MovieClipShared::with_data(0, movie.into(), num_frames, loader_info, Some(parent));

        let data = MovieClipData::new(shared, context.gc());
        data.flags.set(MovieClipFlags::PLAYING);
        MovieClip(Gc::new(context.gc(), data))
    }

    /// Construct a movie clip that represents the root movie
    /// for the entire `Player`.
    pub fn player_root_movie(
        activation: &mut Avm2Activation<'_, 'gc>,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let loader_info = if movie.is_action_script_3() {
            // The root movie doesn't have a `Loader`
            // We will replace this with a `LoaderStream::Swf` later in this function
            let loader_info =
                LoaderInfoObject::not_yet_loaded(activation, movie.clone(), None, None, false)
                    .expect("Failed to construct LoaderInfoObject");
            loader_info.set_expose_content();
            loader_info.set_content_type(ContentType::Swf);
            Some(loader_info)
        } else {
            None
        };

        let shared = MovieClipShared::with_data(
            0,
            movie.clone().into(),
            movie.num_frames(),
            loader_info,
            None,
        );
        let data = MovieClipData::new(shared, activation.gc());
        data.flags.set(MovieClipFlags::PLAYING);
        data.base.base.set_is_root(true);

        let mc = MovieClip(Gc::new(activation.gc(), data));
        if let Some(loader_info) = loader_info {
            loader_info.set_loader_stream(LoaderStream::Swf(movie, mc.into()), activation.gc());
        }
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
        let write = Gc::write(context.gc(), self.0);
        let movie =
            movie.unwrap_or_else(|| Arc::new(SwfMovie::empty(write.movie().version(), None)));
        let total_frames = movie.num_frames();
        assert!(
            write.shared.get().loader_info.is_none(),
            "Called replace_movie on a clip with LoaderInfo set"
        );

        write.base.base.reset_for_movie_load();
        write.base.base.set_is_root(is_root);

        unlock!(write, MovieClipData, cell).borrow_mut().container = ChildContainer::new(&movie);

        unlock!(write, MovieClipData, shared).set(Gc::new(
            context.gc(),
            MovieClipShared::with_data(0, movie.into(), total_frames, loader_info, None),
        ));
        write.tag_stream_pos.set(0);
        write.flags.set(MovieClipFlags::PLAYING);
        write.current_frame.set(0);
        write.audio_stream.take();
    }

    pub fn set_initialized(self) {
        self.0.set_initialized(true);
    }

    pub fn next_avm1_clip(self) -> Option<MovieClip<'gc>> {
        self.0.next_avm1_clip.get()
    }

    pub fn set_next_avm1_clip(&self, mc: &Mutation<'gc>, node: Option<MovieClip<'gc>>) {
        unlock!(Gc::write(mc, self.0), MovieClipData, next_avm1_clip).set(node);
    }

    /// Tries to fire events from our `LoaderInfo` object if we're ready - returns
    /// `true` if both `init` and `complete` have been fired
    pub fn try_fire_loaderinfo_events(self, context: &mut UpdateContext<'gc>) -> bool {
        if self.0.initialized() {
            if let Some(loader_info) = self.loader_info() {
                return loader_info.fire_init_and_complete_events(context, 0, false);
            }
        }
        false
    }

    /// Execute all other timeline actions on this object.
    pub fn run_frame_avm1(self, context: &mut UpdateContext<'gc>) {
        if !self.movie().is_action_script_3() {
            // Run my load/enterFrame clip event.
            let is_load_frame = !self.0.contains_flag(MovieClipFlags::INITIALIZED);
            if is_load_frame {
                self.event_dispatch(context, ClipEvent::Load);
                self.0.set_initialized(true);
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
        let shared = Gc::as_ref(self.0.shared.get());
        let (swf, progress) = (&shared.swf, &shared.preload_progress);

        if progress.awaiting_import.get() {
            // No matter how much of this movie we have loaded, we must not continue preloading
            // until the import is finished.
            return false;
        }

        if progress.next_preload_chunk.get() >= swf.len() as u64 {
            return true;
        }

        let reader = &mut swf.read_from(progress.next_preload_chunk.get());

        let mut sub_preload_done = true;
        if let Some(symbol) = progress.cur_preload_symbol.take() {
            match context
                .library
                .library_for_movie_mut(swf.movie.clone())
                .character_by_id(symbol)
            {
                Some(Character::MovieClip(mc)) => {
                    sub_preload_done = mc.preload(context, chunk_limit);
                    if !sub_preload_done {
                        progress.cur_preload_symbol.set(Some(symbol));
                    }
                }
                Some(unk) => {
                    tracing::error!("Symbol {symbol} changed to unexpected type {unk:?}");
                }
                None => {
                    tracing::error!("Symbol {symbol} disappeared during preloading");
                }
            }
        }

        let mut end_tag_found = false;
        let tag_callback = |reader: &mut SwfStream<'_>, tag_code, tag_len| {
            match tag_code {
                TagCode::CsmTextSettings => shared.csm_text_settings(context, reader),
                TagCode::DefineBits => shared.define_bits(context, reader),
                TagCode::DefineBitsJpeg2 => shared.define_bits_jpeg_2(context, reader),
                TagCode::DefineBitsJpeg3 => shared.define_bits_jpeg_3_or_4(context, reader, 3),
                TagCode::DefineBitsJpeg4 => shared.define_bits_jpeg_3_or_4(context, reader, 4),
                TagCode::DefineBitsLossless => shared.define_bits_lossless(context, reader, 1),
                TagCode::DefineBitsLossless2 => shared.define_bits_lossless(context, reader, 2),
                TagCode::DefineButton => shared.define_button_1(context, reader),
                TagCode::DefineButton2 => shared.define_button_2(context, reader),
                TagCode::DefineButtonCxform => shared.define_button_cxform(context, reader),
                TagCode::DefineButtonSound => shared.define_button_sound(context, reader),
                TagCode::DefineEditText => shared.define_edit_text(context, reader),
                TagCode::DefineFont => shared.define_font_1(context, reader),
                TagCode::DefineFont2 => shared.define_font_2(context, reader),
                TagCode::DefineFont3 => shared.define_font_3(context, reader),
                TagCode::DefineFont4 => shared.define_font_4(context, reader),
                TagCode::DefineMorphShape => shared.define_morph_shape(context, reader, 1),
                TagCode::DefineMorphShape2 => shared.define_morph_shape(context, reader, 2),
                TagCode::DefineScalingGrid => shared.define_scaling_grid(context, reader),
                TagCode::DefineShape => shared.define_shape(context, reader, 1),
                TagCode::DefineShape2 => shared.define_shape(context, reader, 2),
                TagCode::DefineShape3 => shared.define_shape(context, reader, 3),
                TagCode::DefineShape4 => shared.define_shape(context, reader, 4),
                TagCode::DefineSound => shared.define_sound(context, reader),
                TagCode::DefineVideoStream => shared.define_video_stream(context, reader),
                TagCode::DefineSprite => {
                    return shared.define_sprite(context, reader, tag_len, chunk_limit)
                }
                TagCode::DefineText => shared.define_text(context, reader, 1),
                TagCode::DefineText2 => shared.define_text(context, reader, 2),
                TagCode::DoInitAction => self.do_init_action(context, reader, tag_len),
                TagCode::DefineSceneAndFrameLabelData => shared.scene_and_frame_labels(reader),
                TagCode::ExportAssets => shared.export_assets(context, reader),
                TagCode::FrameLabel => shared.frame_label(reader),
                TagCode::JpegTables => shared.jpeg_tables(context, reader),
                TagCode::ShowFrame => shared.show_frame(reader, tag_len),
                TagCode::ScriptLimits => shared.script_limits(reader, context.avm1),
                TagCode::SoundStreamHead => shared.sound_stream_head(reader, 1),
                TagCode::SoundStreamHead2 => shared.sound_stream_head(reader, 2),
                TagCode::VideoFrame => shared.preload_video_frame(context, reader),
                TagCode::DefineBinaryData => shared.define_binary_data(context, reader),
                TagCode::ImportAssets => {
                    self.import_assets(context, reader, chunk_limit, 1)?;
                    return Ok(ControlFlow::Exit);
                }
                TagCode::ImportAssets2 => {
                    self.import_assets(context, reader, chunk_limit, 2)?;
                    return Ok(ControlFlow::Exit);
                }
                TagCode::DoAbc | TagCode::DoAbc2 => shared.preload_bytecode_tag(tag_code, reader),
                TagCode::SymbolClass => shared.preload_symbol_class(reader),
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
            tag_utils::decode_tags(reader, tag_callback)
        } else {
            Ok(true)
        };
        let is_finished = !progress.awaiting_import.get()
            && (end_tag_found || result.is_err() || !result.unwrap_or_default());

        shared.import_exports_of_importer(context);

        if is_finished {
            if progress.cur_preload_frame.get() == 1 {
                // If this clip did not have any show frame tags,
                // treat the end-of-clip as a ShowFrame
                shared.show_frame(reader, 0).unwrap();
            }
            // Flag the movie as fully preloaded when we hit the end of the tag stream.
            progress.next_preload_chunk.set(u64::MAX);
        } else {
            let next_chunk =
                (reader.get_ref().as_ptr() as u64).saturating_sub(swf.data().as_ptr() as u64);
            progress.next_preload_chunk.set(next_chunk);
        }

        is_finished
    }

    pub fn finish_importing(self) {
        self.0
            .shared
            .get()
            .preload_progress
            .awaiting_import
            .set(false);
    }

    #[inline]
    fn do_init_action(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'_>,
        tag_len: usize,
    ) -> Result<(), Error> {
        let mut target = self;
        loop {
            let shared = target.0.shared.get();
            if shared.movie().is_action_script_3() {
                tracing::warn!("DoInitAction tag in AVM2 movie");
                return Ok(());
            }

            // `DoInitAction`s always execute in the context of their importer movie.
            let Some(parent) = shared.importer_movie else {
                break;
            };

            target = parent;
        }

        let start = reader.as_slice();

        // TODO: Init actions are supposed to be executed once, and it gives a
        // sprite ID... how does that work?
        // TODO: what happens with `DoInitAction` blocks nested in a `DefineSprite`?
        // The SWF spec forbids this, but Ruffle will currently execute them in the context
        // of the character itself, which is probably nonsense.
        let _sprite_id = reader.read_u16()?;
        let num_read = reader.pos(start);

        let slice = self
            .0
            .shared
            .get()
            .swf
            .resize_to_reader(reader, tag_len - num_read);

        if !slice.is_empty() {
            Avm1::run_stack_frame_for_init_action(target.into(), slice, context);
        }

        Ok(())
    }

    #[inline]
    fn do_abc(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'_>,
    ) -> Result<Option<Script<'gc>>, Error> {
        if !context.root_swf.is_action_script_3() {
            tracing::warn!("DoABC tag with non-AVM2 root");
            return Ok(None);
        }

        let data = reader.read_slice_to_end();
        if !data.is_empty() {
            let movie = self.movie();
            let domain = context.library.library_for_movie_mut(movie).avm2_domain();

            // DoAbc tag seems to be equivalent to a DoAbc2 with no flags (eager)
            match Avm2::do_abc(
                context,
                data,
                None,
                swf::DoAbc2Flag::empty(),
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
        if !context.root_swf.is_action_script_3() {
            tracing::warn!("DoABC2 tag with non-AVM2 root");
            return Ok(None);
        }

        let do_abc = reader.read_do_abc_2()?;
        if !do_abc.data.is_empty() {
            let movie = self.movie();
            let domain = context.library.library_for_movie_mut(movie).avm2_domain();
            let name = AvmString::new(context.gc(), do_abc.name.decode(reader.encoding()));

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
    fn import_assets(
        self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'_>,
        _chunk_limit: &mut ExecutionLimit,
        version: u8,
    ) -> Result<(), Error> {
        let (url, exported_assets) = match version {
            1 => reader.read_import_assets()?,
            2 => reader.read_import_assets_2()?,
            _ => unreachable!(),
        };

        let mc = context.gc();
        let library = context.library.library_for_movie_mut(self.movie());

        let asset_url = url.to_string_lossy(UTF_8);

        let request = Request::get(asset_url);

        for asset in exported_assets {
            let name = asset.name.decode(reader.encoding());
            let name = AvmString::new(mc, name);
            let id = asset.id;
            tracing::debug!("Importing asset: {} (ID: {})", name, id);

            library.register_import(name, id);
        }

        let fut = LoadManager::load_asset_movie(context, request, self);
        self.0
            .shared
            .get()
            .preload_progress
            .awaiting_import
            .set(true);
        context.navigator.spawn_future(fut);

        Ok(())
    }

    pub fn playing(self) -> bool {
        self.0.playing()
    }

    pub fn programmatically_played(self) -> bool {
        self.0.programmatically_played()
    }

    pub fn drop_target(self) -> Option<DisplayObject<'gc>> {
        self.0.drop_target.get()
    }

    pub fn set_drop_target(self, mc: &Mutation<'gc>, drop_target: Option<DisplayObject<'gc>>) {
        unlock!(Gc::write(mc, self.0), MovieClipData, drop_target).set(drop_target);
    }

    pub fn set_programmatically_played(self) {
        if self.header_frames() > 1 {
            self.0.set_programmatically_played()
        }
    }

    pub fn next_frame(self, context: &mut UpdateContext<'gc>) {
        if self.current_frame() < self.header_frames() {
            self.goto_frame(context, self.current_frame() + 1, true);
        }
    }

    pub fn play(self) {
        self.0.play()
    }

    pub fn prev_frame(self, context: &mut UpdateContext<'gc>) {
        if self.current_frame() > 1 {
            self.goto_frame(context, self.current_frame() - 1, true);
        }
    }

    pub fn initialized(self) -> bool {
        self.0.initialized()
    }

    pub fn stop(self, context: &mut UpdateContext<'gc>) {
        self.0.stop(context)
    }

    /// Does this clip have a unload handler
    pub fn has_unload_handler(self) -> bool {
        self.clip_actions()
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
            self.play();
        }

        // Clamp frame number in bounds.
        let frame = frame.max(1);

        // In AS3, no-op gotos have side effects that are visible to user code.
        // Hence, we have to run them anyway.
        if frame != self.current_frame() {
            if self
                .0
                .contains_flag(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT)
            {
                // AVM2 does not allow a clip to see while it is executing a frame script.
                // The goto is instead queued and run once the frame script is completed.
                self.0.queued_goto_frame.set(Some(frame));
            } else {
                self.run_goto(context, frame, false);
            }
        } else if self.movie().is_action_script_3() {
            // Despite not running, the goto still overwrites the currently enqueued frame.
            self.0.queued_goto_frame.set(None);
            // Pretend we actually did a goto, but don't do anything.
            run_inner_goto_frame(context, &[], self);
        }
    }

    pub fn current_frame(self) -> FrameNumber {
        self.0.current_frame()
    }

    /// Return the current scene.
    pub fn current_scene(self) -> Option<Scene> {
        let current_frame = self.0.current_frame();

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
        let current_frame = self.0.current_frame();

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
        let mut out: Vec<_> = self.0.shared_cell().scene_labels.clone();
        out.sort_unstable_by(|Scene { start: a, .. }, Scene { start: b, .. }| a.cmp(b));
        out
    }

    /// Scan through the list of scenes and yield the best one, if available,
    /// according to a given criterion function.
    fn filter_scenes<F>(self, mut cond: F) -> Option<Scene>
    where
        F: FnMut(Option<&Scene>, &Scene) -> bool,
    {
        let read = self.0.shared_cell();
        let mut best: Option<&Scene> = None;
        for scene in read.scene_labels.iter() {
            if cond(best, scene) {
                best = Some(scene);
            }
        }

        best.cloned()
    }

    /// Yield the current frame label as a tuple of string and frame number.
    pub fn current_label(self) -> Option<(WString, FrameNumber)> {
        let read = self.0.shared_cell();
        let current_frame = self.0.current_frame();

        let mut best: Option<(&WString, FrameNumber)> = None;
        for (frame, label) in read.frame_labels.iter() {
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
        let mut values: Vec<(WString, FrameNumber)> = self
            .0
            .shared_cell()
            .frame_labels
            .iter()
            .filter(|(frame, _label)| *frame >= from && *frame < to)
            .map(|(frame, label)| (label.clone(), *frame))
            .collect();

        values.sort_unstable_by(|(_, framea), (_, frameb)| framea.cmp(frameb));

        values
    }

    /// Returns the frame count defined in the header.
    /// Note that this may not be how many frames there actually are.
    pub fn header_frames(self) -> FrameNumber {
        self.0.header_frames()
    }

    pub fn has_frame_script(self, frame: FrameNumber) -> bool {
        self.frame_script(frame).is_some()
    }

    fn frame_script(self, frame: FrameNumber) -> Option<Avm2Object<'gc>> {
        self.0
            .cell
            .borrow()
            .frame_scripts
            .get(frame as usize)
            .and_then(|&v| v)
    }

    /// This sets the current preload frame of this MovieClipto a given number (resulting
    /// in the _framesloaded / framesLoaded property being the given number - 1).
    pub fn set_cur_preload_frame(self, cur_preload_frame: u16) {
        self.0
            .shared
            .get()
            .preload_progress
            .cur_preload_frame
            .set(cur_preload_frame);
    }

    /// This sets the current frame of this MovieClip to a given number.
    pub fn set_current_frame(self, current_frame: FrameNumber) {
        self.0.current_frame.set(current_frame);
    }

    /// The amount of frames loaded in this movieclip.
    /// This is independent of the frame count defined in the header.
    pub fn frames_loaded(self) -> i32 {
        self.0.frames_loaded()
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
        let progress = &Gc::as_ref(self.0.shared.get()).preload_progress;
        if progress.next_preload_chunk.get() == u64::MAX {
            // u64::MAX is a sentinel for load complete
            return max(self.total_bytes(), 0) as u32;
        }

        let swf_header_size = max(self.total_bytes(), 0) as u32 - self.tag_stream_len() as u32;

        swf_header_size + progress.next_preload_chunk.get() as u32
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
    /// run through `progress`, we instead emulate this property by scaling the
    /// loaded bytes by the compression ratio of the SWF.
    pub fn compressed_loaded_bytes(self) -> u32 {
        (self.loaded_bytes() as f64 * self.compressed_total_bytes() as f64
            / self.total_bytes() as f64) as u32
    }

    pub fn avm2_class(self) -> Option<Avm2ClassObject<'gc>> {
        self.0.shared.get().avm2_class.get()
    }

    pub fn set_avm2_class(self, mc: &Mutation<'gc>, constr: Option<Avm2ClassObject<'gc>>) {
        let data = Gc::write(mc, self.0.shared.get());
        unlock!(data, MovieClipShared, avm2_class).set(constr);
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
                .shared_cell()
                .frame_labels_map
                .get(frame_label)
                .copied()
        } else {
            let label = frame_label.to_ascii_lowercase();
            self.0.shared_cell().frame_labels_map.get(&label).copied()
        }
    }

    pub fn scene_label_to_number(self, scene_label: &WStr) -> Option<FrameNumber> {
        // Never used in AVM1, so always be case sensitive.
        self.0
            .shared_cell()
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
            let mut end = self.header_frames() + 1;
            for Scene {
                start: new_scene_start,
                ..
            } in self.0.shared_cell().scene_labels.iter()
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
    pub fn clip_actions(self) -> &'gc [ClipEventHandler] {
        match Gc::as_ref(self.0).clip_event_handlers.get() {
            Some(handlers) => handlers,
            None => &[],
        }
    }

    /// Sets the clip actions (a.k.a. clip events) for this MovieClip.
    /// Clip actions are created in the Flash IDE by using the `onEnterFrame`
    /// tag on a MovieClip instance.
    pub fn init_clip_event_handlers(self, event_handlers: Box<[ClipEventHandler]>) {
        if self.0.clip_event_handlers.get().is_some() {
            panic!("Clip event handlers already initialized");
        }

        let mut all_event_flags = ClipEventFlag::empty();
        for handler in self.0.clip_event_handlers.get_or_init(|| event_handlers) {
            all_event_flags |= handler.events;
        }

        self.0.clip_event_flags.set(all_event_flags);
    }

    /// Returns an iterator of AVM1 `DoAction` blocks on the given frame number.
    /// Used by the AVM `Call` action.
    pub fn actions_on_frame(self, frame: FrameNumber) -> impl DoubleEndedIterator<Item = SwfSlice> {
        use swf::read::Reader;

        let mut actions: SmallVec<[SwfSlice; 2]> = SmallVec::new();

        // Iterate through this clip's tags, counting frames until we reach the target frame.
        if frame > 0 && frame <= self.header_frames() {
            let mut cur_frame = 1;
            let shared = self.0.shared.get();
            let mut reader = shared.swf.read_from(0);
            while cur_frame <= frame && !reader.get_ref().is_empty() {
                let tag_callback = |reader: &mut Reader<'_>, tag_code, tag_len| {
                    match tag_code {
                        TagCode::ShowFrame => {
                            cur_frame += 1;
                            Ok(ControlFlow::Exit)
                        }
                        TagCode::DoAction if cur_frame == frame => {
                            // On the target frame, add any DoAction tags to the array.
                            let slice = shared.swf.resize_to_reader(reader, tag_len);
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
        let mc = self.0.shared.get();
        // We know that we are not on the last frame if either condition is true:
        // 1. The movieclip is not done preloading frames (indicated by next_preload_chunk not being u64::MAX)
        // 2. The current frame is less than the amount of frames loaded.
        if mc.preload_progress.next_preload_chunk.get() != u64::MAX
            || self.current_frame() < self.frames_loaded() as u16
        {
            NextFrame::Next
        // The `current_frame` can be larger than `header_frames` if the SWF header
        // declared fewer frames than we actually have. We only stop the swf if there
        // was *really* at most a single frame (we declared at most 1 frame, and reached the end
        // of the stream after executing 0 or 1 frames)
        } else if self.header_frames() <= 1 && self.current_frame() <= 1 {
            NextFrame::Same
        } else {
            NextFrame::First
        }
    }

    fn run_frame_internal(
        self,
        context: &mut UpdateContext<'gc>,
        run_display_actions: bool,
        run_sounds: bool,
        is_action_script_3: bool,
    ) {
        let shared = Gc::as_ref(self.0.shared.get());

        let next_frame = self.determine_next_frame();
        match next_frame {
            NextFrame::Next => {
                if (self.0.current_frame() + 1) >= shared.preload_progress.cur_preload_frame.get() {
                    return;
                }

                // AS3 removals need to happen before frame advance (see below)
                if !is_action_script_3 {
                    self.0.increment_current_frame();
                }
            }
            NextFrame::First => return self.run_goto(context, 1, true),
            NextFrame::Same => self.stop(context),
        }

        let tag_stream_start = shared.swf.as_ref().as_ptr() as u64;
        let data = shared.swf.clone();
        let mut reader = data.read_from(self.0.tag_stream_pos.get());

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
                    self.queue_place_object(reader, 1)
                }
                TagCode::PlaceObject2 if run_display_actions && is_action_script_3 => {
                    self.queue_place_object(reader, 2)
                }
                TagCode::PlaceObject3 if run_display_actions && is_action_script_3 => {
                    self.queue_place_object(reader, 3)
                }
                TagCode::PlaceObject4 if run_display_actions && is_action_script_3 => {
                    self.queue_place_object(reader, 4)
                }
                TagCode::RemoveObject if run_display_actions && is_action_script_3 => {
                    self.queue_remove_object(reader, 1)
                }
                TagCode::RemoveObject2 if run_display_actions && is_action_script_3 => {
                    self.queue_remove_object(reader, 2)
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
        if let Err(e) = self.run_abc_and_symbol_tags(context, self.0.current_frame()) {
            tracing::error!("Error running abc/symbol in frame: {e:?}");
        }

        // On AS3, we deliberately run all removals before the frame number or
        // tag position updates. This ensures that code that runs gotos when a
        // display object is added or removed does not catch the movie clip in
        // an invalid state.
        let remove_actions = self.unqueue_filtered(|q| q.unqueue_remove());

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

        self.0
            .tag_stream_pos
            .set(reader.get_ref().as_ptr() as u64 - tag_stream_start);

        // Check if our audio track has finished playing.
        if let Some(audio_stream) = self.0.audio_stream.get() {
            if !context.is_sound_playing(audio_stream) {
                self.0.audio_stream.take();
            }
        }

        if matches!(next_frame, NextFrame::Next) && is_action_script_3 {
            self.0.increment_current_frame();
        }

        self.0.queued_script_frame.set(self.0.current_frame.get());
        if self.0.last_queued_script_frame.get() != Some(self.0.current_frame.get()) {
            // We explicitly clear this variable since AS3 may later GOTO back
            // to the already-ran frame. Since the frame number *has* changed
            // in the meantime, it should absolutely run again.
            self.0.last_queued_script_frame.set(None);
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
        if self.has_child_at_depth(depth) {
            context.avm_warning(&format!("Failed to place object at depth {depth}."));
            return None;
        }

        let movie = self.movie();
        let library = context.library.library_for_movie_mut(movie.clone());
        match library.instantiate_by_id(id, context.gc_context) {
            Some(child) => {
                // Remove previous child from children list,
                // and add new child onto front of the list.
                let prev_child = self.replace_at_depth(context, child, depth);
                {
                    // Set initial properties for child.
                    child.set_instantiated_by_timeline(true);
                    child.set_depth(depth);
                    child.set_parent(context, Some(self.into()));
                    child.set_place_frame(self.current_frame());

                    // Apply PlaceObject parameters.
                    child.apply_place_object(context, place_object);
                    if let Some(name) = &place_object.name {
                        let encoding = swf::SwfStr::encoding_for_version(self.swf_version());
                        let name = AvmString::new(context.gc(), name.decode(encoding));
                        child.set_name(context.gc(), name);
                        child.set_has_explicit_name(true);
                    }
                    if let Some(clip_depth) = place_object.clip_depth {
                        child.set_clip_depth(clip_depth.into());
                    }
                    // Clip events only apply to movie clips.
                    if let (Some(clip_actions), Some(clip)) =
                        (&place_object.clip_actions, child.as_movie_clip())
                    {
                        // Convert from `swf::ClipAction` to Ruffle's `ClipEventHandler`.
                        clip.init_clip_event_handlers(
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
                    if let Some(child) = child.as_movie_clip() {
                        if !movie.is_action_script_3() {
                            child.run_frame_avm1(context);
                        }
                    }
                }

                if let Some(prev_child) = prev_child {
                    dispatch_removed_event(prev_child, context);
                }

                Some(child)
            }
            None => {
                tracing::error!("Unable to instantiate display node id {}", id,);
                None
            }
        }
    }

    #[cfg(not(feature = "timeline_debug"))]
    fn assert_expected_tag_start(self) {}

    #[cfg(feature = "timeline_debug")]
    fn assert_expected_tag_start(self) {
        assert_eq!(
            Some(self.0.tag_stream_pos.get()),
            self.0
                .shared_cell()
                .tag_frame_boundaries
                .get(&self.0.current_frame())
                .map(|(_start, end)| *end), // Yes, this is correct, at least for AVM1.
            "[{:?}] Gotos must start from the correct tag position for frame {}",
            self.base().name,
            self.0.current_frame()
        );
    }

    #[cfg(not(feature = "timeline_debug"))]
    fn assert_expected_tag_end(self, _hit_target_frame: bool) {}

    #[cfg(feature = "timeline_debug")]
    fn assert_expected_tag_end(self, hit_target_frame: bool) {
        let tag_frame_end = self
            .0
            .shared_cell()
            .tag_frame_boundaries
            .get(&self.0.current_frame.get())
            .map(|(_start, end)| *end);

        // Gotos that do *not* hit their target frame will not update their tag
        // stream position, as they do not run the final frame's tags, and thus
        // cannot derive the end position of the clip anyway. This is not
        // observable to user code as any further timeline interaction would
        // trigger a rewind, so we ignore it here for now.
        if hit_target_frame {
            assert_eq!(
                Some(self.0.tag_stream_pos.get()),
                tag_frame_end,
                "[{:?}] Gotos must end at the correct tag position for frame {}",
                self.base().name,
                self.0.current_frame()
            );
        } else {
            // Of course, the target frame desync absolutely will break our
            // other asserts, so fix them up here.
            if let Some(end) = tag_frame_end {
                self.0.tag_stream_pos.set(end);
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
                self.name().map(|s| s.to_string()).unwrap_or_default(),
                if is_implicit { "looping" } else { "goto" },
                self.current_frame(),
                frame
            );
            self.assert_expected_tag_start();
        }

        let frame_before_rewind = self.current_frame();
        self.base().set_skip_next_enter_frame(false);

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

        self.0.stop_audio_stream(context);

        let is_rewind = if frame <= self.current_frame() {
            // Because we can only step forward, we have to start at frame 1
            // when rewinding. We don't actually remove children yet because
            // otherwise AS3 can observe byproducts of the rewinding process.
            self.0.tag_stream_pos.set(0);
            self.0.current_frame.set(0);

            true
        } else {
            false
        };

        let from_frame = self.current_frame();

        // Explicit gotos in the middle of an AS3 loop cancel the loop's queued
        // tags. The rest of the goto machinery can handle the side effects of
        // a half-executed loop.
        if self.0.loop_queued() {
            self.0.queued_tags.borrow_mut().clear();
        }

        if is_implicit {
            self.0.set_loop_queued();
        }

        // Step through the intermediate frames, and aggregate the deltas of each frame.
        let data = self.0.shared.get().swf.clone();
        let tag_stream_start = data.as_ref().as_ptr() as u64;
        let mut frame_pos = self.0.tag_stream_pos.get();
        let mut index = 0;

        // Sanity; let's make sure we don't seek way too far.
        let clamped_frame = frame.min(max(self.0.frames_loaded(), 0) as FrameNumber);

        let mut removed_frame_scripts: Vec<DisplayObject<'gc>> = vec![];

        let mut reader = data.read_from(frame_pos);
        while self.current_frame() < clamped_frame && !reader.get_ref().is_empty() {
            self.0.increment_current_frame();
            frame_pos = reader.get_ref().as_ptr() as u64 - tag_stream_start;

            let tag_callback = |reader: &mut _, tag_code, _tag_len| {
                enum Action {
                    Place(u8),
                    Remove(u8),
                }

                let action = match tag_code {
                    TagCode::PlaceObject => Action::Place(1),
                    TagCode::PlaceObject2 => Action::Place(2),
                    TagCode::PlaceObject3 => Action::Place(3),
                    TagCode::PlaceObject4 => Action::Place(4),
                    TagCode::RemoveObject => Action::Remove(1),
                    TagCode::RemoveObject2 => Action::Remove(2),
                    TagCode::ShowFrame => return Ok(ControlFlow::Exit),
                    _ => return Ok(ControlFlow::Continue),
                };

                match action {
                    Action::Place(version) => {
                        index += 1;
                        self.0.goto_place_object(
                            reader,
                            version,
                            &mut goto_commands,
                            is_rewind,
                            index,
                        )
                    }
                    Action::Remove(version) => self.goto_remove_object(
                        reader,
                        version,
                        context,
                        &mut goto_commands,
                        is_rewind,
                        from_frame,
                        &mut removed_frame_scripts,
                    ),
                }?;

                Ok(ControlFlow::Continue)
            };
            let _ = tag_utils::decode_tags(&mut reader, tag_callback);
            if let Err(e) = self.run_abc_and_symbol_tags(context, self.current_frame() - 1) {
                tracing::error!("Error running abc/symbols in goto: {e:?}");
            }
        }
        let hit_target_frame = self.0.current_frame() == frame;

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

            let final_placements: HashMap<Depth, &GotoPlaceObject<'_>> =
                goto_commands.iter().map(|cmd| (cmd.depth(), cmd)).collect();

            let children: SmallVec<[_; 16]> = self
                .iter_render_list()
                .filter(|child| !self.survives_rewind(*child, &final_placements, frame))
                .collect();

            for child in children {
                if !child.placed_by_avm2_script() {
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
                let new_tag = QueuedTag {
                    tag_type: QueuedTagAction::Place(params.version),
                    tag_start: params.tag_start,
                };
                let mut queued_tags = self.0.queued_tags.borrow_mut();
                let bucket = queued_tags
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
                    prev_child.set_place_frame(params.frame);
                }
                (PlaceObjectAction::Place(id), _, _)
                | (swf::PlaceObjectAction::Replace(id), _, _) => {
                    if let Some(child) =
                        clip.instantiate_child(context, id, params.depth(), &params.place_object)
                    {
                        // Set the place frame to the frame where the object *would* have been placed.
                        child.set_place_frame(params.frame);
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
            self.0.decrement_current_frame();
            self.0.tag_stream_pos.set(frame_pos);
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
            self.0.current_frame.set(clamped_frame);
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

        self.assert_expected_tag_end(hit_target_frame);
    }

    fn survives_rewind(
        self,
        old_object: DisplayObject<'_>,
        final_placements: &HashMap<Depth, &GotoPlaceObject<'_>>,
        frame: FrameNumber,
    ) -> bool {
        // TODO [KJ] This logic is not 100% tested. It's possible it's a bit
        //    different in reality, but the spirit is there :)

        let is_candidate_for_removal = if self.movie().is_action_script_3() {
            old_object.place_frame() > frame || old_object.placed_by_avm2_script()
        } else {
            old_object.depth() < AVM_DEPTH_BIAS
        };

        if !is_candidate_for_removal && old_object.as_morph_shape().is_none() {
            return true;
        }
        let Some(final_placement) = final_placements.get(&old_object.depth()) else {
            return false;
        };

        let new_params = &final_placement.place_object;

        if !old_object.movie().is_action_script_3()
            && old_object.placed_by_avm1_script()
            && old_object.depth() < AVM_DEPTH_BIAS
        {
            return false;
        }

        let id_equals = match new_params.action {
            swf::PlaceObjectAction::Place(id) | swf::PlaceObjectAction::Replace(id) => {
                old_object.id() == id
            }
            _ => false,
        };

        let ratio_equals = match new_params.ratio {
            Some(ratio) => old_object.ratio() == ratio,
            None => true,
        };

        let clip_depth_equals = match new_params.clip_depth {
            Some(clip_depth) => old_object.clip_depth() == clip_depth as Depth,
            None => true,
        };

        let color_transform_equals = match new_params.color_transform {
            Some(color_transform) => old_object.base().color_transform() == color_transform,
            None => true,
        };

        let base_matrix_equals = match new_params.matrix {
            Some(matrix) => old_object.base().matrix() == matrix.into(),
            None => true,
        };

        match old_object {
            DisplayObject::MorphShape(_) | DisplayObject::Graphic(_) | DisplayObject::Text(_) => {
                ratio_equals
                    && id_equals
                    && clip_depth_equals
                    && base_matrix_equals
                    && color_transform_equals
            }
            DisplayObject::Avm1Button(_)
            | DisplayObject::Avm2Button(_)
            | DisplayObject::EditText(_)
            | DisplayObject::Bitmap(_)
            | DisplayObject::Video(_) => ratio_equals && id_equals && clip_depth_equals,
            DisplayObject::MovieClip(_)
            | DisplayObject::Stage(_)
            | DisplayObject::LoaderDisplay(_) => ratio_equals,
        }
    }

    fn construct_as_avm1_object(
        self,
        context: &mut UpdateContext<'gc>,
        init_object: Option<Avm1Object<'gc>>,
        instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if self.0.object1.get().is_none() {
            let avm1_constructor = self.0.get_registered_avm1_constructor(context);

            // If we are running within the AVM, this must be an immediate action.
            // If we are not, then this must be queued to be ran first-thing
            if let Some(constructor) = avm1_constructor.filter(|_| instantiated_by.is_avm()) {
                let mut activation = Avm1Activation::from_nothing(
                    context,
                    ActivationIdentifier::root("[Construct]"),
                    self.into(),
                );

                if let Ok(prototype) = constructor
                    .get(istr!("prototype"), &mut activation)
                    .map(|v| v.coerce_to_object(&mut activation))
                {
                    let object = Avm1Object::new_with_native(
                        &activation.context.strings,
                        Some(prototype),
                        Avm1NativeObject::MovieClip(self),
                    );
                    let write = Gc::write(activation.gc(), self.0);
                    unlock!(write, MovieClipData, object1).set(Some(object));

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

            let object = Avm1Object::new_with_native(
                &context.strings,
                Some(context.avm1.prototypes(self.swf_version()).movie_clip),
                Avm1NativeObject::MovieClip(self),
            );
            let write = Gc::write(context.gc(), self.0);
            unlock!(write, MovieClipData, object1).set(Some(object));

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

            for event_handler in self.clip_actions().iter() {
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
        Avm1::run_with_stack_frame_for_display_object(
            self.into(),
            context,
            Avm1TextFieldBinding::bind_variables,
        );
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
        let class_object = self.0.shared.get().avm2_class.get();
        let class_object = class_object.unwrap_or_else(|| context.avm2.classes().movieclip);

        let object =
            Avm2StageObject::for_display_object(context.gc(), display_object, class_object);

        self.set_object2(context, object);
    }

    /// Construct the AVM2 side of this object.
    ///
    /// This function does *not* allocate the object; it is intended that you
    /// will allocate the object first before doing so. This function is
    /// intended to be called from `post_instantiate`.
    #[inline(never)]
    fn construct_as_avm2_object(self, context: &mut UpdateContext<'gc>) {
        let class_object = self.0.shared.get().avm2_class.get();
        let class_object = class_object.unwrap_or_else(|| context.avm2.classes().movieclip);

        if let Some(object) = self.0.object2.get() {
            let mut activation = Avm2Activation::from_nothing(context);
            let result =
                class_object.call_init(object.into(), Avm2FunctionArgs::empty(), &mut activation);

            if let Err(e) = result {
                tracing::error!(
                    "Got \"{:?}\" when constructing AVM2 side of movie clip of type {}",
                    e,
                    class_object
                        .inner_class_definition()
                        .name()
                        .to_qualified_name(context.gc())
                );
            }
        }
    }

    /// Called on an AVM1 MovieClip that has been loaded by AVM2 (i.e. a
    /// mixed-AVM MovieClip) to create its AVM2-side `AVM1Movie` object.
    pub fn set_avm1movie(self, context: &mut UpdateContext<'gc>) {
        let class_object = context.avm2.classes().avm1movie;
        let object = Avm2StageObject::for_display_object(context.gc(), self.into(), class_object);

        self.set_object2(context, object);
    }

    pub fn register_frame_script(
        self,
        frame_id: FrameNumber,
        callable: Option<Avm2Object<'gc>>,
        context: &mut UpdateContext<'gc>,
    ) {
        let write = Gc::write(context.gc(), self.0);
        let current_frame = write.current_frame();
        let mut frame_scripts =
            RefMut::map(unlock!(write, MovieClipData, cell).borrow_mut(), |r| {
                &mut r.frame_scripts
            });

        let index = frame_id as usize;
        if let Some(callable) = callable {
            if frame_scripts.len() <= index {
                frame_scripts.resize(index + 1, None);
            }
            frame_scripts[index] = Some(callable);
            if frame_id == current_frame {
                if *context.frame_phase == FramePhase::FrameScripts {
                    context.frame_script_cleanup_queue.push_back(self);
                } else {
                    // Ensure newly registered frame scripts are executed,
                    // even if the frame is repeated due to goto.
                    write.last_queued_script_frame.set(None);
                    write.has_pending_script.set(true);
                }
            }
        } else if frame_scripts.len() > index {
            frame_scripts[index] = None;
        }
    }

    /// Handle a RemoveObject tag when running a goto action.
    #[inline]
    #[expect(clippy::too_many_arguments)]
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
            self.0.current_frame.set(from_frame);

            let child = self.child_by_depth(depth);
            if let Some(child) = child {
                if !child.placed_by_avm2_script() {
                    self.remove_child(context, child);
                } else {
                    self.remove_child_from_depth_list(context, child);
                }

                removed_frame_scripts.push(child);
            }

            self.0.current_frame.set(to_frame);
        }
        Ok(())
    }

    fn enabled(self, context: &mut UpdateContext<'gc>) -> bool {
        if !self.movie().is_action_script_3() {
            self.get_avm1_boolean_property(istr!(context, "enabled"), context, |_| true)
        } else {
            self.avm2_enabled()
        }
    }

    pub fn avm2_enabled(self) -> bool {
        self.0.avm2_enabled.get()
    }

    pub fn set_avm2_enabled(self, enabled: bool) {
        self.0.avm2_enabled.set(enabled);
    }

    fn use_hand_cursor(self, context: &mut UpdateContext<'gc>) -> bool {
        if !self.movie().is_action_script_3() {
            self.get_avm1_boolean_property(istr!(context, "useHandCursor"), context, |_| true)
        } else {
            self.avm2_use_hand_cursor()
        }
    }

    pub fn avm2_use_hand_cursor(self) -> bool {
        self.0.avm2_use_hand_cursor.get()
    }

    pub fn set_avm2_use_hand_cursor(self, use_hand_cursor: bool) {
        self.0.avm2_use_hand_cursor.set(use_hand_cursor);
    }

    pub fn hit_area(self) -> Option<DisplayObject<'gc>> {
        self.0.hit_area.get()
    }

    pub fn set_hit_area(self, mc: &Mutation<'gc>, hit_area: Option<DisplayObject<'gc>>) {
        unlock!(Gc::write(mc, self.0), MovieClipData, hit_area).set(hit_area);
    }

    pub fn tag_stream_len(self) -> usize {
        self.0.tag_stream_len()
    }

    pub fn forced_button_mode(self) -> bool {
        self.0.button_mode.get()
    }

    pub fn set_forced_button_mode(self, button_mode: bool) {
        self.0.button_mode.set(button_mode);
    }

    pub fn drawing_mut(&self) -> RefMut<'_, Drawing> {
        // We're about to change graphics, so invalidate on the next frame
        self.invalidate_cached_bitmap();
        self.0.drawing.get_or_init(Default::default).borrow_mut()
    }

    pub fn drawing(&self) -> Option<Ref<'_, Drawing>> {
        self.0.drawing.get().map(|d| d.borrow())
    }

    pub fn is_button_mode(self, context: &mut UpdateContext<'gc>) -> bool {
        if self.forced_button_mode()
            || self
                .0
                .clip_event_flags
                .get()
                .intersects(ClipEvent::BUTTON_EVENT_FLAGS)
        {
            true
        } else if self.avm1_parent().is_none() {
            false
        } else if let Some(object) = self.0.object1.get() {
            let mut activation = Avm1Activation::from_nothing(
                context,
                ActivationIdentifier::root("[Mouse Pick]"),
                self.avm1_root(),
            );

            ClipEvent::BUTTON_EVENT_METHODS
                .iter()
                .copied()
                .any(|handler| {
                    let handler = AvmString::new_utf8(activation.gc(), handler);
                    object.has_property(&mut activation, handler)
                })
        } else {
            false
        }
    }

    /// Remove all tags matching the given filter off the internal tag queue.
    fn unqueue_filtered(
        self,
        mut filter: impl FnMut(&mut QueuedTagList) -> Option<QueuedTag>,
    ) -> Vec<(Depth, QueuedTag)> {
        use std::collections::hash_map::Entry;

        let mut queued_tags = self.0.queued_tags.borrow_mut();
        let mut unqueued: Vec<_> = queued_tags
            .iter_mut()
            .filter_map(|(d, q)| filter(q).map(|q| (*d, q)))
            .collect();

        unqueued.sort_by_key(|(_, t)| t.tag_start);

        for (depth, _tag) in unqueued.iter() {
            match queued_tags.entry(*depth) {
                Entry::Occupied(e) if matches!(e.get(), QueuedTagList::None) => {
                    e.remove();
                }
                _ => (),
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
    pub fn avm1_unload_movie(self, context: &mut UpdateContext<'gc>) {
        // TODO: In Flash player, the MovieClip properties change to the unloaded state
        // one frame after the unloadMovie command has been read, even if the MovieClip
        // is not a root MovieClip (see the movieclip_library_state_values test).
        // However, if avm1_unload and transform_to_unloaded_state are called with a one
        // frame delay when the MovieClip is not a root MovieClip, regressions appear.
        // Ruffle is probably replacing a MovieClip differently to Flash, therefore
        // introducing these regressions when trying to emulate that delay.

        if self.is_root() {
            let player = context.player_handle();
            let mc = MovieClipHandle::stash(context, self);
            let future = Box::pin(async move {
                player
                    .lock()
                    .unwrap()
                    .update(|uc| -> Result<(), loader::Error> {
                        let mc = mc.fetch(uc);
                        mc.avm1_unload(uc);
                        mc.transform_to_unloaded_state(uc);
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
    fn transform_to_unloaded_state(self, context: &mut UpdateContext<'gc>) {
        let movie = if let Some(DisplayObject::MovieClip(parent_mc)) = self.parent() {
            let parent_movie = parent_mc.movie();
            let parent_version = parent_movie.version();
            let parent_url = parent_movie.url();
            let mut unloaded_movie = SwfMovie::empty(parent_version, None);
            unloaded_movie.set_url(parent_url.to_string());

            Some(Arc::new(unloaded_movie))
        } else {
            None
        };

        self.replace_with_movie(context, movie, self.is_root(), None);
    }

    pub fn attach_audio(self, context: &mut UpdateContext<'gc>, netstream: Option<NetStream<'gc>>) {
        let old_netstream = self.0.attached_audio.get();
        if netstream != old_netstream {
            if let Some(old_netstream) = old_netstream {
                old_netstream.was_detached(context);
            }

            if let Some(netstream) = netstream {
                let write = Gc::write(context.gc(), self.0);
                unlock!(write, MovieClipData, attached_audio).set(Some(netstream));
                netstream.was_attached(context, self);
            } else {
                self.0.attached_audio.take();
            }
        }
    }

    pub fn run_frame_script_cleanup(context: &mut UpdateContext<'gc>) {
        while let Some(clip) = context.frame_script_cleanup_queue.pop_front() {
            clip.0.has_pending_script.set(true);
            clip.0.last_queued_script_frame.set(None);
            clip.run_local_frame_scripts(context);
        }
    }

    fn run_local_frame_scripts(self, context: &mut UpdateContext<'gc>) {
        let avm2_object = self.0.object2.get();

        if let Some(avm2_object) = avm2_object {
            if self.0.has_pending_script.get() {
                let frame_id = self.0.queued_script_frame.get();
                // If we are already executing frame scripts, then we shouldn't
                // run frame scripts recursively. This is because AVM2 can run
                // gotos, which will both queue and run frame scripts for the
                // whole movie again. If a goto is attempting to queue frame
                // scripts on us AGAIN, we should allow the current stack to
                // wind down before handling that.
                if !self
                    .0
                    .contains_flag(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT)
                {
                    let is_fresh_frame = self.0.last_queued_script_frame.get() != Some(frame_id);

                    if is_fresh_frame {
                        if let Some(callable) = self.frame_script(frame_id) {
                            self.0.last_queued_script_frame.set(Some(frame_id));
                            self.0.has_pending_script.set(false);
                            self.0
                                .set_flag(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT, true);

                            let movie = self.movie();
                            let domain = context
                                .library
                                .library_for_movie(movie)
                                .unwrap()
                                .avm2_domain();

                            if let Err(e) = Avm2::run_stack_frame_for_callable(
                                callable,
                                avm2_object.into(),
                                domain,
                                context,
                            ) {
                                tracing::error!(
                                    "Error occurred when running AVM2 frame script: {}",
                                    e
                                );
                            }

                            self.0
                                .set_flag(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT, false);
                        }
                    }
                }
            }
        }

        let goto_frame = self.0.queued_goto_frame.take();
        if let Some(frame) = goto_frame {
            self.run_goto(context, frame, false);
        }
    }
}

impl<'gc> TDisplayObject<'gc> for MovieClip<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.raw_interactive())
    }

    fn instantiate(self, mc: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(mc, (*self.0).clone())).into()
    }

    fn id(self) -> CharacterId {
        self.0.id()
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie()
    }

    fn enter_frame(self, context: &mut UpdateContext<'gc>) {
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
                child.base().set_skip_next_enter_frame(true);
            }
            child.enter_frame(context);
        }

        if skip_frame {
            self.base().set_skip_next_enter_frame(false);
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
            let data = self.0.shared.get().swf.clone();
            let place_actions = self.unqueue_filtered(|q| q.unqueue_add());

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
    fn construct_frame(self, context: &mut UpdateContext<'gc>) {
        // AVM1 code expects to execute in line with timeline instructions, so
        // it's exempted from frame construction.
        if self.movie().is_action_script_3()
            && (self.frames_loaded() >= 1 || self.header_frames() == 0)
        {
            let is_load_frame = !self.0.initialized();
            let needs_construction = if self.0.object2.get().is_none() {
                self.allocate_as_avm2_object(context, self.into());
                true
            } else {
                false
            };

            self.0.unset_loop_queued();

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
            } else if !(is_load_frame && self.placed_by_avm2_script()) {
                let running_construct_frame = self
                    .0
                    .contains_flag(MovieClipFlags::RUNNING_CONSTRUCT_FRAME);
                // The supercall constructor for display objects is responsible
                // for triggering construct_frame on frame 1.
                for child in self.iter_render_list() {
                    if running_construct_frame && child.object2().is_none() {
                        continue;
                    }
                    child.construct_frame(context);
                }
            }
        }

        if *context.frame_phase == FramePhase::Construct {
            // Check for frame-scripts before starting the frame-script phase,
            // to differentiate the pre-existing scripts from those introduced during frame-script phase.
            let has_pending_script = self.has_frame_script(self.0.current_frame.get());
            self.0.has_pending_script.set(has_pending_script);
        }
    }

    fn run_frame_scripts(self, context: &mut UpdateContext<'gc>) {
        self.run_local_frame_scripts(context);

        for child in self.iter_render_list() {
            child.run_frame_scripts(context);
        }
    }

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        if let Some(drawing) = self.drawing() {
            drawing.render(context);
        }
        self.render_children(context);
    }

    fn self_bounds(self) -> Rectangle<Twips> {
        self.drawing().map(|d| d.self_bounds()).unwrap_or_default()
    }

    fn hit_test_shape(
        self,
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

            if !options.contains(HitTestOptions::SKIP_CHILDREN) {
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
            }

            let point = local_matrix * point;
            if let Some(drawing) = self.drawing() {
                if drawing.hit_test(point, &local_matrix) {
                    return true;
                }
            }
        }

        false
    }

    fn as_drawing(&self) -> Option<RefMut<'_, Drawing>> {
        Some(self.drawing_mut())
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        init_object: Option<Avm1Object<'gc>>,
        instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if self.0.contains_flag(MovieClipFlags::POST_INSTANTIATED) {
            // Ensure that the same clip doesn't get post-instantiated twice.
            return;
        }
        self.0.set_flag(MovieClipFlags::POST_INSTANTIATED, true);

        self.set_default_instance_name(context);

        if !self.movie().is_action_script_3() {
            context.avm1.add_to_exec_list(context.gc(), self);

            self.construct_as_avm1_object(context, init_object, instantiated_by, run_frame);
        }
    }

    fn object1(self) -> Option<Avm1Object<'gc>> {
        self.0.object1.get()
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.object2.get()
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        let write = Gc::write(context.gc(), self.0);
        unlock!(write, MovieClipData, object2).set(Some(to));
        if self.parent().is_none() {
            context.orphan_manager.add_orphan_obj(self.into());
        }
    }

    fn set_perspective_projection(self, mut perspective_projection: Option<PerspectiveProjection>) {
        if perspective_projection.is_none() && self.is_root() {
            // `root` doesn't allow null PerspectiveProjection.
            perspective_projection = Some(PerspectiveProjection {
                field_of_view: 55.0,
                center: (
                    self.movie().width().to_pixels() / 2.0,
                    self.movie().height().to_pixels() / 2.0,
                ),
            });
        }

        if self
            .base()
            .set_perspective_projection(perspective_projection)
        {
            if let Some(parent) = self.parent() {
                // Self-transform changes are automatically handled,
                // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                parent.invalidate_cached_bitmap();
            }
        }
    }

    fn on_parent_removed(self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() {
            context.orphan_manager.add_orphan_obj(self.into())
        }
    }

    fn avm1_unload(self, context: &mut UpdateContext<'gc>) {
        for child in self.iter_render_list() {
            child.avm1_unload(context);
        }

        if let Some(node) = self.maskee() {
            node.set_masker(context.gc(), None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc(), None, true);
        }

        // Unregister any text field variable bindings.
        Avm1TextFieldBinding::unregister_bindings(self.into(), context);

        self.drop_focus(context);

        self.0.stop_audio_stream(context);

        if self.is_root() {
            context
                .audio_manager
                .stop_sounds_on_parent_and_children(context.audio, self.into());
        }

        // If this clip is currently pending removal, then it unload event will have already been dispatched
        if !self.avm1_pending_removal() {
            self.event_dispatch(context, ClipEvent::Unload);
        }

        self.set_avm1_removed(true);
    }

    fn avm1_text_field_bindings(&self) -> Option<Ref<'_, [Avm1TextFieldBinding<'gc>]>> {
        let obj = self.0.object1.get();
        obj.map(|_| {
            let read = self.0.cell.borrow();
            Ref::map(read, |r| r.avm1_text_field_bindings.as_slice())
        })
    }

    fn avm1_text_field_bindings_mut(
        &self,
        mc: &Mutation<'gc>,
    ) -> Option<RefMut<'_, Vec<Avm1TextFieldBinding<'gc>>>> {
        let obj = self.0.object1.get();
        obj.map(|_| {
            let write = unlock!(Gc::write(mc, self.0), MovieClipData, cell);
            RefMut::map(write.borrow_mut(), |r| &mut r.avm1_text_field_bindings)
        })
    }

    fn loader_info(self) -> Option<LoaderInfoObject<'gc>> {
        self.0.shared.get().loader_info
    }

    fn allow_as_mask(self) -> bool {
        !self.is_empty()
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for MovieClip<'gc> {
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>> {
        Ref::map(self.0.cell.borrow(), |r| &r.container)
    }

    fn raw_container_mut(&self, mc: &Mutation<'gc>) -> RefMut<'_, ChildContainer<'gc>> {
        let write = unlock!(Gc::write(mc, self.0), MovieClipData, cell);
        RefMut::map(write.borrow_mut(), |r| &mut r.container)
    }

    fn is_tab_children_avm1(self, context: &mut UpdateContext<'gc>) -> bool {
        self.get_avm1_boolean_property(istr!(context, "tabChildren"), context, |_| true)
    }
}

impl<'gc> TInteractiveObject<'gc> for MovieClip<'gc> {
    fn raw_interactive(self) -> Gc<'gc, InteractiveObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
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
        if let Some(object) = self.0.object1.get() {
            let swf_version = self.0.movie().version();
            if swf_version >= 5 {
                if let Some(flag) = event.flag() {
                    for event_handler in self
                        .clip_actions()
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
                    if let Some(name) = event.method_name(&context.strings) {
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
        self,
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        // Don't do anything if run in an AVM2 context.
        if self.movie().is_action_script_3() {
            return None;
        }

        if self.visible() {
            let this: InteractiveObject<'gc> = self.into();
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
                if let Some(drawing) = self.drawing() {
                    if drawing.hit_test(point, &local_matrix) {
                        return Some(this);
                    }
                }
            }
        }

        None
    }

    fn mouse_pick_avm2(
        self,
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        // Don't do anything if run in an AVM1 context.
        if !self.movie().is_action_script_3() {
            return Avm2MousePick::Miss;
        }

        if self.visible() {
            let this: InteractiveObject<'gc> = self.into();
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
                } else if child.hit_test_shape(context, point, options)
                    && child
                        .masker()
                        .map(|mask| mask.hit_test_shape(context, point, options))
                        .unwrap_or(true)
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
                        return res.combine_with_parent(self.into());
                    }
                    Avm2MousePick::PropagateToParent => {
                        found_propagate = Some(res);
                    }
                    Avm2MousePick::Miss => {}
                }
            }

            // A 'propagated' event from a child seems to have lower 'priority' than anything else.
            if let Some(propagate) = found_propagate {
                return propagate.combine_with_parent(self.into());
            }

            // Check drawing, because this selects the current clip, it must have mouse enabled
            if self.world_bounds().contains(point) {
                let point = local_matrix * point;

                if let Some(drawing) = self.drawing() {
                    if drawing.hit_test(point, &local_matrix) {
                        return if self.mouse_enabled() {
                            Avm2MousePick::Hit(self.into())
                        } else {
                            Avm2MousePick::PropagateToParent
                        };
                    }
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

    fn is_focusable(self, context: &mut UpdateContext<'gc>) -> bool {
        if self.is_root() {
            false
        } else if self.is_button_mode(context) {
            true
        } else {
            self.get_avm1_boolean_property(istr!(context, "focusEnabled"), context, |_| false)
        }
    }

    fn is_highlightable(self, context: &mut UpdateContext<'gc>) -> bool {
        // Root movie clips are not highlightable.
        // This applies only to AVM2, as in AVM1 they are also not focusable.
        !self.is_root() && self.is_highlight_enabled(context)
    }

    fn is_tabbable(self, context: &mut UpdateContext<'gc>) -> bool {
        if self.is_root() {
            // Root movie clips are never tabbable.
            return false;
        }
        self.tab_enabled(context)
    }

    fn tab_enabled_default(self, context: &mut UpdateContext<'gc>) -> bool {
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
        self.shared.get().id
    }

    fn set_flag(&self, flag: MovieClipFlags, value: bool) {
        let mut flags = self.flags.get();
        flags.set(flag, value);
        self.flags.set(flags);
    }

    fn contains_flag(&self, flag: MovieClipFlags) -> bool {
        self.flags.get().contains(flag)
    }

    fn current_frame(&self) -> FrameNumber {
        self.current_frame.get()
    }

    fn increment_current_frame(&self) {
        let frame = self.current_frame.get();
        self.current_frame.set(frame + 1);
    }

    fn decrement_current_frame(&self) {
        let frame = self.current_frame.get();
        self.current_frame.set(frame - 1);
    }

    fn header_frames(&self) -> FrameNumber {
        self.shared.get().header_frames
    }

    fn frames_loaded(&self) -> i32 {
        (self.shared.get().preload_progress.cur_preload_frame.get()) as i32 - 1
    }

    fn playing(&self) -> bool {
        self.contains_flag(MovieClipFlags::PLAYING)
    }

    fn set_playing(&self, value: bool) {
        self.set_flag(MovieClipFlags::PLAYING, value);
    }

    fn programmatically_played(&self) -> bool {
        self.contains_flag(MovieClipFlags::PROGRAMMATICALLY_PLAYED)
    }

    fn set_programmatically_played(&self) {
        self.set_flag(MovieClipFlags::PROGRAMMATICALLY_PLAYED, true);
    }

    fn loop_queued(&self) -> bool {
        self.contains_flag(MovieClipFlags::LOOP_QUEUED)
    }

    fn set_loop_queued(&self) {
        self.set_flag(MovieClipFlags::LOOP_QUEUED, true);
    }

    fn unset_loop_queued(&self) {
        self.set_flag(MovieClipFlags::LOOP_QUEUED, false);
    }

    fn play(&self) {
        self.set_playing(true);
    }

    fn stop(&self, context: &mut UpdateContext<'gc>) {
        self.set_playing(false);
        self.stop_audio_stream(context);
    }

    fn tag_stream_len(&self) -> usize {
        self.shared.get().swf.end - self.shared.get().swf.start
    }

    /// Handles a PlaceObject tag when running a goto action.
    #[inline]
    fn goto_place_object<'a>(
        &self,
        reader: &mut SwfStream<'a>,
        version: u8,
        goto_commands: &mut Vec<GotoPlaceObject<'a>>,
        is_rewind: bool,
        index: usize,
    ) -> Result<(), Error> {
        let swf_ptr = self.shared.get().swf.as_ref().as_ptr();
        let tag_ptr = reader.get_ref().as_ptr();
        let tag_start = (tag_ptr.addr() - swf_ptr.addr()) as u64;
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

    fn initialized(&self) -> bool {
        self.contains_flag(MovieClipFlags::INITIALIZED)
    }

    fn set_initialized(&self, value: bool) -> bool {
        let ret = self.contains_flag(MovieClipFlags::INITIALIZED);
        self.set_flag(MovieClipFlags::INITIALIZED, value);
        !ret
    }

    /// Stops the audio stream if one is playing.
    fn stop_audio_stream(&self, context: &mut UpdateContext<'gc>) {
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
        let symbol_name = self.shared.get().exported_name.get();
        let symbol_name = symbol_name.as_ref()?;
        context
            .avm1
            .get_registered_constructor(self.movie().version(), *symbol_name)
    }

    pub fn movie(&self) -> Arc<SwfMovie> {
        self.shared.get().movie()
    }
}

// Preloading of definition tags
impl<'gc, 'a> MovieClipShared<'gc> {
    #[inline]
    fn define_bits_lossless(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let define_bits_lossless = reader.read_define_bits_lossless(version)?;
        let bitmap = Gc::new(
            context.gc(),
            BitmapCharacter::new(CompressedBitmap::Lossless(DefineBitsLossless {
                id: define_bits_lossless.id,
                format: define_bits_lossless.format,
                width: define_bits_lossless.width,
                height: define_bits_lossless.height,
                version: define_bits_lossless.version,
                data: Cow::Owned(define_bits_lossless.data.into_owned()),
            })),
        );
        self.library_mut(context)
            .register_character(define_bits_lossless.id, Character::Bitmap(bitmap));
        Ok(())
    }

    #[inline]
    fn define_scaling_grid(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let rect = reader.read_rectangle()?;
        if let Some(character) = self.library_mut(context).character_by_id(id) {
            if let Character::MovieClip(clip) = character {
                clip.set_scaling_grid(rect);
            } else {
                tracing::warn!("DefineScalingGrid for invalid ID {}", id);
            }
        }
        Ok(())
    }

    #[inline]
    fn define_morph_shape(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let tag = reader.read_define_morph_shape(version)?;
        let id = tag.id;
        let morph_shape = MorphShape::from_swf_tag(context.gc(), tag, self.movie());
        self.library_mut(context)
            .register_character(id, Character::MorphShape(morph_shape));
        Ok(())
    }

    #[inline]
    fn define_shape(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let swf_shape = reader.read_define_shape(version)?;
        let id = swf_shape.id;
        let graphic = Graphic::from_swf_tag(context, swf_shape, self.movie());
        self.library_mut(context)
            .register_character(id, Character::Graphic(graphic));
        Ok(())
    }

    #[inline]
    fn sound_stream_head(&self, reader: &mut SwfStream<'a>, _version: u8) -> Result<(), Error> {
        let mut shared = self.cell.borrow_mut();
        shared.audio_stream_info = Some(reader.read_sound_stream_head()?);
        Ok(())
    }

    #[inline]
    fn csm_text_settings(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let settings = reader.read_csm_text_settings()?;
        match self.library_mut(context).character_by_id(settings.id) {
            Some(Character::Text(text)) => {
                text.set_render_settings(settings.into());
            }
            Some(Character::EditText(edit_text)) => {
                edit_text.set_render_settings(settings.into());
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
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream,
    ) -> Result<(), Error> {
        let vframe = reader.read_video_frame()?;
        match self.library_mut(context).character_by_id(vframe.stream_id) {
            Some(Character::Video(v)) => {
                v.preload_swf_frame(vframe);
                Ok(())
            }
            _ => Err(Error::PreloadVideoIntoInvalidCharacter(vframe.stream_id)),
        }
    }

    #[inline]
    fn define_bits(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let mc = context.gc();
        let library = self.library_mut(context);
        let (id, jpeg_data) = reader.read_define_bits()?;
        let jpeg_tables = library.jpeg_tables();
        let jpeg_data =
            ruffle_render::utils::glue_tables_to_jpeg(jpeg_data, jpeg_tables).into_owned();
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(&jpeg_data)?;
        let bitmap = Character::Bitmap(Gc::new(
            mc,
            BitmapCharacter::new(CompressedBitmap::Jpeg {
                data: jpeg_data,
                alpha: None,
                width,
                height,
            }),
        ));
        library.register_character(id, bitmap);
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_2(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let (id, jpeg_data) = reader.read_define_bits_jpeg_2()?;
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(jpeg_data)?;
        let bitmap = Character::Bitmap(Gc::new(
            context.gc(),
            BitmapCharacter::new(CompressedBitmap::Jpeg {
                data: jpeg_data.to_owned(),
                alpha: None,
                width,
                height,
            }),
        ));
        self.library_mut(context).register_character(id, bitmap);
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_3_or_4(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let jpeg = reader.read_define_bits_jpeg_3(version)?;
        let (width, height) = ruffle_render::utils::decode_define_bits_jpeg_dimensions(jpeg.data)?;

        let bitmap = Character::Bitmap(Gc::new(
            context.gc(),
            BitmapCharacter::new(CompressedBitmap::Jpeg {
                data: jpeg.data.to_owned(),
                alpha: Some(jpeg.alpha_data.to_owned()),
                width,
                height,
            }),
        ));
        self.library_mut(context)
            .register_character(jpeg.id, bitmap);
        Ok(())
    }

    #[inline]
    fn define_button_1(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let swf_button = reader.read_define_button_1()?;

        self.define_button_any(context, swf_button)
    }

    #[inline]
    fn define_button_2(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let swf_button = reader.read_define_button_2()?;

        self.define_button_any(context, swf_button)
    }

    #[inline]
    fn define_button_any(
        &self,
        context: &mut UpdateContext<'gc>,
        swf_button: swf::Button<'a>,
    ) -> Result<(), Error> {
        let button = if self.swf.movie.is_action_script_3() {
            Character::Avm2Button(Avm2Button::from_swf_tag(
                &swf_button,
                &self.swf,
                context,
                true,
            ))
        } else {
            Character::Avm1Button(Avm1Button::from_swf_tag(
                &swf_button,
                &self.swf,
                context.gc(),
            ))
        };
        self.library_mut(context)
            .register_character(swf_button.id, button);
        Ok(())
    }

    #[inline]
    fn define_button_cxform(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let button_colors = reader.read_define_button_cxform()?;
        match self.library_mut(context).character_by_id(button_colors.id) {
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
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let button_sounds = reader.read_define_button_sound()?;
        match self.library_mut(context).character_by_id(button_sounds.id) {
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
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let swf_edit_text = reader.read_define_edit_text()?;
        let edit_text = EditText::from_swf_tag(context, self.movie(), swf_edit_text);
        self.library_mut(context)
            .register_character(edit_text.id(), Character::EditText(edit_text));
        Ok(())
    }

    #[inline]
    fn define_font_1(
        &self,
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
            context.gc(),
            context.renderer,
            font,
            reader.encoding(),
            FontType::Embedded,
        );
        self.library_mut(context)
            .register_character(font_id, Character::Font(font_object));
        Ok(())
    }

    #[inline]
    fn define_font_2(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_2(2)?;
        let font_id = font.id;
        let font_object = Font::from_swf_tag(
            context.gc(),
            context.renderer,
            font,
            reader.encoding(),
            FontType::Embedded,
        );
        self.library_mut(context)
            .register_character(font_id, Character::Font(font_object));
        Ok(())
    }

    #[inline]
    fn define_font_3(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_2(3)?;
        let font_id = font.id;
        let font_object = Font::from_swf_tag(
            context.gc(),
            context.renderer,
            font,
            reader.encoding(),
            FontType::Embedded,
        );
        self.library_mut(context)
            .register_character(font_id, Character::Font(font_object));
        Ok(())
    }

    #[inline]
    fn define_font_4(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_4()?;
        let font_id = font.id;
        let font_object = Font::from_font4_tag(context.gc(), font, reader.encoding())?;
        self.library_mut(context)
            .register_character(font_id, Character::Font(font_object));
        Ok(())
    }

    #[inline]
    fn define_sound(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let sound = reader.read_define_sound()?;
        if let Ok(handle) = context.audio.register_sound(&sound) {
            self.library_mut(context)
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
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream,
    ) -> Result<(), Error> {
        let streamdef = reader.read_define_video_stream()?;
        let id = streamdef.id;
        let video = Video::from_swf_tag(self.movie(), streamdef, context.gc());
        self.library_mut(context)
            .register_character(id, Character::Video(video));
        Ok(())
    }

    fn define_sprite(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        tag_len: usize,
        chunk_limit: &mut ExecutionLimit,
    ) -> Result<ControlFlow, Error> {
        let start = reader.as_slice();
        let id = reader.read_character_id()?;
        let num_frames = reader.read_u16()?;
        let num_read = reader.pos(start);

        let movie_clip = MovieClip::new_with_data(
            context.gc(),
            id,
            self.swf.resize_to_reader(reader, tag_len - num_read),
            num_frames,
        );

        if self
            .library_mut(context)
            .register_character(id, Character::MovieClip(movie_clip))
        {
            self.preload_progress.cur_preload_symbol.set(Some(id));
        } else {
            // This character was already defined, so we can skip preloading it, as the
            // character ID refers to the pre-existing character, and not this one.
            return Ok(ControlFlow::Exit);
        }

        let should_exit = chunk_limit.did_ops_breach_limit(context, 4);
        if should_exit {
            return Ok(ControlFlow::Exit);
        }

        if movie_clip.preload(context, chunk_limit) {
            self.preload_progress.cur_preload_symbol.set(None);

            Ok(ControlFlow::Continue)
        } else {
            Ok(ControlFlow::Exit)
        }
    }

    #[inline]
    fn define_text(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let text = reader.read_define_text(version)?;
        let text_object = Text::from_swf_tag(context, self.movie(), &text);
        self.library_mut(context)
            .register_character(text.id, Character::Text(text_object));
        Ok(())
    }

    #[inline]
    fn define_binary_data(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let tag_data = reader.read_define_binary_data()?;
        let binary_data = BinaryData::from_swf_tag(self.movie(), &tag_data);
        let binary_data = Gc::new(context.gc(), binary_data);
        self.library_mut(context)
            .register_character(tag_data.id, Character::BinaryData(binary_data));
        Ok(())
    }

    #[inline]
    fn import_exports_of_importer(&self, context: &mut UpdateContext<'gc>) {
        let Some(importer_library) = self
            .importer_movie
            .and_then(|mc| context.library.library_for_movie(mc.movie()))
        else {
            return;
        };

        let exported_from_importer = importer_library
            .export_characters()
            .iter()
            .map(|(name, id)| {
                let character = importer_library.character_by_id(*id).unwrap();
                (name, (*id, character))
            })
            .collect::<HashMap<AvmString<'gc>, (CharacterId, Character<'gc>)>>();

        let self_library = self.library_mut(context);
        for (name, (id, character)) in exported_from_importer {
            if self_library.character_by_id(id).is_none() {
                self_library.register_character(id, character);
                self_library.register_export(id, name);
            }
        }
    }

    #[inline]
    fn script_limits(&self, reader: &mut SwfStream<'a>, avm: &mut Avm1<'gc>) -> Result<(), Error> {
        let max_recursion_depth = reader.read_u16()?;
        let _timeout_in_seconds = reader.read_u16()?;

        avm.set_max_recursion_depth(max_recursion_depth);

        Ok(())
    }

    fn register_export(
        context: &mut UpdateContext<'gc>,
        id: CharacterId,
        name: AvmString<'gc>,
        movie: Arc<SwfMovie>,
    ) {
        let mc = context.gc();
        let library = context.library.library_for_movie_mut(movie);
        library.register_export(id, name);

        // TODO: do other types of Character need to know their exported name?
        if let Some(character) = library.character_by_id(id) {
            if let Character::MovieClip(clip) = character {
                let data = Gc::write(mc, clip.0.shared.get());
                unlock!(data, MovieClipShared, exported_name).set(Some(name));
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
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let exports = reader.read_export_assets()?;
        let importer_movie = self.importer_movie.map(|mc| mc.movie());
        for export in exports {
            let name = export.name.decode(reader.encoding());
            let name = AvmString::new(context.gc(), name);

            if let Some(character) = self
                .library(context)
                .and_then(|l| l.character_by_id(export.id))
            {
                Self::register_export(context, export.id, name, self.movie());
                tracing::debug!("register_export asset: {} (ID: {})", name, export.id);

                if let Some(parent) = &importer_movie {
                    let parent_library = context.library.library_for_movie_mut(parent.clone());

                    if let Some(id) = parent_library.character_id_by_import_name(name) {
                        parent_library.register_character(id, character);

                        Self::register_export(context, id, name, parent.clone());
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
    fn frame_label(&self, reader: &mut SwfStream<'a>) -> Result<(), Error> {
        let cur_frame = self.preload_progress.cur_preload_frame.get();
        let mut shared = self.cell.borrow_mut();

        // This tag is ignored if scene labels exist.
        if !shared.scene_labels.is_empty() {
            return Ok(());
        }

        let frame_label = reader.read_frame_label()?;
        let mut label = frame_label.label.decode(reader.encoding()).into_owned();

        // In AVM1, frame labels are case insensitive (ASCII), but in AVM2 they are case sensitive.
        if !self.swf.movie.is_action_script_3() {
            label.make_ascii_lowercase();
        }

        shared.frame_labels.push((cur_frame, label.clone()));
        if let std::collections::hash_map::Entry::Vacant(v) = shared.frame_labels_map.entry(label) {
            v.insert(cur_frame);
        } else {
            tracing::warn!("Movie clip {}: Duplicated frame label", self.id);
        }
        Ok(())
    }

    #[inline]
    fn scene_and_frame_labels(&self, reader: &mut SwfStream<'_>) -> Result<(), Error> {
        let mut shared = self.cell.borrow_mut();
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
                .unwrap_or_else(|| self.header_frames + 1);

            let scene = Scene {
                name: label.decode(reader.encoding()).into_owned(),
                start,
                length: end - start,
            };
            shared.scene_labels.push(scene.clone());
            if let std::collections::hash_map::Entry::Vacant(v) =
                shared.scene_labels_map.entry(scene.name.clone())
            {
                v.insert(scene);
            } else {
                tracing::warn!("Movie clip {}: Duplicated scene label", self.id);
            }
        }

        for FrameLabelData { frame_num, label } in sfl_data.frame_labels {
            let label = label.decode(reader.encoding()).into_owned();
            shared
                .frame_labels
                .push((frame_num as u16 + 1, label.clone()));
            if let std::collections::hash_map::Entry::Vacant(v) =
                shared.frame_labels_map.entry(label)
            {
                v.insert(frame_num as u16 + 1);
            } else {
                tracing::warn!("Movie clip {}: Duplicated frame label", self.id);
            }
        }

        Ok(())
    }

    #[inline]
    fn jpeg_tables(
        &self,
        context: &mut UpdateContext<'gc>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let jpeg_data = reader.read_slice_to_end();
        self.library_mut(context).set_jpeg_tables(jpeg_data);
        Ok(())
    }

    #[inline]
    fn show_frame(
        &self,
        #[allow(unused)] reader: &mut SwfStream<'a>,
        #[allow(unused)] tag_len: usize,
    ) -> Result<(), Error> {
        let progress = &self.preload_progress;

        #[cfg(feature = "timeline_debug")]
        {
            let tag_stream_start = self.swf.as_ref().as_ptr() as u64;
            let end_pos = reader.get_ref().as_ptr() as u64 - tag_stream_start;

            // We add tag_len because the reader position doesn't take it into
            // account. Strictly speaking ShowFrame should not have tag data, but
            // who *knows* what weird obfuscation hacks people have done with it.
            self.cell.borrow_mut().tag_frame_boundaries.insert(
                progress.cur_preload_frame.get(),
                (progress.start_pos.get(), end_pos + tag_len as u64),
            );

            progress.start_pos.set(end_pos);
        }

        let cur = &progress.cur_preload_frame;
        cur.set(cur.get() + 1);

        Ok(())
    }

    /// Handles a DoAbc or DoAbc2 tag
    #[inline]
    fn preload_bytecode_tag(
        &self,
        tag_code: TagCode,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let cur_frame = self.preload_progress.cur_preload_frame.get() - 1;
        let abc = match tag_code {
            TagCode::DoAbc | TagCode::DoAbc2 => {
                self.swf.resize_to_reader(reader, reader.as_slice().len())
            }
            _ => unreachable!(),
        };

        // If we got an eager script (which happens for non-lazy DoAbc2 tags).
        // we store it for later. It will be run the first time we execute this frame
        // (for any instance of this MovieClip) in `run_eager_script_and_symbol`
        let mut shared = self.cell.borrow_mut();
        let eager_tags = shared.eager_tags.entry(cur_frame).or_default();
        eager_tags.abc_tags.push((tag_code, abc));
        Ok(())
    }

    #[inline]
    fn preload_symbol_class(&self, reader: &mut SwfStream<'a>) -> Result<(), Error> {
        let cur_frame = self.preload_progress.cur_preload_frame.get() - 1;
        let mut shared = self.cell.borrow_mut();
        let eager_tags = shared.eager_tags.entry(cur_frame).or_default();
        let num_symbols = reader.read_u16()?;

        for _ in 0..num_symbols {
            let id = reader.read_u16()?;
            let class_name = reader.read_str()?.decode(reader.encoding());

            // Store the name and symbol with in the global data for this frame. The first time
            // we execute this frame (for any instance of this MovieClip), we will load the symbolclass
            // from `run_eager_script_and_symbol`
            eager_tags.symbolclass_names.push((class_name.into(), id));
        }
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
        let slice = self.0.shared.get().swf.resize_to_reader(reader, tag_len);
        if !slice.is_empty() {
            context.action_queue.queue_action(
                self.into(),
                ActionType::Normal { bytecode: slice },
                false,
            );
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
        let Some(eager_tags) = self.0.shared.get().take_eager_tags(current_frame) else {
            return Ok(());
        };

        let mut eager_scripts = Vec::new();

        for (tag_code, abc) in eager_tags.abc_tags {
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

        if !eager_tags.symbolclass_names.is_empty() {
            let mut activation = Avm2Activation::from_nothing(context);

            let movie = self.movie();

            let library = activation
                .context
                .library
                .library_for_movie_mut(movie.clone());
            let domain = library.avm2_domain();

            for (class_name, id) in eager_tags.symbolclass_names {
                let name = AvmString::new(activation.gc(), class_name);
                match Avm2::lookup_class_for_character(&mut activation, self, domain, name, id) {
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
                            Some(Character::EditText(edit_text)) => {
                                edit_text.set_avm2_class(activation.gc(), class_object)
                            }
                            Some(Character::Graphic(graphic)) => {
                                graphic.set_avm2_class(activation.gc(), class_object)
                            }
                            Some(Character::MovieClip(mc)) => {
                                mc.set_avm2_class(activation.gc(), Some(class_object))
                            }
                            Some(Character::Avm2Button(btn)) => {
                                btn.set_avm2_class(activation.gc(), class_object)
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

                                    let Some(Character::Bitmap(bitmap)) =
                                        library.character_by_id(id)
                                    else {
                                        unreachable!();
                                    };
                                    BitmapCharacter::set_avm2_class(
                                        bitmap,
                                        bitmap_class,
                                        activation.gc(),
                                    );
                                } else {
                                    tracing::error!("Associated class {:?} for symbol {} must extend flash.display.Bitmap or BitmapData, does neither", class_object.inner_class_definition().name(), id);
                                }
                            }
                            None => {
                                // Most SWFs use id 0 here, but some obfuscated SWFs can use other invalid IDs.
                                if self.avm2_class().is_none() {
                                    self.set_avm2_class(activation.gc(), Some(class_object));
                                }

                                // We also need to register this MovieClip as a character now.
                                // Use 'instantiate' to clone movie clip data, so future
                                // instantiations don't reflect changes made to the loaded
                                // main timeline instance.

                                let instantiated =
                                    self.instantiate(activation.gc()).as_movie_clip().unwrap();

                                let library = activation
                                    .context
                                    .library
                                    .library_for_movie_mut(movie.clone());

                                library.register_character(id, Character::MovieClip(instantiated));
                            }
                            _ => {
                                tracing::warn!(
                                    "Symbol class {name} cannot be assigned to character id {id}",
                                );
                            }
                        }
                    }
                    Err(e) => tracing::error!(
                        "Got AVM2 error when attempting to lookup symbol class: {e:?}",
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

    fn queue_place_object(self, reader: &mut SwfStream<'a>, version: u8) -> Result<(), Error> {
        let swf_ptr = self.0.shared.get().swf.as_ref().as_ptr();
        let tag_ptr = reader.get_ref().as_ptr();
        let tag_start = (tag_ptr.addr() - swf_ptr.addr()) as u64;
        let place_object = if version == 1 {
            reader.read_place_object()
        } else {
            reader.read_place_object_2_or_3(version)
        }?;

        let new_tag = QueuedTag {
            tag_type: QueuedTagAction::Place(version),
            tag_start,
        };
        let mut queued_tags = self.0.queued_tags.borrow_mut();
        let bucket = queued_tags
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
                    child.set_place_frame(self.current_frame());
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
            if !child.placed_by_avm2_script() {
                self.remove_child(context, child);
            } else {
                self.remove_child_from_depth_list(context, child);
            }
        }

        Ok(())
    }

    #[inline]
    fn queue_remove_object(self, reader: &mut SwfStream<'a>, version: u8) -> Result<(), Error> {
        let swf_ptr = self.0.shared.get().swf.as_ref().as_ptr();
        let tag_ptr = reader.get_ref().as_ptr();
        let tag_start = (tag_ptr.addr() - swf_ptr.addr()) as u64;
        let remove_object = if version == 1 {
            reader.read_remove_object_1()
        } else {
            reader.read_remove_object_2()
        }?;

        let new_tag = QueuedTag {
            tag_type: QueuedTagAction::Remove(version),
            tag_start,
        };
        let mut queued_tags = self.0.queued_tags.borrow_mut();
        let bucket = queued_tags
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
            context.stage.set_background_color(Some(background_color));
        }
        Ok(())
    }

    #[inline]
    fn sound_stream_block(
        self,
        context: &mut UpdateContext<'gc>,
        _reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let audio_stream = self.0.playing().then(|| {
            let read = self.0.shared_cell();
            if let (Some(stream_info), None) = (&read.audio_stream_info, self.0.audio_stream.get())
            {
                let slice = self.0.shared.get().swf.to_start_and_end(
                    self.0.tag_stream_pos.get() as usize,
                    self.0.tag_stream_len(),
                );
                Some(context.start_stream(self, self.0.current_frame(), slice, stream_info))
            } else {
                None
            }
        });

        if let Some(stream) = audio_stream.flatten() {
            self.0.audio_stream.set(stream);
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

    pub fn set_constructing_frame(self, val: bool) {
        self.0
            .set_flag(MovieClipFlags::RUNNING_CONSTRUCT_FRAME, val);
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
struct PreloadProgress {
    /// The SWF offset to start the next progress chunk from.
    next_preload_chunk: Cell<u64>,

    /// The current frame being preloaded.
    cur_preload_frame: Cell<u16>,

    /// The SWF offset that the current frame started in.
    #[cfg(feature = "timeline_debug")]
    start_pos: Cell<u64>,

    /// The symbol we are currently asynchronously preloading.
    cur_preload_symbol: Cell<Option<CharacterId>>,

    /// If this movie is currently executing an ImportAssets/2.
    /// If true, this movie should **not** execute, and should be considered as still loading.
    awaiting_import: Cell<bool>,
}

impl Default for PreloadProgress {
    fn default() -> Self {
        Self {
            next_preload_chunk: Cell::new(0),
            cur_preload_frame: Cell::new(1),
            #[cfg(feature = "timeline_debug")]
            start_pos: Cell::new(0),
            cur_preload_symbol: Cell::new(None),
            awaiting_import: Cell::new(false),
        }
    }
}

/// Data shared between all instances of a movie clip.
#[derive(Collect)]
#[collect(no_drop)]
struct MovieClipShared<'gc> {
    cell: RefCell<MovieClipSharedMut>,
    id: CharacterId,
    swf: SwfSlice,
    header_frames: FrameNumber,
    /// Preload progress for the given clip's tag stream.
    #[collect(require_static)]
    preload_progress: PreloadProgress,
    /// The last known symbol name under which this movie clip was exported.
    /// Used for looking up constructors registered with `Object.registerClass`.
    exported_name: Lock<Option<AvmString<'gc>>>,
    avm2_class: Lock<Option<Avm2ClassObject<'gc>>>,
    /// Only set if this MovieClip is the root movie in an SWF
    /// (either the root SWF initially loaded by the player,
    /// or an SWF dynamically loaded by `Loader`)
    ///
    /// This is always `None` for the AVM1 root movie.
    /// However, it will be set for an AVM1 movie loaded from AVM2
    /// via `Loader`
    loader_info: Option<LoaderInfoObject<'gc>>,

    // If this movie was loaded from ImportAssets(2), this will be the root MovieClip of the parent movie.
    importer_movie: Option<MovieClip<'gc>>,
}

#[derive(Default)]
struct MovieClipSharedMut {
    frame_labels: Vec<(FrameNumber, WString)>,
    frame_labels_map: HashMap<WString, FrameNumber>,
    scene_labels: Vec<Scene>,
    scene_labels_map: HashMap<WString, Scene>,
    audio_stream_info: Option<swf::SoundStreamHead>,

    /// The tag stream start and stop positions for each frame in the clip.
    #[cfg(feature = "timeline_debug")]
    tag_frame_boundaries: HashMap<FrameNumber, (u64, u64)>,

    // This map holds DoAbc/SymbolClass data that was loaded during preloading, but hasn't
    // yet been executed. The first time we encounter a frame, we will remove the entry
    // from this map, and process it in `run_eager_script_and_symbol`
    eager_tags: HashMap<FrameNumber, EagerTags>,
}

#[derive(Default)]
struct EagerTags {
    abc_tags: Vec<(TagCode, SwfSlice)>,
    symbolclass_names: Vec<(WString, u16)>,
}

impl<'gc> MovieClipShared<'gc> {
    fn empty(movie: Arc<SwfMovie>) -> Self {
        let mut s = Self::with_data(0, SwfSlice::empty(movie), 1, None, None);

        *s.preload_progress.cur_preload_frame.get_mut() = s.header_frames + 1;

        s
    }

    fn with_data(
        id: CharacterId,
        swf: SwfSlice,
        header_frames: FrameNumber,
        loader_info: Option<LoaderInfoObject<'gc>>,
        importer_movie: Option<MovieClip<'gc>>,
    ) -> Self {
        Self {
            cell: Default::default(),
            id,
            swf,
            header_frames,
            preload_progress: Default::default(),
            exported_name: Lock::new(None),
            avm2_class: Lock::new(None),
            loader_info,
            importer_movie,
        }
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.swf.movie.clone()
    }

    fn library<'a>(&self, context: &'a UpdateContext<'gc>) -> Option<&'a MovieLibrary<'gc>> {
        context.library.library_for_movie(self.movie())
    }

    fn library_mut<'a>(&self, context: &'a mut UpdateContext<'gc>) -> &'a mut MovieLibrary<'gc> {
        context.library.library_for_movie_mut(self.movie())
    }

    fn take_eager_tags(&self, frame: FrameNumber) -> Option<EagerTags> {
        let mut write = self.cell.borrow_mut();
        let tags = write.eager_tags.remove(&frame);
        // Optimization: free memory when the last tag is removed.
        if tags.is_some() && write.eager_tags.is_empty() {
            write.eager_tags = HashMap::new();
        }
        tags
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
        const POST_INSTANTIATED = 1 << 6;
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
