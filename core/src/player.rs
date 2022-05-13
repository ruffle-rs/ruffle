use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::debug::VariableDumper;
use crate::avm1::globals::system::SystemProperties;
use crate::avm1::object::Object;
use crate::avm1::property::Attribute;
use crate::avm1::{Avm1, ScriptObject, TObject, Timers, Value};
use crate::avm2::{Activation as Avm2Activation, Avm2, Domain as Avm2Domain};
use crate::backend::{
    audio::{AudioBackend, AudioManager},
    log::LogBackend,
    navigator::{NavigatorBackend, RequestOptions},
    render::RenderBackend,
    storage::StorageBackend,
    ui::{InputManager, MouseCursor, UiBackend},
    video::VideoBackend,
};
use crate::config::Letterbox;
use crate::context::{ActionQueue, ActionType, RenderContext, UpdateContext};
use crate::context_menu::{ContextMenuCallback, ContextMenuItem, ContextMenuState};
use crate::display_object::{
    EditText, InteractiveObject, MovieClip, Stage, StageAlign, StageDisplayState, StageQuality,
    StageScaleMode, TInteractiveObject, WindowMode,
};
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult, KeyCode, MouseButton, PlayerEvent};
use crate::external::Value as ExternalValue;
use crate::external::{ExternalInterface, ExternalInterfaceProvider};
use crate::focus_tracker::FocusTracker;
use crate::library::Library;
use crate::loader::LoadManager;
use crate::prelude::*;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use crate::transform::TransformStack;
use crate::vminterface::{AvmType, Instantiator};
use gc_arena::{make_arena, ArenaParameters, Collect, GcCell};
use instant::Instant;
use log::info;
use rand::{rngs::SmallRng, SeedableRng};
use std::collections::{HashMap, VecDeque};
use std::ops::DerefMut;
use std::str::FromStr;
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

    /// The display object that the mouse is currently hovering over.
    mouse_hovered_object: Option<InteractiveObject<'gc>>,

    /// If the mouse is down, the display object that the mouse is currently pressing.
    mouse_pressed_object: Option<InteractiveObject<'gc>>,

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

    current_context_menu: Option<ContextMenuState<'gc>>,

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
        &mut Option<ContextMenuState<'gc>>,
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
            &mut self.current_context_menu,
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

    input: InputManager,

    mouse_pos: (Twips, Twips),

    /// The current mouse cursor icon.
    mouse_cursor: MouseCursor,
    mouse_cursor_needs_check: bool,

    system: SystemProperties,

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
}

impl Player {
    /// Fetch the root movie.
    ///
    /// This should not be called if a root movie fetch has already been kicked
    /// off.
    pub fn fetch_root_movie(
        &mut self,
        movie_url: &str,
        parameters: Vec<(String, String)>,
        on_metadata: Box<dyn FnOnce(&swf::HeaderExt)>,
    ) {
        self.mutate_with_update_context(|context| {
            let future = context.load_manager.load_root_movie(
                context.player.clone(),
                movie_url,
                RequestOptions::get(),
                parameters,
                on_metadata,
            );
            context.navigator.spawn_future(future);
        });
    }

    /// Change the root movie.
    ///
    /// This should only be called once, as it makes no attempt at removing
    /// previous stage contents. If you need to load a new root movie, you
    /// should destroy and recreate the player instance.
    pub fn set_root_movie(&mut self, movie: SwfMovie) {
        info!(
            "Loaded SWF version {}, with a resolution of {}x{}",
            movie.version(),
            movie.width(),
            movie.height()
        );

        self.frame_rate = movie.frame_rate().into();
        self.swf = Arc::new(movie);
        self.instance_counter = 0;

        self.mutate_with_update_context(|context| {
            context.stage.set_movie_size(
                context.gc_context,
                context.swf.width().to_pixels() as u32,
                context.swf.height().to_pixels() as u32,
            );

            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            let global_domain = activation.avm2().global_domain();
            let domain = Avm2Domain::movie_domain(&mut activation, global_domain);

            drop(activation);

            context
                .library
                .library_for_movie_mut(context.swf.clone())
                .set_avm2_domain(domain);
            context.ui.set_mouse_visible(true);

            let root: DisplayObject =
                MovieClip::from_movie(context.gc_context, context.swf.clone()).into();

            root.set_depth(context.gc_context, 0);
            let flashvars = if !context.swf.parameters().is_empty() {
                let object = ScriptObject::object(context.gc_context, None);
                for (key, value) in context.swf.parameters().iter() {
                    object.define_value(
                        context.gc_context,
                        AvmString::new_utf8(context.gc_context, key),
                        AvmString::new_utf8(context.gc_context, value).into(),
                        Attribute::empty(),
                    );
                }
                Some(object.into())
            } else {
                None
            };

            root.post_instantiation(context, flashvars, Instantiator::Movie, false);
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
                AvmString::new_utf8(activation.context.gc_context, version_string).into(),
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
            let frame_rate = self.frame_rate;
            let frame_time = 1000.0 / frame_rate;

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

            // Adjust playback speed for next frame to stay in sync with timeline audio tracks ("stream" sounds).
            let cur_frame_offset = self.frame_accumulator;
            self.frame_accumulator += self.mutate_with_update_context(|context| {
                context
                    .audio_manager
                    .audio_skew_time(context.audio, cur_frame_offset)
                    * 1000.0
            });

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

    pub fn prepare_context_menu(&mut self) -> Vec<ContextMenuItem> {
        self.mutate_with_update_context(|context| {
            if !context.stage.show_menu() {
                return vec![];
            }

            let mut activation = Activation::from_stub(
                context.reborrow(),
                ActivationIdentifier::root("[ContextMenu]"),
            );

            // TODO: This should use a pointed display object with `.menu`
            let menu_object = {
                let dobj = activation.context.stage.root_clip();
                if let Value::Object(obj) = dobj.object() {
                    if let Ok(Value::Object(menu)) = obj.get("menu", &mut activation) {
                        Some(menu)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(menu) = menu_object {
                if let Ok(Value::Object(on_select)) = menu.get("onSelect", &mut activation) {
                    Self::run_context_menu_custom_callback(
                        menu,
                        on_select,
                        &mut activation.context,
                    );
                }
            }

            let menu = crate::avm1::globals::context_menu::make_context_menu_state(
                menu_object,
                &mut activation,
            );
            let ret = menu.info().clone();
            *activation.context.current_context_menu = Some(menu);
            ret
        })
    }

    pub fn clear_custom_menu_items(&mut self) {
        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut root_data = gc_root.0.write(gc_context);
            root_data.current_context_menu = None;
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
                    _ => {}
                }
                Self::run_actions(context);
            }
        });
    }

    fn run_context_menu_custom_callback<'gc>(
        item: Object<'gc>,
        callback: Object<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        let globals = context.avm1.global_object_cell();
        let root_clip = context.stage.root_clip();

        let mut activation = Activation::from_nothing(
            context.reborrow(),
            ActivationIdentifier::root("[Context Menu Callback]"),
            globals,
            root_clip,
        );

        // TODO: Remember to also change the first arg
        // when we support contextmenu on non-root-movie
        let params = vec![root_clip.object(), Value::Object(item)];

        let _ = callback.call(
            "[Context Menu Callback]".into(),
            &mut activation,
            Value::Undefined,
            &params,
        );
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

    fn toggle_play_root_movie<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
        if let Some(mc) = context.stage.root_clip().as_movie_clip() {
            if mc.playing() {
                mc.stop(context);
            } else {
                mc.play(context);
            }
        }
    }
    fn rewind_root_movie<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
        if let Some(mc) = context.stage.root_clip().as_movie_clip() {
            mc.goto_frame(context, 1, true)
        }
    }
    fn forward_root_movie<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
        if let Some(mc) = context.stage.root_clip().as_movie_clip() {
            mc.next_frame(context);
        }
    }
    fn back_root_movie<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
        if let Some(mc) = context.stage.root_clip().as_movie_clip() {
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

    pub fn viewport_dimensions(&mut self) -> (u32, u32) {
        self.mutate_with_update_context(|context| context.stage.viewport_size())
    }

    pub fn set_viewport_dimensions(&mut self, width: u32, height: u32, scale_factor: f64) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            stage.set_viewport_size(context, width, height, scale_factor);
        })
    }

    pub fn set_show_menu(&mut self, show_menu: bool) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            stage.set_show_menu(context, show_menu);
        })
    }

    pub fn set_stage_align(&mut self, stage_align: &str) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            if let Ok(stage_align) = StageAlign::from_str(stage_align) {
                stage.set_align(context, stage_align);
            }
        })
    }

    pub fn set_quality(&mut self, quality: &str) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            if let Ok(quality) = StageQuality::from_str(quality) {
                stage.set_quality(context.gc_context, quality);
            }
        })
    }

    pub fn set_scale_mode(&mut self, scale_mode: &str) {
        self.mutate_with_update_context(|context| {
            let stage = context.stage;
            if let Ok(scale_mode) = StageScaleMode::from_str(scale_mode) {
                stage.set_scale_mode(context, scale_mode);
            }
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

    /// Handle an event sent into the player from the external windowing system
    /// or an HTML element.
    ///
    /// Event handling is a complicated affair, involving several different
    /// concerns that need to resolve with specific priority.
    ///
    /// 1. (In `avm_debug` builds) If Ctrl-Alt-V is pressed, dump all AVM1
    ///    variables in the player.
    /// 2. (In `avm_debug` builds) If Ctrl-Alt-D is pressed, toggle debug
    ///    output for AVM1 and AVM2.
    /// 3. If the incoming event is text input or key input that could be
    ///    related to text input (e.g. pressing a letter key), we dispatch a
    ///    key press event onto the stage.
    /// 4. If the event from step 3 was not handled, we check if an `EditText`
    ///    object is in focus and dispatch a text-control event to said object.
    /// 5. If the incoming event is text input, and neither step 3 nor step 4
    ///    resulted in an event being handled, we dispatch a text input event
    ///    to the currently focused `EditText` (if present).
    /// 6. Regardless of all prior event handling, we dispatch the event
    ///    through the stage normally.
    /// 7. Then, we dispatch the event through AVM1 global listener objects.
    /// 8. The AVM1 action queue is drained.
    /// 9. Mouse state is updated. This triggers button rollovers, which are a
    ///    second wave of event processing.
    pub fn handle_event(&mut self, event: PlayerEvent) {
        let prev_is_mouse_down = self.input.is_mouse_down();
        self.input.handle_event(&event);
        let is_mouse_button_changed = self.input.is_mouse_down() != prev_is_mouse_down;

        if cfg!(feature = "avm_debug") {
            match event {
                PlayerEvent::KeyDown {
                    key_code: KeyCode::V,
                    ..
                } if self.input.is_key_down(KeyCode::Control)
                    && self.input.is_key_down(KeyCode::Alt) =>
                {
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
                PlayerEvent::KeyDown {
                    key_code: KeyCode::D,
                    ..
                } if self.input.is_key_down(KeyCode::Control)
                    && self.input.is_key_down(KeyCode::Alt) =>
                {
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
                _ => {}
            }
        }

        self.mutate_with_update_context(|context| {
            // Propagate button events.
            let button_event = match event {
                // ASCII characters convert directly to keyPress button events.
                PlayerEvent::TextInput { codepoint }
                    if codepoint as u32 >= 32 && codepoint as u32 <= 126 =>
                {
                    Some(ClipEvent::KeyPress {
                        key_code: ButtonKeyCode::from_u8(codepoint as u8).unwrap(),
                    })
                }

                // Special keys have custom values for keyPress.
                PlayerEvent::KeyDown { key_code, .. } => {
                    if let Some(key_code) = crate::events::key_code_to_button_key_code(key_code) {
                        Some(ClipEvent::KeyPress { key_code })
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let mut key_press_handled = false;
            if let Some(button_event) = button_event {
                let levels: Vec<_> = context.stage.iter_depth_list().collect();
                for (_depth, level) in levels {
                    let state = if let Some(interactive) = level.as_interactive() {
                        interactive.handle_clip_event(context, button_event)
                    } else {
                        ClipEventResult::NotHandled
                    };

                    if state == ClipEventResult::Handled {
                        key_press_handled = true;
                        break;
                    } else if let Some(text) =
                        context.focus_tracker.get().and_then(|o| o.as_edit_text())
                    {
                        // Text fields listen for arrow key presses, etc.
                        if text.handle_text_control_event(context, button_event)
                            == ClipEventResult::Handled
                        {
                            key_press_handled = true;
                            break;
                        }
                    }
                }
            }

            // keyPress events take precedence over text input.
            if !key_press_handled {
                if let PlayerEvent::TextInput { codepoint } = event {
                    if let Some(text) = context.focus_tracker.get().and_then(|o| o.as_edit_text()) {
                        text.text_input(codepoint, context);
                    }
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
                _ => (None, None),
            };

            // Fire clip event on all clips.
            if let Some(clip_event) = clip_event {
                let levels: Vec<_> = context.stage.iter_depth_list().collect();
                for (_depth, level) in levels {
                    if let Some(interactive) = level.as_interactive() {
                        interactive.handle_clip_event(context, clip_event);
                    }
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

            Self::run_actions(context);
        });

        // Update mouse state.
        if let PlayerEvent::MouseMove { x, y }
        | PlayerEvent::MouseDown {
            x,
            y,
            button: MouseButton::Left,
        }
        | PlayerEvent::MouseUp {
            x,
            y,
            button: MouseButton::Left,
        } = event
        {
            let inverse_view_matrix =
                self.mutate_with_update_context(|context| context.stage.inverse_view_matrix());
            let old_pos = self.mouse_pos;
            self.mouse_pos = inverse_view_matrix * (Twips::from_pixels(x), Twips::from_pixels(y));

            // Update the dragged object here to keep it constantly in sync with the mouse position.
            self.update_drag();

            let is_mouse_moved = old_pos != self.mouse_pos;

            // This fires button rollover/press events, which should run after the above mouseMove events.
            if self.update_mouse_state(is_mouse_button_changed, is_mouse_moved) {
                self.needs_render = true;
            }
        }

        if let PlayerEvent::MouseWheel { delta } = event {
            self.mutate_with_update_context(|context| {
                if let Some(over_object) = context.mouse_over_object {
                    if !over_object.as_displayobject().removed() {
                        over_object.handle_clip_event(context, ClipEvent::MouseWheel { delta });
                    }
                } else {
                    context
                        .stage
                        .handle_clip_event(context, ClipEvent::MouseWheel { delta });
                }
            });
        }
    }

    /// Update dragged object, if any.
    fn update_drag(&mut self) {
        let (mouse_x, mouse_y) = self.mouse_pos;
        self.mutate_with_update_context(|context| {
            if let Some(drag_object) = &mut context.drag_object {
                let display_object = drag_object.display_object;
                if drag_object.display_object.removed() {
                    // Be sure to clear the drag if the object was removed.
                    *context.drag_object = None;
                } else {
                    let (offset_x, offset_y) = drag_object.offset;
                    let mut drag_point = (mouse_x + offset_x, mouse_y + offset_y);
                    if let Some(parent) = display_object.parent() {
                        drag_point = parent.global_to_local(drag_point);
                    }
                    drag_point = drag_object.constraint.clamp(drag_point);
                    display_object.set_x(context.gc_context, drag_point.0.to_pixels());
                    display_object.set_y(context.gc_context, drag_point.1.to_pixels());

                    // Update _droptarget property of dragged object.
                    if let Some(movie_clip) = display_object.as_movie_clip() {
                        // Turn the dragged object invisible so that we don't pick it.
                        // TODO: This could be handled via adding a `HitTestOptions::SKIP_DRAGGED`.
                        let was_visible = display_object.visible();
                        display_object.set_visible(context.gc_context, false);
                        // Set _droptarget to the object the mouse is hovering over.
                        let drop_target_object =
                            context
                                .stage
                                .iter_depth_list()
                                .rev()
                                .find_map(|(_depth, level)| {
                                    level.as_interactive().and_then(|l| {
                                        l.mouse_pick(context, *context.mouse_position, false)
                                    })
                                });
                        movie_clip.set_drop_target(
                            context.gc_context,
                            drop_target_object.map(|d| d.as_displayobject()),
                        );
                        display_object.set_visible(context.gc_context, was_visible);
                    }
                }
            }
        });
    }

    /// Updates the hover state of buttons.
    fn update_mouse_state(&mut self, is_mouse_button_changed: bool, is_mouse_moved: bool) -> bool {
        let mut new_cursor = self.mouse_cursor;
        let mut mouse_cursor_needs_check = self.mouse_cursor_needs_check;

        // Determine the display object the mouse is hovering over.
        // Search through levels from top-to-bottom, returning the first display object that is under the mouse.
        let needs_render = self.mutate_with_update_context(|context| {
            let new_over_object =
                context
                    .stage
                    .iter_depth_list()
                    .rev()
                    .find_map(|(_depth, level)| {
                        level
                            .as_interactive()
                            .and_then(|l| l.mouse_pick(context, *context.mouse_position, true))
                    });

            let mut events: smallvec::SmallVec<[(InteractiveObject<'_>, ClipEvent); 2]> =
                Default::default();

            if is_mouse_moved {
                events.push((
                    new_over_object.unwrap_or_else(|| context.stage.into()),
                    ClipEvent::MouseMoveInside,
                ));
            }

            // Cancel hover if an object is removed from the stage.
            if let Some(hovered) = context.mouse_over_object {
                if hovered.as_displayobject().removed() {
                    context.mouse_over_object = None;
                }
            }
            if let Some(pressed) = context.mouse_down_object {
                if pressed.as_displayobject().removed() {
                    context.mouse_down_object = None;
                }
            }

            // Update the cursor if the object was removed from the stage.
            if new_cursor != MouseCursor::Arrow {
                let object_removed =
                    context.mouse_over_object.is_none() && context.mouse_down_object.is_none();
                if !object_removed {
                    mouse_cursor_needs_check = false;
                    if is_mouse_button_changed {
                        // The object is pressed/released and may be removed immediately, we need to check
                        // in the next frame if it still exists. If it doesn't, we'll update the cursor.
                        mouse_cursor_needs_check = true;
                    }
                } else if mouse_cursor_needs_check {
                    mouse_cursor_needs_check = false;
                    new_cursor = MouseCursor::Arrow;
                } else if !context.input.is_mouse_down()
                    && (is_mouse_moved || is_mouse_button_changed)
                {
                    // In every other case, the cursor remains until the user interacts with the mouse again.
                    new_cursor = MouseCursor::Arrow;
                }
            } else {
                mouse_cursor_needs_check = false;
            }

            let cur_over_object = context.mouse_over_object;
            // Check if a new object has been hovered over.
            if !InteractiveObject::option_ptr_eq(cur_over_object, new_over_object) {
                // If the mouse button is down, the object the user clicked on grabs the focus
                // and fires "drag" events. Other objects are ignored.
                if context.input.is_mouse_down() {
                    context.mouse_over_object = new_over_object;
                    if let Some(down_object) = context.mouse_down_object {
                        if InteractiveObject::option_ptr_eq(
                            context.mouse_down_object,
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
                            context.mouse_down_object,
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
            context.mouse_over_object = new_over_object;

            // Handle presses and releases.
            if is_mouse_button_changed {
                if context.input.is_mouse_down() {
                    // Pressed on a hovered object.
                    if let Some(over_object) = context.mouse_over_object {
                        events.push((over_object, ClipEvent::Press));
                        context.mouse_down_object = context.mouse_over_object;
                    } else {
                        events.push((context.stage.into(), ClipEvent::Press));
                    }
                } else {
                    if let Some(over_object) = context.mouse_over_object {
                        events.push((over_object, ClipEvent::MouseUpInside));
                    } else {
                        events.push((context.stage.into(), ClipEvent::MouseUpInside));
                    }

                    let released_inside = InteractiveObject::option_ptr_eq(
                        context.mouse_down_object,
                        context.mouse_over_object,
                    );
                    if released_inside {
                        // Released inside the clicked object.
                        if let Some(down_object) = context.mouse_down_object {
                            new_cursor = down_object.mouse_cursor(context);
                            events.push((down_object, ClipEvent::Release));
                        } else {
                            events.push((context.stage.into(), ClipEvent::Release));
                        }
                    } else {
                        // Released outside the clicked object.
                        if let Some(down_object) = context.mouse_down_object {
                            events.push((down_object, ClipEvent::ReleaseOutside));
                        } else {
                            events.push((context.stage.into(), ClipEvent::ReleaseOutside));
                        }
                        // The new object is rolled over immediately.
                        if let Some(over_object) = context.mouse_over_object {
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
                    context.mouse_down_object = None;
                }
            }

            // Fire any pending mouse events.
            let needs_render = if events.is_empty() {
                false
            } else {
                let mut refresh = false;
                for (object, event) in events {
                    let display_object = object.as_displayobject();
                    if !display_object.removed() {
                        object.handle_clip_event(context, event);
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

    /// Preload the first movie in the player.
    ///
    /// This should only be called once. Further movie loads should preload the
    /// specific `MovieClip` referenced.
    fn preload(&mut self) {
        self.mutate_with_update_context(|context| {
            let root = context.stage.root_clip();
            root.as_movie_clip().unwrap().preload(context);
        });
        if self.swf.avm_type() == AvmType::Avm2 && self.warn_on_unsupported_content {
            self.ui.display_unsupported_message();
        }
    }

    pub fn run_frame(&mut self) {
        self.update(|context| {
            let stage = context.stage;
            match context.swf.avm_type() {
                AvmType::Avm1 => {
                    // AVM1 execution order is determined by the global execution list, based on instantiation order.
                    for clip in context.avm1.clip_exec_iter() {
                        if clip.removed() {
                            // Clean up removed objects from this frame or a previous frame.
                            // Can be safely removed while iterating here, because the iterator advances
                            // to the next node before returning the current node.
                            context.avm1.remove_from_exec_list(context.gc_context, clip);
                        } else {
                            clip.run_frame(context);
                        }
                    }

                    // Fire "onLoadInit" events.
                    context
                        .load_manager
                        .movie_clip_on_load(context.action_queue);
                }
                AvmType::Avm2 => {
                    stage.exit_frame(context);
                    stage.enter_frame(context);
                    stage.construct_frame(context);
                    stage.frame_constructed(context);
                    stage.run_frame_avm2(context);
                    stage.run_frame_scripts(context);
                }
            }
            context.update_sounds();
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

    pub fn navigator(&self) -> &Navigator {
        &self.navigator
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

    pub fn run_actions<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
        // Note that actions can queue further actions, so a while loop is necessary here.
        while let Some(actions) = context.action_queue.pop_action() {
            // We don't run frame actions if the clip was removed after it queued the action.
            if !actions.is_unload && actions.clip.removed() {
                continue;
            }

            match actions.action_type {
                // DoAction/clip event code.
                ActionType::Normal { bytecode } | ActionType::Initialize { bytecode } => {
                    Avm1::run_stack_frame_for_action(actions.clip, "[Frame]", bytecode, context);
                }
                // Change the prototype of a MovieClip and run constructor events.
                ActionType::Construct {
                    constructor: Some(constructor),
                    events,
                } => {
                    let globals = context.avm1.global_object_cell();

                    let mut activation = Activation::from_nothing(
                        context.reborrow(),
                        ActivationIdentifier::root("[Construct]"),
                        globals,
                        actions.clip,
                    );
                    if let Ok(prototype) = constructor.get("prototype", &mut activation) {
                        if let Value::Object(object) = actions.clip.object() {
                            object.define_value(
                                activation.context.gc_context,
                                "__proto__",
                                prototype,
                                Attribute::empty(),
                            );
                            for event in events {
                                let _ = activation.run_child_frame_for_action(
                                    "[Actions]",
                                    actions.clip,
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
                            actions.clip,
                            "[Construct]",
                            event,
                            context,
                        );
                    }
                }
                // Event handler method call (e.g. onEnterFrame).
                ActionType::Method { object, name, args } => {
                    Avm1::run_stack_frame_for_method(
                        actions.clip,
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
                        actions.clip,
                        context,
                        listener.into(),
                        method.into(),
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

                ActionType::Event2 { event, target } => {
                    if let Err(e) = Avm2::dispatch_event(context, event, target) {
                        log::error!("Unhandled AVM2 exception in event handler: {}", e);
                    }
                }
            }
        }
    }

    /// Runs the closure `f` with an `UpdateContext`.
    /// This takes cares of populating the `UpdateContext` struct, avoiding borrow issues.
    pub(crate) fn mutate_with_update_context<F, R>(&mut self, f: F) -> R
    where
        F: for<'a, 'gc> FnOnce(&mut UpdateContext<'a, 'gc, '_>) -> R,
    {
        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut root_data = gc_root.0.write(gc_context);
            let mouse_hovered_object = root_data.mouse_hovered_object;
            let mouse_pressed_object = root_data.mouse_pressed_object;
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
                current_context_menu,
                external_interface,
                audio_manager,
            ) = root_data.update_context_params();

            let mut update_context = UpdateContext {
                player_version: self.player_version,
                swf: &self.swf,
                library,
                rng: &mut self.rng,
                renderer: self.renderer.deref_mut(),
                audio: self.audio.deref_mut(),
                navigator: self.navigator.deref_mut(),
                ui: self.ui.deref_mut(),
                action_queue,
                gc_context,
                stage,
                mouse_over_object: mouse_hovered_object,
                mouse_down_object: mouse_pressed_object,
                input: &self.input,
                mouse_position: &self.mouse_pos,
                drag_object,
                player: self.self_reference.clone(),
                load_manager,
                system: &mut self.system,
                instance_counter: &mut self.instance_counter,
                storage: self.storage.deref_mut(),
                log: self.log.deref_mut(),
                video: self.video.deref_mut(),
                shared_objects,
                unbound_text_fields,
                timers,
                current_context_menu,
                needs_render: &mut self.needs_render,
                avm1,
                avm2,
                external_interface,
                start_time: self.start_time,
                update_start: Instant::now(),
                max_execution_duration: self.max_execution_duration,
                focus_tracker,
                times_get_time_called: 0,
                time_offset: &mut self.time_offset,
                audio_manager,
                frame_rate: &mut self.frame_rate,
            };

            let old_frame_rate = *update_context.frame_rate;

            let ret = f(&mut update_context);

            let new_frame_rate = *update_context.frame_rate;

            // If we changed the framerate, let the audio handler now.
            #[allow(clippy::float_cmp)]
            if old_frame_rate != new_frame_rate {
                update_context.audio.set_frame_rate(new_frame_rate);
            }

            self.current_frame = update_context
                .stage
                .root_clip()
                .as_movie_clip()
                .map(|clip| clip.current_frame());

            // Hovered object may have been updated; copy it back to the GC root.
            let mouse_hovered_object = update_context.mouse_over_object;
            let mouse_pressed_object = update_context.mouse_down_object;
            root_data.mouse_hovered_object = mouse_hovered_object;
            root_data.mouse_pressed_object = mouse_pressed_object;

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
            reader.read_define_font_2(3)?,
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
        let rval = self.mutate_with_update_context(|context| {
            let rval = func(context);

            Self::run_actions(context);

            rval
        });

        // Update mouse state (check for new hovered button, etc.)
        self.update_drag();
        self.update_mouse_state(false, false);

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
    fullscreen: bool,
    letterbox: Letterbox,
    max_execution_duration: Duration,
    viewport_width: u32,
    viewport_height: u32,
    viewport_scale_factor: f64,
    warn_on_unsupported_content: bool,
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
            warn_on_unsupported_content: true,
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

    /// Sets the storage backend of the player.
    #[inline]
    pub fn with_storage(mut self, storage: impl 'static + StorageBackend) -> Self {
        self.storage = Some(Box::new(storage));
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

    /// Configures the player to use software video decoding.
    #[inline]
    pub fn with_software_video(mut self) -> Self {
        self.video = Some(Box::new(crate::backend::video::SoftwareVideoBackend::new()));
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

    /// Configures the player to warn if unsupported content is detected (ActionScript 3.0).
    #[inline]
    pub fn with_warn_on_unsupported_content(mut self, value: bool) -> Self {
        self.warn_on_unsupported_content = value;
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

    // Sets whether the stage is fullscreen.
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Builds the player, wiring up the backends and configuring the specified settings.
    pub fn build(self) -> Arc<Mutex<Player>> {
        use crate::backend::*;
        let audio = self
            .audio
            .unwrap_or_else(|| Box::new(audio::NullAudioBackend::new()));
        let log = self
            .log
            .unwrap_or_else(|| Box::new(log::NullLogBackend::new()));
        let navigator = self
            .navigator
            .unwrap_or_else(|| Box::new(navigator::NullNavigatorBackend::new()));
        let renderer = self
            .renderer
            .unwrap_or_else(|| Box::new(render::NullRenderer::new()));
        let storage = self
            .storage
            .unwrap_or_else(|| Box::new(storage::MemoryStorageBackend::new()));
        let ui = self
            .ui
            .unwrap_or_else(|| Box::new(ui::NullUiBackend::new()));
        let video = self
            .video
            .unwrap_or_else(|| Box::new(video::NullVideoBackend::new()));

        // Instantiate the player.
        let fake_movie = Arc::new(SwfMovie::empty(NEWEST_PLAYER_VERSION));
        let frame_rate = 12.0;
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
                frame_accumulator: 0.0,
                recent_run_frame_timings: VecDeque::with_capacity(10),
                start_time: Instant::now(),
                time_offset: 0,
                time_til_next_timer: None,
                max_execution_duration: self.max_execution_duration,

                // Input
                input: Default::default(),
                mouse_pos: (Twips::ZERO, Twips::ZERO),
                mouse_cursor: MouseCursor::Arrow,
                mouse_cursor_needs_check: false,

                // Misc. state
                rng: SmallRng::seed_from_u64(chrono::Utc::now().timestamp_millis() as u64),
                system: SystemProperties::default(),
                transform_stack: TransformStack::new(),
                instance_counter: 0,
                player_version: NEWEST_PLAYER_VERSION,
                is_playing: self.autoplay,
                needs_render: true,
                warn_on_unsupported_content: self.warn_on_unsupported_content,
                self_reference: self_ref.clone(),

                // GC data
                gc_arena: GcArena::new(ArenaParameters::default(), |gc_context| {
                    GcRoot(GcCell::allocate(
                        gc_context,
                        GcRootData {
                            audio_manager: AudioManager::new(),
                            action_queue: ActionQueue::new(),
                            avm1: Avm1::new(gc_context, NEWEST_PLAYER_VERSION),
                            avm2: Avm2::new(gc_context),
                            current_context_menu: None,
                            drag_object: None,
                            external_interface: ExternalInterface::new(),
                            focus_tracker: FocusTracker::new(gc_context),
                            library: Library::empty(),
                            load_manager: LoadManager::new(),
                            mouse_hovered_object: None,
                            mouse_pressed_object: None,
                            shared_objects: HashMap::new(),
                            stage: Stage::empty(
                                gc_context,
                                self.viewport_width,
                                self.viewport_height,
                                self.fullscreen,
                            ),
                            timers: Timers::new(),
                            unbound_text_fields: Vec::new(),
                        },
                    ))
                }),
            })
        });

        // Finalize configuration and load the movie.
        let mut player_lock = player.lock().unwrap();
        player_lock.mutate_with_update_context(|context| {
            // Instantiate an empty root before the main movie loads.
            let fake_root = MovieClip::from_movie(context.gc_context, fake_movie);
            fake_root.post_instantiation(context, None, Instantiator::Movie, false);
            context.stage.replace_at_depth(context, fake_root.into(), 0);
            Avm2::load_player_globals(context).expect("Unable to load AVM2 globals");
            let stage = context.stage;
            stage.post_instantiation(context, None, Instantiator::Movie, false);
            stage.build_matrices(context);
        });
        player_lock.audio.set_frame_rate(frame_rate);
        player_lock.set_letterbox(self.letterbox);
        player_lock.set_viewport_dimensions(
            self.viewport_width,
            self.viewport_height,
            self.viewport_scale_factor,
        );
        if let Some(movie) = self.movie {
            player_lock.set_root_movie(movie);
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

    /// The offset from the mouse position to the center of the clip.
    #[collect(require_static)]
    pub offset: (Twips, Twips),

    /// The bounding rectangle where the clip will be maintained.
    #[collect(require_static)]
    pub constraint: BoundingBox,
}
