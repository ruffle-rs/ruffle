//! `MovieClip` display object and support code.
use crate::avm1::{Object as Avm1Object, StageObject, TObject as Avm1TObject, Value as Avm1Value};
use crate::avm2::object::LoaderInfoObject;
use crate::avm2::object::LoaderStream;
use crate::avm2::Activation as Avm2Activation;
use crate::avm2::{
    Avm2, ClassObject as Avm2ClassObject, Error as Avm2Error, Multiname as Avm2Multiname,
    Object as Avm2Object, QName as Avm2QName, StageObject as Avm2StageObject,
    TObject as Avm2TObject, Value as Avm2Value,
};
use crate::backend::audio::{SoundHandle, SoundInstanceHandle};
use crate::backend::ui::MouseCursor;
use bitflags::bitflags;

use crate::avm1::Avm1;
use crate::avm1::{Activation as Avm1Activation, ActivationIdentifier};
use crate::binary_data::BinaryData;
use crate::character::Character;
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::container::{
    dispatch_added_event_only, dispatch_added_to_stage_event_only, dispatch_removed_event,
    ChildContainer, TDisplayObjectContainer,
};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{
    Avm1Button, Avm2Button, Bitmap, DisplayObjectBase, DisplayObjectPtr, EditText, Graphic,
    MorphShape, TDisplayObject, Text, Video,
};
use crate::drawing::Drawing;
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult};
use crate::font::Font;
use crate::frame_lifecycle::catchup_display_object_to_frame;
use crate::limits::ExecutionLimit;
use crate::prelude::*;
use crate::string::{AvmString, WStr, WString};
use crate::tag_utils::{self, ControlFlow, DecodeResult, Error, SwfMovie, SwfSlice, SwfStream};
use crate::vminterface::{AvmObject, Instantiator};
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use smallvec::SmallVec;
use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::sync::Arc;
use swf::extensions::ReadSwfExt;
use swf::{ClipEventFlag, FrameLabelData};

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
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct MovieClip<'gc>(GcCell<'gc, MovieClipData<'gc>>);

#[derive(Clone, Debug, Collect)]
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
    clip_event_handlers: Vec<ClipEventHandler>,
    #[collect(require_static)]
    clip_event_flags: ClipEventFlag,
    frame_scripts: Vec<Avm2FrameScript<'gc>>,
    flags: MovieClipFlags,
    avm2_class: Option<Avm2ClassObject<'gc>>,
    drawing: Drawing,
    is_focusable: bool,
    has_focus: bool,
    enabled: bool,

    /// Show a hand cursor when the clip is in button mode.
    use_hand_cursor: bool,

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
    queued_tags: HashMap<Depth, QueuedTagList>,
}

impl<'gc> MovieClip<'gc> {
    pub fn new(movie: Arc<SwfMovie>, gc_context: MutationContext<'gc, '_>) -> Self {
        MovieClip(GcCell::allocate(
            gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::allocate(gc_context, MovieClipStatic::empty(movie, gc_context)),
                tag_stream_pos: 0,
                current_frame: 0,
                audio_stream: None,
                container: ChildContainer::new(),
                object: None,
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::empty(),
                avm2_class: None,
                drawing: Drawing::new(),
                is_focusable: false,
                has_focus: false,
                enabled: true,
                use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
            },
        ))
    }

    pub fn new_with_avm2(
        movie: Arc<SwfMovie>,
        this: Avm2Object<'gc>,
        class: Avm2ClassObject<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> Self {
        MovieClip(GcCell::allocate(
            gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::allocate(gc_context, MovieClipStatic::empty(movie, gc_context)),
                tag_stream_pos: 0,
                current_frame: 0,
                audio_stream: None,
                container: ChildContainer::new(),
                object: Some(this.into()),
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::empty(),
                avm2_class: Some(class),
                drawing: Drawing::new(),
                is_focusable: false,
                has_focus: false,
                enabled: true,
                use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
            },
        ))
    }

    /// Constructs a non-root movie
    pub fn new_with_data(
        gc_context: MutationContext<'gc, '_>,
        id: CharacterId,
        swf: SwfSlice,
        num_frames: u16,
    ) -> Self {
        MovieClip(GcCell::allocate(
            gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::allocate(
                    gc_context,
                    MovieClipStatic::with_data(id, swf, num_frames, None, gc_context),
                ),
                tag_stream_pos: 0,
                current_frame: 0,
                audio_stream: None,
                container: ChildContainer::new(),
                object: None,
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::PLAYING,
                avm2_class: None,
                drawing: Drawing::new(),
                is_focusable: false,
                has_focus: false,
                enabled: true,
                use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
            },
        ))
    }

    /// Construct a movie clip that represents the root movie
    /// for the entire `Player`.
    pub fn player_root_movie(
        activation: &mut Avm2Activation<'_, 'gc, '_>,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let num_frames = movie.num_frames();

        let loader_info = if movie.is_action_script_3() {
            // The root movie doesn't have a `Loader`
            // We will replace this with a `LoaderStream::Swf` later in this function
            Some(
                LoaderInfoObject::not_yet_loaded(activation, movie.clone(), None, None, false)
                    .expect("Failed to construct LoaderInfoObject"),
            )
        } else {
            None
        };

        let mc = MovieClip(GcCell::allocate(
            activation.context.gc_context,
            MovieClipData {
                base: Default::default(),
                static_data: Gc::allocate(
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
                container: ChildContainer::new(),
                object: None,
                clip_event_handlers: Vec::new(),
                clip_event_flags: ClipEventFlag::empty(),
                frame_scripts: Vec::new(),
                flags: MovieClipFlags::PLAYING,
                avm2_class: None,
                drawing: Drawing::new(),
                is_focusable: false,
                has_focus: false,
                enabled: true,
                use_hand_cursor: true,
                button_mode: false,
                last_queued_script_frame: None,
                queued_script_frame: None,
                queued_goto_frame: None,
                drop_target: None,

                #[cfg(feature = "timeline_debug")]
                tag_frame_boundaries: Default::default(),
                queued_tags: HashMap::new(),
            },
        ));

        if movie.is_action_script_3() {
            mc.0.read()
                .static_data
                .loader_info
                .as_ref()
                .unwrap()
                .as_loader_info_object()
                .unwrap()
                .set_loader_stream(
                    LoaderStream::Swf(movie, mc.into()),
                    activation.context.gc_context,
                );
        }
        mc.set_is_root(activation.context.gc_context, true);
        mc
    }

    /// Replace the current MovieClipData with a completely new SwfMovie.
    ///
    /// Playback will start at position zero, any existing streamed audio will
    /// be terminated, and so on. Children and AVM data will NOT be kept across
    /// the load boundary.
    ///
    /// If no movie is provided, then the movie clip will be replaced with an
    /// empty movie of the same SWF version.
    pub fn replace_with_movie(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        movie: Option<Arc<SwfMovie>>,
        loader_info: Option<LoaderInfoObject<'gc>>,
    ) {
        let mut mc = self.0.write(context.gc_context);
        let is_swf = movie.is_some();
        let movie = movie.unwrap_or_else(|| Arc::new(SwfMovie::empty(mc.movie().version())));
        let total_frames = movie.num_frames();
        assert_eq!(
            mc.static_data.loader_info, None,
            "Called replace_movie on a clip with LoaderInfo set"
        );

        mc.base.base.reset_for_movie_load();
        mc.static_data = Gc::allocate(
            context.gc_context,
            MovieClipStatic::with_data(
                0,
                movie.into(),
                total_frames,
                loader_info.map(|l| l.into()),
                context.gc_context,
            ),
        );
        mc.tag_stream_pos = 0;
        mc.flags = MovieClipFlags::PLAYING;
        mc.base.base.set_is_root(is_swf);
        mc.current_frame = 0;
        mc.audio_stream = None;
        mc.container = ChildContainer::new();
        drop(mc);
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        chunk_limit: &mut ExecutionLimit,
    ) -> bool {
        use swf::TagCode;

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
        let mut static_data = (&*self.0.read().static_data).clone();
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
            if let Some(movie) = self.movie() {
                match context
                    .library
                    .library_for_movie_mut(movie)
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
                        log::error!(
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
                        log::error!(
                            "Symbol {} disappeared during preloading",
                            cur_preload_symbol
                        );

                        static_data
                            .preload_progress
                            .write(context.gc_context)
                            .cur_preload_symbol = None;
                    }
                }
            } else {
                log::error!(
                    "Attempted to preload symbol {} in movie clip not associated with movie!",
                    cur_preload_symbol
                );
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
                TagCode::DoAbc => self.do_abc(context, reader),
                TagCode::SymbolClass => self.symbol_class(context, reader),
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

        self.0.write(context.gc_context).static_data =
            Gc::allocate(context.gc_context, static_data);

        is_finished
    }

    #[inline]
    fn do_init_action(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'_>,
        tag_len: usize,
    ) -> Result<(), Error> {
        if context.is_action_script_3() {
            log::warn!("DoInitAction tag in AVM2 movie");
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'_>,
    ) -> Result<(), Error> {
        if !context.is_action_script_3() {
            log::warn!("DoABC tag in AVM1 movie");
            return Ok(());
        }

        let do_abc = reader.read_do_abc()?;
        if !do_abc.data.is_empty() {
            let movie = self.movie().unwrap();
            let domain = context.library.library_for_movie_mut(movie).avm2_domain();

            if let Err(e) = Avm2::do_abc(context, do_abc, domain) {
                log::warn!("Error loading ABC file: {}", e);
            }
        }

        Ok(())
    }

    #[inline]
    fn symbol_class(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'_>,
    ) -> Result<(), Error> {
        let movie = self.movie().ok_or(Error::NoSymbolClasses)?;
        let mut activation = Avm2Activation::from_nothing(context.reborrow());

        let num_symbols = reader.read_u16()?;

        for _ in 0..num_symbols {
            let id = reader.read_u16()?;
            let class_name = reader.read_str()?.to_str_lossy(reader.encoding());
            let class_name = AvmString::new_utf8(activation.context.gc_context, class_name);

            let name = Avm2QName::from_qualified_name(class_name, activation.context.gc_context);
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
                            format!(
                                "Attempted to assign a non-class {} to symbol {}",
                                class_name,
                                name.to_qualified_name(activation.context.gc_context)
                            )
                            .into()
                        })
                });

            match class_object {
                Ok(class_object) => {
                    activation
                        .context
                        .library
                        .avm2_class_registry_mut()
                        .set_class_symbol(class_object, movie.clone(), id);

                    let library = activation
                        .context
                        .library
                        .library_for_movie_mut(movie.clone());

                    if id == 0 {
                        //TODO: This assumes only the root movie has `SymbolClass` tags.
                        self.set_avm2_class(activation.context.gc_context, Some(class_object));
                    } else {
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
                            Some(Character::Bitmap(bitmap)) => {
                                bitmap.set_avm2_bitmapdata_class(
                                    &mut activation.context,
                                    class_object,
                                );
                            }
                            _ => {
                                log::warn!(
                                    "Symbol class {} cannot be assigned to invalid character id {}",
                                    class_name,
                                    id
                                );
                            }
                        }
                    }
                }
                Err(e) => log::warn!(
                    "Got AVM2 error {} when attempting to assign symbol class {}",
                    e,
                    class_name
                ),
            }
        }

        Ok(())
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

            let label = WString::from_utf8(&label.to_string_lossy(reader.encoding()));
            static_data.scene_labels.insert(
                label.clone(),
                Scene {
                    name: label,
                    start,
                    length: end - start,
                },
            );
        }

        for FrameLabelData { frame_num, label } in sfl_data.frame_labels {
            static_data.frame_labels.insert(
                WString::from_utf8(&label.to_string_lossy(reader.encoding())),
                frame_num as u16 + 1,
            );
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
        gc_context: MutationContext<'gc, '_>,
        drop_target: Option<DisplayObject<'gc>>,
    ) {
        self.0.write(gc_context).drop_target = drop_target;
    }

    pub fn set_programmatically_played(self, mc: MutationContext<'gc, '_>) {
        self.0.write(mc).set_programmatically_played()
    }

    pub fn next_frame(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if self.current_frame() < self.total_frames() {
            self.goto_frame(context, self.current_frame() + 1, true);
        }
    }

    pub fn play(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).play()
    }

    pub fn prev_frame(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if self.current_frame() > 1 {
            self.goto_frame(context, self.current_frame() - 1, true);
        }
    }

    pub fn stop(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).stop(context)
    }

    /// Queues up a goto to the specified frame.
    /// `frame` should be 1-based.
    ///
    /// This is treated as an 'explicit' goto: frame scripts and other frame
    /// lifecycle events will be retriggered.
    pub fn goto_frame(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        frame: FrameNumber,
        stop: bool,
    ) {
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
        if frame != self.current_frame() || context.is_action_script_3() {
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
        }
    }

    pub fn current_frame(self) -> FrameNumber {
        self.0.read().current_frame
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
    }

    /// Return all scenes in the movie.
    ///
    /// Scenes will be sorted in playback order.
    pub fn scenes(self) -> Vec<Scene> {
        let mut out: Vec<_> = self
            .0
            .read()
            .static_data
            .scene_labels
            .values()
            .cloned()
            .collect();
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

        for (_, scene) in read.static_data.scene_labels.iter() {
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

        for (label, frame) in read.static_data.frame_labels.iter() {
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
            .filter(|(_label, frame)| **frame >= from && **frame < to)
            .map(|(label, frame)| (label.clone(), *frame))
            .collect();

        values.sort_unstable_by(|(_, framea), (_, frameb)| framea.cmp(frameb));

        values
    }

    pub fn total_frames(self) -> FrameNumber {
        self.0.read().static_data.total_frames
    }

    pub fn frames_loaded(self) -> FrameNumber {
        self.0
            .read()
            .static_data
            .preload_progress
            .read()
            .cur_preload_frame
            .saturating_sub(1)
    }

    pub fn total_bytes(self) -> u32 {
        // For a loaded SWF, returns the uncompressed size of the SWF.
        // Otherwise, returns the size of the tag list in the clip's DefineSprite tag.
        if self.is_root() {
            self.movie()
                .map(|mv| mv.uncompressed_len())
                .unwrap_or_default()
        } else {
            self.tag_stream_len() as u32
        }
    }

    pub fn loaded_bytes(self) -> u32 {
        let read = self.0.read();
        let progress_read = read.static_data.preload_progress.read();
        if progress_read.next_preload_chunk == u64::MAX {
            // u64::MAX is a sentinel for load complete
            return self.total_bytes();
        }

        let swf_header_size = self.total_bytes() - self.tag_stream_len() as u32;

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
        let compressed_movie_size = movie
            .as_ref()
            .map(|mv| mv.compressed_len())
            .unwrap_or_default();

        if self.is_root() {
            compressed_movie_size as u32
        } else {
            let uncompressed_movie_size = movie.map(|mv| mv.data().len()).unwrap_or_default();
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

    pub fn set_avm2_class(
        self,
        gc_context: MutationContext<'gc, '_>,
        constr: Option<Avm2ClassObject<'gc>>,
    ) {
        let mut write = self.0.write(gc_context);
        write.avm2_class = constr;
    }

    pub fn frame_label_to_number(self, frame_label: &WStr) -> Option<FrameNumber> {
        // Frame labels are case insensitive (ASCII).
        // TODO: Should be case sensitive in AVM2.
        let label = frame_label.to_ascii_lowercase();
        self.0.read().static_data.frame_labels.get(&label).copied()
    }

    pub fn scene_label_to_number(self, scene_label: &WStr) -> Option<FrameNumber> {
        // Never used in AVM1, so always be case sensitive.
        self.0
            .read()
            .static_data
            .scene_labels
            .get(&WString::from(scene_label))
            .map(|Scene { start, .. }| start)
            .copied()
    }

    pub fn frame_exists_within_scene(self, frame_label: &WStr, scene_label: &WStr) -> bool {
        let scene = self.scene_label_to_number(scene_label);
        let frame = self.frame_label_to_number(frame_label);

        if scene.is_none() || frame.is_none() {
            return false;
        }

        let scene = scene.unwrap();
        let frame = frame.unwrap();

        if scene <= frame {
            let mut end = self.total_frames();
            for (
                _label,
                Scene {
                    start: new_scene_start,
                    ..
                },
            ) in self.0.read().static_data.scene_labels.iter()
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
        gc_context: MutationContext<'gc, '_>,
        event_handlers: Vec<ClipEventHandler>,
    ) {
        let mut mc = self.0.write(gc_context);
        mc.set_clip_event_handlers(event_handlers);
    }

    /// Returns an iterator of AVM1 `DoAction` blocks on the given frame number.
    /// Used by the AVM `Call` action.
    pub fn actions_on_frame(
        self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        frame: FrameNumber,
    ) -> impl DoubleEndedIterator<Item = SwfSlice> {
        use swf::{read::Reader, TagCode};

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
        context: &mut UpdateContext<'_, 'gc, '_>,
        run_display_actions: bool,
        run_sounds: bool,
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
                if !context.is_action_script_3() {
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

        use swf::TagCode;
        let tag_callback = |reader: &mut SwfStream<'_>, tag_code, tag_len| {
            match tag_code {
                TagCode::DoAction => self.do_action(context, reader, tag_len),
                TagCode::PlaceObject if run_display_actions && !context.is_action_script_3() => {
                    self.place_object(context, reader, tag_len, 1)
                }
                TagCode::PlaceObject2 if run_display_actions && !context.is_action_script_3() => {
                    self.place_object(context, reader, tag_len, 2)
                }
                TagCode::PlaceObject3 if run_display_actions && !context.is_action_script_3() => {
                    self.place_object(context, reader, tag_len, 3)
                }
                TagCode::PlaceObject4 if run_display_actions && !context.is_action_script_3() => {
                    self.place_object(context, reader, tag_len, 4)
                }
                TagCode::RemoveObject if run_display_actions && !context.is_action_script_3() => {
                    self.remove_object(context, reader, 1)
                }
                TagCode::RemoveObject2 if run_display_actions && !context.is_action_script_3() => {
                    self.remove_object(context, reader, 2)
                }
                TagCode::PlaceObject if run_display_actions && context.is_action_script_3() => {
                    self.queue_place_object(context, reader, tag_len, 1)
                }
                TagCode::PlaceObject2 if run_display_actions && context.is_action_script_3() => {
                    self.queue_place_object(context, reader, tag_len, 2)
                }
                TagCode::PlaceObject3 if run_display_actions && context.is_action_script_3() => {
                    self.queue_place_object(context, reader, tag_len, 3)
                }
                TagCode::PlaceObject4 if run_display_actions && context.is_action_script_3() => {
                    self.queue_place_object(context, reader, tag_len, 4)
                }
                TagCode::RemoveObject if run_display_actions && context.is_action_script_3() => {
                    self.queue_remove_object(context, reader, tag_len, 1)
                }
                TagCode::RemoveObject2 if run_display_actions && context.is_action_script_3() => {
                    self.queue_remove_object(context, reader, tag_len, 2)
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
                log::error!("Error running queued tag: {:?}, got {}", tag.tag_type, e);
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

        if matches!(next_frame, NextFrame::Next) && context.is_action_script_3() {
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        id: CharacterId,
        depth: Depth,
        place_object: &swf::PlaceObject,
    ) -> Option<DisplayObject<'gc>> {
        let movie = self.movie().unwrap();
        let library = context.library.library_for_movie_mut(movie.clone());
        match library.instantiate_by_id(id, context.gc_context) {
            Ok(child) => {
                // Remove previous child from children list,
                // and add new child onto front of the list.
                let prev_child = self.replace_at_depth(context, child, depth);
                let mut placed_with_name = false;
                {
                    // Set initial properties for child.
                    child.set_instantiated_by_timeline(context.gc_context, true);
                    child.set_depth(context.gc_context, depth);
                    child.set_parent(context.gc_context, Some(self.into()));
                    child.set_place_frame(context.gc_context, self.current_frame());

                    // Apply PlaceObject parameters.
                    child.apply_place_object(context, place_object);
                    if let Some(name) = &place_object.name {
                        placed_with_name = true;
                        let encoding = swf::SwfStr::encoding_for_version(self.swf_version());
                        let name = name.to_str_lossy(encoding);
                        child.set_name(
                            context.gc_context,
                            AvmString::new_utf8(context.gc_context, name),
                        );
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
                    // TODO: Missing PlaceObject properties: amf_data, filters

                    // Run first frame.
                    catchup_display_object_to_frame(context, child);
                    child.post_instantiation(context, None, Instantiator::Movie, false);
                    // In AVM1, children are added in `run_frame` so this is necessary.
                    // In AVM2 we add them in `construct_frame` so calling this causes
                    // duplicate frames
                    if !movie.is_action_script_3() {
                        child.run_frame(context);
                    }
                }

                dispatch_added_event_only(child, context);
                dispatch_added_to_stage_event_only(child, context);
                if let Some(prev_child) = prev_child {
                    dispatch_removed_event(prev_child, context);
                }

                if placed_with_name {
                    if let Avm2Value::Object(mut p) = self.object2() {
                        if let Avm2Value::Object(c) = child.object2() {
                            let name = Avm2Multiname::public(child.name());
                            let mut activation = Avm2Activation::from_nothing(context.reborrow());
                            if let Err(e) = p.init_property(&name, c.into(), &mut activation) {
                                log::error!(
                                    "Got error when setting AVM2 child named \"{}\": {}",
                                    &child.name(),
                                    e
                                );
                            }
                        }
                    }
                }

                Some(child)
            }
            Err(e) => {
                log::error!(
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
            "[{}] Gotos must start from the correct tag position for frame {}",
            read.base.base.name,
            read.current_frame
        );
    }

    #[cfg(not(feature = "timeline_debug"))]
    fn assert_expected_tag_end(
        self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        _hit_target_frame: bool,
    ) {
    }

    #[cfg(feature = "timeline_debug")]
    fn assert_expected_tag_end(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        hit_target_frame: bool,
    ) {
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
                "[{}] Gotos must end at the correct tag position for frame {}",
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        frame: FrameNumber,
        is_implicit: bool,
    ) {
        if cfg!(feature = "timeline_debug") {
            log::debug!(
                "[{}]: {} from frame {} to frame {}",
                self.name(),
                if is_implicit { "looping" } else { "goto" },
                self.current_frame(),
                frame
            );
            self.assert_expected_tag_start();
        }

        let frame_before_rewind = self.current_frame();

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
        let clamped_frame = frame.min(mc.frames_loaded());
        drop(mc);

        let mut removed_frame_scripts: Vec<DisplayObject<'gc>> = vec![];

        let mut reader = data.read_from(frame_pos);
        while self.current_frame() < clamped_frame && !reader.get_ref().is_empty() {
            self.0.write(context.gc_context).current_frame += 1;
            frame_pos = reader.get_ref().as_ptr() as u64 - tag_stream_start;

            use swf::TagCode;
            let tag_callback = |reader: &mut _, tag_code, tag_len| {
                match tag_code {
                    TagCode::PlaceObject => {
                        index += 1;
                        let mut mc = self.0.write(context.gc_context);

                        mc.goto_place_object(
                            reader,
                            tag_len,
                            1,
                            &mut goto_commands,
                            is_rewind,
                            index,
                        )
                    }
                    TagCode::PlaceObject2 => {
                        index += 1;
                        let mut mc = self.0.write(context.gc_context);

                        mc.goto_place_object(
                            reader,
                            tag_len,
                            2,
                            &mut goto_commands,
                            is_rewind,
                            index,
                        )
                    }
                    TagCode::PlaceObject3 => {
                        index += 1;
                        let mut mc = self.0.write(context.gc_context);

                        mc.goto_place_object(
                            reader,
                            tag_len,
                            3,
                            &mut goto_commands,
                            is_rewind,
                            index,
                        )
                    }
                    TagCode::PlaceObject4 => {
                        index += 1;
                        let mut mc = self.0.write(context.gc_context);

                        mc.goto_place_object(
                            reader,
                            tag_len,
                            4,
                            &mut goto_commands,
                            is_rewind,
                            index,
                        )
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
            // TODO: AS3 children don't live on the depth list. Do they respect
            // or ignore GOTOs?
            let children: SmallVec<[_; 16]> = self
                .0
                .read()
                .container
                .iter_children_by_depth()
                .filter_map(|(depth, clip)| {
                    if clip.place_frame() > frame {
                        Some((depth, clip))
                    } else {
                        None
                    }
                })
                .collect();
            for (_depth, child) in children {
                if !child.placed_by_script() {
                    self.remove_child(context, child, Lists::all());
                } else {
                    self.remove_child(context, child, Lists::DEPTH);
                }
            }
        }

        // Run the list of goto commands to actually create and update the display objects.
        let run_goto_command = |clip: MovieClip<'gc>,
                                context: &mut UpdateContext<'_, 'gc, '_>,
                                params: &GotoPlaceObject<'_>| {
            use swf::PlaceObjectAction;
            let child_entry = clip.child_by_depth(params.depth());
            if context.is_action_script_3() && is_implicit && child_entry.is_none() {
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
                    tag_len: params.tag_len,
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
                    log::error!(
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
            self.run_frame_internal(context, false, frame != frame_before_rewind);
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
            self.construct_frame(context);
            self.frame_constructed(context);
            self.avm2_root(context)
                .unwrap_or_else(|| self.into())
                .run_frame_scripts(context);

            for child in removed_frame_scripts {
                child.run_frame_scripts(context);
            }

            self.exit_frame(context);
        }

        self.assert_expected_tag_end(context, hit_target_frame);
    }

    fn construct_as_avm1_object(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        init_object: Option<Avm1Object<'gc>>,
        instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        //TODO: This will break horribly when AVM2 starts touching the display list
        if self.0.read().object.is_none() {
            let globals = context.avm1.global_object_cell();
            let avm1_constructor = self.0.read().get_registered_avm1_constructor(context);

            // If we are running within the AVM, this must be an immediate action.
            // If we are not, then this must be queued to be ran first-thing
            if let Some(constructor) = avm1_constructor.filter(|_| instantiated_by.is_avm()) {
                let mut activation = Avm1Activation::from_nothing(
                    context.reborrow(),
                    ActivationIdentifier::root("[Construct]"),
                    globals,
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
                        self.run_frame(&mut activation.context);
                    }

                    if let Some(init_object) = init_object {
                        // AVM1 sets keys in reverse order (compared to enumeration order).
                        // This behavior is visible to setters, and some SWFs depend on it.
                        for key in init_object.get_keys(&mut activation).into_iter().rev() {
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
                self.run_frame(context);
            }

            if let Some(init_object) = init_object {
                let mut activation = Avm1Activation::from_nothing(
                    context.reborrow(),
                    ActivationIdentifier::root("[Init]"),
                    globals,
                    self.into(),
                );

                for key in init_object.get_keys(&mut activation) {
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
            self.run_frame(context);
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
    fn allocate_as_avm2_object(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
    ) {
        let class_object = self
            .0
            .read()
            .avm2_class
            .unwrap_or_else(|| context.avm2.classes().movieclip);

        let mut constr_thing = || {
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            let object =
                Avm2StageObject::for_display_object(&mut activation, display_object, class_object)?
                    .into();

            Ok(object)
        };
        let result: Result<Avm2Object<'gc>, Avm2Error> = constr_thing();

        if let Ok(object) = result {
            self.0.write(context.gc_context).object = Some(object.into());
        } else if let Err(e) = result {
            log::error!("Got {} when allocating AVM2 side of display object", e);
        }
    }

    /// Construct the AVM2 side of this object.
    ///
    /// This function does *not* allocate the object; it is intended that you
    /// will allocate the object first before doing so. This function is
    /// intended to be called from `post_instantiate`.
    fn construct_as_avm2_object(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let class_object = self
            .0
            .read()
            .avm2_class
            .unwrap_or_else(|| context.avm2.classes().movieclip);

        if let Avm2Value::Object(object) = self.object2() {
            let mut constr_thing = || {
                let mut activation = Avm2Activation::from_nothing(context.reborrow());
                class_object.call_native_init(Some(object), &[], &mut activation)?;

                Ok(())
            };
            let result: Result<(), Avm2Error> = constr_thing();

            if let Err(e) = result {
                log::error!(
                    "Got {} when constructing AVM2 side of movie clip of type {}",
                    e,
                    class_object
                        .try_inner_class_definition()
                        .map(|c| c.read().name().to_qualified_name(context.gc_context))
                        .unwrap_or_else(|_| "[BorrowError!]".into())
                );
            }
        }
    }

    pub fn register_frame_script(
        self,
        frame_id: FrameNumber,
        callable: Avm2Object<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        let mut write = self.0.write(context.gc_context);

        write
            .frame_scripts
            .push(Avm2FrameScript { frame_id, callable });
    }

    pub fn set_focusable(self, focusable: bool, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0.write(context.gc_context).is_focusable = focusable;
    }

    /// Handle a RemoveObject tag when running a goto action.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn goto_remove_object<'a>(
        mut self,
        reader: &mut SwfStream<'a>,
        version: u8,
        context: &mut UpdateContext<'_, 'gc, '_>,
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

            let child = self.0.read().container.get_depth(depth);
            if let Some(child) = child {
                if !child.placed_by_script() {
                    self.remove_child(context, child, Lists::all());
                } else {
                    self.remove_child(context, child, Lists::DEPTH);
                }

                removed_frame_scripts.push(child);
            }

            self.0.write(context.gc_context).current_frame = to_frame;
        }
        Ok(())
    }

    pub fn enabled(self) -> bool {
        self.0.read().enabled
    }

    pub fn set_enabled(self, context: &mut UpdateContext<'_, 'gc, '_>, enabled: bool) {
        self.0.write(context.gc_context).enabled = enabled;
    }

    pub fn use_hand_cursor(self) -> bool {
        self.0.read().use_hand_cursor
    }

    pub fn set_use_hand_cursor(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        use_hand_cursor: bool,
    ) {
        self.0.write(context.gc_context).use_hand_cursor = use_hand_cursor;
    }

    pub fn tag_stream_len(&self) -> usize {
        self.0.read().tag_stream_len()
    }

    pub fn forced_button_mode(self) -> bool {
        self.0.read().button_mode
    }

    pub fn set_forced_button_mode(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        button_mode: bool,
    ) {
        self.0.write(context.gc_context).button_mode = button_mode;
    }

    pub fn drawing(&self, gc_context: MutationContext<'gc, '_>) -> RefMut<'_, Drawing> {
        RefMut::map(self.0.write(gc_context), |s| &mut s.drawing)
    }

    pub fn is_button_mode(&self, context: &mut UpdateContext<'_, 'gc, '_>) -> bool {
        if self.forced_button_mode()
            || self
                .0
                .read()
                .clip_event_flags
                .intersects(ClipEvent::BUTTON_EVENT_FLAGS)
        {
            true
        } else {
            let mut activation = Avm1Activation::from_stub(
                context.reborrow(),
                ActivationIdentifier::root("[Mouse Pick]"),
            );
            let object = self.object().coerce_to_object(&mut activation);

            ClipEvent::BUTTON_EVENT_METHODS
                .iter()
                .copied()
                .any(|handler| object.has_property(&mut activation, handler.into()))
        }
    }

    /// Remove all `PlaceObject` tags off the internal tag queue.
    fn unqueue_adds(&self, context: &mut UpdateContext<'_, 'gc, '_>) -> Vec<(Depth, QueuedTag)> {
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
    fn unqueue_removes(&self, context: &mut UpdateContext<'_, 'gc, '_>) -> Vec<(Depth, QueuedTag)> {
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
}

impl<'gc> TDisplayObject<'gc> for MovieClip<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().id()
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        Some(self.0.read().movie())
    }

    fn swf_version(&self) -> u8 {
        self.0.read().movie().version()
    }

    fn enter_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        //Child removals from looping gotos appear to resolve in reverse order.
        for child in self.iter_render_list().rev() {
            child.enter_frame(context);
        }

        if context.is_action_script_3() {
            let is_playing = self.playing();

            if is_playing {
                self.run_frame_internal(context, true, true);
            }
        }
    }

    /// Construct objects placed on this frame.
    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        // New children will be constructed when they are instantiated and thus
        // if we construct before our children, they'll get double-constructed.
        for child in self.iter_render_list() {
            child.construct_frame(context);
        }

        // AVM1 code expects to execute in line with timeline instructions, so
        // it's exempted from frame construction.
        if context.is_action_script_3() && self.frames_loaded() >= 1 {
            let needs_construction = if matches!(self.object2(), Avm2Value::Undefined) {
                self.allocate_as_avm2_object(context, (*self).into());
                true
            } else {
                false
            };

            // PlaceObject tags execute at this time.
            let data = self.0.read().static_data.swf.clone();
            let place_actions = self.unqueue_adds(context);

            for (_, tag) in place_actions {
                let mut reader = data.read_from(tag.tag_start);
                let version = match tag.tag_type {
                    QueuedTagAction::Place(v) => v,
                    _ => unreachable!(),
                };

                if let Err(e) = self.place_object(context, &mut reader, tag.tag_len, version) {
                    log::error!("Error running queued tag: {:?}, got {}", tag.tag_type, e);
                }
            }

            self.0.write(context.gc_context).unset_loop_queued();

            if needs_construction {
                self.construct_as_avm2_object(context);

                // AVM2 roots work exactly the same as any other timeline- or
                // script-constructed object in terms of events received on
                // them. However, because roots are added by the player itself,
                // we can't fire the events until we run our first frame, so we
                // have to actually check if we've just built the root and act
                // like it just got added to the timeline.
                let self_dobj: DisplayObject<'gc> = (*self).into();
                if self_dobj.is_root() {
                    dispatch_added_event_only(self_dobj, context);
                    dispatch_added_to_stage_event_only(self_dobj, context);
                }
            }
        }
    }

    fn run_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
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
        if self.playing() && !context.is_action_script_3() {
            self.run_frame_internal(context, true, true);
        }
    }

    fn run_frame_scripts(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut index = 0;
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

                    write.last_queued_script_frame = Some(frame_id);
                    write.queued_script_frame = None;
                    write
                        .flags
                        .insert(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT);

                    if is_fresh_frame {
                        while let Some(fs) = write.frame_scripts.get(index) {
                            if fs.frame_id == frame_id {
                                let callable = fs.callable;
                                drop(write);
                                if let Err(e) = Avm2::run_stack_frame_for_callable(
                                    callable,
                                    Some(avm2_object),
                                    &[],
                                    context,
                                ) {
                                    log::error!(
                                        "Error occured when running AVM2 frame script: {}",
                                        e
                                    );
                                }
                                write = self.0.write(context.gc_context);
                            }

                            index += 1;
                        }
                    }

                    write
                        .flags
                        .remove(MovieClipFlags::EXECUTING_AVM2_FRAME_SCRIPT);
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

    fn on_exit_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        // Attempt to fire an "init" event on our `LoaderInfo`.
        // This fires after we've exited our first frame, but before
        // but before we enter a new frame. `loader_stream_init`
        // keeps track if an "init" event has already been fired,
        // so this becomes a no-op after the event has been fired.
        if self.0.read().initialized() {
            if let Some(loader_info) = self
                .loader_info()
                .as_ref()
                .and_then(|o| o.as_loader_info_object())
            {
                loader_info.fire_init_and_complete_events(context);
            }
        }

        for child in self.iter_render_list() {
            child.on_exit_frame(context);
        }
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc, '_>) {
        self.0.read().drawing.render(context);
        self.render_children(context);
    }

    fn self_bounds(&self) -> BoundingBox {
        self.0.read().drawing.self_bounds()
    }

    fn hit_test_shape(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        point: (Twips, Twips),
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
                } else if child.depth() > clip_depth
                    && child.hit_test_shape(context, point, options)
                {
                    return true;
                }
            }

            let local_matrix = self.global_to_local_matrix();
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

    fn as_drawing(&self, gc_context: MutationContext<'gc, '_>) -> Option<RefMut<'_, Drawing>> {
        Some(self.drawing(gc_context))
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        init_object: Option<Avm1Object<'gc>>,
        instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        if !context.is_action_script_3() {
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
            .unwrap_or(Avm2Value::Undefined)
    }

    fn set_object2(&mut self, mc: MutationContext<'gc, '_>, to: Avm2Object<'gc>) {
        self.0.write(mc).object = Some(to.into());
    }

    fn unload(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.iter_render_list() {
            child.unload(context);
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

        let had_focus = self.0.read().has_focus;
        if had_focus {
            let tracker = context.focus_tracker;
            tracker.set(None, context);
        }

        {
            let mut mc = self.0.write(context.gc_context);
            mc.stop_audio_stream(context);
        }
        self.event_dispatch(context, ClipEvent::Unload);
        self.set_removed(context.gc_context, true);
    }

    fn loader_info(&self) -> Option<Avm2Object<'gc>> {
        self.0.read().static_data.loader_info
    }

    fn allow_as_mask(&self) -> bool {
        !self.is_empty()
    }

    fn is_focusable(&self) -> bool {
        self.0.read().is_focusable
    }

    fn on_focus_changed(&self, gc_context: MutationContext<'gc, '_>, focused: bool) {
        self.0.write(gc_context).has_focus = focused;
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for MovieClip<'gc> {
    impl_display_object_container!(container);
}

impl<'gc> TInteractiveObject<'gc> for MovieClip<'gc> {
    fn ibase(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn ibase_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<InteractiveObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(self, event: ClipEvent) -> ClipEventResult {
        if event.is_button_event() && !self.visible() && !matches!(event, ClipEvent::ReleaseOutside)
        {
            return ClipEventResult::NotHandled;
        }

        if !self.enabled()
            && event.is_button_event()
            && !matches!(event, ClipEvent::KeyPress { .. })
        {
            return ClipEventResult::NotHandled;
        }

        ClipEventResult::Handled
    }

    fn event_dispatch(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        let frame_name = match event {
            ClipEvent::RollOut { .. } | ClipEvent::ReleaseOutside => Some(WStr::from_units(b"_up")),
            ClipEvent::RollOver { .. } | ClipEvent::Release | ClipEvent::DragOut { .. } => {
                Some(WStr::from_units(b"_over"))
            }
            ClipEvent::Press | ClipEvent::DragOver { .. } => Some(WStr::from_units(b"_down")),
            _ => None,
        };

        if let Some(frame_name) = frame_name {
            if let Some(frame_number) = self.frame_label_to_number(frame_name) {
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
                if swf_version >= 6 {
                    if let Some(name) = event.method_name() {
                        // Keyboard events don't fire their methods unless the MovieClip has focus (#2120).
                        if !event.is_key_event() || read.has_focus {
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
        } else {
            drop(read);
            handled = self.event_dispatch_to_avm2(context, event);
        }

        handled
    }

    fn mouse_pick(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        point: (Twips, Twips),
        require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        if self.visible() && self.mouse_enabled() {
            let this: InteractiveObject<'gc> = (*self).into();

            if let Some(masker) = self.masker() {
                if !masker.hit_test_shape(context, point, HitTestOptions::SKIP_INVISIBLE) {
                    return None;
                }
            }

            if self.world_bounds().contains(point) {
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
            // AVM2 allows movie clips to recieve mouse events without explicitly enabling button mode.
            let check_non_interactive =
                !require_button_mode || matches!(self.object2(), Avm2Value::Object(_));

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
                        result = child.mouse_pick(context, point, require_button_mode);
                    } else if check_non_interactive && child.hit_test_shape(context, point, options)
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

            // Check drawing.
            if check_non_interactive {
                let local_matrix = self.global_to_local_matrix();
                let point = local_matrix * point;
                if self.0.read().drawing.hit_test(point, &local_matrix) {
                    return Some(this);
                }
            }
        }

        None
    }

    fn mouse_cursor(self, context: &mut UpdateContext<'_, 'gc, '_>) -> MouseCursor {
        if self.use_hand_cursor() && self.enabled() && self.is_button_mode(context) {
            MouseCursor::Hand
        } else {
            MouseCursor::Arrow
        }
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

    fn frames_loaded(&self) -> FrameNumber {
        self.static_data
            .preload_progress
            .read()
            .cur_preload_frame
            .saturating_sub(1)
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

    fn stop(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
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
        tag_len: usize,
        version: u8,
        goto_commands: &mut Vec<GotoPlaceObject<'a>>,
        is_rewind: bool,
        index: usize,
    ) -> Result<(), Error> {
        let tag_start =
            reader.get_ref().as_ptr() as u64 - self.static_data.swf.as_ref().as_ptr() as u64;
        let place_object = if version == 1 {
            reader.read_place_object(tag_len)
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
            tag_len,
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
    fn stop_audio_stream(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if let Some(audio_stream) = self.audio_stream.take() {
            context.stop_sound(audio_stream);
        }
    }

    /// Fetch the avm1 constructor associated with this MovieClip by `Object.registerClass`.
    /// Return `None` if this MovieClip isn't exported, or if no constructor is associated
    /// to its symbol name.
    fn get_registered_avm1_constructor(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
        version: u8,
    ) -> Result<(), Error> {
        let define_bits_lossless = reader.read_define_bits_lossless(version)?;
        let bitmap = ruffle_render::utils::decode_define_bits_lossless(&define_bits_lossless)?;
        let bitmap = Bitmap::new(context, define_bits_lossless.id, bitmap)?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(define_bits_lossless.id, Character::Bitmap(bitmap));
        Ok(())
    }

    #[inline]
    fn define_morph_shape(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
                log::warn!(
                    "Tried to apply CSMTextSettings to non-text character ID {}",
                    settings.id
                );
            }
            None => {
                log::warn!(
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let jpeg_data = reader.read_slice_to_end();
        let jpeg_tables = context
            .library
            .library_for_movie_mut(self.movie())
            .jpeg_tables();
        let jpeg_data = ruffle_render::utils::glue_tables_to_jpeg(jpeg_data, jpeg_tables);
        let bitmap = ruffle_render::utils::decode_define_bits_jpeg(&jpeg_data, None)?;
        let bitmap = Bitmap::new(context, id, bitmap)?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(id, Character::Bitmap(bitmap));
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let id = reader.read_u16()?;
        let jpeg_data = reader.read_slice_to_end();
        let bitmap = ruffle_render::utils::decode_define_bits_jpeg(jpeg_data, None)?;
        let bitmap = Bitmap::new(context, id, bitmap)?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(id, Character::Bitmap(bitmap));
        Ok(())
    }

    #[inline]
    fn define_bits_jpeg_3_or_4(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        let bitmap = ruffle_render::utils::decode_define_bits_jpeg(jpeg_data, Some(alpha_data))?;
        let bitmap = Bitmap::new(context, id, bitmap)?;
        context
            .library
            .library_for_movie_mut(self.movie())
            .register_character(id, Character::Bitmap(bitmap));
        Ok(())
    }

    #[inline]
    fn define_button_1(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let swf_button = reader.read_define_button_1()?;

        self.define_button_any(context, swf_button)
    }

    #[inline]
    fn define_button_2(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let swf_button = reader.read_define_button_2()?;

        self.define_button_any(context, swf_button)
    }

    #[inline]
    fn define_button_any(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_button: swf::Button<'a>,
    ) -> Result<(), Error> {
        let movie = self.movie();
        let button = if movie.is_action_script_3() {
            Character::Avm2Button(Avm2Button::from_swf_tag(
                &swf_button,
                &self.static_data.swf,
                context,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let button_colors = reader.read_define_button_cxform()?;
        match context
            .library
            .library_for_movie_mut(self.movie())
            .character_by_id(button_colors.id)
        {
            Some(Character::Avm1Button(button)) => {
                button.set_colors(context.gc_context, &button_colors.color_transforms[..]);
            }
            Some(_) => {
                log::warn!(
                    "DefineButtonCxform: Tried to apply on non-button ID {}",
                    button_colors.id
                );
            }
            None => {
                log::warn!(
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let button_sounds = reader.read_define_button_sound()?;
        match context
            .library
            .library_for_movie_mut(self.movie())
            .character_by_id(button_sounds.id)
        {
            Some(Character::Avm1Button(button)) => {
                button.set_sounds(context.gc_context, button_sounds);
            }
            Some(_) => {
                log::warn!(
                    "DefineButtonSound: Tried to apply on non-button ID {}",
                    button_sounds.id
                );
            }
            None => {
                log::warn!(
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_2(2)?;
        let font_id = font.id;
        let font_object = Font::from_swf_tag(
            context.gc_context,
            context.renderer,
            font,
            reader.encoding(),
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let font = reader.read_define_font_2(3)?;
        let font_id = font.id;
        let font_object = Font::from_swf_tag(
            context.gc_context,
            context.renderer,
            font,
            reader.encoding(),
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
        _context: &mut UpdateContext<'_, 'gc, '_>,
        _reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        log::warn!("DefineFont4 tag (TLF text) is not implemented");
        Ok(())
    }

    #[inline]
    fn define_sound(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let sound = reader.read_define_sound()?;
        if let Ok(handle) = context.audio.register_sound(&sound) {
            context
                .library
                .library_for_movie_mut(self.movie())
                .register_character(sound.id, Character::Sound(handle));
        } else {
            log::error!(
                "MovieClip::define_sound: Unable to register sound ID {}",
                sound.id
            );
        }
        Ok(())
    }

    #[inline]
    fn define_video_stream(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
    fn export_assets(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let exports = reader.read_export_assets()?;
        for export in exports {
            let name = export.name.to_str_lossy(reader.encoding());
            let name = AvmString::new_utf8(context.gc_context, name);
            let character = context
                .library
                .library_for_movie_mut(self.movie())
                .register_export(export.id, name);

            // TODO: do other types of Character need to know their exported name?
            if let Some(Character::MovieClip(movie_clip)) = character {
                *movie_clip
                    .0
                    .read()
                    .static_data
                    .exported_name
                    .write(context.gc_context) = Some(name);
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
    ) -> Result<(), Error> {
        let frame_label = reader.read_frame_label()?;
        let mut label = frame_label
            .label
            .to_str_lossy(reader.encoding())
            .into_owned();

        // Frame labels are case insensitive (ASCII).
        label.make_ascii_lowercase();
        let label = WString::from_utf8_owned(label);
        if let std::collections::hash_map::Entry::Vacant(v) = static_data.frame_labels.entry(label)
        {
            v.insert(cur_frame);
        } else {
            log::warn!("Movie clip {}: Duplicated frame label", self.id());
        }
        Ok(())
    }

    #[inline]
    fn jpeg_tables(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
        tag_len: usize,
    ) -> Result<(), Error> {
        if context.is_action_script_3() {
            log::warn!("DoAction tag in AVM2 movie");
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

    fn queue_place_object(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
        tag_len: usize,
        version: u8,
    ) -> Result<(), Error> {
        let mut write = self.0.write(context.gc_context);
        let tag_start =
            reader.get_ref().as_ptr() as u64 - write.static_data.swf.as_ref().as_ptr() as u64;
        let place_object = if version == 1 {
            reader.read_place_object(tag_len)
        } else {
            reader.read_place_object_2_or_3(version)
        }?;

        let new_tag = QueuedTag {
            tag_type: QueuedTagAction::Place(version),
            tag_start,
            tag_len,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
        tag_len: usize,
        version: u8,
    ) -> Result<(), Error> {
        let place_object = if version == 1 {
            reader.read_place_object(tag_len)
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
                self.remove_child(context, child, Lists::all());
            } else {
                self.remove_child(context, child, Lists::DEPTH);
            }
        }

        Ok(())
    }

    #[inline]
    fn queue_remove_object(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
        tag_len: usize,
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
            tag_len,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
                let audio_stream = context.start_stream(
                    mc.static_data.audio_stream_handle,
                    self,
                    mc.current_frame(),
                    slice,
                    stream_info,
                );
                drop(mc);
                self.0.write(context.gc_context).audio_stream = audio_stream;
            }
        }

        Ok(())
    }

    #[inline]
    fn start_sound_1(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reader: &mut SwfStream<'a>,
    ) -> Result<(), Error> {
        let start_sound = reader.read_start_sound_1()?;
        if let Some(handle) = context
            .library
            .library_for_movie_mut(self.movie().unwrap()) // TODO
            .get_sound(start_sound.id)
        {
            use swf::SoundEvent;
            // The sound event type is controlled by the "Sync" setting in the Flash IDE.
            match start_sound.sound_info.event {
                // "Event" sounds always play, independent of the timeline.
                SoundEvent::Event => {
                    let _ = context.start_sound(
                        handle,
                        &start_sound.sound_info,
                        Some(self.into()),
                        None,
                    );
                }

                // "Start" sounds only play if an instance of the same sound is not already playing.
                SoundEvent::Start => {
                    if !context.is_sound_playing_with_handle(handle) {
                        let _ = context.start_sound(
                            handle,
                            &start_sound.sound_info,
                            Some(self.into()),
                            None,
                        );
                    }
                }

                // "Stop" stops any active instances of a given sound.
                SoundEvent::Stop => context.stop_sounds_with_handle(handle),
            }
        }
        Ok(())
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
            start: 0,
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
    frame_labels: HashMap<WString, FrameNumber>,
    #[collect(require_static)]
    scene_labels: HashMap<WString, Scene>,
    #[collect(require_static)]
    audio_stream_info: Option<swf::SoundStreamHead>,
    #[collect(require_static)]
    audio_stream_handle: Option<SoundHandle>,
    total_frames: FrameNumber,
    /// The last known symbol name under which this movie clip was exported.
    /// Used for looking up constructors registered with `Object.registerClass`.
    exported_name: GcCell<'gc, Option<AvmString<'gc>>>,
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
}

impl<'gc> MovieClipStatic<'gc> {
    fn empty(movie: Arc<SwfMovie>, gc_context: MutationContext<'gc, '_>) -> Self {
        let s = Self::with_data(0, SwfSlice::empty(movie), 1, None, gc_context);

        s.preload_progress.write(gc_context).cur_preload_frame = s.total_frames + 1;

        s
    }

    fn with_data(
        id: CharacterId,
        swf: SwfSlice,
        total_frames: FrameNumber,
        loader_info: Option<Avm2Object<'gc>>,
        gc_context: MutationContext<'gc, '_>,
    ) -> Self {
        Self {
            id,
            swf,
            total_frames,
            frame_labels: HashMap::new(),
            scene_labels: HashMap::new(),
            audio_stream_info: None,
            audio_stream_handle: None,
            exported_name: GcCell::allocate(gc_context, None),
            loader_info,
            preload_progress: GcCell::allocate(gc_context, Default::default()),
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

    /// The length of the PlaceObject tag at `tag_start`.
    tag_len: usize,

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
        tag_len: usize,
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
                // Purposely omitted properties:
                // name, clip_depth, clip_actions, amf_data
                // These properties are only set on initial placement in `MovieClip::instantiate_child`
                // and can not be modified by subsequent PlaceObject tags.
                // Also, is_visible flag persists during rewind unlike all other properties.
                // TODO: Filters need to be applied here. Rewinding will erase filters if initial
                // PlaceObject tag has none.
            }
        }

        Self {
            frame,
            place_object,
            index,
            tag_start,
            tag_len,
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
        // Purposely omitted properties:
        // name, clip_depth, clip_actions, amf_data
        // These properties are only set on initial placement in `MovieClip::instantiate_child`
        // and can not be modified by subsequent PlaceObject tags.
        // TODO: Filters need to be applied here. New filters will overwrite old filters.
    }
}

/// A list of add/remove tags to process on a given depth this frame.
///
/// There are only a handful of valid tag configurations per depth: namely,
/// no tags, one removal, one add, or a removal followed by an add.
///
/// Any other configuration in the SWF tag stream is normalized to one of
/// these patterns.
#[derive(Default, Debug, Eq, PartialEq, Clone, Copy, Collect)]
#[collect(require_static)]
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
            QueuedTagList::Add(_) => QueuedTagList::Add(add_tag),
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
#[derive(Debug, Eq, PartialEq, Clone, Copy, Collect)]
#[collect(require_static)]
pub struct QueuedTag {
    pub tag_type: QueuedTagAction,
    pub tag_start: u64,
    pub tag_len: usize,
}

/// The type of queued tag.
///
/// The u8 parameter is the tag version.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Collect)]
#[collect(require_static)]
pub enum QueuedTagAction {
    Place(u8),
    Remove(u8),
}

bitflags! {
    /// Boolean state flags used by `MovieClip`.
    #[derive(Collect)]
    #[collect(require_static)]
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
    }
}

/// Actions that are attached to a `MovieClip` event in
/// an `onClipEvent`/`on` handler.
#[derive(Debug, Clone, Collect)]
#[collect(require_static)]
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

/// An AVM2 frame script attached to a (presumably AVM2) MovieClip.
#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct Avm2FrameScript<'gc> {
    /// The frame to invoke this frame script on.
    pub frame_id: FrameNumber,

    /// The AVM2 callable object to invoke when the frame script runs.
    pub callable: Avm2Object<'gc>,
}
