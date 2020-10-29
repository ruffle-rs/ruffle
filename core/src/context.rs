//! Contexts and helper types passed between functions.
use crate::avm1;

use crate::avm1::globals::system::SystemProperties;
use crate::avm1::{Avm1, Object as Avm1Object, Timers, Value as Avm1Value};
use crate::avm2::{Avm2, Object as Avm2Object, Value as Avm2Value};
use crate::backend::input::InputBackend;
use crate::backend::locale::LocaleBackend;
use crate::backend::log::LogBackend;
use crate::backend::storage::StorageBackend;
use crate::backend::{audio::AudioBackend, navigator::NavigatorBackend, render::RenderBackend};
use crate::display_object::EditText;
use crate::external::ExternalInterface;
use crate::focus_tracker::FocusTracker;
use crate::library::Library;
use crate::loader::LoadManager;
use crate::player::Player;
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::transform::TransformStack;
use core::fmt;
use gc_arena::{Collect, CollectionContext, MutationContext};
use instant::Instant;
use rand::rngs::SmallRng;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

/// `UpdateContext` holds shared data that is used by the various subsystems of Ruffle.
/// `Player` crates this when it begins a tick and passes it through the call stack to
/// children and the VM.
pub struct UpdateContext<'a, 'gc, 'gc_context> {
    /// The queue of actions that will be run after the display list updates.
    /// Display objects and actions can push actions onto the queue.
    pub action_queue: &'a mut ActionQueue<'gc>,

    /// The background color of the Stage. Changed by the `SetBackgroundColor` SWF tag.
    /// TODO: Move this into a `Stage` display object.
    pub background_color: &'a mut Color,

    /// The mutation context to allocate and mutate `GcCell` types.
    pub gc_context: MutationContext<'gc, 'gc_context>,

    /// The library containing character definitions for this SWF.
    /// Used to instantiate a `DisplayObject` of a given ID.
    pub library: &'a mut Library<'gc>,

    /// The version of the Flash Player we are emulating.
    /// TODO: This is a little confusing because this represents the player's max SWF version,
    /// which is an integer (e.g. 13), but "Flash Player version" is a triplet (11.6.0), and these
    /// aren't in sync. It may be better to have separate `player_swf_version` and `player_version`
    /// variables.
    pub player_version: u8,

    /// Requests a that the player re-renders after this execution (e.g. due to `updateAfterEvent`).
    pub needs_render: &'a mut bool,

    /// The root SWF file.
    pub swf: &'a Arc<SwfMovie>,

    /// The audio backend, used by display objects and AVM to play audio.
    pub audio: &'a mut (dyn AudioBackend + 'a),

    /// The navigator backend, used by the AVM to make HTTP requests and visit webpages.
    pub navigator: &'a mut (dyn NavigatorBackend + 'a),

    /// The renderer, used by the display objects to draw themselves.
    pub renderer: &'a mut dyn RenderBackend,

    /// The input backend, used to detect user interactions.
    pub input: &'a mut dyn InputBackend,

    /// The storage backend, used for storing persistent state
    pub storage: &'a mut dyn StorageBackend,

    /// The locale backend, used for localisation and personalisation
    pub locale: &'a mut dyn LocaleBackend,

    /// The logging backend, used for trace output capturing
    pub log: &'a mut dyn LogBackend,

    /// The RNG, used by the AVM `RandomNumber` opcode,  `Math.random(),` and `random()`.
    pub rng: &'a mut SmallRng,

    /// All loaded levels of the current player.
    pub levels: &'a mut BTreeMap<u32, DisplayObject<'gc>>,

    /// The current set of system-specified prototypes to use when constructing
    /// new built-in objects.
    pub system_prototypes: avm1::SystemPrototypes<'gc>,

    /// The display object that the mouse is currently hovering over.
    pub mouse_hovered_object: Option<DisplayObject<'gc>>,

    /// The location of the mouse when it was last over the player.
    pub mouse_position: &'a (Twips, Twips),

    /// The object being dragged via a `startDrag` action.
    pub drag_object: &'a mut Option<crate::player::DragObject<'gc>>,

    /// The dimensions of the stage.
    pub stage_size: (Twips, Twips),

    /// Weak reference to the player.
    ///
    /// Recipients of an update context may upgrade the reference to ensure
    /// that the player lives across I/O boundaries.
    pub player: Option<Weak<Mutex<Player>>>,

    /// The player's load manager.
    ///
    /// This is required for asynchronous behavior, such as fetching data from
    /// a URL.
    pub load_manager: &'a mut LoadManager<'gc>,

    /// The system properties
    pub system: &'a mut SystemProperties,

    /// The current instance ID. Used to generate default `instanceN` names.
    pub instance_counter: &'a mut i32,

    /// Shared objects cache
    pub shared_objects: &'a mut HashMap<String, Avm1Object<'gc>>,

    /// Text fields with unbound variable bindings.
    pub unbound_text_fields: &'a mut Vec<EditText<'gc>>,

    /// Timed callbacks created with `setInterval`/`setTimeout`.
    pub timers: &'a mut Timers<'gc>,

    /// The AVM1 global state.
    pub avm1: &'a mut Avm1<'gc>,

    /// The AVM2 global state.
    pub avm2: &'a mut Avm2<'gc>,

    /// External interface for (for example) Javascript <-> Actionscript interaction
    pub external_interface: &'a mut ExternalInterface<'gc>,

    /// The instant at which the current update started.
    pub update_start: Instant,

    /// The maximum amount of time that can be called before a `Error::ExecutionTimeout`
    /// is raised. This defaults to 15 seconds but can be changed.
    pub max_execution_duration: Duration,

    /// A tracker for the current keyboard focused element
    pub focus_tracker: FocusTracker<'gc>,
}

unsafe impl<'a, 'gc, 'gc_context> Collect for UpdateContext<'a, 'gc, 'gc_context> {
    fn trace(&self, cc: CollectionContext) {
        self.action_queue.trace(cc);
        self.background_color.trace(cc);
        self.library.trace(cc);
        self.player_version.trace(cc);
        self.needs_render.trace(cc);
        self.swf.trace(cc);
        self.audio.trace(cc);
        self.navigator.trace(cc);
        self.renderer.trace(cc);
        self.input.trace(cc);
        self.storage.trace(cc);
        self.rng.trace(cc);
        self.levels.trace(cc);
        self.system_prototypes.trace(cc);
        self.mouse_hovered_object.trace(cc);
        self.mouse_position.trace(cc);
        self.drag_object.trace(cc);
        self.load_manager.trace(cc);
        self.system.trace(cc);
        self.instance_counter.trace(cc);
        self.shared_objects.trace(cc);
        self.unbound_text_fields.trace(cc);
        self.timers.trace(cc);
        self.avm1.trace(cc);
        self.avm2.trace(cc);
        self.focus_tracker.trace(cc);
    }
}

impl<'a, 'gc, 'gc_context> UpdateContext<'a, 'gc, 'gc_context> {
    /// Transform a borrowed update context into an owned update context with
    /// a shorter internal lifetime.
    ///
    /// This is particularly useful for structures that may wish to hold an
    /// update context without adding further lifetimes for it's borrowing.
    /// Please note that you will not be able to use the original update
    /// context until this reborrowed copy has fallen out of scope.
    pub fn reborrow<'b>(&'b mut self) -> UpdateContext<'b, 'gc, 'gc_context>
    where
        'a: 'b,
    {
        UpdateContext {
            action_queue: self.action_queue,
            background_color: self.background_color,
            gc_context: self.gc_context,
            library: self.library,
            player_version: self.player_version,
            needs_render: self.needs_render,
            swf: self.swf,
            audio: self.audio,
            navigator: self.navigator,
            renderer: self.renderer,
            locale: self.locale,
            log: self.log,
            input: self.input,
            storage: self.storage,
            rng: self.rng,
            levels: self.levels,
            system_prototypes: self.system_prototypes.clone(),
            mouse_hovered_object: self.mouse_hovered_object,
            mouse_position: self.mouse_position,
            drag_object: self.drag_object,
            stage_size: self.stage_size,
            player: self.player.clone(),
            load_manager: self.load_manager,
            system: self.system,
            instance_counter: self.instance_counter,
            shared_objects: self.shared_objects,
            unbound_text_fields: self.unbound_text_fields,
            timers: self.timers,
            avm1: self.avm1,
            avm2: self.avm2,
            external_interface: self.external_interface,
            update_start: self.update_start,
            max_execution_duration: self.max_execution_duration,
            focus_tracker: self.focus_tracker,
        }
    }
}

/// A queued ActionScript call.
pub struct QueuedActions<'gc> {
    /// The movie clip this ActionScript is running on.
    pub clip: DisplayObject<'gc>,

    /// The type of action this is, along with the corresponding bytecode/method data.
    pub action_type: ActionType<'gc>,

    /// Whether this is an unload action, which can still run if the clip is removed.
    pub is_unload: bool,
}

unsafe impl<'gc> Collect for QueuedActions<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.clip.trace(cc);
        self.action_type.trace(cc);
    }
}

/// Action and gotos need to be queued up to execute at the end of the frame.
pub struct ActionQueue<'gc> {
    change_prototype_queue: VecDeque<QueuedActions<'gc>>,
    action_queue: VecDeque<QueuedActions<'gc>>,
}

impl<'gc> ActionQueue<'gc> {
    const DEFAULT_CAPACITY: usize = 32;

    /// Crates a new `ActionQueue` with an empty queue.
    pub fn new() -> Self {
        Self {
            change_prototype_queue: VecDeque::with_capacity(Self::DEFAULT_CAPACITY),
            action_queue: VecDeque::with_capacity(Self::DEFAULT_CAPACITY),
        }
    }

    /// Queues ActionScript to run for the given movie clip.
    /// `actions` is the slice of ActionScript bytecode to run.
    /// The actions will be skipped if the clip is removed before the actions run.
    pub fn queue_actions(
        &mut self,
        clip: DisplayObject<'gc>,
        action_type: ActionType<'gc>,
        is_unload: bool,
    ) {
        // Prototype change goes a higher priority queue.
        if let ActionType::Construct { .. } = action_type {
            self.change_prototype_queue.push_back(QueuedActions {
                clip,
                action_type,
                is_unload,
            })
        } else {
            self.action_queue.push_back(QueuedActions {
                clip,
                action_type,
                is_unload,
            })
        }
    }

    /// Sorts and drains the actions from the queue.
    pub fn pop_action(&mut self) -> Option<QueuedActions<'gc>> {
        if !self.change_prototype_queue.is_empty() {
            self.change_prototype_queue.pop_front()
        } else {
            self.action_queue.pop_front()
        }
    }
}

impl<'gc> Default for ActionQueue<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<'gc> Collect for ActionQueue<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.change_prototype_queue.iter().for_each(|o| o.trace(cc));
        self.action_queue.iter().for_each(|o| o.trace(cc));
    }
}

/// Shared data used during rendering.
/// `Player` creates this when it renders a frame and passes it down to display objects.
pub struct RenderContext<'a, 'gc> {
    /// The renderer, used by the display objects to draw themselves.
    pub renderer: &'a mut dyn RenderBackend,

    /// The library, which provides access to fonts and other definitions when rendering.
    pub library: &'a Library<'gc>,

    /// The transform stack controls the matrix and color transform as we traverse the display hierarchy.
    pub transform_stack: &'a mut TransformStack,
    /// The bounds of the current viewport in twips. Used for culling.
    pub view_bounds: BoundingBox,

    /// The stack of clip depths, used in masking.
    pub clip_depth_stack: Vec<Depth>,

    /// Whether to allow pushing a new mask. A masker-inside-a-masker does not work in Flash, instead
    /// causing the inner mask to be included as part of the outer mask. Maskee-inside-a-maskee works as one expects.
    pub allow_mask: bool,
}

/// The type of action being run.
#[derive(Clone)]
pub enum ActionType<'gc> {
    /// Normal frame or event actions.
    Normal { bytecode: SwfSlice },

    /// Construct a movie with a custom class or on(construct) events
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

    /// A system listener method,
    NotifyListeners {
        listener: &'static str,
        method: &'static str,
        args: Vec<Avm1Value<'gc>>,
    },

    /// An AVM2 callable, e.g. a frame script or event handler.
    Callable2 {
        callable: Avm2Object<'gc>,
        reciever: Option<Avm2Object<'gc>>,
        args: Vec<Avm2Value<'gc>>,
    },
}

impl fmt::Debug for ActionType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActionType::Normal { bytecode } => f
                .debug_struct("ActionType::Normal")
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
            ActionType::Callable2 {
                callable,
                reciever,
                args,
            } => f
                .debug_struct("ActionType::Callable2")
                .field("callable", callable)
                .field("reciever", reciever)
                .field("args", args)
                .finish(),
        }
    }
}

unsafe impl<'gc> Collect for ActionType<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        match self {
            ActionType::Construct { constructor, .. } => {
                constructor.trace(cc);
            }
            ActionType::Method { object, args, .. } => {
                object.trace(cc);
                args.trace(cc);
            }
            ActionType::NotifyListeners { args, .. } => {
                args.trace(cc);
            }
            _ => {}
        }
    }
}
