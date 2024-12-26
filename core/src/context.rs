//! Contexts and helper types passed between functions.

use crate::avm1::Activation;
use crate::avm1::ActivationIdentifier;
use crate::avm1::Attribute;
use crate::avm1::Avm1;
use crate::avm1::ScriptObject;
use crate::avm1::SystemProperties;
use crate::avm1::TObject;
use crate::avm1::{Object as Avm1Object, Value as Avm1Value};
use crate::avm2::api_version::ApiVersion;
use crate::avm2::object::LoaderInfoObject;
use crate::avm2::Activation as Avm2Activation;
use crate::avm2::TObject as _;
use crate::avm2::{Avm2, Object as Avm2Object, SoundChannelObject};
use crate::backend::{
    audio::{AudioBackend, AudioManager, SoundHandle, SoundInstanceHandle},
    log::LogBackend,
    navigator::NavigatorBackend,
    storage::StorageBackend,
    ui::UiBackend,
};
use crate::context_menu::ContextMenuState;
use crate::display_object::{EditText, MovieClip, SoundTransform, Stage};
use crate::external::ExternalInterface;
use crate::focus_tracker::FocusTracker;
use crate::frame_lifecycle::FramePhase;
use crate::input::InputManager;
use crate::library::Library;
use crate::loader::LoadManager;
use crate::local_connection::LocalConnections;
use crate::net_connection::NetConnections;
use crate::player::PostFrameCallback;
use crate::player::{MouseData, Player};
use crate::prelude::*;
use crate::socket::Sockets;
use crate::streams::StreamManager;
use crate::string::{AvmString, StringContext};
use crate::stub::StubCollection;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::timer::Timers;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::{Collect, Mutation};
use rand::rngs::SmallRng;
use ruffle_render::backend::{BitmapCacheEntry, RenderBackend};
use ruffle_render::commands::{CommandHandler, CommandList};
use ruffle_render::transform::TransformStack;
use ruffle_video::backend::VideoBackend;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use web_time::Instant;

/// `UpdateContext` holds shared data that is used by the various subsystems of Ruffle.
/// `Player` creates this when it begins a tick and passes it through the call stack to
/// children and the VM.
pub struct UpdateContext<'gc> {
    /// The mutation context to allocate and mutate `Gc` pointers.
    ///
    /// NOTE: This is redundant with `strings.gc()`, but is used by
    /// too much code to remove.
    pub gc_context: &'gc Mutation<'gc>,

    /// The string context.
    pub strings: StringContext<'gc>,

    /// The queue of actions that will be run after the display list updates.
    /// Display objects and actions can push actions onto the queue.
    pub action_queue: &'gc mut ActionQueue<'gc>,

    /// A collection of stubs encountered during this movie.
    pub stub_tracker: &'gc mut StubCollection,

    /// The library containing character definitions for this SWF.
    /// Used to instantiate a `DisplayObject` of a given ID.
    pub library: &'gc mut Library<'gc>,

    /// The version of the Flash Player we are emulating.
    /// TODO: This is a little confusing because this represents the player's max SWF version,
    /// which is an integer (e.g. 13), but "Flash Player version" is a triplet (11.6.0), and these
    /// aren't in sync. It may be better to have separate `player_swf_version` and `player_version`
    /// variables.
    pub player_version: u8,

    /// Requests that the player re-renders after this execution (e.g. due to `updateAfterEvent`).
    pub needs_render: &'gc mut bool,

    /// The root SWF file.
    pub swf: &'gc mut Arc<SwfMovie>,

    /// The audio backend, used by display objects and AVM to play audio.
    pub audio: &'gc mut dyn AudioBackend,

    /// The audio manager, managing all actively playing sounds.
    pub audio_manager: &'gc mut AudioManager<'gc>,

    /// The navigator backend, used by the AVM to make HTTP requests and visit webpages.
    pub navigator: &'gc mut dyn NavigatorBackend,

    /// The renderer, used by the display objects to draw themselves.
    pub renderer: &'gc mut dyn RenderBackend,

    /// The UI backend, used to detect user interactions.
    pub ui: &'gc mut dyn UiBackend,

    /// The storage backend, used for storing persistent state
    pub storage: &'gc mut dyn StorageBackend,

    /// The logging backend, used for trace output capturing.
    ///
    /// **DO NOT** use this field directly, use the `avm_trace` method instead.
    pub log: &'gc mut dyn LogBackend,

    /// The video backend, used for video decoding
    pub video: &'gc mut dyn VideoBackend,

    /// The RNG, used by the AVM `RandomNumber` opcode, `Math.random(),` and `random()`.
    pub rng: &'gc mut SmallRng,

    /// The current player's stage (including all loaded levels)
    pub stage: Stage<'gc>,

    pub mouse_data: &'gc mut MouseData<'gc>,

    /// The input manager, tracking keys state.
    pub input: &'gc InputManager,

    /// The location of the mouse when it was last over the player.
    pub mouse_position: &'gc Point<Twips>,

    /// The object being dragged via a `startDrag` action.
    pub drag_object: &'gc mut Option<crate::player::DragObject<'gc>>,

    /// Weak reference to the player.
    ///
    /// Recipients of an update context may upgrade the reference to ensure
    /// that the player lives across I/O boundaries.
    pub player: Weak<Mutex<Player>>,

    /// The player's load manager.
    ///
    /// This is required for asynchronous behavior, such as fetching data from
    /// a URL.
    pub load_manager: &'gc mut LoadManager<'gc>,

    /// The system properties
    pub system: &'gc mut SystemProperties,

    pub page_url: &'gc mut Option<String>,

    /// The current instance ID. Used to generate default `instanceN` names.
    pub instance_counter: &'gc mut i32,

    /// Shared objects cache
    pub avm1_shared_objects: &'gc mut HashMap<String, Avm1Object<'gc>>,

    /// Shared objects cache
    pub avm2_shared_objects: &'gc mut HashMap<String, Avm2Object<'gc>>,

    /// Text fields with unbound variable bindings.
    pub unbound_text_fields: &'gc mut Vec<EditText<'gc>>,

    /// Timed callbacks created with `setInterval`/`setTimeout`.
    pub timers: &'gc mut Timers<'gc>,

    pub current_context_menu: &'gc mut Option<ContextMenuState<'gc>>,

    /// The AVM1 global state.
    pub avm1: &'gc mut Avm1<'gc>,

    /// The AVM2 global state.
    pub avm2: &'gc mut Avm2<'gc>,

    /// External interface for (for example) JavaScript <-> ActionScript interaction
    pub external_interface: &'gc mut ExternalInterface<'gc>,

    /// The instant at which the SWF was launched.
    pub start_time: Instant,

    /// The instant at which the current update started.
    pub update_start: Instant,

    /// The maximum amount of time that can be called before a `Error::ExecutionTimeout`
    /// is raised. This defaults to 15 seconds but can be changed.
    pub max_execution_duration: Duration,

    /// A tracker for the current keyboard focused element
    pub focus_tracker: FocusTracker<'gc>,

    /// How many times getTimer() was called so far. Used to detect busy-loops.
    pub times_get_time_called: u32,

    /// This frame's current fake time offset, used to pretend passage of time in time functions
    pub time_offset: &'gc mut u32,

    /// The current stage frame rate.
    pub frame_rate: &'gc mut f64,

    /// Whether movies are prevented from changing the stage frame rate.
    pub forced_frame_rate: bool,

    /// Amount of actions performed since the last timeout check
    pub actions_since_timeout_check: &'gc mut u16,

    /// The current frame processing phase.
    ///
    /// If we are not doing frame processing, then this is `FramePhase::Enter`.
    pub frame_phase: &'gc mut FramePhase,

    /// Manager of in-progress media streams.
    pub stream_manager: &'gc mut StreamManager<'gc>,

    pub sockets: &'gc mut Sockets<'gc>,

    /// List of active NetConnection instances.
    pub net_connections: &'gc mut NetConnections<'gc>,

    pub local_connections: &'gc mut LocalConnections<'gc>,

    /// Dynamic root for allowing handles to GC objects to exist outside of the GC.
    pub dynamic_root: gc_arena::DynamicRootSet<'gc>,

    /// These functions are run at the end of each frame execution.
    /// Currently, this is just used for handling `Loader.loadBytes`
    #[allow(clippy::type_complexity)]
    pub post_frame_callbacks: &'gc mut Vec<PostFrameCallback<'gc>>,
}

/// Convenience methods for controlling audio.
impl<'gc> UpdateContext<'gc> {
    pub fn global_sound_transform(&self) -> &SoundTransform {
        self.audio_manager.global_sound_transform()
    }

    pub fn set_global_sound_transform(&mut self, sound_transform: SoundTransform) {
        self.audio_manager
            .set_global_sound_transform(sound_transform);
    }

    /// Get the local sound transform of a single sound instance.
    pub fn local_sound_transform(&self, instance: SoundInstanceHandle) -> Option<&SoundTransform> {
        self.audio_manager.local_sound_transform(instance)
    }

    /// Set the local sound transform of a single sound instance.
    pub fn set_local_sound_transform(
        &mut self,
        instance: SoundInstanceHandle,
        sound_transform: SoundTransform,
    ) {
        self.audio_manager
            .set_local_sound_transform(instance, sound_transform);
    }

    pub fn start_sound(
        &mut self,
        sound: SoundHandle,
        settings: &swf::SoundInfo,
        owner: Option<DisplayObject<'gc>>,
        avm1_object: Option<Avm1Object<'gc>>,
    ) -> Option<SoundInstanceHandle> {
        self.audio_manager
            .start_sound(self.audio, sound, settings, owner, avm1_object)
    }

    pub fn attach_avm2_sound_channel(
        &mut self,
        instance: SoundInstanceHandle,
        avm2_object: SoundChannelObject<'gc>,
    ) {
        self.audio_manager
            .attach_avm2_sound_channel(instance, avm2_object);
    }

    pub fn stop_sound(&mut self, instance: SoundInstanceHandle) {
        self.audio_manager.stop_sound(self.audio, instance)
    }

    pub fn stop_sounds_with_handle(&mut self, sound: SoundHandle) {
        self.audio_manager
            .stop_sounds_with_handle(self.audio, sound)
    }

    pub fn stop_sounds_with_display_object(&mut self, display_object: DisplayObject<'gc>) {
        self.audio_manager
            .stop_sounds_with_display_object(self.audio, display_object)
    }

    pub fn stop_sounds_on_parent_and_children(&mut self, display_object: DisplayObject<'gc>) {
        self.audio_manager
            .stop_sounds_on_parent_and_children(self.audio, display_object)
    }

    pub fn stop_all_sounds(&mut self) {
        self.audio_manager.stop_all_sounds(self.audio)
    }

    pub fn is_sound_playing(&self, sound: SoundInstanceHandle) -> bool {
        self.audio_manager.is_sound_playing(sound)
    }

    pub fn is_sound_playing_with_handle(&self, sound: SoundHandle) -> bool {
        self.audio_manager.is_sound_playing_with_handle(sound)
    }

    pub fn start_stream(
        &mut self,
        movie_clip: MovieClip<'gc>,
        frame: u16,
        data: SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> Option<SoundInstanceHandle> {
        self.audio_manager
            .start_stream(self.audio, movie_clip, frame, data, stream_info)
    }

    pub fn set_sound_transforms_dirty(&mut self) {
        self.audio_manager.set_sound_transforms_dirty()
    }

    /// Change the root movie.
    ///
    /// This should only be called once, as it makes no attempt at removing
    /// previous stage contents. If you need to load a new root movie, you
    /// should use `replace_root_movie`.
    pub fn set_root_movie(&mut self, movie: SwfMovie) {
        if !self.forced_frame_rate {
            *self.frame_rate = movie.frame_rate().into();
        }

        info!(
            "Loaded SWF version {}, resolution {}x{} @ {} FPS",
            movie.version(),
            movie.width(),
            movie.height(),
            self.frame_rate,
        );

        *self.swf = Arc::new(movie);
        *self.instance_counter = 0;

        if self.swf.is_action_script_3() {
            self.avm2.root_api_version =
                ApiVersion::from_swf_version(self.swf.version(), self.avm2.player_runtime)
                    .unwrap_or_else(|| panic!("Unknown SWF version {}", self.swf.version()));
        }

        self.stage.set_movie_size(
            self.gc(),
            self.swf.width().to_pixels() as u32,
            self.swf.height().to_pixels() as u32,
        );
        self.stage.set_movie(self.gc(), self.swf.clone());

        let stage_domain = self.avm2.stage_domain();
        let mut activation = Avm2Activation::from_domain(self, stage_domain);

        activation
            .context
            .library
            .library_for_movie_mut(activation.context.swf.clone())
            .set_avm2_domain(stage_domain);
        activation.context.ui.set_mouse_visible(true);

        let swf = activation.context.swf.clone();
        let root: DisplayObject = MovieClip::player_root_movie(&mut activation, swf.clone()).into();

        // The Stage `LoaderInfo` is permanently in the 'not yet loaded' state,
        // and has no associated `Loader` instance.
        // However, some properties are always accessible, and take their values
        // from the root SWF.
        let stage_loader_info =
            LoaderInfoObject::not_yet_loaded(&mut activation, swf, None, Some(root), true)
                .expect("Failed to construct Stage LoaderInfo");
        stage_loader_info
            .as_loader_info_object()
            .unwrap()
            .set_expose_content();
        activation
            .context
            .stage
            .set_loader_info(activation.gc(), stage_loader_info);

        drop(activation);

        root.set_depth(self.gc(), 0);
        let flashvars = if !self.swf.parameters().is_empty() {
            let object = ScriptObject::new(self.gc(), None);
            for (key, value) in self.swf.parameters().iter() {
                object.define_value(
                    self.gc(),
                    AvmString::new_utf8(self.gc(), key),
                    AvmString::new_utf8(self.gc(), value).into(),
                    Attribute::empty(),
                );
            }
            Some(object.into())
        } else {
            None
        };

        root.post_instantiation(self, flashvars, Instantiator::Movie, false);
        root.set_default_root_name(self);
        self.stage.replace_at_depth(self, root, 0);

        // Set the version parameter on the root.
        let mut activation =
            Activation::from_stub(self, ActivationIdentifier::root("[Version Setter]"));
        let object = root.object().coerce_to_object(&mut activation);
        let version_string = activation
            .context
            .system
            .get_version_string(activation.context.avm1);
        object.define_value(
            activation.gc(),
            "$version",
            AvmString::new_utf8(activation.gc(), version_string).into(),
            Attribute::empty(),
        );

        let stage = activation.context.stage;
        stage.build_matrices(activation.context);

        drop(activation);

        self.audio.set_frame_rate(*self.frame_rate);
    }

    pub fn replace_root_movie(&mut self, movie: SwfMovie) {
        // FIXME Use RAII here, e.g. destroy and recreate
        //       the player instance instead of cleaning up.

        // Clean up the stage before loading another root movie.
        self.sockets.close_all();
        self.timers.remove_all();

        self.set_root_movie(movie);
    }
}

impl<'gc> UpdateContext<'gc> {
    /// Convenience method to retrieve the current GC context. Note that explicitly writing
    /// `self.gc_context` can be sometimes necessary to satisfy the borrow checker.
    #[inline(always)]
    pub fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
    }

    pub fn avm_trace(&self, message: &str) {
        self.log.avm_trace(&message.replace('\r', "\n"));
    }
}

/// A queued ActionScript call.
#[derive(Collect)]
#[collect(no_drop)]
pub struct QueuedAction<'gc> {
    /// The movie clip this ActionScript is running on.
    pub clip: DisplayObject<'gc>,

    /// The type of action this is, along with the corresponding bytecode/method data.
    pub action_type: ActionType<'gc>,

    /// Whether this is an unload action, which can still run if the clip is removed.
    pub is_unload: bool,
}

/// Action and gotos need to be queued up to execute at the end of the frame.
#[derive(Collect)]
#[collect(no_drop)]
pub struct ActionQueue<'gc> {
    /// Each priority is kept in a separate bucket.
    action_queue: [VecDeque<QueuedAction<'gc>>; ActionQueue::NUM_PRIORITIES],
}

impl<'gc> ActionQueue<'gc> {
    const DEFAULT_CAPACITY: usize = 32;
    const NUM_PRIORITIES: usize = 3;

    /// Crates a new `ActionQueue` with an empty queue.
    pub fn new() -> Self {
        let action_queue = std::array::from_fn(|_| VecDeque::with_capacity(Self::DEFAULT_CAPACITY));
        Self { action_queue }
    }

    /// Queues an action to run for the given movie clip.
    /// The action will be skipped if the clip is removed before the action runs.
    pub fn queue_action(
        &mut self,
        clip: DisplayObject<'gc>,
        action_type: ActionType<'gc>,
        is_unload: bool,
    ) {
        let priority = action_type.priority();
        let action = QueuedAction {
            clip,
            action_type,
            is_unload,
        };
        debug_assert!(priority < Self::NUM_PRIORITIES);
        if let Some(queue) = self.action_queue.get_mut(priority) {
            queue.push_back(action)
        }
    }

    /// Sorts and drains the actions from the queue.
    pub fn pop_action(&mut self) -> Option<QueuedAction<'gc>> {
        self.action_queue
            .iter_mut()
            .rev()
            .find_map(VecDeque::pop_front)
    }
}

impl Default for ActionQueue<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared data used during rendering.
/// `Player` creates this when it renders a frame and passes it down to display objects.
///
/// As a convenience, this type can be deref-coerced to `Mutation<'gc>`, but note that explicitly
/// writing `context.gc()` can be sometimes necessary to satisfy the borrow checker.
pub struct RenderContext<'a, 'gc> {
    /// The renderer, used by the display objects to register themselves.
    pub renderer: &'a mut dyn RenderBackend,

    /// The command list, used by the display objects to draw themselves.
    pub commands: CommandList,

    /// Any offscreen draws that should be used to redraw a cacheAsBitmap
    pub cache_draws: &'a mut Vec<BitmapCacheEntry>,

    /// The GC context, used to perform any `Gc` writes that must occur during rendering.
    pub gc_context: &'gc Mutation<'gc>,

    /// The library, which provides access to fonts and other definitions when rendering.
    pub library: &'a Library<'gc>,

    /// The transform stack controls the matrix and color transform as we traverse the display hierarchy.
    pub transform_stack: &'a mut TransformStack,

    /// Whether we're rendering offscreen. This can disable some logic like Ruffle-side render culling
    pub is_offscreen: bool,

    /// Whether to use cacheAsBitmap, vs drawing everything explicitly
    pub use_bitmap_cache: bool,

    /// The current player's stage (including all loaded levels)
    pub stage: Stage<'gc>,
}

impl<'gc> RenderContext<'_, 'gc> {
    /// Convenience method to retrieve the current GC context. Note that explicitly writing
    /// `self.gc_context` can be sometimes necessary to satisfy the borrow checker.
    #[inline(always)]
    pub fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
    }

    /// Draw a rectangle outline.
    ///
    /// The outline is contained within the given bounds.
    pub fn draw_rect_outline(&mut self, color: Color, bounds: Rectangle<Twips>, thickness: Twips) {
        let bounds = self.transform_stack.transform().matrix * bounds;
        let width = bounds.width().to_pixels() as f32;
        let height = bounds.height().to_pixels() as f32;
        let thickness_pixels = thickness.to_pixels() as f32;
        // Top
        self.commands.draw_rect(
            color,
            Matrix::create_box(width, thickness_pixels, bounds.x_min, bounds.y_min),
        );
        // Bottom
        self.commands.draw_rect(
            color,
            Matrix::create_box(
                width,
                thickness_pixels,
                bounds.x_min,
                bounds.y_max - thickness,
            ),
        );
        // Left
        self.commands.draw_rect(
            color,
            Matrix::create_box(thickness_pixels, height, bounds.x_min, bounds.y_min),
        );
        // Right
        self.commands.draw_rect(
            color,
            Matrix::create_box(
                thickness_pixels,
                height,
                bounds.x_max - thickness,
                bounds.y_min,
            ),
        );
    }
}

/// The type of action being run.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum ActionType<'gc> {
    /// Normal frame or event actions.
    Normal { bytecode: SwfSlice },

    /// AVM1 initialize clip event.
    Initialize { bytecode: SwfSlice },

    /// Construct a movie with a custom class or on(construct) events.
    Construct {
        constructor: Option<Avm1Object<'gc>>,
        events: Vec<SwfSlice>,
    },

    /// An event handler method, e.g. `onEnterFrame`.
    Method {
        object: Avm1Object<'gc>,
        name: &'static str,
        args: Vec<Avm1Value<'gc>>,
    },

    /// A system listener method.
    NotifyListeners {
        listener: &'static str,
        method: &'static str,
        args: Vec<Avm1Value<'gc>>,
    },
}

impl ActionType<'_> {
    fn priority(&self) -> usize {
        match self {
            ActionType::Initialize { .. } => 2,
            ActionType::Construct { .. } => 1,
            _ => 0,
        }
    }
}

impl fmt::Debug for ActionType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActionType::Normal { bytecode } => f
                .debug_struct("ActionType::Normal")
                .field("bytecode", bytecode)
                .finish(),
            ActionType::Initialize { bytecode } => f
                .debug_struct("ActionType::Initialize")
                .field("bytecode", bytecode)
                .finish(),
            ActionType::Construct {
                constructor,
                events,
            } => f
                .debug_struct("ActionType::Construct")
                .field("constructor", constructor)
                .field("events", events)
                .finish(),
            ActionType::Method { object, name, args } => f
                .debug_struct("ActionType::Method")
                .field("object", object)
                .field("name", name)
                .field("args", args)
                .finish(),
            ActionType::NotifyListeners {
                listener,
                method,
                args,
            } => f
                .debug_struct("ActionType::NotifyListeners")
                .field("listener", listener)
                .field("method", method)
                .field("args", args)
                .finish(),
        }
    }
}
