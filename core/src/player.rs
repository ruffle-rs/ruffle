use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::debug::VariableDumper;
use crate::avm1::globals::system::SystemProperties;
use crate::avm1::object::Object;
use crate::avm1::property::Attribute;
use crate::avm1::{Avm1, AvmString, ScriptObject, TObject, Timers, Value};
use crate::avm2::{Avm2, Domain as Avm2Domain};
use crate::backend::{
    audio::{AudioBackend, AudioManager},
    locale::LocaleBackend,
    log::LogBackend,
    navigator::{NavigatorBackend, RequestOptions},
    render::RenderBackend,
    storage::StorageBackend,
    ui::{MouseCursor, UiBackend},
    video::VideoBackend,
};
use crate::config::Letterbox;
use crate::context::{ActionQueue, ActionType, RenderContext, UpdateContext};
use crate::display_object::{EditText, MorphShape, MovieClip, Stage};
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult, KeyCode, PlayerEvent};
use crate::external::Value as ExternalValue;
use crate::external::{ExternalInterface, ExternalInterfaceProvider};
use crate::focus_tracker::FocusTracker;
use crate::library::Library;
use crate::loader::LoadManager;
use crate::prelude::*;
use crate::property_map::PropertyMap;
use crate::tag_utils::SwfMovie;
use crate::transform::TransformStack;
use crate::vminterface::{AvmType, Instantiator};
use gc_arena::{make_arena, ArenaParameters, Collect, GcCell};
use instant::Instant;
use log::info;
use rand::{rngs::SmallRng, SeedableRng};
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

pub static DEVICE_FONT_TAG: &[u8] = include_bytes!("../assets/noto-sans-definefont3.bin");

/// The newest known Flash Player version, serves as a default to
/// `player_version`.
pub const NEWEST_PLAYER_VERSION: u8 = 32;

#[derive(Collect)]
#[collect(no_drop)]
struct GcRoot<'gc>(GcCell<'gc, GcRootData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
struct GcRootData<'gc> {
    library: Library<'gc>,

    /// The root of the display object hierarchy.
    ///
    /// It's children are the `level`s of AVM1, it may also be directly
    /// accessed in AVM2.
    stage: Stage<'gc>,

    mouse_hovered_object: Option<DisplayObject<'gc>>, // TODO: Remove GcCell wrapped inside GcCell.

    /// The object being dragged via a `startDrag` action.
    drag_object: Option<DragObject<'gc>>,

    /// Interpreter state for AVM1 code.
    avm1: Avm1<'gc>,

    /// Interpreter state for AVM2 code.
    avm2: Avm2<'gc>,

    action_queue: ActionQueue<'gc>,

    /// Object which manages asynchronous processes that need to interact with
    /// data in the GC arena.
    load_manager: LoadManager<'gc>,

    shared_objects: HashMap<String, Object<'gc>>,

    /// Text fields with unbound variable bindings.
    unbound_text_fields: Vec<EditText<'gc>>,

    /// Timed callbacks created with `setInterval`/`setTimeout`.
    timers: Timers<'gc>,

    /// External interface for (for example) JavaScript <-> ActionScript interaction
    external_interface: ExternalInterface<'gc>,

    /// A tracker for the current keyboard focused element
    focus_tracker: FocusTracker<'gc>,

    /// Manager of active sound instances.
    audio_manager: AudioManager<'gc>,
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
        &mut Avm1<'gc>,
        &mut Avm2<'gc>,
        &mut Option<DragObject<'gc>>,
        &mut LoadManager<'gc>,
        &mut HashMap<String, Object<'gc>>,
        &mut Vec<EditText<'gc>>,
        &mut Timers<'gc>,
        &mut ExternalInterface<'gc>,
        &mut AudioManager<'gc>,
    ) {
        (
            self.stage,
            &mut self.library,
            &mut self.action_queue,
            &mut self.avm1,
            &mut self.avm2,
            &mut self.drag_object,
            &mut self.load_manager,
            &mut self.shared_objects,
            &mut self.unbound_text_fields,
            &mut self.timers,
            &mut self.external_interface,
            &mut self.audio_manager,
        )
    }
}
type Error = Box<dyn std::error::Error>;

make_arena!(GcArena, GcRoot);

type Audio = Box<dyn AudioBackend>;
type Navigator = Box<dyn NavigatorBackend>;
type Renderer = Box<dyn RenderBackend>;
type Storage = Box<dyn StorageBackend>;
type Locale = Box<dyn LocaleBackend>;
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

    swf: Arc<SwfMovie>,

    warn_on_unsupported_content: bool,

    is_playing: bool,
    needs_render: bool,

    renderer: Renderer,
    audio: Audio,
    navigator: Navigator,
    storage: Storage,
    locale: Locale,
    log: Log,
    ui: Ui,
    video: Video,

    transform_stack: TransformStack,

    rng: SmallRng,

    gc_arena: GcArena,

    frame_rate: f64,

    /// A time budget for executing frames.
    /// Gained by passage of time between host frames, spent by executing SWF frames.
    /// This is how we support custom SWF framerates
    /// and compensate for small lags by "catching up" (up to MAX_FRAMES_PER_TICK).
    frame_accumulator: f64,
    recent_run_frame_timings: VecDeque<f64>,

    /// Faked time passage for fooling hand-written busy-loop FPS limiters.
    time_offset: u32,

    mouse_pos: (Twips, Twips),
    is_mouse_down: bool,

    /// The current mouse cursor icon.
    mouse_cursor: MouseCursor,

    system: SystemProperties,

    /// The current instance ID. Used to generate default `instanceN` names.
    instance_counter: i32,

    /// Time remaining until the next timer will fire.
    time_til_next_timer: Option<f64>,

    /// The maximum amount of time that can be called before a `Error::ExecutionTimeout`
    /// is raised. This defaults to 15 seconds but can be changed.
    max_execution_duration: Duration,

    /// Self-reference to ourselves.
    ///
    /// This is a weak reference that is upgraded and handed out in various
    /// contexts to other parts of the player. It can be used to ensure the
    /// player lives across `await` calls in async code.
    self_reference: Option<Weak<Mutex<Self>>>,

    /// The current frame of the main timeline, if available.
    /// The first frame is frame 1.
    current_frame: Option<u16>,
}

#[allow(clippy::too_many_arguments)]
impl Player {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        renderer: Renderer,
        audio: Audio,
        navigator: Navigator,
        storage: Storage,
        locale: Locale,
        video: Video,
        log: Log,
        ui: Ui,
    ) -> Result<Arc<Mutex<Self>>, Error> {
        let fake_movie = Arc::new(SwfMovie::empty(NEWEST_PLAYER_VERSION));
        let movie_width = 550;
        let movie_height = 400;
        let frame_rate = 12.0;
        // Disable script timeout in debug builds by default.
        let max_execution_duration = if cfg!(debug_assertions) { u64::MAX } else { 15 };

        let mut player = Player {
            player_version: NEWEST_PLAYER_VERSION,

            swf: fake_movie.clone(),

            warn_on_unsupported_content: true,

            is_playing: false,
            needs_render: true,

            transform_stack: TransformStack::new(),

            rng: SmallRng::seed_from_u64(chrono::Utc::now().timestamp_millis() as u64),

            gc_arena: GcArena::new(ArenaParameters::default(), |gc_context| {
                GcRoot(GcCell::allocate(
                    gc_context,
                    GcRootData {
                        library: Library::empty(gc_context),
                        stage: Stage::empty(gc_context, movie_width, movie_height),
                        mouse_hovered_object: None,
                        drag_object: None,
                        avm1: Avm1::new(gc_context, NEWEST_PLAYER_VERSION),
                        avm2: Avm2::new(gc_context),
                        action_queue: ActionQueue::new(),
                        load_manager: LoadManager::new(),
                        shared_objects: HashMap::new(),
                        unbound_text_fields: Vec::new(),
                        timers: Timers::new(),
                        external_interface: ExternalInterface::new(),
                        focus_tracker: FocusTracker::new(gc_context),
                        audio_manager: AudioManager::new(),
                    },
                ))
            }),

            frame_rate,
            frame_accumulator: 0.0,
            recent_run_frame_timings: VecDeque::with_capacity(10),
            time_offset: 0,

            mouse_pos: (Twips::zero(), Twips::zero()),
            is_mouse_down: false,
            mouse_cursor: MouseCursor::Arrow,

            renderer,
            audio,
            navigator,
            locale,
            log,
            ui,
            video,
            self_reference: None,
            system: SystemProperties::default(),
            instance_counter: 0,
            time_til_next_timer: None,
            storage,
            max_execution_duration: Duration::from_secs(max_execution_duration),
            current_frame: None,
        };

        player.mutate_with_update_context(|context| {
            // Instantiate an empty root before the main movie loads.
            let fake_root = MovieClip::from_movie(context.gc_context, fake_movie);
            fake_root.post_instantiation(
                context,
                fake_root.into(),
                None,
                Instantiator::Movie,
                false,
            );
            context.stage.replace_at_depth(context, fake_root.into(), 0);

            let result = Avm2::load_player_globals(context);

            let stage = context.stage;
            stage.build_matrices(context);

            result
        })?;

        player.audio.set_frame_rate(frame_rate);
        let player_box = Arc::new(Mutex::new(player));
        let mut player_lock = player_box.lock().unwrap();
        player_lock.self_reference = Some(Arc::downgrade(&player_box));

        std::mem::drop(player_lock);

        Ok(player_box)
    }

    /// Fetch the root movie.
    ///
    /// This should not be called if a root movie fetch has already been kicked
    /// off.
    pub fn fetch_root_movie(
        &mut self,
        movie_url: &str,
        parameters: PropertyMap<String>,
        on_metadata: Box<dyn FnOnce(&swf::Header)>,
    ) {
        self.mutate_with_update_context(|context| {
            let fetch = context.navigator.fetch(movie_url, RequestOptions::get());
            let process = context.load_manager.load_root_movie(
                context.player.clone().unwrap(),
                fetch,
                movie_url.to_string(),
                parameters,
                on_metadata,
            );

            context.navigator.spawn_future(process);
        });
    }

    /// Change the root movie.
    ///
    /// This should only be called once, as it makes no attempt at removing
    /// previous stage contents. If you need to load a new root movie, you
    /// should destroy and recreate the player instance.
    pub fn set_root_movie(&mut self, movie: Arc<SwfMovie>) {
        info!(
            "Loaded SWF version {}, with a resolution of {}x{}",
            movie.header().version,
            movie.header().stage_size.x_max,
            movie.header().stage_size.y_max
        );

        self.frame_rate = movie.header().frame_rate.into();
        self.swf = movie;
        self.instance_counter = 0;

        self.mutate_with_update_context(|context| {
            context.stage.set_stage_size(
                context.gc_context,
                context.swf.width(),
                context.swf.height(),
            );
            let domain = Avm2Domain::movie_domain(context.gc_context, context.avm2.global_domain());
            context
                .library
                .library_for_movie_mut(context.swf.clone())
                .set_avm2_domain(domain);

            let root: DisplayObject =
                MovieClip::from_movie(context.gc_context, context.swf.clone()).into();

            root.set_depth(context.gc_context, 0);
            let flashvars = if !context.swf.parameters().is_empty() {
                let object = ScriptObject::object(context.gc_context, None);
                for (key, value) in context.swf.parameters().iter() {
                    object.define_value(
                        context.gc_context,
                        key,
                        AvmString::new(context.gc_context, value).into(),
                        Attribute::empty(),
                    );
                }
                Some(object.into())
            } else {
                None
            };

            root.post_instantiation(context, root, flashvars, Instantiator::Movie, false);
            root.set_default_root_name(context);
            context.stage.replace_at_depth(context, root, 0);

            // Load and parse the device font.
            if context.library.device_font().is_none() {
                let device_font =
                    Self::load_device_font(context.gc_context, DEVICE_FONT_TAG, context.renderer);
                if let Err(e) = &device_font {
                    log::error!("Unable to load device font: {}", e);
                }
                context.library.set_device_font(device_font.ok());
            }

            // Set the version parameter on the root.
            let mut activation = Activation::from_stub(
                context.reborrow(),
                ActivationIdentifier::root("[Version Setter]"),
            );
            let object = root.object().coerce_to_object(&mut activation);
            let version_string = activation
                .context
                .system
                .get_version_string(activation.context.avm1);
            object.define_value(
                activation.context.gc_context,
                "$version",
                AvmString::new(activation.context.gc_context, version_string).into(),
                Attribute::empty(),
            );

            let stage = activation.context.stage;
            stage.build_matrices(&mut activation.context);
        });

        self.preload();
        self.audio.set_frame_rate(self.frame_rate);
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
            ((frame_time / average_run_frame_time) as u32)
                .max(1)
                .min(MAX_FRAMES_PER_TICK)
        }
    }

    fn add_frame_timing(&mut self, elapsed: f64) {
        self.recent_run_frame_timings.push_back(elapsed);
        if self.recent_run_frame_timings.len() >= 10 {
            self.recent_run_frame_timings.pop_front();
        }
    }

    pub fn tick(&mut self, dt: f64) {
        // Don't run until preloading is complete.
        // TODO: Eventually we want to stream content similar to the Flash player.
        if !self.audio.is_loading_complete() {
            return;
        }

        if self.is_playing() {
            self.frame_accumulator += dt;
            let frame_time = 1000.0 / self.frame_rate;

            let max_frames_per_tick = self.max_frames_per_tick();
            let mut frame = 0;

            while frame < max_frames_per_tick && self.frame_accumulator >= frame_time {
                let timer = Instant::now();
                self.run_frame();
                let elapsed = timer.elapsed().as_millis() as f64;

                self.add_frame_timing(elapsed);

                self.frame_accumulator -= frame_time;
                frame += 1;
                // The script probably tried implementing an FPS limiter with a busy loop.
                // We fooled the busy loop by pretending that more time has passed that actually did.
                // Then we need to actually pass this time, by decreasing frame_accumulator
                // to delay the future frame.
                if self.time_offset > 0 {
                    self.frame_accumulator -= self.time_offset as f64;
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

            self.update_timers(dt);
            self.audio.tick();
        }
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

    pub fn warn_on_unsupported_content(&self) -> bool {
        self.warn_on_unsupported_content
    }

    pub fn set_warn_on_unsupported_content(&mut self, warn_on_unsupported_content: bool) {
        self.warn_on_unsupported_content = warn_on_unsupported_content
    }

    pub fn movie_width(&mut self) -> u32 {
        self.mutate_with_update_context(|context| context.stage.stage_size().0)
    }

    pub fn movie_height(&mut self) -> u32 {
        self.mutate_with_update_context(|context| context.stage.stage_size().1)
    }

    pub fn viewport_dimensions(&mut self) -> (u32, u32) {
        self.mutate_with_update_context(|context| context.stage.viewport_size())
    }

    pub fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            stage.set_viewport_size(context, width, height);
        })
    }

    pub fn handle_event(&mut self, event: PlayerEvent) {
        let mut needs_render = self.needs_render;
        let inverse_view_matrix =
            self.mutate_with_update_context(|context| context.stage.inverse_view_matrix());

        if cfg!(feature = "avm_debug") {
            if let PlayerEvent::KeyDown {
                key_code: KeyCode::V,
            } = event
            {
                if self.ui.is_key_down(KeyCode::Control) && self.ui.is_key_down(KeyCode::Alt) {
                    self.mutate_with_update_context(|context| {
                        let mut dumper = VariableDumper::new("  ");
                        let levels: Vec<_> = context.stage.iter_depth_list().collect();

                        let mut activation = Activation::from_stub(
                            context.reborrow(),
                            ActivationIdentifier::root("[Variable Dumper]"),
                        );

                        dumper.print_variables(
                            "Global Variables:",
                            "_global",
                            &activation.context.avm1.global_object_cell(),
                            &mut activation,
                        );

                        for (level, display_object) in levels {
                            let object = display_object.object().coerce_to_object(&mut activation);
                            dumper.print_variables(
                                &format!("Level #{}:", level),
                                &format!("_level{}", level),
                                &object,
                                &mut activation,
                            );
                        }
                        log::info!("Variable dump:\n{}", dumper.output());
                    });
                }
            }

            if let PlayerEvent::KeyDown {
                key_code: KeyCode::D,
            } = event
            {
                if self.ui.is_key_down(KeyCode::Control) && self.ui.is_key_down(KeyCode::Alt) {
                    self.mutate_with_update_context(|context| {
                        if context.avm1.show_debug_output() {
                            log::info!(
                                "AVM Debugging turned off! Press CTRL+ALT+D to turn off again."
                            );
                            context.avm1.set_show_debug_output(false);
                            context.avm2.set_show_debug_output(false);
                        } else {
                            log::info!(
                                "AVM Debugging turned on! Press CTRL+ALT+D to turn on again."
                            );
                            context.avm1.set_show_debug_output(true);
                            context.avm2.set_show_debug_output(true);
                        }
                    });
                }
            }
        }

        // Update mouse position from mouse events.
        if let PlayerEvent::MouseMove { x, y }
        | PlayerEvent::MouseDown { x, y }
        | PlayerEvent::MouseUp { x, y } = event
        {
            self.mouse_pos = inverse_view_matrix * (Twips::from_pixels(x), Twips::from_pixels(y));
            if self.update_roll_over() {
                needs_render = true;
            }
        }

        // Propagate button events.
        let button_event = match event {
            // ASCII characters convert directly to keyPress button events.
            PlayerEvent::TextInput { codepoint }
                if codepoint as u32 >= 32 && codepoint as u32 <= 126 =>
            {
                Some(ClipEvent::KeyPress {
                    key_code: ButtonKeyCode::try_from(codepoint as u8).unwrap(),
                })
            }

            // Special keys have custom values for keyPress.
            PlayerEvent::KeyDown { key_code } => {
                if let Some(key_code) = crate::events::key_code_to_button_key_code(key_code) {
                    Some(ClipEvent::KeyPress { key_code })
                } else {
                    None
                }
            }
            _ => None,
        };

        if button_event.is_some() {
            self.mutate_with_update_context(|context| {
                let levels: Vec<_> = context.stage.iter_depth_list().collect();
                for (_depth, level) in levels {
                    if let Some(button_event) = button_event {
                        let state = level.handle_clip_event(context, button_event);
                        if state == ClipEventResult::Handled {
                            return;
                        }
                    }
                }
            });
        }

        if let PlayerEvent::TextInput { codepoint } = event {
            self.mutate_with_update_context(|context| {
                if let Some(text) = context.focus_tracker.get().and_then(|o| o.as_edit_text()) {
                    text.text_input(codepoint, context);
                }
            });
        }

        // Propagate clip events.
        self.mutate_with_update_context(|context| {
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
                PlayerEvent::MouseUp { .. } => (
                    Some(ClipEvent::MouseUp),
                    Some(("Mouse", "onMouseUp", vec![])),
                ),
                PlayerEvent::MouseDown { .. } => (
                    Some(ClipEvent::MouseDown),
                    Some(("Mouse", "onMouseDown", vec![])),
                ),
                PlayerEvent::MouseWheel { delta } => {
                    let delta = Value::from(delta.lines());
                    (None, Some(("Mouse", "onMouseWheel", vec![delta])))
                }
                _ => (None, None),
            };

            // Fire clip event on all clips.
            if let Some(clip_event) = clip_event {
                let levels: Vec<_> = context.stage.iter_depth_list().collect();
                for (_depth, level) in levels {
                    level.handle_clip_event(context, clip_event);
                }
            }

            // Fire event listener on appropriate object
            if let Some((listener_type, event_name, args)) = listener {
                context.action_queue.queue_actions(
                    context.stage.root_clip(),
                    ActionType::NotifyListeners {
                        listener: listener_type,
                        method: event_name,
                        args,
                    },
                    false,
                );
            }
        });

        let mut is_mouse_down = self.is_mouse_down;
        self.mutate_with_update_context(|context| {
            if let Some(node) = context.mouse_hovered_object {
                if node.removed() {
                    context.mouse_hovered_object = None;
                }
            }

            match event {
                PlayerEvent::MouseDown { .. } => {
                    is_mouse_down = true;
                    needs_render = true;
                    if let Some(node) = context.mouse_hovered_object {
                        node.handle_clip_event(context, ClipEvent::Press);
                    }
                }

                PlayerEvent::MouseUp { .. } => {
                    is_mouse_down = false;
                    needs_render = true;
                    if let Some(node) = context.mouse_hovered_object {
                        node.handle_clip_event(context, ClipEvent::Release);
                    }
                }

                _ => (),
            }

            Self::run_actions(context);
        });
        self.is_mouse_down = is_mouse_down;
        if needs_render {
            self.needs_render = true;
        }
    }

    /// Update dragged object, if any.
    fn update_drag(&mut self) {
        let mouse_pos = self.mouse_pos;
        self.mutate_with_update_context(|context| {
            if let Some(drag_object) = &mut context.drag_object {
                if drag_object.display_object.removed() {
                    // Be sure to clear the drag if the object was removed.
                    *context.drag_object = None;
                } else {
                    let mut drag_point = (
                        mouse_pos.0 + drag_object.offset.0,
                        mouse_pos.1 + drag_object.offset.1,
                    );
                    if let Some(parent) = drag_object.display_object.parent() {
                        drag_point = parent.global_to_local(drag_point);
                    }
                    drag_point = drag_object.constraint.clamp(drag_point);
                    drag_object
                        .display_object
                        .set_x(context.gc_context, drag_point.0.to_pixels());
                    drag_object
                        .display_object
                        .set_y(context.gc_context, drag_point.1.to_pixels());
                }
            }
        });
    }

    /// Checks to see if a recent update has caused the current mouse hover
    /// node to change.
    fn update_roll_over(&mut self) -> bool {
        // TODO: While the mouse is down, maintain the hovered node.
        if self.is_mouse_down {
            return false;
        }
        let mouse_pos = self.mouse_pos;

        let mut new_cursor = self.mouse_cursor;
        let hover_changed = self.mutate_with_update_context(|context| {
            // Check hovered object.
            let mut new_hovered = None;
            let levels: Vec<_> = context.stage.iter_depth_list().collect();
            for (_depth, level) in levels.iter().rev() {
                if new_hovered.is_none() {
                    new_hovered = level.mouse_pick(context, *level, mouse_pos);
                } else {
                    break;
                }
            }

            let cur_hovered = context.mouse_hovered_object;

            if cur_hovered.map(|d| d.as_ptr()) != new_hovered.map(|d| d.as_ptr()) {
                // RollOut of previous node.
                if let Some(node) = cur_hovered {
                    if !node.removed() {
                        node.handle_clip_event(context, ClipEvent::RollOut);
                    }
                }

                // RollOver on new node.I still
                new_cursor = MouseCursor::Arrow;
                if let Some(node) = new_hovered {
                    new_cursor = node.mouse_cursor();
                    node.handle_clip_event(context, ClipEvent::RollOver);
                }

                context.mouse_hovered_object = new_hovered;

                Self::run_actions(context);
                true
            } else {
                false
            }
        });

        // Update mouse cursor if it has changed.
        if new_cursor != self.mouse_cursor {
            self.mouse_cursor = new_cursor;
            self.ui.set_mouse_cursor(new_cursor)
        }

        hover_changed
    }

    /// Preload the first movie in the player.
    ///
    /// This should only be called once. Further movie loads should preload the
    /// specific `MovieClip` referenced.
    fn preload(&mut self) {
        let mut is_action_script_3 = false;
        self.mutate_with_update_context(|context| {
            let mut morph_shapes = fnv::FnvHashMap::default();
            let root = context.stage.root_clip();
            root.as_movie_clip()
                .unwrap()
                .preload(context, &mut morph_shapes);

            let lib = context
                .library
                .library_for_movie_mut(root.as_movie_clip().unwrap().movie().unwrap());

            is_action_script_3 = lib.avm_type() == AvmType::Avm2;
            // Finalize morph shapes.
            for (id, static_data) in morph_shapes {
                let morph_shape = MorphShape::new(context.gc_context, static_data);
                lib.register_character(id, crate::character::Character::MorphShape(morph_shape));
            }
        });
        if is_action_script_3 && self.warn_on_unsupported_content {
            self.ui.display_unsupported_message();
        }
    }

    pub fn run_frame(&mut self) {
        self.update(|update_context| {
            // TODO: In what order are levels run?
            let stage = update_context.stage;

            stage.exit_frame(update_context);
            stage.enter_frame(update_context);
            stage.construct_frame(update_context);
            stage.frame_constructed(update_context);
            stage.run_frame(update_context);
            stage.run_frame_scripts(update_context);

            update_context.update_sounds();
        });
        self.needs_render = true;
    }

    pub fn render(&mut self) {
        let (renderer, ui, transform_stack) =
            (&mut self.renderer, &mut self.ui, &mut self.transform_stack);

        self.gc_arena.mutate(|_gc_context, gc_root| {
            let root_data = gc_root.0.read();
            let mut render_context = RenderContext {
                renderer: renderer.deref_mut(),
                ui: ui.deref_mut(),
                library: &root_data.library,
                transform_stack,
                stage: root_data.stage,
                clip_depth_stack: vec![],
                allow_mask: true,
            };

            root_data.stage.render(&mut render_context);
        });

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

    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    pub fn run_actions<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
        // Note that actions can queue further actions, so a while loop is necessary here.
        while let Some(actions) = context.action_queue.pop_action() {
            // We don't run frame actions if the clip was removed after it queued the action.
            if !actions.is_unload && actions.clip.removed() {
                continue;
            }

            match actions.action_type {
                // DoAction/clip event code
                ActionType::Normal { bytecode } | ActionType::Initialize { bytecode } => {
                    Avm1::run_stack_frame_for_action(
                        actions.clip,
                        "[Frame]",
                        context.swf.header().version,
                        bytecode,
                        context,
                    );
                }
                // Change the prototype of a movieclip & run constructor events
                ActionType::Construct {
                    constructor: Some(constructor),
                    events,
                } => {
                    let version = context.swf.version();
                    let globals = context.avm1.global_object_cell();

                    let mut activation = Activation::from_nothing(
                        context.reborrow(),
                        ActivationIdentifier::root("[Construct]"),
                        version,
                        globals,
                        actions.clip,
                    );
                    if let Ok(prototype) = constructor.get("prototype", &mut activation) {
                        if let Value::Object(object) = actions.clip.object() {
                            object.set_proto(activation.context.gc_context, prototype);
                            for event in events {
                                let _ = activation.run_child_frame_for_action(
                                    "[Actions]",
                                    actions.clip,
                                    activation.context.swf.header().version,
                                    event,
                                );
                            }

                            let _ = constructor.construct_on_existing(&mut activation, object, &[]);
                        }
                    }
                }
                // Run constructor events without changing the prototype
                ActionType::Construct {
                    constructor: None,
                    events,
                } => {
                    for event in events {
                        Avm1::run_stack_frame_for_action(
                            actions.clip,
                            "[Construct]",
                            context.swf.header().version,
                            event,
                            context,
                        );
                    }
                }
                // Event handler method call (e.g. onEnterFrame)
                ActionType::Method { object, name, args } => {
                    Avm1::run_stack_frame_for_method(
                        actions.clip,
                        object,
                        context.swf.header().version,
                        context,
                        name,
                        &args,
                    );
                }

                // Event handler method call (e.g. onEnterFrame)
                ActionType::NotifyListeners {
                    listener,
                    method,
                    args,
                } => {
                    // A native function ends up resolving immediately,
                    // so this doesn't require any further execution.
                    Avm1::notify_system_listeners(
                        actions.clip,
                        context.swf.version(),
                        context,
                        listener,
                        method,
                        &args,
                    );
                }

                ActionType::Callable2 {
                    callable,
                    reciever,
                    args,
                } => {
                    if let Err(e) =
                        Avm2::run_stack_frame_for_callable(callable, reciever, &args[..], context)
                    {
                        log::error!("Unhandled AVM2 exception in event handler: {}", e);
                    }
                }
            }
        }
    }

    /// Runs the closure `f` with an `UpdateContext`.
    /// This takes cares of populating the `UpdateContext` struct, avoiding borrow issues.
    fn mutate_with_update_context<F, R>(&mut self, f: F) -> R
    where
        F: for<'a, 'gc> FnOnce(&mut UpdateContext<'a, 'gc, '_>) -> R,
    {
        // We have to do this piecewise borrowing of fields before the closure to avoid
        // completely borrowing `self`.
        let (
            player_version,
            swf,
            renderer,
            audio,
            navigator,
            ui,
            rng,
            mouse_position,
            player,
            system_properties,
            instance_counter,
            storage,
            locale,
            logging,
            video,
            needs_render,
            max_execution_duration,
            current_frame,
            time_offset,
        ) = (
            self.player_version,
            &self.swf,
            self.renderer.deref_mut(),
            self.audio.deref_mut(),
            self.navigator.deref_mut(),
            self.ui.deref_mut(),
            &mut self.rng,
            &self.mouse_pos,
            self.self_reference.clone(),
            &mut self.system,
            &mut self.instance_counter,
            self.storage.deref_mut(),
            self.locale.deref_mut(),
            self.log.deref_mut(),
            self.video.deref_mut(),
            &mut self.needs_render,
            self.max_execution_duration,
            &mut self.current_frame,
            &mut self.time_offset,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut root_data = gc_root.0.write(gc_context);
            let mouse_hovered_object = root_data.mouse_hovered_object;
            let focus_tracker = root_data.focus_tracker;
            let (
                stage,
                library,
                action_queue,
                avm1,
                avm2,
                drag_object,
                load_manager,
                shared_objects,
                unbound_text_fields,
                timers,
                external_interface,
                audio_manager,
            ) = root_data.update_context_params();

            let mut update_context = UpdateContext {
                player_version,
                swf,
                library,
                rng,
                renderer,
                audio,
                navigator,
                ui,
                action_queue,
                gc_context,
                stage,
                mouse_hovered_object,
                mouse_position,
                drag_object,
                player,
                load_manager,
                system: system_properties,
                instance_counter,
                storage,
                locale,
                log: logging,
                video,
                shared_objects,
                unbound_text_fields,
                timers,
                needs_render,
                avm1,
                avm2,
                external_interface,
                update_start: Instant::now(),
                max_execution_duration,
                focus_tracker,
                times_get_time_called: 0,
                time_offset,
                audio_manager,
            };

            let ret = f(&mut update_context);

            *current_frame = update_context
                .stage
                .root_clip()
                .as_movie_clip()
                .map(|clip| clip.current_frame());

            // Hovered object may have been updated; copy it back to the GC root.
            root_data.mouse_hovered_object = update_context.mouse_hovered_object;

            ret
        })
    }

    /// Loads font data from the given buffer.
    /// The buffer should be the `DefineFont3` info for the tag.
    /// The tag header should not be included.
    pub fn load_device_font<'gc>(
        gc_context: gc_arena::MutationContext<'gc, '_>,
        data: &[u8],
        renderer: &mut dyn RenderBackend,
    ) -> Result<crate::font::Font<'gc>, Error> {
        let mut reader = swf::read::Reader::new(data, 8);
        let device_font = crate::font::Font::from_swf_tag(
            gc_context,
            renderer,
            &reader.read_define_font_2(3)?,
            reader.encoding(),
        )?;
        Ok(device_font)
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
        F: for<'a, 'gc, 'gc_context> FnOnce(&mut UpdateContext<'a, 'gc, 'gc_context>) -> R,
    {
        self.update_drag();

        let rval = self.mutate_with_update_context(|context| {
            let rval = func(context);

            Self::run_actions(context);

            rval
        });

        // Update mouse state (check for new hovered button, etc.)
        self.update_roll_over();

        // GC
        self.gc_arena.collect_debt();

        rval
    }

    pub fn flush_shared_objects(&mut self) {
        self.update(|context| {
            let mut activation =
                Activation::from_stub(context.reborrow(), ActivationIdentifier::root("[Flush]"));
            let shared_objects = activation.context.shared_objects.clone();
            for so in shared_objects.values() {
                let _ = crate::avm1::globals::shared_object::flush(&mut activation, *so, &[]);
            }
        });
    }

    /// Update all AVM-based timers (such as created via setInterval).
    /// Returns the approximate amount of time until the next timer tick.
    pub fn update_timers(&mut self, dt: f64) {
        self.time_til_next_timer =
            self.mutate_with_update_context(|context| Timers::update_timers(context, dt));
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

    pub fn log_backend(&self) -> &Log {
        &self.log
    }

    pub fn max_execution_duration(&self) -> Duration {
        self.max_execution_duration
    }

    pub fn set_max_execution_duration(&mut self, max_execution_duration: Duration) {
        self.max_execution_duration = max_execution_duration
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct DragObject<'gc> {
    /// The display object being dragged.
    pub display_object: DisplayObject<'gc>,

    /// The offset from the mouse position to the center of the clip.
    #[collect(require_static)]
    pub offset: (Twips, Twips),

    /// The bounding rectangle where the clip will be maintained.
    #[collect(require_static)]
    pub constraint: BoundingBox,
}
