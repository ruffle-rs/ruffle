//! Contexts and helper types passed between functions.

use crate::avm1::Avm1;
use crate::avm1::SystemProperties;
use crate::avm1::{Object as Avm1Object, Value as Avm1Value};
use crate::avm2::{Avm2, Object as Avm2Object, SoundChannelObject};
use crate::backend::{
    audio::{AudioBackend, AudioManager, SoundHandle, SoundInstanceHandle},
    log::LogBackend,
    navigator::NavigatorBackend,
    storage::StorageBackend,
    ui::{InputManager, UiBackend},
};
use crate::context_menu::ContextMenuState;
use crate::display_object::{EditText, InteractiveObject, MovieClip, SoundTransform, Stage};
use crate::external::ExternalInterface;
use crate::focus_tracker::FocusTracker;
use crate::frame_lifecycle::FramePhase;
use crate::library::Library;
use crate::loader::LoadManager;
use crate::local_connection::LocalConnections;
use crate::net_connection::NetConnections;
use crate::player::Player;
use crate::prelude::*;
use crate::socket::Sockets;
use crate::streams::StreamManager;
use crate::string::AvmStringInterner;
use crate::stub::StubCollection;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::timer::Timers;
use core::fmt;
use gc_arena::{Collect, Mutation};
use rand::rngs::SmallRng;
use ruffle_render::backend::{BitmapCacheEntry, RenderBackend};
use ruffle_render::commands::CommandList;
use ruffle_render::transform::TransformStack;
use ruffle_video::backend::VideoBackend;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use web_time::Instant;

/// Minimal context, useful for manipulating the GC heap.
pub struct GcContext<'a, 'gc> {
    /// The mutation context to allocate and mutate `Gc` pointers.
    pub gc_context: &'gc Mutation<'gc>,

    /// The global string interner.
    pub interner: &'a mut AvmStringInterner<'gc>,
}

impl<'a, 'gc> GcContext<'a, 'gc> {
    #[inline(always)]
    pub fn reborrow<'b>(&'b mut self) -> GcContext<'b, 'gc>
    where
        'a: 'b,
    {
        GcContext {
            gc_context: self.gc_context,
            interner: self.interner,
        }
    }

    /// Convenience method to retrieve the current GC context. Note that explicitely writing
    /// `self.gc_context` can be sometimes necessary to satisfy the borrow checker.
    #[inline(always)]
    pub fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
    }
}

/// `UpdateContext` holds shared data that is used by the various subsystems of Ruffle.
/// `Player` creates this when it begins a tick and passes it through the call stack to
/// children and the VM.
pub struct UpdateContext<'a, 'gc> {
    /// The queue of actions that will be run after the display list updates.
    /// Display objects and actions can push actions onto the queue.
    pub action_queue: &'a mut ActionQueue<'gc>,

    /// The mutation context to allocate and mutate `Gc` pointers.
    pub gc_context: &'gc Mutation<'gc>,

    /// The global string interner.
    pub interner: &'a mut AvmStringInterner<'gc>,

    /// A collection of stubs encountered during this movie.
    pub stub_tracker: &'a mut StubCollection,

    /// The library containing character definitions for this SWF.
    /// Used to instantiate a `DisplayObject` of a given ID.
    pub library: &'a mut Library<'gc>,

    /// The version of the Flash Player we are emulating.
    /// TODO: This is a little confusing because this represents the player's max SWF version,
    /// which is an integer (e.g. 13), but "Flash Player version" is a triplet (11.6.0), and these
    /// aren't in sync. It may be better to have separate `player_swf_version` and `player_version`
    /// variables.
    pub player_version: u8,

    /// Requests that the player re-renders after this execution (e.g. due to `updateAfterEvent`).
    pub needs_render: &'a mut bool,

    /// The root SWF file.
    pub swf: &'a Arc<SwfMovie>,

    /// The audio backend, used by display objects and AVM to play audio.
    pub audio: &'a mut dyn AudioBackend,

    /// The audio manager, managing all actively playing sounds.
    pub audio_manager: &'a mut AudioManager<'gc>,

    /// The navigator backend, used by the AVM to make HTTP requests and visit webpages.
    pub navigator: &'a mut (dyn NavigatorBackend + 'a),

    /// The renderer, used by the display objects to draw themselves.
    pub renderer: &'a mut dyn RenderBackend,

    /// The UI backend, used to detect user interactions.
    pub ui: &'a mut dyn UiBackend,

    /// The storage backend, used for storing persistent state
    pub storage: &'a mut dyn StorageBackend,

    /// The logging backend, used for trace output capturing.
    ///
    /// **DO NOT** use this field directly, use the `avm_trace` method instead.
    pub log: &'a mut dyn LogBackend,

    /// The video backend, used for video decoding
    pub video: &'a mut dyn VideoBackend,

    /// The RNG, used by the AVM `RandomNumber` opcode, `Math.random(),` and `random()`.
    pub rng: &'a mut SmallRng,

    /// The current player's stage (including all loaded levels)
    pub stage: Stage<'gc>,

    /// The display object that the mouse is currently hovering over.
    pub mouse_over_object: Option<InteractiveObject<'gc>>,

    /// If the mouse is down, the display object that the mouse is currently pressing.
    pub mouse_down_object: Option<InteractiveObject<'gc>>,

    /// The input manager, tracking keys state.
    pub input: &'a InputManager,

    /// The location of the mouse when it was last over the player.
    pub mouse_position: &'a Point<Twips>,

    /// The object being dragged via a `startDrag` action.
    pub drag_object: &'a mut Option<crate::player::DragObject<'gc>>,

    /// Weak reference to the player.
    ///
    /// Recipients of an update context may upgrade the reference to ensure
    /// that the player lives across I/O boundaries.
    pub player: Weak<Mutex<Player>>,

    /// The player's load manager.
    ///
    /// This is required for asynchronous behavior, such as fetching data from
    /// a URL.
    pub load_manager: &'a mut LoadManager<'gc>,

    /// The system properties
    pub system: &'a mut SystemProperties,

    pub page_url: &'a mut Option<String>,

    /// The current instance ID. Used to generate default `instanceN` names.
    pub instance_counter: &'a mut i32,

    /// Shared objects cache
    pub avm1_shared_objects: &'a mut HashMap<String, Avm1Object<'gc>>,

    /// Shared objects cache
    pub avm2_shared_objects: &'a mut HashMap<String, Avm2Object<'gc>>,

    /// Text fields with unbound variable bindings.
    pub unbound_text_fields: &'a mut Vec<EditText<'gc>>,

    /// Timed callbacks created with `setInterval`/`setTimeout`.
    pub timers: &'a mut Timers<'gc>,

    pub current_context_menu: &'a mut Option<ContextMenuState<'gc>>,

    /// The AVM1 global state.
    pub avm1: &'a mut Avm1<'gc>,

    /// The AVM2 global state.
    pub avm2: &'a mut Avm2<'gc>,

    /// External interface for (for example) JavaScript <-> ActionScript interaction
    pub external_interface: &'a mut ExternalInterface<'gc>,

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
    pub time_offset: &'a mut u32,

    /// The current stage frame rate.
    pub frame_rate: &'a mut f64,

    /// Whether movies are prevented from changing the stage frame rate.
    pub forced_frame_rate: bool,

    /// Amount of actions performed since the last timeout check
    pub actions_since_timeout_check: &'a mut u16,

    /// The current frame processing phase.
    ///
    /// If we are not doing frame processing, then this is `FramePhase::Enter`.
    pub frame_phase: &'a mut FramePhase,

    /// Manager of in-progress media streams.
    pub stream_manager: &'a mut StreamManager<'gc>,

    pub sockets: &'a mut Sockets<'gc>,

    /// List of active NetConnection instances.
    pub net_connections: &'a mut NetConnections<'gc>,

    pub local_connections: &'a mut LocalConnections<'gc>,

    /// Dynamic root for allowing handles to GC objects to exist outside of the GC.
    pub dynamic_root: gc_arena::DynamicRootSet<'gc>,
}

/// Convenience methods for controlling audio.
impl<'a, 'gc> UpdateContext<'a, 'gc> {
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
        avm1_object: Option<crate::avm1::SoundObject<'gc>>,
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

    pub fn stop_all_sounds(&mut self) {
        self.audio_manager.stop_all_sounds(self.audio)
    }

    pub fn is_sound_playing(&mut self, sound: SoundInstanceHandle) -> bool {
        self.audio_manager.is_sound_playing(sound)
    }

    pub fn is_sound_playing_with_handle(&mut self, sound: SoundHandle) -> bool {
        self.audio_manager.is_sound_playing_with_handle(sound)
    }

    pub fn start_stream(
        &mut self,
        movie_clip: MovieClip<'gc>,
        frame: u16,
        data: crate::tag_utils::SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> Option<SoundInstanceHandle> {
        self.audio_manager
            .start_stream(self.audio, movie_clip, frame, data, stream_info)
    }

    pub fn set_sound_transforms_dirty(&mut self) {
        self.audio_manager.set_sound_transforms_dirty()
    }
}

impl<'a, 'gc> UpdateContext<'a, 'gc> {
    /// Convenience method to retrieve the current GC context. Note that explicitely writing
    /// `self.gc_context` can be sometimes necessary to satisfy the borrow checker.
    #[inline(always)]
    pub fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
    }

    /// Transform a borrowed update context into an owned update context with
    /// a shorter internal lifetime.
    ///
    /// This is particularly useful for structures that may wish to hold an
    /// update context without adding further lifetimes for its borrowing.
    /// Please note that you will not be able to use the original update
    /// context until this reborrowed copy has fallen out of scope.
    #[inline]
    pub fn reborrow<'b>(&'b mut self) -> UpdateContext<'b, 'gc>
    where
        'a: 'b,
    {
        UpdateContext {
            action_queue: self.action_queue,
            gc_context: self.gc_context,
            interner: self.interner,
            stub_tracker: self.stub_tracker,
            library: self.library,
            player_version: self.player_version,
            needs_render: self.needs_render,
            swf: self.swf,
            audio: self.audio,
            audio_manager: self.audio_manager,
            navigator: self.navigator,
            renderer: self.renderer,
            log: self.log,
            ui: self.ui,
            video: self.video,
            storage: self.storage,
            rng: self.rng,
            stage: self.stage,
            mouse_over_object: self.mouse_over_object,
            mouse_down_object: self.mouse_down_object,
            input: self.input,
            mouse_position: self.mouse_position,
            drag_object: self.drag_object,
            player: self.player.clone(),
            load_manager: self.load_manager,
            system: self.system,
            page_url: self.page_url,
            instance_counter: self.instance_counter,
            avm1_shared_objects: self.avm1_shared_objects,
            avm2_shared_objects: self.avm2_shared_objects,
            unbound_text_fields: self.unbound_text_fields,
            timers: self.timers,
            current_context_menu: self.current_context_menu,
            avm1: self.avm1,
            avm2: self.avm2,
            external_interface: self.external_interface,
            start_time: self.start_time,
            update_start: self.update_start,
            max_execution_duration: self.max_execution_duration,
            focus_tracker: self.focus_tracker,
            times_get_time_called: self.times_get_time_called,
            time_offset: self.time_offset,
            frame_rate: self.frame_rate,
            forced_frame_rate: self.forced_frame_rate,
            actions_since_timeout_check: self.actions_since_timeout_check,
            frame_phase: self.frame_phase,
            stream_manager: self.stream_manager,
            sockets: self.sockets,
            net_connections: self.net_connections,
            local_connections: self.local_connections,
            dynamic_root: self.dynamic_root,
        }
    }

    #[inline]
    pub fn borrow_gc<'b>(&'b mut self) -> GcContext<'b, 'gc>
    where
        'a: 'b,
    {
        GcContext {
            gc_context: self.gc_context,
            interner: self.interner,
        }
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

impl<'gc> Default for ActionQueue<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared data used during rendering.
/// `Player` creates this when it renders a frame and passes it down to display objects.
///
/// As a convenience, this type can be deref-coerced to `Mutation<'gc>`, but note that explicitely
/// writing `context.gc_context` can be sometimes necessary to satisfy the borrow checker.
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

    /// Whether or not to use cacheAsBitmap, vs drawing everything explicitly
    pub use_bitmap_cache: bool,

    /// The current player's stage (including all loaded levels)
    pub stage: Stage<'gc>,
}

impl<'a, 'gc> RenderContext<'a, 'gc> {
    /// Convenience method to retrieve the current GC context. Note that explicitely writing
    /// `self.gc_context` can be sometimes necessary to satisfy the borrow checker.
    #[inline(always)]
    pub fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
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
