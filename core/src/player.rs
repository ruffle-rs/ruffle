use crate::avm1::Attribute;
use crate::avm1::Avm1;
use crate::avm1::Object;
use crate::avm1::SystemProperties;
use crate::avm1::VariableDumper;
use crate::avm1::{Activation, ActivationIdentifier};
use crate::avm1::{TObject, Value};
use crate::avm2::{
    object::TObject as _, Activation as Avm2Activation, Avm2, CallStack, Object as Avm2Object,
};
use crate::backend::ui::FontDefinition;
use crate::backend::{
    audio::{AudioBackend, AudioManager},
    log::LogBackend,
    navigator::{NavigatorBackend, Request},
    storage::StorageBackend,
    ui::{MouseCursor, UiBackend},
};
use crate::compatibility_rules::CompatibilityRules;
use crate::config::Letterbox;
use crate::context::{ActionQueue, ActionType, RenderContext, UpdateContext};
use crate::context_menu::{
    BuiltInItemFlags, ContextMenuCallback, ContextMenuItem, ContextMenuState,
};
use crate::display_object::Avm2MousePick;
use crate::display_object::{
    EditText, InteractiveObject, Stage, StageAlign, StageDisplayState, StageScaleMode,
    TInteractiveObject, WindowMode,
};
use crate::events::GamepadButton;
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult, KeyCode, MouseButton, PlayerEvent};
use crate::external::{ExternalInterface, ExternalInterfaceProvider, NullFsCommandProvider};
use crate::external::{FsCommandProvider, Value as ExternalValue};
use crate::focus_tracker::NavigationDirection;
use crate::frame_lifecycle::{run_all_phases_avm2, FramePhase};
use crate::input::InputManager;
use crate::library::Library;
use crate::limits::ExecutionLimit;
use crate::loader::{LoadBehavior, LoadManager};
use crate::local_connection::LocalConnections;
use crate::locale::get_current_date_time;
use crate::net_connection::NetConnections;
use crate::prelude::*;
use crate::socket::Sockets;
use crate::streams::StreamManager;
use crate::string::StringContext;
use crate::string::{AvmString, AvmStringInterner};
use crate::stub::StubCollection;
use crate::tag_utils::SwfMovie;
use crate::timer::Timers;
use crate::vminterface::Instantiator;
use crate::DefaultFont;
use gc_arena::lock::GcRefLock;
use gc_arena::{Collect, DynamicRootSet, Mutation, Rootable};
use rand::{rngs::SmallRng, SeedableRng};
use ruffle_render::backend::{null::NullRenderer, RenderBackend, ViewportDimensions};
use ruffle_render::commands::CommandList;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::TransformStack;
use ruffle_video::backend::VideoBackend;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak as RcWeak};
use std::str::FromStr;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use tracing::instrument;
use web_time::Instant;

/// The newest known Flash Player version, serves as a default to
/// `player_version`.
pub const NEWEST_PLAYER_VERSION: u8 = 32;

#[cfg(feature = "default_font")]
pub const FALLBACK_DEVICE_FONT_TAG: &[u8] = include_bytes!("../assets/noto-sans-definefont3.bin");

#[derive(Collect)]
#[collect(no_drop)]
struct GcRoot<'gc> {
    avm2_callstack: GcRefLock<'gc, CallStack<'gc>>,
    data: GcRefLock<'gc, GcRootData<'gc>>,
}

#[derive(Clone)]
pub struct StaticCallstack {
    arena: RcWeak<RefCell<GcArena>>,
}

impl StaticCallstack {
    pub fn avm2(&self, f: impl for<'gc> FnOnce(&CallStack<'gc>)) {
        if let Some(arena) = self.arena.upgrade() {
            if let Ok(arena) = arena.try_borrow() {
                arena.mutate(|_, root| {
                    let callstack = root.avm2_callstack.borrow();
                    if !callstack.is_empty() {
                        f(&callstack);
                    }
                })
            }
        }
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct MouseData<'gc> {
    /// The object that the mouse is currently hovering over.
    pub hovered: Option<InteractiveObject<'gc>>,

    /// If the mouse is down, the object that the mouse is currently pressing.
    pub pressed: Option<InteractiveObject<'gc>>,
    pub right_pressed: Option<InteractiveObject<'gc>>,
    pub middle_pressed: Option<InteractiveObject<'gc>>,
}

impl<'gc> MouseData<'gc> {
    pub fn pressed(&self, button: MouseButton) -> Option<InteractiveObject<'gc>> {
        match button {
            MouseButton::Unknown => None,
            MouseButton::Left => self.pressed,
            MouseButton::Right => self.right_pressed,
            MouseButton::Middle => self.middle_pressed,
        }
    }

    pub fn set_pressed(&mut self, button: MouseButton, value: Option<InteractiveObject<'gc>>) {
        match button {
            MouseButton::Unknown => {}
            MouseButton::Left => self.pressed = value,
            MouseButton::Right => self.right_pressed = value,
            MouseButton::Middle => self.middle_pressed = value,
        }
    }
}

#[derive(Collect)]
#[collect(no_drop)]
struct GcRootData<'gc> {
    library: Library<'gc>,

    /// The root of the display object hierarchy.
    ///
    /// It's children are the `level`s of AVM1, it may also be directly
    /// accessed in AVM2.
    stage: Stage<'gc>,

    mouse_data: MouseData<'gc>,

    /// The object being dragged via a `startDrag` action.
    drag_object: Option<DragObject<'gc>>,

    /// Interpreter state for AVM1 code.
    avm1: Avm1<'gc>,

    /// Interpreter state for AVM2 code.
    avm2: Avm2<'gc>,

    action_queue: ActionQueue<'gc>,
    interner: AvmStringInterner<'gc>,

    /// Object which manages asynchronous processes that need to interact with
    /// data in the GC arena.
    load_manager: LoadManager<'gc>,

    avm1_shared_objects: HashMap<String, Object<'gc>>,

    avm2_shared_objects: HashMap<String, Avm2Object<'gc>>,

    /// Text fields with unbound variable bindings.
    unbound_text_fields: Vec<EditText<'gc>>,

    /// Timed callbacks created with `setInterval`/`setTimeout`.
    timers: Timers<'gc>,

    current_context_menu: Option<ContextMenuState<'gc>>,

    /// External interface for (for example) JavaScript <-> ActionScript interaction
    external_interface: ExternalInterface<'gc>,

    /// Manager of active sound instances.
    audio_manager: AudioManager<'gc>,

    /// List of actively playing streams to decode.
    stream_manager: StreamManager<'gc>,

    sockets: Sockets<'gc>,

    /// List of active NetConnection objects.
    net_connections: NetConnections<'gc>,

    local_connections: LocalConnections<'gc>,

    /// Dynamic root for allowing handles to GC objects to exist outside of the GC.
    dynamic_root: DynamicRootSet<'gc>,

    post_frame_callbacks: Vec<PostFrameCallback<'gc>>,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct PostFrameCallback<'gc> {
    #[collect(require_static)]
    #[allow(clippy::type_complexity)]
    pub callback: Box<dyn for<'b> FnOnce(&mut UpdateContext<'b>, DisplayObject<'b>) + 'static>,
    pub data: DisplayObject<'gc>,
}

impl<'gc> GcRootData<'gc> {
    /// Splits out parameters for creating an `UpdateContext`
    /// (because we can borrow fields of `self` independently)
    #[allow(clippy::type_complexity)]
    fn update_context_params(
        &mut self,
    ) -> (
        Stage<'gc>,
        &mut Library<'gc>,
        &mut ActionQueue<'gc>,
        &mut AvmStringInterner<'gc>,
        &mut Avm1<'gc>,
        &mut Avm2<'gc>,
        &mut Option<DragObject<'gc>>,
        &mut LoadManager<'gc>,
        &mut HashMap<String, Object<'gc>>,
        &mut HashMap<String, Avm2Object<'gc>>,
        &mut Vec<EditText<'gc>>,
        &mut Timers<'gc>,
        &mut Option<ContextMenuState<'gc>>,
        &mut ExternalInterface<'gc>,
        &mut AudioManager<'gc>,
        &mut StreamManager<'gc>,
        &mut Sockets<'gc>,
        &mut NetConnections<'gc>,
        &mut LocalConnections<'gc>,
        &mut Vec<PostFrameCallback<'gc>>,
        &mut MouseData<'gc>,
        DynamicRootSet<'gc>,
    ) {
        (
            self.stage,
            &mut self.library,
            &mut self.action_queue,
            &mut self.interner,
            &mut self.avm1,
            &mut self.avm2,
            &mut self.drag_object,
            &mut self.load_manager,
            &mut self.avm1_shared_objects,
            &mut self.avm2_shared_objects,
            &mut self.unbound_text_fields,
            &mut self.timers,
            &mut self.current_context_menu,
            &mut self.external_interface,
            &mut self.audio_manager,
            &mut self.stream_manager,
            &mut self.sockets,
            &mut self.net_connections,
            &mut self.local_connections,
            &mut self.post_frame_callbacks,
            &mut self.mouse_data,
            self.dynamic_root,
        )
    }
}

type GcArena = gc_arena::Arena<Rootable![GcRoot<'_>]>;

type Audio = Box<dyn AudioBackend>;
type Navigator = Box<dyn NavigatorBackend>;
type Renderer = Box<dyn RenderBackend>;
type Storage = Box<dyn StorageBackend>;
type Log = Box<dyn LogBackend>;
type Ui = Box<dyn UiBackend>;
type Video = Box<dyn VideoBackend>;

pub struct Player {
    /// The version of the player we're emulating.
    ///
    /// This serves a few purposes, primarily for compatibility:
    ///
    /// * ActionScript can query the player version, ostensibly for graceful
    ///   degradation on older platforms. Certain SWF files broke with the
    ///   release of Flash Player 10 because the version string contains two
    ///   digits. This allows the user to play those old files.
    /// * Player-specific behavior that was not properly versioned in Flash
    ///   Player can be enabled by setting a particular player version.
    player_version: u8,

    /// The runtime we're emulating (Flash Player or Adobe AIR).
    /// In Adobe AIR mode, additional classes are available
    #[allow(unused)]
    player_runtime: PlayerRuntime,

    swf: Arc<SwfMovie>,

    is_playing: bool,
    needs_render: bool,

    renderer: Renderer,
    audio: Audio,
    navigator: Navigator,
    storage: Storage,
    log: Log,
    ui: Ui,
    video: Video,

    transform_stack: TransformStack,

    rng: SmallRng,

    gc_arena: Rc<RefCell<GcArena>>,

    frame_rate: f64,
    forced_frame_rate: bool,
    actions_since_timeout_check: u16,

    frame_phase: FramePhase,

    stub_tracker: StubCollection,

    /// A time budget for executing frames.
    /// Gained by passage of time between host frames, spent by executing SWF frames.
    /// This is how we support custom SWF framerates
    /// and compensate for small lags by "catching up" (up to MAX_FRAMES_PER_TICK).
    frame_accumulator: f64,
    recent_run_frame_timings: VecDeque<f64>,

    /// Faked time passage for fooling hand-written busy-loop FPS limiters.
    time_offset: u32,

    input: InputManager,

    mouse_in_stage: bool,
    mouse_position: Point<Twips>,

    /// The current mouse cursor icon.
    mouse_cursor: MouseCursor,
    mouse_cursor_needs_check: bool,

    system: SystemProperties,

    page_url: Option<String>,

    /// The current instance ID. Used to generate default `instanceN` names.
    instance_counter: i32,

    /// Time remaining until the next timer will fire.
    time_til_next_timer: Option<f64>,

    /// The instant at which the SWF was launched.
    start_time: Instant,

    /// The maximum amount of time that can be called before a `Error::ExecutionTimeout`
    /// is raised. This defaults to 15 seconds but can be changed.
    max_execution_duration: Duration,

    /// Self-reference to ourselves.
    ///
    /// This is a weak reference that is upgraded and handed out in various
    /// contexts to other parts of the player. It can be used to ensure the
    /// player lives across `await` calls in async code.
    self_reference: Weak<Mutex<Self>>,

    /// The current frame of the main timeline, if available.
    /// The first frame is frame 1.
    current_frame: Option<u16>,

    /// How Ruffle should load movies.
    load_behavior: LoadBehavior,

    /// The root SWF URL provided to ActionScript. If None,
    /// the actual loaded url will be used
    spoofed_url: Option<String>,

    /// Any compatibility rules to apply for this movie.
    compatibility_rules: CompatibilityRules,

    /// Debug UI windows
    #[cfg(feature = "egui")]
    debug_ui: Rc<RefCell<crate::debug_ui::DebugUi>>,
}

impl Player {
    // This method will panic if called inside an `enter_arena_mut` call.
    fn enter_arena<F, T>(&self, f: F) -> T
    where
        F: for<'gc> FnOnce(&'gc Mutation<'gc>, &'gc GcRootData<'gc>, &'gc Self) -> T,
    {
        let borrow = self.gc_arena.try_borrow().ok();
        let result = borrow.and_then(|arena| {
            arena.mutate(|mc, root| {
                let root = root.data.try_borrow().ok()?;

                // SAFETY: The 'gc lifetime is generative, and can be soundly conflated with
                // the lifetime of shorter borrows, as `&'gc T`s aren't `Collect` and cannot
                // outlive the closure.
                Some(unsafe {
                    let root = &*(root.deref() as *const _);
                    let this = &*(self as *const _);
                    f(mc, root, this)
                })
            })
        });
        result.expect("arena already mutably borrowed")
    }

    // This method will panic if called inside another `enter_arena_mut` call.
    fn enter_arena_mut<F, T>(&mut self, f: F) -> T
    where
        F: for<'gc> FnOnce(&'gc Mutation<'gc>, &'gc mut GcRootData<'gc>, &'gc mut Self) -> T,
    {
        // To allow passing a `&mut Self` to the user-provided function, we avoid borrowing directly from self.
        let arena = Rc::clone(&self.gc_arena);

        // Do not borrow the arena mutably, to keep it accessible while inside a panic handler.
        let borrow = arena.try_borrow().ok();

        let result = borrow.and_then(|arena| {
            arena.mutate(|mc, root| {
                let mut root = root.data.try_borrow_mut(mc).ok()?;

                // SAFETY: The 'gc lifetime is generative, and can be soundly conflated with
                // the lifetime of shorter borrows, as `&'gc T`s aren't `Collect` and cannot
                // outlive the closure.
                Some(unsafe {
                    let root = &mut *(root.deref_mut() as *mut _);
                    let this = &mut *(self as *mut _);
                    f(mc, root, this)
                })
            })
        });
        result.expect("arena already borrowed")
    }

    /// Fetch the root movie.
    ///
    /// This should not be called if a root movie fetch has already been kicked
    /// off.
    ///
    /// `parameters` are *extra* parameters to set on the LoaderInfo -
    /// parameters from `movie_url` query parameters will be automatically added.
    pub fn fetch_root_movie(
        &mut self,
        movie_url: String,
        parameters: Vec<(String, String)>,
        on_metadata: Box<dyn FnOnce(&swf::HeaderExt)>,
    ) {
        self.mutate_with_update_context(|context| {
            let future = context.load_manager.load_root_movie(
                context.player.clone(),
                Request::get(movie_url),
                parameters,
                on_metadata,
            );
            context.navigator.spawn_future(future);
        });
    }

    /// Get rough estimate of the max # of times we can update the frame.
    ///
    /// In some cases, we might want to update several times in a row.
    /// For example, if the game runs at 60FPS, but the host runs at 30FPS
    /// Or if for some reason the we miss a couple of frames.
    /// However, if the code is simply slow, this is the opposite of what we want;
    /// If run_frame() consistently takes say 100ms, we don't want `tick` to try to "catch up",
    /// as this will only make it worse.
    ///
    /// This rough heuristic manages this job; for example if average run_frame()
    /// takes more than 1/3 of frame_time, we shouldn't run it more than twice in a row.
    /// This logic is far from perfect, as it doesn't take into account
    /// that things like rendering also take time. But for now it's good enough.
    fn max_frames_per_tick(&self) -> u32 {
        const MAX_FRAMES_PER_TICK: u32 = 5;

        if self.recent_run_frame_timings.is_empty() {
            5
        } else {
            let frame_time = 1000.0 / self.frame_rate;
            let average_run_frame_time = self.recent_run_frame_timings.iter().sum::<f64>()
                / self.recent_run_frame_timings.len() as f64;
            ((frame_time / average_run_frame_time) as u32).clamp(1, MAX_FRAMES_PER_TICK)
        }
    }

    fn add_frame_timing(&mut self, elapsed: f64) {
        self.recent_run_frame_timings.push_back(elapsed);
        if self.recent_run_frame_timings.len() >= 10 {
            self.recent_run_frame_timings.pop_front();
        }
    }

    pub fn tick(&mut self, dt: f64) {
        if self.is_playing() {
            self.frame_accumulator += dt;
            let frame_rate = self.frame_rate;
            let frame_time = 1000.0 / frame_rate;

            let max_frames_per_tick = self.max_frames_per_tick();
            let mut frame = 0;
            let mut last_time_offset = self.time_offset;

            while frame < max_frames_per_tick && self.frame_accumulator >= frame_time {
                let timer = Instant::now();
                self.run_frame();
                let elapsed = timer.elapsed().as_millis();
                if elapsed == 0 {
                    // To make sure that `getTimer()` never returns the same value between frames,
                    // pretend that at least 1ms elapsed if we ran the frame too fast.
                    self.time_offset += 1;
                }

                self.add_frame_timing(elapsed as f64);

                self.frame_accumulator -= frame_time;
                frame += 1;

                // We fooled the movie by pretending that more time has passed that actually did.
                // Then we need to actually pass this time, by decreasing frame_accumulator
                // to delay the future frame.
                let delta_time_offset = self.time_offset - last_time_offset;
                if delta_time_offset > 0 {
                    self.frame_accumulator -= delta_time_offset as f64;
                    last_time_offset = self.time_offset;
                }
            }

            // Now that we're done running code,
            // we can stop pretending that more time passed than actually did.
            // Note: update_timers(dt) doesn't need to see this either.
            // Timers will run at correct times and see correct time.
            // Also note that in Flash, a blocking busy loop would delay setTimeout
            // and cancel some setInterval callbacks, but here busy loops don't block
            // so timer callbacks won't get cancelled/delayed.
            self.time_offset = 0;

            // Sanity: If we had too many frames to tick, just reset the accumulator
            // to prevent running at turbo speed.
            if self.frame_accumulator >= frame_time {
                self.frame_accumulator = 0.0;
            }

            // Adjust playback speed for next frame to stay in sync with timeline audio tracks ("stream" sounds).
            let cur_frame_offset = self.frame_accumulator;
            self.frame_accumulator += self.mutate_with_update_context(|context| {
                context
                    .audio_manager
                    .audio_skew_time(context.audio, cur_frame_offset)
                    * 1000.0
            });

            self.update_sockets();
            self.update_net_connections();
            self.update_timers(dt);
            self.update(|context| {
                StreamManager::tick(context, dt);
            });
            self.audio.tick();
        }
    }
    pub fn time_til_next_timer(&self) -> Option<f64> {
        self.time_til_next_timer
    }

    /// Returns the approximate duration of time until the next frame is due to run.
    /// This is only an approximation to be used for sleep durations.
    pub fn time_til_next_frame(&self) -> std::time::Duration {
        let frame_time = 1000.0 / self.frame_rate;
        let mut dt = if self.frame_accumulator <= 0.0 {
            frame_time
        } else if self.frame_accumulator >= frame_time {
            0.0
        } else {
            frame_time - self.frame_accumulator
        };

        if let Some(time_til_next_timer) = self.time_til_next_timer {
            dt = dt.min(time_til_next_timer)
        }

        dt = dt.max(0.0);

        std::time::Duration::from_micros(dt as u64 * 1000)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn mouse_in_stage(&self) -> bool {
        self.mouse_in_stage
    }

    pub fn set_mouse_in_stage(&mut self, is_in: bool) {
        self.mouse_in_stage = is_in;
    }

    /// Returns the master volume of the player. 1.0 is 100% volume.
    ///
    /// The volume is linear and not adapted for logarithmic hearing.
    pub fn volume(&self) -> f32 {
        self.audio.volume()
    }

    /// Sets the master volume of the player. 1.0 is 100% volume.
    ///
    /// The volume should be linear and not adapted for logarithmic hearing.
    pub fn set_volume(&mut self, volume: f32) {
        self.audio.set_volume(volume)
    }

    pub fn prepare_context_menu(&mut self) -> Vec<ContextMenuItem> {
        self.mutate_with_update_context(|context| {
            if !context.stage.show_menu() {
                return vec![];
            }

            let display_obj = Player::get_context_menu_display_object(context);

            let menu = if let Some(Value::Object(obj)) = display_obj.map(|obj| obj.object()) {
                let mut activation =
                    Activation::from_stub(context, ActivationIdentifier::root("[ContextMenu]"));
                let menu_object = if let Ok(Value::Object(menu)) = obj.get("menu", &mut activation)
                {
                    if let Ok(Value::Object(on_select)) = menu.get("onSelect", &mut activation) {
                        Self::run_context_menu_custom_callback(menu, on_select, activation.context);
                    }
                    Some(menu)
                } else {
                    None
                };

                crate::avm1::make_context_menu_state(menu_object, display_obj, &mut activation)
            } else if let Some(Avm2Value::Object(hit_obj)) = display_obj.map(|obj| obj.object2()) {
                let mut activation = Avm2Activation::from_nothing(context);

                let menu_object = display_obj
                    .expect("Root is confirmed to exist here")
                    .as_interactive()
                    .map(|iobj| iobj.context_menu())
                    .and_then(|v| v.as_object());

                if let Some(menu_object) = menu_object {
                    // TODO: contextMenuOwner and mouseTarget might not be the same
                    let menu_evt = activation
                        .avm2()
                        .classes()
                        .contextmenuevent
                        .construct(
                            &mut activation,
                            &[
                                "menuSelect".into(),
                                false.into(),
                                false.into(),
                                hit_obj.into(),
                                hit_obj.into(),
                            ],
                        )
                        .expect("Context menu event should be constructed!");

                    Avm2::dispatch_event(activation.context, menu_evt, menu_object);
                }

                crate::avm2::make_context_menu_state(menu_object, display_obj, &mut activation)
            } else {
                // no AVM1 or AVM2 object - so just prepare the builtin items
                let mut menu = ContextMenuState::new();
                let builtin_items = BuiltInItemFlags::for_stage(context.stage);
                menu.build_builtin_items(builtin_items, context);
                menu
            };

            let ret = menu.info().clone();
            *context.current_context_menu = Some(menu);
            ret
        })
    }

    pub fn clear_custom_menu_items(&mut self) {
        self.enter_arena_mut(|_, gc_root, _| {
            gc_root.current_context_menu = None;
        });
    }

    pub fn run_context_menu_callback(&mut self, index: usize) {
        self.mutate_with_update_context(|context| {
            let menu = &context.current_context_menu;
            if let Some(ref menu) = menu {
                match menu.callback(index) {
                    ContextMenuCallback::Avm1 { item, callback } => {
                        Self::run_context_menu_custom_callback(*item, *callback, context)
                    }
                    ContextMenuCallback::Play => Self::toggle_play_root_movie(context),
                    ContextMenuCallback::Forward => Self::forward_root_movie(context),
                    ContextMenuCallback::Back => Self::back_root_movie(context),
                    ContextMenuCallback::Rewind => Self::rewind_root_movie(context),
                    ContextMenuCallback::Avm2 { item } => {
                        if let Some(display_obj) = menu.get_display_object() {
                            let menu_item = *item;
                            let mut activation = Avm2Activation::from_nothing(context);

                            let menu_obj = display_obj
                                .as_interactive()
                                .map(|iobj| iobj.context_menu())
                                .and_then(|v| v.as_object());

                            if menu_obj.is_some() {
                                // TODO: contextMenuOwner and mouseTarget might not be the same (see above comment)
                                let menu_evt = activation
                                    .avm2()
                                    .classes()
                                    .contextmenuevent
                                    .construct(
                                        &mut activation,
                                        &[
                                            "menuItemSelect".into(),
                                            false.into(),
                                            false.into(),
                                            display_obj.object2(),
                                            display_obj.object2(),
                                        ],
                                    )
                                    .expect("Context menu event should be constructed!");

                                Avm2::dispatch_event(context, menu_evt, menu_item);
                            }
                        }
                    }
                    ContextMenuCallback::QualityLow => {
                        context.stage.set_quality(context, StageQuality::Low)
                    }
                    ContextMenuCallback::QualityMedium => {
                        context.stage.set_quality(context, StageQuality::Medium)
                    }
                    ContextMenuCallback::QualityHigh => {
                        context.stage.set_quality(context, StageQuality::High)
                    }
                    ContextMenuCallback::TextControl { code, text } => {
                        text.text_control_input(*code, context)
                    }
                    _ => {}
                }
                Self::run_actions(context);
            }
        });
    }

    fn run_context_menu_custom_callback<'gc>(
        item: Object<'gc>,
        callback: Object<'gc>,
        context: &mut UpdateContext<'gc>,
    ) {
        if let Some(menu_state) = context.current_context_menu {
            if let Some(display_object) = menu_state.get_display_object() {
                let mut activation = Activation::from_nothing(
                    context,
                    ActivationIdentifier::root("[Context Menu Callback]"),
                    display_object,
                );

                let params = vec![display_object.object(), Value::Object(item)];

                let _ = callback.call(
                    "[Context Menu Callback]".into(),
                    &mut activation,
                    Value::Undefined,
                    &params,
                );
            }
        }
    }

    ///Returns the first display object that the mouse is hovering over that has a custom context menu. Returns root if none is found.
    fn get_context_menu_display_object<'gc>(
        context: &mut UpdateContext<'gc>,
    ) -> Option<DisplayObject<'gc>> {
        let mut picked_obj =
            run_mouse_pick(context, false).map(|picked_obj| picked_obj.as_displayobject());

        while let Some(display_obj) = picked_obj {
            if let Value::Object(obj) = display_obj.object() {
                let mut activation =
                    Activation::from_stub(context, ActivationIdentifier::root("[ContextMenu]"));

                if let Ok(Value::Object(_)) = obj.get("menu", &mut activation) {
                    return Some(display_obj);
                }
            }

            picked_obj = display_obj.parent();
        }

        context.stage.root_clip()
    }

    pub fn is_fullscreen(&mut self) -> bool {
        self.mutate_with_update_context(|context| {
            context.stage.display_state() != StageDisplayState::Normal
        })
    }

    pub fn set_fullscreen(&mut self, is_fullscreen: bool) {
        self.mutate_with_update_context(|context| {
            let display_state = if is_fullscreen {
                StageDisplayState::FullScreen
            } else {
                StageDisplayState::Normal
            };
            context.stage.set_display_state(context, display_state);
        });
    }

    fn toggle_play_root_movie(context: &mut UpdateContext<'_>) {
        if let Some(mc) = context
            .stage
            .root_clip()
            .and_then(|root| root.as_movie_clip())
        {
            if mc.playing() {
                mc.stop(context);
            } else {
                mc.play(context);
            }
        }
    }
    fn rewind_root_movie(context: &mut UpdateContext<'_>) {
        if let Some(mc) = context
            .stage
            .root_clip()
            .and_then(|root| root.as_movie_clip())
        {
            mc.goto_frame(context, 1, true)
        }
    }
    fn forward_root_movie(context: &mut UpdateContext<'_>) {
        if let Some(mc) = context
            .stage
            .root_clip()
            .and_then(|root| root.as_movie_clip())
        {
            mc.next_frame(context);
        }
    }
    fn back_root_movie(context: &mut UpdateContext<'_>) {
        if let Some(mc) = context
            .stage
            .root_clip()
            .and_then(|root| root.as_movie_clip())
        {
            mc.prev_frame(context);
        }
    }

    pub fn set_is_playing(&mut self, v: bool) {
        if v {
            // Allow auto-play after user gesture for web backends.
            self.audio.play();
        } else {
            self.audio.pause();
        }
        self.is_playing = v;
    }

    pub fn needs_render(&self) -> bool {
        self.needs_render
    }

    pub fn background_color(&mut self) -> Option<Color> {
        self.mutate_with_update_context(|context| context.stage.background_color())
    }

    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.mutate_with_update_context(|context| {
            context
                .stage
                .set_background_color(context.gc_context, color)
        })
    }

    pub fn letterbox(&mut self) -> Letterbox {
        self.mutate_with_update_context(|context| context.stage.letterbox())
    }

    pub fn set_letterbox(&mut self, letterbox: Letterbox) {
        self.mutate_with_update_context(|context| {
            context.stage.set_letterbox(context.gc_context, letterbox)
        })
    }

    pub fn movie_width(&mut self) -> u32 {
        self.mutate_with_update_context(|context| context.stage.movie_size().0)
    }

    pub fn movie_height(&mut self) -> u32 {
        self.mutate_with_update_context(|context| context.stage.movie_size().1)
    }

    pub fn viewport_dimensions(&mut self) -> ViewportDimensions {
        self.mutate_with_update_context(|context| context.renderer.viewport_dimensions())
    }

    pub fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions) {
        self.mutate_with_update_context(|context| {
            context.renderer.set_viewport_dimensions(dimensions);
            context.stage.build_matrices(context);
        })
    }

    pub fn set_show_menu(&mut self, show_menu: bool) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            stage.set_show_menu(context, show_menu);
        })
    }

    /// Set whether the Stage's display state can be changed.
    pub fn set_allow_fullscreen(&mut self, allow_fullscreen: bool) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            stage.set_allow_fullscreen(context, allow_fullscreen);
        })
    }

    pub fn quality(&mut self) -> StageQuality {
        self.mutate_with_update_context(|context| context.stage.quality())
    }

    pub fn set_quality(&mut self, quality: StageQuality) {
        self.mutate_with_update_context(|context| {
            context.stage.set_quality(context, quality);
        })
    }

    pub fn set_window_mode(&mut self, window_mode: &str) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            if let Ok(window_mode) = WindowMode::from_str(window_mode) {
                stage.set_window_mode(context, window_mode);
            }
        })
    }

    pub fn scale_mode(&mut self) -> StageScaleMode {
        self.mutate_with_update_context(|context| context.stage.scale_mode())
    }

    pub fn set_scale_mode(&mut self, scale_mode: StageScaleMode) {
        self.mutate_with_update_context(|context| {
            context.stage.set_scale_mode(context, scale_mode, false);
        })
    }

    pub fn forced_scale_mode(&mut self) -> bool {
        self.mutate_with_update_context(|context| context.stage.forced_scale_mode())
    }

    pub fn set_forced_scale_mode(&mut self, force: bool) {
        self.mutate_with_update_context(|context| {
            context.stage.set_forced_scale_mode(context, force);
        })
    }

    /// Handle an event sent into the player from the external windowing system
    /// or an HTML element.
    pub fn handle_event(&mut self, event: PlayerEvent) -> bool {
        match event {
            PlayerEvent::FocusGained | PlayerEvent::FocusLost => self.handle_focus_event(event),
            PlayerEvent::KeyDown { .. }
            | PlayerEvent::KeyUp { .. }
            | PlayerEvent::MouseMove { .. }
            | PlayerEvent::MouseUp { .. }
            | PlayerEvent::MouseDown { .. }
            | PlayerEvent::MouseLeave
            | PlayerEvent::MouseWheel { .. }
            | PlayerEvent::GamepadButtonDown { .. }
            | PlayerEvent::GamepadButtonUp { .. }
            | PlayerEvent::TextInput { .. }
            | PlayerEvent::TextControl { .. } => self.handle_input_event(event),
        }
    }

    fn handle_focus_event(&mut self, event: PlayerEvent) -> bool {
        if let PlayerEvent::FocusLost = event {
            self.mutate_with_update_context(|context| {
                let focus_tracker = context.focus_tracker;
                focus_tracker.reset_focus(context);
            });
        }

        true
    }

    /// Input event handling is a complicated affair, involving several different
    /// concerns that need to resolve with specific priority.
    ///
    /// 0. Transform gamepad button events into key events.
    /// 1. (In `avm_debug` builds)
    ///    If Ctrl-Alt-V is pressed, dump all AVM1 variables in the player.
    ///    If Ctrl-Alt-D is pressed, toggle debug output for AVM1 and AVM2.
    ///    If Ctrl-Alt-F is pressed, dump the display object tree.
    /// 2. If the incoming event is text input or key input that could be
    ///    related to text input (e.g. pressing a letter key), we dispatch a
    ///    key press event onto the stage.
    /// 3. If the event from step 3 was not handled, we check if an `EditText`
    ///    object is in focus and dispatch a text-control event to said object.
    /// 4. If the incoming event is text input, and neither step 3 nor step 4
    ///    resulted in an event being handled, we dispatch a text input event
    ///    to the currently focused `EditText` (if present).
    /// 5. Regardless of all prior event handling, we dispatch the event
    ///    through the stage normally.
    /// 6. Then, we dispatch the event through AVM1 global listener objects.
    /// 7. The AVM1 action queue is drained.
    /// 8. Mouse state is updated. This triggers button rollovers, which are a
    ///    second wave of event processing.
    fn handle_input_event(&mut self, event: PlayerEvent) -> bool {
        let mut player_event_handled = false;
        let Some(event) = self.input.map_input_event(event) else {
            return false;
        };

        let prev_mouse_buttons = self.input.get_mouse_down_buttons();
        self.input.handle_event(&event);
        let changed_mouse_buttons = self
            .input
            .get_mouse_down_buttons()
            .symmetric_difference(&prev_mouse_buttons)
            .cloned()
            .collect();

        if cfg!(feature = "avm_debug") {
            match event {
                PlayerEvent::KeyDown {
                    key_code: KeyCode::V,
                    ..
                } if self.input.is_key_down(KeyCode::CONTROL)
                    && self.input.is_key_down(KeyCode::ALT) =>
                {
                    self.mutate_with_update_context(|context| {
                        let mut dumper = VariableDumper::new("  ");

                        let mut activation = Activation::from_stub(
                            context,
                            ActivationIdentifier::root("[Variable Dumper]"),
                        );

                        dumper.print_variables(
                            "Global Variables:",
                            "_global",
                            &activation.context.avm1.global_object(),
                            &mut activation,
                        );

                        for display_object in activation.context.stage.iter_render_list() {
                            let level = display_object.depth();
                            let object = display_object.object().coerce_to_object(&mut activation);
                            dumper.print_variables(
                                &format!("Level #{level}:"),
                                &format!("_level{level}"),
                                &object,
                                &mut activation,
                            );
                        }
                        tracing::info!("Variable dump:\n{}", dumper.output());
                    });
                }
                PlayerEvent::KeyDown {
                    key_code: KeyCode::D,
                    ..
                } if self.input.is_key_down(KeyCode::CONTROL)
                    && self.input.is_key_down(KeyCode::ALT) =>
                {
                    self.mutate_with_update_context(|context| {
                        if context.avm1.show_debug_output() {
                            tracing::info!(
                                "AVM Debugging turned off! Press CTRL+ALT+D to turn on again."
                            );
                            context.avm1.set_show_debug_output(false);
                            context.avm2.set_show_debug_output(false);
                        } else {
                            tracing::info!(
                                "AVM Debugging turned on! Press CTRL+ALT+D to turn off."
                            );
                            context.avm1.set_show_debug_output(true);
                            context.avm2.set_show_debug_output(true);
                        }
                    });
                }
                PlayerEvent::KeyDown {
                    key_code: KeyCode::F,
                    ..
                } if self.input.is_key_down(KeyCode::CONTROL)
                    && self.input.is_key_down(KeyCode::ALT) =>
                {
                    self.mutate_with_update_context(|context| {
                        context.stage.display_render_tree(0);
                    });
                }
                _ => {}
            }
        }

        self.mutate_with_update_context(|context| {
            let button_event = ButtonKeyCode::from_player_event(event)
                .map(|key_code| ClipEvent::KeyPress { key_code });

            if let PlayerEvent::KeyDown { key_code, key_char }
            | PlayerEvent::KeyUp { key_code, key_char } = event
            {
                let ctrl_key = context.input.is_key_down(KeyCode::CONTROL);
                let alt_key = context.input.is_key_down(KeyCode::ALT);
                let shift_key = context.input.is_key_down(KeyCode::SHIFT);

                let mut activation = Avm2Activation::from_nothing(context);

                let event_name = match event {
                    PlayerEvent::KeyDown { .. } => "keyDown",
                    PlayerEvent::KeyUp { .. } => "keyUp",
                    _ => unreachable!(),
                };

                let keyboardevent_class = activation.avm2().classes().keyboardevent;
                let event_name_val: Avm2Value<'_> =
                    AvmString::new_utf8(activation.context.gc_context, event_name).into();

                // TODO: keyLocation should not be a dummy value.
                // ctrlKey and controlKey can be different from each other on Mac.
                // commandKey should be supported.
                let keyboard_event = keyboardevent_class
                    .construct(
                        &mut activation,
                        &[
                            event_name_val,                          /* type */
                            true.into(),                             /* bubbles */
                            false.into(),                            /* cancelable */
                            key_char.map_or(0, |c| c as u32).into(), /* charCode */
                            key_code.value().into(),                 /* keyCode */
                            0.into(),                                /* keyLocation */
                            ctrl_key.into(),                         /* ctrlKey */
                            alt_key.into(),                          /* altKey */
                            shift_key.into(),                        /* shiftKey */
                            ctrl_key.into(),                         /* controlKey */
                        ],
                    )
                    .expect("Failed to construct KeyboardEvent");
                let target_object = activation
                    .context
                    .focus_tracker
                    .get()
                    .map(|o| o.as_displayobject())
                    .unwrap_or_else(|| activation.context.stage.into());

                if target_object.movie().is_action_script_3() {
                    let target = target_object
                        .object2()
                        .coerce_to_object(&mut activation)
                        .expect("DisplayObject is not an object!");

                    Avm2::dispatch_event(activation.context, keyboard_event, target);
                }
            }

            // Propagate clip events.
            let (clip_event, listener) = match event {
                PlayerEvent::KeyDown { .. } => {
                    (Some(ClipEvent::KeyDown), Some(("Key", "onKeyDown", vec![])))
                }
                PlayerEvent::KeyUp { .. } => {
                    (Some(ClipEvent::KeyUp), Some(("Key", "onKeyUp", vec![])))
                }
                PlayerEvent::MouseMove { .. } => (
                    Some(ClipEvent::MouseMove),
                    Some(("Mouse", "onMouseMove", vec![])),
                ),
                PlayerEvent::MouseUp {
                    button: MouseButton::Left,
                    ..
                } => (
                    Some(ClipEvent::MouseUp),
                    Some(("Mouse", "onMouseUp", vec![])),
                ),
                PlayerEvent::MouseDown {
                    button: MouseButton::Left,
                    ..
                } => (
                    Some(ClipEvent::MouseDown),
                    Some(("Mouse", "onMouseDown", vec![])),
                ),
                PlayerEvent::MouseWheel { delta } => {
                    let delta = Value::from(delta.lines());
                    (None, Some(("Mouse", "onMouseWheel", vec![delta])))
                }
                PlayerEvent::MouseUp {
                    button: MouseButton::Right,
                    ..
                } => (Some(ClipEvent::RightMouseUp), None),
                PlayerEvent::MouseDown {
                    button: MouseButton::Right,
                    ..
                } => (Some(ClipEvent::RightMouseDown), None),
                PlayerEvent::MouseUp {
                    button: MouseButton::Middle,
                    ..
                } => (Some(ClipEvent::MiddleMouseUp), None),
                PlayerEvent::MouseDown {
                    button: MouseButton::Middle,
                    ..
                } => (Some(ClipEvent::MiddleMouseDown), None),
                _ => (None, None),
            };

            // Fire clip event on all clips.
            if let Some(clip_event) = clip_event {
                if context.stage.handle_clip_event(context, clip_event) == ClipEventResult::Handled
                {
                    player_event_handled = true;
                }
            }

            // Fire event listener on appropriate object
            if let Some((listener_type, event_name, args)) = listener {
                if let Some(root_clip) = context.stage.root_clip() {
                    context.action_queue.queue_action(
                        root_clip,
                        ActionType::NotifyListeners {
                            listener: listener_type,
                            method: event_name,
                            args,
                        },
                        false,
                    );
                }
            }

            // Propagate button events.
            // It has to be done after propagating the clip event,
            // so that KeyPress is always fired after KeyDown.
            let key_press_handled = if let Some(button_event) = button_event {
                context.stage.handle_clip_event(context, button_event) == ClipEventResult::Handled
            } else {
                false
            };

            if key_press_handled {
                player_event_handled = true;
            }

            // KeyPress events take precedence over text input.
            if !key_press_handled {
                if let Some(text) = context.focus_tracker.get_as_edit_text() {
                    if let PlayerEvent::TextInput { codepoint } = event {
                        text.text_input(codepoint, context);
                    }
                    if let PlayerEvent::TextControl { code } = event {
                        text.text_control_input(code, context);
                    }
                }
            }

            // KeyPress events also take precedence over tabbing.
            if !key_press_handled {
                if let PlayerEvent::KeyDown {
                    key_code: KeyCode::TAB,
                    ..
                } = event
                {
                    let reversed = context.input.is_key_down(KeyCode::SHIFT);
                    let tracker = context.focus_tracker;
                    tracker.cycle(context, reversed);
                }
            }

            // KeyPress events also take precedence over keyboard navigation.
            // Note that keyboard navigation works only when the highlight is visible.
            if !key_press_handled && context.focus_tracker.highlight().is_visible() {
                if let Some(focus) = context.focus_tracker.get() {
                    if matches!(
                        event,
                        PlayerEvent::KeyDown {
                            key_code: KeyCode::RETURN,
                            ..
                        } | PlayerEvent::TextInput { codepoint: ' ' }
                    ) {
                        // The button/clip is pressed and then immediately released.
                        // We do not have to wait for KeyUp.
                        focus.handle_clip_event(context, ClipEvent::Press { index: 0 });
                        focus.handle_clip_event(context, ClipEvent::Release { index: 0 });
                    }

                    if let PlayerEvent::KeyDown { key_code, .. } = event {
                        if let Some(direction) = NavigationDirection::from_key_code(key_code) {
                            let tracker = context.focus_tracker;
                            tracker.navigate(context, direction);
                        }
                    }
                }
            }

            Self::run_actions(context);
        });

        // Update mouse state.
        if let PlayerEvent::MouseMove { x, y }
        | PlayerEvent::MouseDown { x, y, .. }
        | PlayerEvent::MouseUp { x, y, .. } = event
        {
            let inverse_view_matrix =
                self.mutate_with_update_context(|context| context.stage.inverse_view_matrix());
            let prev_mouse_position = self.mouse_position;
            self.mouse_position = inverse_view_matrix * Point::from_pixels(x, y);

            // Update the dragged object here to keep it constantly in sync with the mouse position.
            self.mutate_with_update_context(|context| {
                Self::update_drag(context);
            });

            let is_mouse_moved = prev_mouse_position != self.mouse_position;

            // This fires button rollover/press events, which should run after the above mouseMove events.
            if self.update_mouse_state(
                &changed_mouse_buttons,
                is_mouse_moved,
                &mut player_event_handled,
            ) {
                self.needs_render = true;
            }
        }

        if let PlayerEvent::MouseWheel { delta } = event {
            self.mutate_with_update_context(|context| {
                let target = if let Some(over_object) = context.mouse_data.hovered {
                    if over_object.as_displayobject().movie().is_action_script_3()
                        || !over_object.as_displayobject().avm1_removed()
                    {
                        Some(over_object)
                    } else {
                        None
                    }
                } else {
                    context.stage.as_interactive()
                };
                if let Some(target) = target {
                    let event = ClipEvent::MouseWheel { delta };
                    target.event_dispatch_to_avm2(context, event);
                    target.handle_clip_event(context, event);
                }
            });
        }

        if let PlayerEvent::MouseLeave = event {
            if self.update_mouse_state(&changed_mouse_buttons, true, &mut player_event_handled) {
                self.needs_render = true;
            }
        }

        if self.should_reset_highlight(event) {
            self.mutate_with_update_context(|context| {
                context.focus_tracker.reset_highlight();
            });
        }

        player_event_handled
    }

    fn should_reset_highlight(&self, event: PlayerEvent) -> bool {
        if matches!(
            event,
            PlayerEvent::MouseDown {
                button: MouseButton::Left,
                ..
            }
        ) {
            // Left mouse button down always resets the highlight.
            return true;
        }

        if self.swf.version() < 9
            && matches!(
                event,
                PlayerEvent::MouseDown {
                    button: MouseButton::Left | MouseButton::Right,
                    ..
                } | PlayerEvent::MouseUp {
                    button: MouseButton::Left | MouseButton::Right,
                    ..
                } | PlayerEvent::MouseMove { .. }
            )
        {
            // For SWF8 and older, other mouse events also reset the highlight.
            return true;
        }

        false
    }

    /// Update dragged object, if any.
    pub fn update_drag(context: &mut UpdateContext<'_>) {
        let mouse_position = *context.mouse_position;
        if let Some(drag_object) = context.drag_object {
            let display_object = drag_object.display_object;
            if !display_object.movie().is_action_script_3() && display_object.avm1_removed() {
                // Be sure to clear the drag if the object was removed.
                *context.drag_object = None;
                return;
            }

            let local_to_global_matrix = match display_object.parent() {
                Some(parent) => parent.local_to_global_matrix(),
                None => Matrix::IDENTITY,
            };
            let global_to_local_matrix = local_to_global_matrix.inverse().unwrap_or_default();

            let new_position = if drag_object.lock_center {
                let new_position = global_to_local_matrix * mouse_position;
                drag_object.constraint.clamp(new_position)
            } else {
                // TODO: Introduce `DisplayObject::position()`?
                let position = Point::new(display_object.x(), display_object.y());
                let mouse_delta = mouse_position - drag_object.last_mouse_position;
                let new_position = position + global_to_local_matrix * mouse_delta;
                let new_position = drag_object.constraint.clamp(new_position);

                let mouse_delta = local_to_global_matrix * (new_position - position);
                drag_object.last_mouse_position += mouse_delta;

                new_position
            };

            // TODO: Introduce `DisplayObject::set_position()`?
            display_object.set_x(context.gc_context, new_position.x);
            display_object.set_y(context.gc_context, new_position.y);

            // Update `_droptarget` property of dragged object.
            if let Some(movie_clip) = display_object.as_movie_clip() {
                // Turn the dragged object invisible so that we don't pick it.
                // TODO: This could be handled via adding a `HitTestOptions::SKIP_DRAGGED`.
                let was_visible = display_object.visible();
                display_object.set_visible(context, false);
                // Set `_droptarget` to the object the mouse is hovering over.
                let drop_target_object = run_mouse_pick(context, false);
                movie_clip.set_drop_target(
                    context.gc_context,
                    drop_target_object.map(|d| d.as_displayobject()),
                );
                display_object.set_visible(context, was_visible);
            }
        }
    }

    /// Updates the hover state of buttons.
    fn update_mouse_state(
        &mut self,
        changed_mouse_buttons: &HashSet<MouseButton>,
        is_mouse_moved: bool,
        player_event_handled: &mut bool,
    ) -> bool {
        let mut new_cursor = self.mouse_cursor;
        let mut mouse_cursor_needs_check = self.mouse_cursor_needs_check;
        let mouse_in_stage = self.mouse_in_stage();

        // Determine the display object the mouse is hovering over.
        // Search through levels from top-to-bottom, returning the first display object that is under the mouse.
        let needs_render = self.mutate_with_update_context(|context| {
            // Objects may be hovered using Tab,
            // skip mouse hover when it's not necessary.
            let mut skip_mouse_hover = !is_mouse_moved
                && changed_mouse_buttons.is_empty()
                && context.mouse_data.hovered.is_some();

            let new_over_object = if mouse_in_stage {
                run_mouse_pick(context, true)
            } else {
                None
            };
            let mut events: smallvec::SmallVec<[(InteractiveObject<'_>, ClipEvent); 2]> =
                Default::default();

            if is_mouse_moved {
                events.push((
                    new_over_object.unwrap_or_else(|| context.stage.into()),
                    ClipEvent::MouseMoveInside,
                ));
            }

            let mut new_over_object_updated = false;
            if let Some(hovered) = context.mouse_data.hovered {
                // Cancel hover if an object is removed from the stage.
                if !hovered.as_displayobject().movie().is_action_script_3()
                    && hovered.as_displayobject().avm1_removed()
                {
                    context.mouse_data.hovered = None;
                    if let Some(new_object) = new_over_object {
                        if Self::check_display_object_equality(
                            new_object.as_displayobject(),
                            hovered.as_displayobject(),
                        ) {
                            if let Some(state) = hovered.as_displayobject().state() {
                                new_object.as_displayobject().set_state(context, state);
                            }
                            context.mouse_data.hovered = Some(new_object);
                            new_over_object_updated = true;
                        }
                    }
                }

                // Ensure that hover is canceled when an object disappears,
                // even if the mouse was idle.
                if !hovered.as_displayobject().visible() {
                    skip_mouse_hover = false;
                }
            }

            if let Some(pressed) = context.mouse_data.pressed {
                if !pressed.as_displayobject().movie().is_action_script_3()
                    && pressed.as_displayobject().avm1_removed()
                {
                    context.mouse_data.pressed = None;
                    let mut display_object = None;
                    if let Some(root_clip) = context.stage.root_clip() {
                        display_object = Self::find_first_character_instance(
                            root_clip,
                            pressed.as_displayobject(),
                        );
                    }

                    if let Some(new_down_object) = display_object {
                        if let Some(state) = pressed.as_displayobject().state() {
                            new_down_object.set_state(context, state);
                        }

                        context.mouse_data.pressed = new_down_object.as_interactive();
                    }
                }
            }

            // Update the cursor if the object was removed from the stage.
            if new_cursor != MouseCursor::Arrow {
                let object_removed =
                    context.mouse_data.hovered.is_none() && context.mouse_data.pressed.is_none();
                if !object_removed {
                    mouse_cursor_needs_check = false;
                    if changed_mouse_buttons.contains(&MouseButton::Left) {
                        // The object is pressed/released and may be removed immediately, we need to check
                        // in the next frame if it still exists. If it doesn't, we'll update the cursor.
                        mouse_cursor_needs_check = true;
                    }
                } else if mouse_cursor_needs_check {
                    mouse_cursor_needs_check = false;
                    new_cursor = MouseCursor::Arrow;
                } else if !context.input.is_mouse_down(MouseButton::Left)
                    && (is_mouse_moved || changed_mouse_buttons.contains(&MouseButton::Left))
                {
                    // In every other case, the cursor remains until the user interacts with the mouse again.
                    new_cursor = MouseCursor::Arrow;
                }
            } else {
                mouse_cursor_needs_check = false;
            }

            let cur_over_object = context.mouse_data.hovered;
            // Check if a new object has been hovered over.
            if !skip_mouse_hover
                && !InteractiveObject::option_ptr_eq(cur_over_object, new_over_object)
            {
                // If the mouse button is down, the object the user clicked on grabs the focus
                // and fires "drag" events. Other objects are ignored.
                if context.input.is_mouse_down(MouseButton::Left) {
                    context.mouse_data.hovered = new_over_object;
                    if let Some(down_object) = context.mouse_data.pressed {
                        if InteractiveObject::option_ptr_eq(
                            context.mouse_data.pressed,
                            cur_over_object,
                        ) {
                            // Dragged from outside the clicked object to the inside.
                            events.push((
                                down_object,
                                ClipEvent::DragOut {
                                    to: new_over_object,
                                },
                            ));
                        } else if InteractiveObject::option_ptr_eq(
                            context.mouse_data.pressed,
                            new_over_object,
                        ) {
                            // Dragged from inside the clicked object to the outside.
                            events.push((
                                down_object,
                                ClipEvent::DragOver {
                                    from: cur_over_object,
                                },
                            ));
                        }
                    }
                } else {
                    // The mouse button is up, so fire rollover states for the object we are hovering over.
                    // Rolled out of the previous object.
                    if let Some(cur_over_object) = cur_over_object {
                        events.push((
                            cur_over_object,
                            ClipEvent::RollOut {
                                to: new_over_object,
                            },
                        ));
                    }
                    // Rolled over the new object.
                    if let Some(new_over_object) = new_over_object {
                        new_cursor = new_over_object.mouse_cursor(context);
                        events.push((
                            new_over_object,
                            ClipEvent::RollOver {
                                from: cur_over_object,
                            },
                        ));
                    } else {
                        new_cursor = MouseCursor::Arrow;
                    }
                }
            }
            if !skip_mouse_hover && !new_over_object_updated {
                context.mouse_data.hovered = new_over_object;
            }
            // Handle presses and releases.
            for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
                if !changed_mouse_buttons.contains(&button) {
                    continue;
                }

                if context.input.is_mouse_down(button) {
                    let event = match button {
                        MouseButton::Left => ClipEvent::Press {
                            index: context.input.last_click_index(),
                        },
                        MouseButton::Right => ClipEvent::RightPress,
                        MouseButton::Middle => ClipEvent::MiddlePress,
                        _ => unreachable!(),
                    };
                    // Pressed on a hovered object.
                    if let Some(over_object) = context.mouse_data.hovered {
                        events.push((over_object, event));
                        context
                            .mouse_data
                            .set_pressed(button, context.mouse_data.hovered);
                    } else {
                        events.push((context.stage.into(), event));
                    }
                } else {
                    let event = match button {
                        MouseButton::Left => ClipEvent::MouseUpInside,
                        MouseButton::Right => ClipEvent::RightMouseUpInside,
                        MouseButton::Middle => ClipEvent::MiddleMouseUpInside,
                        _ => unreachable!(),
                    };
                    if let Some(over_object) = context.mouse_data.hovered {
                        events.push((over_object, event));
                    } else {
                        events.push((context.stage.into(), event));
                    }

                    let mut released_inside = InteractiveObject::option_ptr_eq(
                        context.mouse_data.pressed(button),
                        context.mouse_data.hovered,
                    );
                    if let Some(down) = context.mouse_data.pressed(button) {
                        if let Some(over) = context.mouse_data.hovered {
                            if !released_inside {
                                released_inside = Self::check_display_object_equality(
                                    down.as_displayobject(),
                                    over.as_displayobject(),
                                );
                            }
                        }
                    }
                    if released_inside {
                        let event = match button {
                            MouseButton::Left => ClipEvent::Release {
                                index: context.input.last_click_index(),
                            },
                            MouseButton::Right => ClipEvent::RightRelease,
                            MouseButton::Middle => ClipEvent::MiddleRelease,
                            _ => unreachable!(),
                        };
                        // Released inside the clicked object.
                        if let Some(down_object) = context.mouse_data.pressed(button) {
                            if button == MouseButton::Left {
                                new_cursor = down_object.mouse_cursor(context);
                            }
                            events.push((down_object, event));
                        } else {
                            events.push((context.stage.into(), event));
                        }
                    } else {
                        let event = match button {
                            MouseButton::Left => ClipEvent::ReleaseOutside,
                            MouseButton::Right => ClipEvent::RightReleaseOutside,
                            MouseButton::Middle => ClipEvent::MiddleReleaseOutside,
                            _ => unreachable!(),
                        };
                        // Released outside the clicked object.
                        if let Some(down_object) = context.mouse_data.pressed(button) {
                            events.push((down_object, event));
                        } else {
                            events.push((context.stage.into(), event));
                        }

                        if button == MouseButton::Left {
                            // The new object is rolled over immediately.
                            if let Some(over_object) = context.mouse_data.hovered {
                                new_cursor = over_object.mouse_cursor(context);
                                events.push((
                                    over_object,
                                    ClipEvent::RollOver {
                                        from: cur_over_object,
                                    },
                                ));
                            } else {
                                new_cursor = MouseCursor::Arrow;
                            }
                        }
                    }
                    context.mouse_data.set_pressed(button, None);
                }
            }

            // Fire any pending mouse events.
            let needs_render = if events.is_empty() {
                false
            } else {
                let mut refresh = false;
                for (object, event) in events {
                    let display_object = object.as_displayobject();
                    if !display_object.avm1_removed() {
                        if object.handle_clip_event(context, event) == ClipEventResult::Handled {
                            *player_event_handled = true;
                        }
                        if matches!(event, ClipEvent::Press { .. }) {
                            Self::update_focus_on_mouse_press(context, display_object);
                        }
                        if object.event_dispatch_to_avm2(context, event) == ClipEventResult::Handled
                        {
                            *player_event_handled = true;
                        }
                    }
                    if !refresh && event.is_button_event() {
                        let is_button_mode = display_object.as_avm1_button().is_some()
                            || display_object.as_avm2_button().is_some()
                            || display_object
                                .as_movie_clip()
                                .map(|mc| mc.is_button_mode(context))
                                .unwrap_or_default();
                        if is_button_mode {
                            refresh = true;
                        }
                    }
                }
                refresh
            };
            Self::run_actions(context);
            needs_render
        });

        // Update mouse cursor if it has changed.
        if new_cursor != self.mouse_cursor {
            self.mouse_cursor = new_cursor;
            self.ui.set_mouse_cursor(new_cursor)
        }
        self.mouse_cursor_needs_check = mouse_cursor_needs_check;

        needs_render
    }

    fn update_focus_on_mouse_press<'gc>(
        context: &mut UpdateContext<'gc>,
        pressed_object: DisplayObject<'gc>,
    ) {
        let tracker = context.focus_tracker;
        let mut pressed_object = pressed_object.as_interactive();
        if InteractiveObject::option_ptr_eq(pressed_object, context.stage.as_interactive()) {
            pressed_object = None;
        }

        let should_focus = pressed_object.is_some_and(|int| {
            // AVM2 fires focus change events even if the related object is not focusable
            int.as_displayobject().movie().is_action_script_3()
                || int.is_focusable_by_mouse(context)
        });
        if should_focus {
            tracker.set_by_mouse(pressed_object, context);
        } else if tracker
            .get()
            .is_some_and(|int| int.is_focusable_by_mouse(context))
        {
            // Need to clear the focus if an object focusable by mouse was un-focused.
            tracker.set_by_mouse(None, context);
        }
    }

    //Checks if two displayObjects have the same depth and id and accur in the same movie.s
    fn check_display_object_equality(object1: DisplayObject, object2: DisplayObject) -> bool {
        object1.depth() == object2.depth()
            && object1.id() == object2.id()
            && Arc::ptr_eq(&object1.movie(), &object2.movie())
    }
    ///This searches for a display object by it's id.
    ///When a button is being held down but the mouse stops hovering over the object
    ///we need to know if th button is still there after goto.
    //TODO: is there a better place to place next two functions?
    fn find_first_character_instance<'gc>(
        obj: DisplayObject<'gc>,
        previous_object: DisplayObject<'gc>,
    ) -> Option<DisplayObject<'gc>> {
        if let Some(parent) = obj.as_container() {
            for child in parent.iter_render_list() {
                if Self::check_display_object_equality(child, previous_object) {
                    return Some(child);
                }

                let display_object = Self::find_first_character_instance(child, previous_object);
                if display_object.is_some() {
                    return display_object;
                }
            }
        }
        None
    }

    /// Preload all pending movies in the player, including the root movie.
    ///
    /// This should be called periodically with a reasonable execution limit.
    /// By default, the Player will do so after every `run_frame` using a limit
    /// derived from the current frame rate and execution time. Clients that
    /// want synchronous or 'lockstep' preloading may call this function with
    /// an unlimited execution limit.
    ///
    /// Returns true if all preloading work has completed. Clients that want to
    /// simulate a particular load condition or stress chunked loading may use
    /// this in lieu of an unlimited execution limit.
    pub fn preload(&mut self, limit: &mut ExecutionLimit) -> bool {
        self.mutate_with_update_context(|context| {
            let mut did_finish = true;

            if let Some(root) = context
                .stage
                .root_clip()
                .and_then(|root| root.as_movie_clip())
            {
                let was_root_movie_loaded = root.loaded_bytes() as i32 == root.total_bytes();
                did_finish = root.preload(context, limit);

                if let Some(loader_info) = root.loader_info().filter(|_| !was_root_movie_loaded) {
                    let mut activation = Avm2Activation::from_nothing(context);

                    let progress_evt = activation.avm2().classes().progressevent.construct(
                        &mut activation,
                        &[
                            "progress".into(),
                            false.into(),
                            false.into(),
                            root.compressed_loaded_bytes().into(),
                            root.compressed_total_bytes().into(),
                        ],
                    );

                    match progress_evt {
                        Err(e) => tracing::error!(
                            "Encountered AVM2 error when constructing `progress` event: {}",
                            e,
                        ),
                        Ok(progress_evt) => {
                            Avm2::dispatch_event(context, progress_evt, loader_info);
                        }
                    }
                }
            }

            if did_finish {
                did_finish = LoadManager::preload_tick(context, limit);
            }

            Self::run_actions(context);

            did_finish
        })
    }

    #[instrument(level = "debug", skip_all)]
    pub fn run_frame(&mut self) {
        let frame_time = Duration::from_nanos((750_000_000.0 / self.frame_rate) as u64);
        let (mut execution_limit, may_execute_while_streaming) = match self.load_behavior {
            LoadBehavior::Streaming => (
                ExecutionLimit::with_max_ops_and_time(10000, frame_time),
                true,
            ),
            LoadBehavior::Delayed => (
                ExecutionLimit::with_max_ops_and_time(10000, frame_time),
                false,
            ),
            LoadBehavior::Blocking => (ExecutionLimit::none(), false),
        };
        let preload_finished = self.preload(&mut execution_limit);

        if !preload_finished && !may_execute_while_streaming {
            return;
        }

        self.update(|context| {
            // TODO: Is this order correct?
            run_all_phases_avm2(context);
            Avm1::run_frame(context);
            AudioManager::update_sounds(context);
            LocalConnections::update_connections(context);

            // Only run the current list of callbacks - any callbacks added during callback execution
            // will be run at the end of the *next* frame.
            for cb in std::mem::take(context.post_frame_callbacks) {
                (cb.callback)(context, cb.data);
            }
        });

        self.needs_render = true;
    }

    #[instrument(level = "debug", skip_all)]
    pub fn render(&mut self) {
        let invalidated = self.enter_arena(|_, gc_root, _| gc_root.stage.invalidated());

        if invalidated {
            self.update(|context| {
                let stage = context.stage;
                stage.broadcast_render(context);
            });
        }

        let mut background_color = Color::WHITE;

        let (cache_draws, commands) = self.enter_arena_mut(|gc_context, gc_root, this| {
            let stage = gc_root.stage;

            let mut cache_draws = vec![];
            let mut render_context = RenderContext {
                renderer: this.renderer.deref_mut(),
                commands: CommandList::new(),
                cache_draws: &mut cache_draws,
                gc_context,
                library: &gc_root.library,
                transform_stack: &mut this.transform_stack,
                is_offscreen: false,
                use_bitmap_cache: true,
                stage,
            };

            stage.render(&mut render_context);

            #[cfg(feature = "egui")]
            {
                this.debug_ui
                    .borrow_mut()
                    .draw_debug_rects(&mut render_context, gc_root.dynamic_root);
            }

            background_color =
                if stage.window_mode() != WindowMode::Transparent || stage.is_fullscreen() {
                    stage.background_color().unwrap_or(Color::WHITE)
                } else {
                    Color::from_rgba(0)
                };

            let commands = render_context.commands;
            (cache_draws, commands)
        });

        self.renderer
            .submit_frame(background_color, commands, cache_draws);

        self.needs_render = false;
    }

    /// The current frame of the main timeline, if available.
    /// The first frame is frame 1.
    pub fn current_frame(&self) -> Option<u16> {
        self.current_frame
    }

    pub fn audio(&self) -> &Audio {
        &self.audio
    }

    pub fn audio_mut(&mut self) -> &mut Audio {
        &mut self.audio
    }

    pub fn navigator(&self) -> &Navigator {
        &self.navigator
    }

    pub fn navigator_mut(&mut self) -> &mut Navigator {
        &mut self.navigator
    }

    // The frame rate of the current movie in FPS.
    pub fn frame_rate(&self) -> f64 {
        self.frame_rate
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }

    pub fn destroy(self) -> Renderer {
        self.renderer
    }

    pub fn ui(&self) -> &Ui {
        &self.ui
    }

    pub fn ui_mut(&mut self) -> &mut Ui {
        &mut self.ui
    }

    pub fn run_actions(context: &mut UpdateContext<'_>) {
        // Note that actions can queue further actions, so a while loop is necessary here.
        while let Some(action) = context.action_queue.pop_action() {
            // We don't run frame actions if the clip was removed (or scheduled to be removed) after it queued the action.
            if !action.is_unload
                && (!action.clip.movie().is_action_script_3()
                    && (action.clip.avm1_removed() || action.clip.avm1_pending_removal()))
            {
                continue;
            }

            match action.action_type {
                // DoAction/clip event code.
                ActionType::Normal { bytecode } | ActionType::Initialize { bytecode } => {
                    Avm1::run_stack_frame_for_action(action.clip, "[Frame]", bytecode, context);
                }
                // Change the prototype of a MovieClip and run constructor events.
                ActionType::Construct {
                    constructor: Some(constructor),
                    events,
                } => {
                    let mut activation = Activation::from_nothing(
                        context,
                        ActivationIdentifier::root("[Construct]"),
                        action.clip,
                    );
                    if let Ok(prototype) = constructor.get("prototype", &mut activation) {
                        if let Value::Object(object) = action.clip.object() {
                            object.define_value(
                                activation.context.gc_context,
                                "__proto__",
                                prototype,
                                Attribute::DONT_ENUM | Attribute::DONT_DELETE,
                            );
                            for event in events {
                                let _ = activation.run_child_frame_for_action(
                                    "[Actions]",
                                    action.clip,
                                    event,
                                );
                            }

                            let _ = constructor.construct_on_existing(&mut activation, object, &[]);
                        }
                    }
                }
                // Run constructor events without changing the prototype.
                ActionType::Construct {
                    constructor: None,
                    events,
                } => {
                    for event in events {
                        Avm1::run_stack_frame_for_action(
                            action.clip,
                            "[Construct]",
                            event,
                            context,
                        );
                    }
                }
                // Event handler method call (e.g. onEnterFrame).
                ActionType::Method { object, name, args } => {
                    Avm1::run_stack_frame_for_method(
                        action.clip,
                        object,
                        context,
                        name.into(),
                        &args,
                    );
                }

                // Event handler method call (e.g. onEnterFrame).
                ActionType::NotifyListeners {
                    listener,
                    method,
                    args,
                } => {
                    // A native function ends up resolving immediately,
                    // so this doesn't require any further execution.
                    Avm1::notify_system_listeners(
                        action.clip,
                        context,
                        listener.into(),
                        method.into(),
                        &args,
                    );
                }
            }

            // AVM1 bytecode may leave the stack unbalanced, so do not let garbage values accumulate
            // across multiple executions and/or frames.
            context.avm1.clear_stack();
        }
    }

    /// Runs the closure `f` with an `UpdateContext`.
    /// This takes cares of populating the `UpdateContext` struct, avoiding borrow issues.
    pub fn mutate_with_update_context<F, R>(&mut self, f: F) -> R
    where
        F: for<'a, 'gc> FnOnce(&mut UpdateContext<'gc>) -> R,
    {
        self.enter_arena_mut(|gc_context, gc_root, this| {
            #[allow(unused_variables)]
            let (
                stage,
                library,
                action_queue,
                interner,
                avm1,
                avm2,
                drag_object,
                load_manager,
                avm1_shared_objects,
                avm2_shared_objects,
                unbound_text_fields,
                timers,
                current_context_menu,
                external_interface,
                audio_manager,
                stream_manager,
                sockets,
                net_connections,
                local_connections,
                post_frame_callbacks,
                mouse_data,
                dynamic_root,
            ) = gc_root.update_context_params();

            let mut update_context = UpdateContext {
                player_version: this.player_version,
                swf: &mut this.swf,
                library,
                rng: &mut this.rng,
                renderer: this.renderer.deref_mut(),
                audio: this.audio.deref_mut(),
                navigator: this.navigator.deref_mut(),
                ui: this.ui.deref_mut(),
                action_queue,
                gc_context,
                strings: StringContext::from_parts(gc_context, interner),
                stage,
                mouse_data,
                input: &this.input,
                mouse_position: &this.mouse_position,
                drag_object,
                player: this.self_reference.clone(),
                load_manager,
                system: &mut this.system,
                page_url: &mut this.page_url,
                instance_counter: &mut this.instance_counter,
                storage: this.storage.deref_mut(),
                log: this.log.deref_mut(),
                video: this.video.deref_mut(),
                avm1_shared_objects,
                avm2_shared_objects,
                unbound_text_fields,
                timers,
                current_context_menu,
                needs_render: &mut this.needs_render,
                avm1,
                avm2,
                external_interface,
                start_time: this.start_time,
                update_start: Instant::now(),
                max_execution_duration: this.max_execution_duration,
                focus_tracker: stage.focus_tracker(),
                times_get_time_called: 0,
                time_offset: &mut this.time_offset,
                audio_manager,
                frame_rate: &mut this.frame_rate,
                forced_frame_rate: this.forced_frame_rate,
                actions_since_timeout_check: &mut this.actions_since_timeout_check,
                frame_phase: &mut this.frame_phase,
                stub_tracker: &mut this.stub_tracker,
                stream_manager,
                sockets,
                net_connections,
                local_connections,
                dynamic_root,
                post_frame_callbacks,
            };

            let prev_frame_rate = *update_context.frame_rate;

            let ret = f(&mut update_context);

            // If we changed the framerate, let the audio handler now.
            #[allow(clippy::float_cmp)]
            if *update_context.frame_rate != prev_frame_rate {
                update_context
                    .audio
                    .set_frame_rate(*update_context.frame_rate);
            }

            this.current_frame = update_context
                .stage
                .root_clip()
                .and_then(|root| root.as_movie_clip())
                .map(|clip| clip.current_frame());
            ret
        })
    }

    #[cfg(feature = "egui")]
    pub fn show_debug_ui(&mut self, egui_ctx: &egui::Context, movie_offset: f64) {
        // To allow using `mutate_with_update_context` and passing the context inside the debug ui,
        // we avoid borrowing directly from self here.
        // This method should only be called once and it will panic if it tries to recursively render.
        let debug_ui = self.debug_ui.clone();
        let mut debug_ui = debug_ui.borrow_mut();
        self.mutate_with_update_context(|context| {
            debug_ui.show(egui_ctx, context, movie_offset);
        });
    }

    #[cfg(feature = "egui")]
    pub fn debug_ui(&mut self) -> core::cell::RefMut<'_, crate::debug_ui::DebugUi> {
        self.debug_ui.borrow_mut()
    }

    /// Update the current state of the player.
    ///
    /// The given function will be called with the current stage root, current
    /// mouse hover node, AVM, and an update context.
    ///
    /// This particular function runs necessary post-update bookkeeping, such
    /// as executing any actions queued on the update context, keeping the
    /// hover state up to date, and running garbage collection.
    pub fn update<F, R>(&mut self, func: F) -> R
    where
        F: for<'a, 'gc> FnOnce(&mut UpdateContext<'gc>) -> R,
    {
        let rval = self.mutate_with_update_context(|context| {
            let rval = func(context);

            Self::run_actions(context);

            rval
        });

        // Update mouse state (check for new hovered button, etc.)
        self.mutate_with_update_context(|context| {
            Self::update_drag(context);
        });
        self.update_mouse_state(&HashSet::new(), false, &mut false);

        // GC
        self.gc_arena.borrow_mut().collect_debt();

        rval
    }

    pub fn flush_shared_objects(&mut self) {
        self.update(|context| {
            if let Some(mut avm1_activation) =
                Activation::try_from_stub(context, ActivationIdentifier::root("[Flush]"))
            {
                for so in avm1_activation.context.avm1_shared_objects.clone().values() {
                    if let Err(e) =
                        crate::avm1::globals::shared_object::flush(&mut avm1_activation, *so, &[])
                    {
                        tracing::error!("Error flushing AVM1 shared object `{:?}`: {:?}", so, e);
                    }
                }
            }

            let mut avm2_activation = Avm2Activation::from_nothing(context);
            for so in avm2_activation.context.avm2_shared_objects.clone().values() {
                if let Err(e) = crate::avm2::globals::flash::net::shared_object::flush(
                    &mut avm2_activation,
                    *so,
                    &[],
                ) {
                    tracing::error!("Error flushing AVM2 shared object `{:?}`: {:?}", so, e);
                }
            }
        });
    }

    /// Update all AVM-based timers (such as created via setInterval).
    /// Returns the approximate amount of time until the next timer tick.
    pub fn update_timers(&mut self, dt: f64) {
        self.time_til_next_timer =
            self.mutate_with_update_context(|context| Timers::update_timers(context, dt));
    }

    /// Update connected Sockets.
    pub fn update_sockets(&mut self) {
        self.mutate_with_update_context(|context| {
            Sockets::update_sockets(context);
        })
    }

    /// Update connected NetConnections.
    pub fn update_net_connections(&mut self) {
        self.mutate_with_update_context(|context| {
            NetConnections::update_connections(context);
        })
    }

    /// Returns whether this player consumes mouse wheel events.
    /// Used by web to prevent scrolling.
    pub fn should_prevent_scrolling(&mut self) -> bool {
        self.mutate_with_update_context(|context| context.avm1.has_mouse_listener())
    }

    pub fn add_external_interface(&mut self, provider: Box<dyn ExternalInterfaceProvider>) {
        self.mutate_with_update_context(|context| {
            context.external_interface.add_provider(provider)
        });
    }

    pub fn call_internal_interface(
        &mut self,
        name: &str,
        args: impl IntoIterator<Item = ExternalValue>,
    ) -> ExternalValue {
        self.mutate_with_update_context(|context| {
            if let Some(callback) = context.external_interface.get_callback(name) {
                callback.call(context, name, args)
            } else {
                ExternalValue::Null
            }
        })
    }

    pub fn spoofed_url(&self) -> Option<&str> {
        self.spoofed_url.as_deref()
    }

    pub fn compatibility_rules(&self) -> &CompatibilityRules {
        &self.compatibility_rules
    }

    pub fn log_backend(&self) -> &Log {
        &self.log
    }

    pub fn max_execution_duration(&self) -> Duration {
        self.max_execution_duration
    }

    pub fn set_max_execution_duration(&mut self, max_execution_duration: Duration) {
        self.max_execution_duration = max_execution_duration
    }

    pub fn callstack(&self) -> StaticCallstack {
        StaticCallstack {
            arena: Rc::downgrade(&self.gc_arena),
        }
    }

    /// Eagerly load any device fonts.
    /// It's preferable to use [UiBackend::load_device_font] for lazy font loading,
    /// but this is for situations where you don't know the names of the fonts you're going to register.
    pub fn register_device_font(&mut self, definition: FontDefinition<'_>) {
        self.mutate_with_update_context(|context| {
            context
                .library
                .register_device_font(context.gc_context, context.renderer, definition);
        });
    }

    pub fn set_default_font(&mut self, font: DefaultFont, names: Vec<String>) {
        self.mutate_with_update_context(|context| {
            context.library.set_default_font(font, names);
        });
    }
}

/// Player factory, which can be used to configure the aspects of a Ruffle player.
pub struct PlayerBuilder {
    movie: Option<SwfMovie>,

    // Backends
    audio: Option<Audio>,
    log: Option<Log>,
    navigator: Option<Navigator>,
    renderer: Option<Renderer>,
    storage: Option<Storage>,
    ui: Option<Ui>,
    video: Option<Video>,

    // Misc. player configuration
    autoplay: bool,
    align: StageAlign,
    forced_align: bool,
    scale_mode: StageScaleMode,
    forced_scale_mode: bool,
    allow_fullscreen: bool,
    fullscreen: bool,
    letterbox: Letterbox,
    max_execution_duration: Duration,
    viewport_width: u32,
    viewport_height: u32,
    viewport_scale_factor: f64,
    load_behavior: LoadBehavior,
    spoofed_url: Option<String>,
    compatibility_rules: CompatibilityRules,
    gamepad_button_mapping: HashMap<GamepadButton, KeyCode>,
    player_version: Option<u8>,
    player_runtime: PlayerRuntime,
    quality: StageQuality,
    page_url: Option<String>,
    frame_rate: Option<f64>,
    external_interface_providers: Vec<Box<dyn ExternalInterfaceProvider>>,
    fs_command_provider: Box<dyn FsCommandProvider>,
    #[cfg(feature = "known_stubs")]
    stub_report_output: Option<std::path::PathBuf>,
    avm2_optimizer_enabled: bool,
}

impl PlayerBuilder {
    /// Generates the base configuration for creating a player.
    ///
    /// All settings will be at their defaults, and "null" backends will be used. The settings
    /// can be changed by chaining the configuration methods.
    #[inline]
    pub fn new() -> Self {
        Self {
            movie: None,

            audio: None,
            log: None,
            navigator: None,
            renderer: None,
            storage: None,
            ui: None,
            video: None,

            autoplay: false,
            align: StageAlign::default(),
            forced_align: false,
            scale_mode: StageScaleMode::default(),
            forced_scale_mode: false,
            allow_fullscreen: true,
            fullscreen: false,
            // Disable script timeout in debug builds by default.
            letterbox: Letterbox::Fullscreen,
            max_execution_duration: Duration::from_secs(if cfg!(debug_assertions) {
                u64::MAX
            } else {
                15
            }),
            viewport_width: 550,
            viewport_height: 400,
            viewport_scale_factor: 1.0,
            load_behavior: LoadBehavior::Streaming,
            spoofed_url: None,
            compatibility_rules: CompatibilityRules::default(),
            gamepad_button_mapping: HashMap::new(),
            player_version: None,
            player_runtime: PlayerRuntime::default(),
            quality: StageQuality::High,
            page_url: None,
            frame_rate: None,
            external_interface_providers: vec![],
            fs_command_provider: Box::new(NullFsCommandProvider),
            #[cfg(feature = "known_stubs")]
            stub_report_output: None,
            avm2_optimizer_enabled: true,
        }
    }

    /// Configures the player to play an already-loaded movie.
    #[inline]
    pub fn with_movie(mut self, movie: SwfMovie) -> Self {
        self.movie = Some(movie);
        self
    }

    /// Sets the audio backend of the player.
    #[inline]
    pub fn with_audio(mut self, audio: impl 'static + AudioBackend) -> Self {
        self.audio = Some(Box::new(audio));
        self
    }

    /// Sets the audio backend of the player.
    #[inline]
    pub fn with_boxed_audio(mut self, audio: Box<dyn AudioBackend>) -> Self {
        self.audio = Some(audio);
        self
    }

    /// Sets the logging backend of the player.
    #[inline]
    pub fn with_log(mut self, log: impl 'static + LogBackend) -> Self {
        self.log = Some(Box::new(log));
        self
    }

    /// Sets the navigator backend of the player.
    #[inline]
    pub fn with_navigator(mut self, navigator: impl 'static + NavigatorBackend) -> Self {
        self.navigator = Some(Box::new(navigator));
        self
    }

    /// Sets the rendering backend of the player.
    #[inline]
    pub fn with_renderer(mut self, renderer: impl 'static + RenderBackend) -> Self {
        self.renderer = Some(Box::new(renderer));
        self
    }

    /// Sets the rendering backend of the player.
    #[inline]
    pub fn with_boxed_renderer(mut self, renderer: Box<dyn RenderBackend>) -> Self {
        self.renderer = Some(renderer);
        self
    }

    /// Sets the storage backend of the player.
    #[inline]
    pub fn with_storage(mut self, storage: Box<dyn StorageBackend>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Sets the UI backend of the player.
    #[inline]
    pub fn with_ui(mut self, ui: impl 'static + UiBackend) -> Self {
        self.ui = Some(Box::new(ui));
        self
    }

    /// Sets the video backend of the player.
    #[inline]
    pub fn with_video(mut self, video: impl 'static + VideoBackend) -> Self {
        self.video = Some(Box::new(video));
        self
    }

    /// Sets the stage scale mode and optionally prevents movies from changing it.
    #[inline]
    pub fn with_align(mut self, align: StageAlign, force: bool) -> Self {
        self.align = align;
        self.forced_align = force;
        self
    }

    /// Sets whether the movie will start playing immediately upon load.
    #[inline]
    pub fn with_autoplay(mut self, autoplay: bool) -> Self {
        self.autoplay = autoplay;
        self
    }

    /// Sets the letterbox setting for the player.
    #[inline]
    pub fn with_letterbox(mut self, letterbox: Letterbox) -> Self {
        self.letterbox = letterbox;
        self
    }

    /// Sets the maximum execution time of ActionScript code.
    #[inline]
    pub fn with_max_execution_duration(mut self, duration: Duration) -> Self {
        self.max_execution_duration = duration;
        self
    }

    /// Sets the dimensions of the stage.
    #[inline]
    pub fn with_viewport_dimensions(
        mut self,
        width: u32,
        height: u32,
        dpi_scale_factor: f64,
    ) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self.viewport_scale_factor = dpi_scale_factor;
        self
    }

    /// Sets the stage scale mode and optionally prevents movies from changing it.
    #[inline]
    pub fn with_scale_mode(mut self, scale: StageScaleMode, force: bool) -> Self {
        self.scale_mode = scale;
        self.forced_scale_mode = force;
        self
    }

    /// Sets whether the stage is fullscreen.
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Sets the default stage quality
    pub fn with_quality(mut self, quality: StageQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Configures how the root movie should be loaded.
    pub fn with_load_behavior(mut self, load_behavior: LoadBehavior) -> Self {
        self.load_behavior = load_behavior;
        self
    }

    /// Sets the root SWF URL provided to ActionScript.
    pub fn with_spoofed_url(mut self, url: Option<String>) -> Self {
        self.spoofed_url = url;
        self
    }

    /// Sets the compatibility rules to use with this movie.
    pub fn with_compatibility_rules(mut self, compatibility_rules: CompatibilityRules) -> Self {
        self.compatibility_rules = compatibility_rules;
        self
    }

    /// Configures the target player version.
    pub fn with_player_version(mut self, version: Option<u8>) -> Self {
        self.player_version = version;
        self
    }

    /// Configures the player runtime (default is `PlayerRuntime::FlashPlayer`)
    pub fn with_player_runtime(mut self, runtime: PlayerRuntime) -> Self {
        self.player_runtime = runtime;
        self
    }

    // Configure the embedding page's URL (if applicable)
    pub fn with_page_url(mut self, page_url: Option<String>) -> Self {
        self.page_url = page_url;
        self
    }

    /// Sets and locks the player's frame rate. If None is provided, this has no effect.
    pub fn with_frame_rate(mut self, frame_rate: Option<f64>) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    /// Adds an External Interface provider for movies to communicate with
    pub fn with_external_interface(mut self, provider: Box<dyn ExternalInterfaceProvider>) -> Self {
        self.external_interface_providers.push(provider);
        self
    }

    /// Adds an FSCommand implementation for movies to communicate with
    pub fn with_fs_commands(mut self, provider: Box<dyn FsCommandProvider>) -> Self {
        self.fs_command_provider = provider;
        self
    }

    pub fn with_gamepad_button_mapping(mut self, mapping: HashMap<GamepadButton, KeyCode>) -> Self {
        self.gamepad_button_mapping = mapping;
        self
    }

    #[cfg(feature = "known_stubs")]
    /// Sets the output path for the stub report. When set, the player
    /// will write the report to this path and exit the process.
    pub fn with_stub_report_output(mut self, output: std::path::PathBuf) -> Self {
        self.stub_report_output = Some(output);
        self
    }

    pub fn with_avm2_optimizer_enabled(mut self, value: bool) -> Self {
        self.avm2_optimizer_enabled = value;
        self
    }

    fn create_gc_root<'gc>(
        gc_context: &'gc Mutation<'gc>,
        player_version: u8,
        player_runtime: PlayerRuntime,
        fullscreen: bool,
        fake_movie: Arc<SwfMovie>,
        external_interface_providers: Vec<Box<dyn ExternalInterfaceProvider>>,
        fs_command_provider: Box<dyn FsCommandProvider>,
    ) -> GcRoot<'gc> {
        let mut interner = AvmStringInterner::new(gc_context);
        let (avm1, avm2) = {
            // SAFETY: Extending this borrow to `'gc` is sound, as the result of this
            // block implements `Collect`, preventing any `&'gc _` outliving it.
            let interner: &'gc mut _ = unsafe { &mut *(&mut interner as *mut _) };
            let mut init = StringContext::from_parts(gc_context, interner);
            (
                Avm1::new(&mut init, player_version),
                Avm2::new(&mut init, player_version, player_runtime),
            )
        };

        let data = GcRootData {
            audio_manager: AudioManager::new(),
            action_queue: ActionQueue::new(),
            avm1,
            avm2,
            interner,
            current_context_menu: None,
            drag_object: None,
            external_interface: ExternalInterface::new(
                external_interface_providers,
                fs_command_provider,
            ),
            library: Library::empty(),
            load_manager: LoadManager::new(),
            mouse_data: MouseData {
                hovered: None,
                pressed: None,
                right_pressed: None,
                middle_pressed: None,
            },
            avm1_shared_objects: HashMap::new(),
            avm2_shared_objects: HashMap::new(),
            stage: Stage::empty(gc_context, fullscreen, fake_movie),
            timers: Timers::new(),
            unbound_text_fields: Vec::new(),
            stream_manager: StreamManager::new(),
            sockets: Sockets::empty(),
            net_connections: NetConnections::default(),
            local_connections: LocalConnections::empty(),
            dynamic_root: DynamicRootSet::new(gc_context),
            post_frame_callbacks: Vec::new(),
        };

        GcRoot {
            avm2_callstack: data.avm2.call_stack(),
            data: GcRefLock::new(gc_context, data.into()),
        }
    }

    /// Builds the player, wiring up the backends and configuring the specified settings.
    pub fn build(self) -> Arc<Mutex<Player>> {
        use crate::backend::*;
        use ruffle_video::null;
        let audio = self
            .audio
            .unwrap_or_else(|| Box::new(audio::NullAudioBackend::new()));
        let log = self
            .log
            .unwrap_or_else(|| Box::new(log::NullLogBackend::new()));
        let navigator = self
            .navigator
            .unwrap_or_else(|| Box::new(navigator::NullNavigatorBackend::new()));
        let renderer = self.renderer.unwrap_or_else(|| {
            Box::new(NullRenderer::new(ViewportDimensions {
                width: self.viewport_width,
                height: self.viewport_height,
                scale_factor: self.viewport_scale_factor,
            }))
        });
        let storage = self
            .storage
            .unwrap_or_else(|| Box::new(storage::MemoryStorageBackend::new()));
        let ui = self
            .ui
            .unwrap_or_else(|| Box::new(ui::NullUiBackend::new()));
        let video = self
            .video
            .unwrap_or_else(|| Box::new(null::NullVideoBackend::new()));

        let player_version = self.player_version.unwrap_or(NEWEST_PLAYER_VERSION);

        // Instantiate the player.
        let fake_movie = Arc::new(SwfMovie::empty(player_version));
        let frame_rate = self.frame_rate.unwrap_or(12.0);
        let forced_frame_rate = self.frame_rate.is_some();
        let player = Arc::new_cyclic(|self_ref| {
            Mutex::new(Player {
                // Backends
                audio,
                log,
                navigator,
                renderer,
                storage,
                ui,
                video,

                // SWF info
                swf: fake_movie.clone(),
                current_frame: None,

                // Timing
                frame_rate,
                forced_frame_rate,
                frame_phase: Default::default(),
                frame_accumulator: 0.0,
                recent_run_frame_timings: VecDeque::with_capacity(10),
                start_time: Instant::now(),
                time_offset: 0,
                time_til_next_timer: None,
                max_execution_duration: self.max_execution_duration,
                actions_since_timeout_check: 0,

                // Input
                input: InputManager::new(self.gamepad_button_mapping),
                mouse_in_stage: true,
                mouse_position: Point::ZERO,
                mouse_cursor: MouseCursor::Arrow,
                mouse_cursor_needs_check: false,

                // Misc. state
                rng: SmallRng::seed_from_u64(get_current_date_time().timestamp_millis() as u64),
                system: SystemProperties::new(),
                page_url: self.page_url.clone(),
                transform_stack: TransformStack::new(),
                instance_counter: 0,
                player_version,
                player_runtime: self.player_runtime,
                is_playing: self.autoplay,
                needs_render: true,
                self_reference: self_ref.clone(),
                load_behavior: self.load_behavior,
                spoofed_url: self.spoofed_url.clone(),
                compatibility_rules: self.compatibility_rules.clone(),
                stub_tracker: StubCollection::new(),
                #[cfg(feature = "egui")]
                debug_ui: Default::default(),

                // GC data
                gc_arena: Rc::new(RefCell::new(GcArena::new(|gc_context| {
                    Self::create_gc_root(
                        gc_context,
                        player_version,
                        self.player_runtime,
                        self.fullscreen,
                        fake_movie.clone(),
                        self.external_interface_providers,
                        self.fs_command_provider,
                    )
                }))),
            })
        });

        // Finalize configuration and load the movie.
        let mut player_lock = player.lock().unwrap();

        #[cfg(feature = "default_font")]
        {
            let mut font_reader = swf::read::Reader::new(FALLBACK_DEVICE_FONT_TAG, 8);
            let font_tag = font_reader
                .read_define_font_2(3)
                .expect("Built-in font should compile");
            player_lock
                .register_device_font(FontDefinition::SwfTag(font_tag, font_reader.encoding()));
            player_lock.set_default_font(DefaultFont::Sans, vec!["Noto Sans".to_string()]);
            player_lock.set_default_font(DefaultFont::Serif, vec!["Noto Sans".to_string()]);
            player_lock.set_default_font(DefaultFont::Typewriter, vec!["Noto Sans".to_string()]);
            player_lock.set_default_font(
                DefaultFont::JapaneseGothicMono,
                vec!["Noto Sans".to_string()],
            );
            player_lock
                .set_default_font(DefaultFont::JapaneseGothic, vec!["Noto Sans".to_string()]);
            player_lock
                .set_default_font(DefaultFont::JapaneseMincho, vec!["Noto Sans".to_string()]);
        }

        player_lock.mutate_with_update_context(|context| {
            if !self.avm2_optimizer_enabled {
                tracing::warn!(
                    "AVM2 optimizer disabled, some bytecode verification will be missing"
                );
            }

            context
                .avm2
                .set_optimizer_enabled(self.avm2_optimizer_enabled);
            Avm2::load_player_globals(context).expect("Unable to load AVM2 globals");

            let stage = context.stage;
            stage.set_align(context, self.align);
            stage.set_forced_align(context, self.forced_align);
            stage.set_scale_mode(context, self.scale_mode, false);
            stage.set_forced_scale_mode(context, self.forced_scale_mode);
            stage.set_allow_fullscreen(context, self.allow_fullscreen);
            stage.post_instantiation(context, None, Instantiator::Movie, false);
            stage.build_matrices(context);
            #[cfg(feature = "known_stubs")]
            if let Some(stub_path) = self.stub_report_output {
                crate::avm2::specification::capture_specification(context, &stub_path);
            }
        });
        player_lock.audio.set_frame_rate(frame_rate);
        player_lock.set_letterbox(self.letterbox);
        player_lock.set_quality(self.quality);
        player_lock.set_viewport_dimensions(ViewportDimensions {
            width: self.viewport_width,
            height: self.viewport_height,
            scale_factor: self.viewport_scale_factor,
        });
        if let Some(mut movie) = self.movie {
            if let Some(url) = self.spoofed_url.clone() {
                movie.set_url(url);
            }
            player_lock.mutate_with_update_context(|context| {
                context.set_root_movie(movie);
            });
        }
        drop(player_lock);
        player
    }
}

impl Default for PlayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct DragObject<'gc> {
    /// The display object being dragged.
    pub display_object: DisplayObject<'gc>,

    /// The last seen mouse position.
    #[collect(require_static)]
    pub last_mouse_position: Point<Twips>,

    /// Whether the dragged object is locked to the center of the mouse position, or locked to the
    /// point where the user first clicked it.
    #[collect(require_static)]
    pub lock_center: bool,

    /// The bounding rectangle where the clip will be maintained.
    #[collect(require_static)]
    pub constraint: Rectangle<Twips>,
}

fn run_mouse_pick<'gc>(
    context: &mut UpdateContext<'gc>,
    require_button_mode: bool,
) -> Option<InteractiveObject<'gc>> {
    context.stage.iter_render_list().rev().find_map(|level| {
        level.as_interactive().and_then(|l| {
            if l.as_displayobject().movie().is_action_script_3() {
                let mut res = None;
                if let Avm2MousePick::Hit(target) =
                    l.mouse_pick_avm2(context, *context.mouse_position, require_button_mode)
                {
                    // Flash Player appears to never target events at the root object
                    if !target.as_displayobject().is_root() {
                        res = Some(target);
                    }
                }

                res
            } else {
                l.mouse_pick_avm1(context, *context.mouse_position, require_button_mode)
            }
        })
    })
}

#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub enum PlayerRuntime {
    #[default]
    FlashPlayer,
    AIR,
}

pub struct ParseEnumError;

impl FromStr for PlayerRuntime {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let player_runtime = match s {
            "air" => PlayerRuntime::AIR,
            "flash_player" => PlayerRuntime::FlashPlayer,
            _ => return Err(ParseEnumError),
        };
        Ok(player_runtime)
    }
}
