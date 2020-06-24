use crate::avm1::debug::VariableDumper;
use crate::avm1::globals::system::SystemProperties;
use crate::avm1::listeners::SystemListener;
use crate::avm1::object::Object;
use crate::avm1::{Activation, Avm1, TObject, Value};
use crate::backend::input::{InputBackend, MouseCursor};
use crate::backend::storage::StorageBackend;
use crate::backend::{
    audio::AudioBackend, navigator::NavigatorBackend, render::Letterbox, render::RenderBackend,
};
use crate::context::{ActionQueue, ActionType, RenderContext, UpdateContext};
use crate::display_object::{MorphShape, MovieClip};
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult, KeyCode, PlayerEvent};
use crate::library::Library;
use crate::loader::LoadManager;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::transform::TransformStack;
use enumset::EnumSet;
use gc_arena::{make_arena, ArenaParameters, Collect, GcCell};
use log::info;
use rand::{rngs::SmallRng, SeedableRng};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, Weak};

static DEVICE_FONT_TAG: &[u8] = include_bytes!("../assets/noto-sans-definefont3.bin");

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

    /// The list of levels on the current stage.
    ///
    /// Each level is a `_root` MovieClip that holds a particular SWF movie, also accessible via
    /// the `_levelN` property.
    /// levels[0] represents the initial SWF file that was loaded.
    levels: BTreeMap<u32, DisplayObject<'gc>>,

    mouse_hovered_object: Option<DisplayObject<'gc>>, // TODO: Remove GcCell wrapped inside GcCell.

    /// The object being dragged via a `startDrag` action.
    drag_object: Option<DragObject<'gc>>,

    avm: Avm1<'gc>,
    action_queue: ActionQueue<'gc>,

    /// Object which manages asynchronous processes that need to interact with
    /// data in the GC arena.
    load_manager: LoadManager<'gc>,

    shared_objects: HashMap<String, Object<'gc>>,
}

impl<'gc> GcRootData<'gc> {
    /// Splits out parameters for creating an `UpdateContext`
    /// (because we can borrow fields of `self` independently)
    #[allow(clippy::type_complexity)]
    fn update_context_params(
        &mut self,
    ) -> (
        &mut BTreeMap<u32, DisplayObject<'gc>>,
        &mut Library<'gc>,
        &mut ActionQueue<'gc>,
        &mut Avm1<'gc>,
        &mut Option<DragObject<'gc>>,
        &mut LoadManager<'gc>,
        &mut HashMap<String, Object<'gc>>,
    ) {
        (
            &mut self.levels,
            &mut self.library,
            &mut self.action_queue,
            &mut self.avm,
            &mut self.drag_object,
            &mut self.load_manager,
            &mut self.shared_objects,
        )
    }
}
type Error = Box<dyn std::error::Error>;

make_arena!(GcArena, GcRoot);

type Audio = Box<dyn AudioBackend>;
type Navigator = Box<dyn NavigatorBackend>;
type Renderer = Box<dyn RenderBackend>;
type Input = Box<dyn InputBackend>;
type Storage = Box<dyn StorageBackend>;

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

    is_playing: bool,
    needs_render: bool,

    audio: Audio,
    renderer: Renderer,
    pub navigator: Navigator,
    input: Input,
    transform_stack: TransformStack,
    view_matrix: Matrix,
    inverse_view_matrix: Matrix,

    storage: Storage,

    rng: SmallRng,

    gc_arena: GcArena,
    background_color: Color,

    frame_rate: f64,
    frame_accumulator: f64,
    global_time: u64,

    viewport_width: u32,
    viewport_height: u32,
    movie_width: u32,
    movie_height: u32,
    letterbox: Letterbox,

    mouse_pos: (Twips, Twips),
    is_mouse_down: bool,

    /// The current mouse cursor icon.
    mouse_cursor: MouseCursor,

    system: SystemProperties,

    /// The current instance ID. Used to generate default `instanceN` names.
    instance_counter: i32,

    /// Self-reference to ourselves.
    ///
    /// This is a weak reference that is upgraded and handed out in various
    /// contexts to other parts of the player. It can be used to ensure the
    /// player lives across `await` calls in async code.
    self_reference: Option<Weak<Mutex<Self>>>,
}

impl Player {
    pub fn new(
        mut renderer: Renderer,
        audio: Audio,
        navigator: Navigator,
        input: Input,
        movie: SwfMovie,
        storage: Storage,
    ) -> Result<Arc<Mutex<Self>>, Error> {
        let movie = Arc::new(movie);

        info!(
            "Loaded SWF version {}, with a resolution of {}x{}",
            movie.header().version,
            movie.header().stage_size.x_max,
            movie.header().stage_size.y_max
        );

        let movie_width = movie.width();
        let movie_height = movie.height();

        let mut player = Player {
            player_version: NEWEST_PLAYER_VERSION,

            swf: movie.clone(),

            is_playing: false,
            needs_render: true,

            background_color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            transform_stack: TransformStack::new(),
            view_matrix: Default::default(),
            inverse_view_matrix: Default::default(),

            rng: SmallRng::from_seed([0u8; 16]), // TODO(Herschel): Get a proper seed on all platforms.

            gc_arena: GcArena::new(ArenaParameters::default(), |gc_context| {
                // Load and parse the device font.
                let device_font =
                    match Self::load_device_font(gc_context, DEVICE_FONT_TAG, &mut renderer) {
                        Ok(font) => Some(font),
                        Err(e) => {
                            log::error!("Unable to load device font: {}", e);
                            None
                        }
                    };

                let mut library = Library::default();

                library
                    .library_for_movie_mut(movie.clone())
                    .set_device_font(device_font);

                GcRoot(GcCell::allocate(
                    gc_context,
                    GcRootData {
                        library,
                        levels: BTreeMap::new(),
                        mouse_hovered_object: None,
                        drag_object: None,
                        avm: Avm1::new(gc_context, NEWEST_PLAYER_VERSION),
                        action_queue: ActionQueue::new(),
                        load_manager: LoadManager::new(),
                        shared_objects: HashMap::new(),
                    },
                ))
            }),

            frame_rate: movie.header().frame_rate.into(),
            frame_accumulator: 0.0,
            global_time: 0,

            movie_width,
            movie_height,
            viewport_width: movie_width,
            viewport_height: movie_height,
            letterbox: Letterbox::None,

            mouse_pos: (Twips::new(0), Twips::new(0)),
            is_mouse_down: false,
            mouse_cursor: MouseCursor::Arrow,

            renderer,
            audio,
            navigator,
            input,
            self_reference: None,
            system: SystemProperties::default(),
            instance_counter: 0,
            storage,
        };

        player.mutate_with_update_context(|avm, context| {
            let mut root: DisplayObject =
                MovieClip::from_movie(context.gc_context, movie.clone()).into();
            root.set_depth(context.gc_context, 0);
            root.post_instantiation(avm, context, root, None, false);
            root.set_name(context.gc_context, "");
            context.levels.insert(0, root);

            let object = root.object().coerce_to_object(avm, context);
            object.define_value(
                context.gc_context,
                "$version",
                context.system.get_version_string(avm).into(),
                EnumSet::empty(),
            );
        });

        player.build_matrices();
        player.preload();

        let player_box = Arc::new(Mutex::new(player));
        let mut player_lock = player_box.lock().unwrap();
        player_lock.self_reference = Some(Arc::downgrade(&player_box));
        std::mem::drop(player_lock);

        Ok(player_box)
    }

    pub fn tick(&mut self, dt: f64) {
        // Don't run until preloading is complete.
        // TODO: Eventually we want to stream content similar to the Flash player.
        if !self.audio.is_loading_complete() {
            return;
        }

        if self.is_playing() {
            self.frame_accumulator += dt;
            self.global_time += dt as u64;
            let frame_time = 1000.0 / self.frame_rate;

            const MAX_FRAMES_PER_TICK: u32 = 5; // Sanity cap on frame tick.
            let mut frame = 0;
            while frame < MAX_FRAMES_PER_TICK && self.frame_accumulator >= frame_time {
                self.frame_accumulator -= frame_time;
                self.run_frame();
                frame += 1;
            }

            // Sanity: If we had too many frames to tick, just reset the accumulator
            // to prevent running at turbo speed.
            if self.frame_accumulator >= frame_time {
                self.frame_accumulator = 0.0;
            }

            self.audio.tick();
        }
    }

    /// Returns the approximate duration of time until the next frame is due to run.
    /// This is only an approximation to be used for sleep durations.
    pub fn time_til_next_frame(&self) -> std::time::Duration {
        let frame_time = 1000.0 / self.frame_rate;
        let dt = if self.frame_accumulator <= 0.0 {
            frame_time
        } else if self.frame_accumulator >= frame_time {
            0.0
        } else {
            frame_time - self.frame_accumulator
        };
        std::time::Duration::from_micros(dt as u64 * 1000)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn set_is_playing(&mut self, v: bool) {
        if v {
            // Allow auto-play after user gesture for web backends.
            self.audio.prime_audio();
        }
        self.is_playing = v;
    }

    pub fn needs_render(&self) -> bool {
        self.needs_render
    }

    pub fn movie_width(&self) -> u32 {
        self.movie_width
    }

    pub fn movie_height(&self) -> u32 {
        self.movie_height
    }

    pub fn viewport_dimensions(&self) -> (u32, u32) {
        (self.viewport_width, self.viewport_height)
    }

    pub fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.build_matrices();
    }

    pub fn handle_event(&mut self, event: PlayerEvent) {
        let mut needs_render = self.needs_render;

        if let PlayerEvent::KeyDown {
            key_code: KeyCode::V,
        } = event
        {
            if self.input.is_key_down(KeyCode::Control) && self.input.is_key_down(KeyCode::Alt) {
                self.mutate_with_update_context(|avm, context| {
                    let mut dumper = VariableDumper::new("  ");
                    dumper.print_variables(
                        "Global Variables:",
                        "_global",
                        &avm.global_object_cell(),
                        avm,
                        context,
                    );
                    let levels = context.levels.clone();
                    for (level, display_object) in levels {
                        let object = display_object.object().coerce_to_object(avm, context);
                        dumper.print_variables(
                            &format!("Level #{}:", level),
                            &format!("_level{}", level),
                            &object,
                            avm,
                            context,
                        );
                    }
                    log::info!("Variable dump:\n{}", dumper.output());
                });
            }
        }

        // Update mouse position from mouse events.
        if let PlayerEvent::MouseMove { x, y }
        | PlayerEvent::MouseDown { x, y }
        | PlayerEvent::MouseUp { x, y } = event
        {
            self.mouse_pos =
                self.inverse_view_matrix * (Twips::from_pixels(x), Twips::from_pixels(y));
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
            self.mutate_with_update_context(|avm, context| {
                let levels: Vec<DisplayObject<'_>> = context.levels.values().copied().collect();
                for level in levels {
                    if let Some(button_event) = button_event {
                        let state = level.handle_clip_event(avm, context, button_event);
                        if state == ClipEventResult::Handled {
                            return;
                        }
                    }
                }
            });
        }

        // Propagte clip events.
        let (clip_event, mouse_event_name) = match event {
            PlayerEvent::KeyDown { .. } => (Some(ClipEvent::KeyDown), Some("onKeyDown")),
            PlayerEvent::KeyUp { .. } => (Some(ClipEvent::KeyUp), Some("onKeyUp")),
            PlayerEvent::MouseMove { .. } => (Some(ClipEvent::MouseMove), Some("onMouseMove")),
            PlayerEvent::MouseUp { .. } => (Some(ClipEvent::MouseUp), Some("onMouseUp")),
            PlayerEvent::MouseDown { .. } => (Some(ClipEvent::MouseDown), Some("onMouseDown")),
            _ => (None, None),
        };

        if clip_event.is_some() || mouse_event_name.is_some() {
            self.mutate_with_update_context(|avm, context| {
                let levels: Vec<DisplayObject<'_>> = context.levels.values().copied().collect();

                for level in levels {
                    if let Some(clip_event) = clip_event {
                        level.handle_clip_event(avm, context, clip_event);
                    }
                }

                if let Some(mouse_event_name) = mouse_event_name {
                    context.action_queue.queue_actions(
                        *context.levels.get(&0).expect("root level"),
                        ActionType::NotifyListeners {
                            listener: SystemListener::Mouse,
                            method: mouse_event_name,
                            args: vec![],
                        },
                        false,
                    );
                }
            });
        }

        let mut is_mouse_down = self.is_mouse_down;
        self.mutate_with_update_context(|avm, context| {
            if let Some(node) = context.mouse_hovered_object {
                match event {
                    PlayerEvent::MouseDown { .. } => {
                        is_mouse_down = true;
                        needs_render = true;
                        node.handle_clip_event(avm, context, ClipEvent::Press);
                    }

                    PlayerEvent::MouseUp { .. } => {
                        is_mouse_down = false;
                        needs_render = true;
                        node.handle_clip_event(avm, context, ClipEvent::Release);
                    }

                    _ => (),
                }
            }

            Self::run_actions(avm, context);
        });
        self.is_mouse_down = is_mouse_down;
        self.needs_render = needs_render;
    }

    /// Update dragged object, if any.
    fn update_drag(&mut self) {
        let mouse_pos = self.mouse_pos;
        self.mutate_with_update_context(|_avm, context| {
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
        let hover_changed = self.mutate_with_update_context(|avm, context| {
            // Check hovered object.
            let mut new_hovered = None;
            for (_depth, level) in context.levels.clone().iter().rev() {
                if new_hovered.is_none() {
                    new_hovered =
                        level.mouse_pick(avm, context, *level, (mouse_pos.0, mouse_pos.1));
                } else {
                    break;
                }
            }

            let cur_hovered = context.mouse_hovered_object;

            if cur_hovered.map(|d| d.as_ptr()) != new_hovered.map(|d| d.as_ptr()) {
                // RollOut of previous node.
                if let Some(node) = cur_hovered {
                    node.handle_clip_event(avm, context, ClipEvent::RollOut);
                }

                // RollOver on new node.
                new_cursor = MouseCursor::Arrow;
                if let Some(node) = new_hovered {
                    new_cursor = MouseCursor::Hand;
                    node.handle_clip_event(avm, context, ClipEvent::RollOver);
                }

                context.mouse_hovered_object = new_hovered;

                Self::run_actions(avm, context);
                true
            } else {
                false
            }
        });

        // Update mouse cursor if it has changed.
        if new_cursor != self.mouse_cursor {
            self.mouse_cursor = new_cursor;
            self.input.set_mouse_cursor(new_cursor)
        }

        hover_changed
    }

    /// Preload the first movie in the player.
    ///
    /// This should only be called once. Further movie loads should preload the
    /// specific `MovieClip` referenced.
    fn preload(&mut self) {
        self.mutate_with_update_context(|avm, context| {
            let mut morph_shapes = fnv::FnvHashMap::default();
            let root = *context.levels.get(&0).expect("root level");
            root.as_movie_clip()
                .unwrap()
                .preload(avm, context, &mut morph_shapes);

            // Finalize morph shapes.
            for (id, static_data) in morph_shapes {
                let morph_shape = MorphShape::new(context.gc_context, static_data);
                context
                    .library
                    .library_for_movie_mut(root.as_movie_clip().unwrap().movie().unwrap())
                    .register_character(id, crate::character::Character::MorphShape(morph_shape));
            }
        });
    }

    pub fn run_frame(&mut self) {
        self.update(|avm, update_context| {
            // TODO: In what order are levels run?
            // NOTE: We have to copy all the layer pointers into a separate list
            // because level updates can create more levels, which we don't
            // want to run frames on
            let levels: Vec<_> = update_context.levels.values().copied().collect();

            for mut level in levels {
                level.run_frame(avm, update_context);
            }
        });
        self.needs_render = true;
    }

    pub fn render(&mut self) {
        let view_bounds = BoundingBox {
            x_min: Twips::new(0),
            y_min: Twips::new(0),
            x_max: Twips::from_pixels(self.movie_width.into()),
            y_max: Twips::from_pixels(self.movie_height.into()),
            valid: true,
        };

        self.renderer.begin_frame(self.background_color.clone());

        let (renderer, transform_stack) = (&mut self.renderer, &mut self.transform_stack);

        transform_stack.push(&crate::transform::Transform {
            matrix: self.view_matrix,
            ..Default::default()
        });
        self.gc_arena.mutate(|_gc_context, gc_root| {
            let root_data = gc_root.0.read();
            let mut render_context = RenderContext {
                renderer: renderer.deref_mut(),
                library: &root_data.library,
                transform_stack,
                view_bounds,
                clip_depth_stack: vec![],
            };

            for (_depth, level) in root_data.levels.iter() {
                level.render(&mut render_context);
            }
        });
        transform_stack.pop();

        self.renderer.draw_letterbox(self.letterbox);
        self.renderer.end_frame();
        self.needs_render = false;
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

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut dyn InputBackend {
        self.input.deref_mut()
    }

    fn run_actions<'gc>(avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>) {
        // Note that actions can queue further actions, so a while loop is necessary here.
        while let Some(actions) = context.action_queue.pop_action() {
            // We don't run frame actions if the clip was removed after it queued the action.
            if !actions.is_unload && actions.clip.removed() {
                continue;
            }

            match actions.action_type {
                // DoAction/clip event code
                ActionType::Normal { bytecode } => {
                    avm.insert_stack_frame_for_action(
                        actions.clip,
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
                    avm.insert_stack_frame(GcCell::allocate(
                        context.gc_context,
                        Activation::from_nothing(
                            context.swf.header().version,
                            avm.global_object_cell(),
                            context.gc_context,
                            actions.clip,
                        ),
                    ));
                    if let Ok(prototype) = constructor
                        .get("prototype", avm, context)
                        .map(|v| v.coerce_to_object(avm, context))
                    {
                        if let Value::Object(object) = actions.clip.object() {
                            object.set_proto(context.gc_context, Some(prototype));
                            for event in events {
                                avm.insert_stack_frame_for_action(
                                    actions.clip,
                                    context.swf.header().version,
                                    event,
                                    context,
                                );
                            }
                            let _ = avm.run_stack_till_empty(context);

                            avm.insert_stack_frame(GcCell::allocate(
                                context.gc_context,
                                Activation::from_nothing(
                                    context.swf.header().version,
                                    avm.global_object_cell(),
                                    context.gc_context,
                                    actions.clip,
                                ),
                            ));
                            let _ = constructor.call(avm, context, object, None, &[]);
                        }
                    }
                }
                // Run constructor events without changing the prototype
                ActionType::Construct {
                    constructor: None,
                    events,
                } => {
                    for event in events {
                        avm.insert_stack_frame_for_action(
                            actions.clip,
                            context.swf.header().version,
                            event,
                            context,
                        );
                    }
                }
                // Event handler method call (e.g. onEnterFrame)
                ActionType::Method { object, name, args } => {
                    avm.insert_stack_frame_for_method(
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
                    avm.notify_system_listeners(
                        actions.clip,
                        context.swf.version(),
                        context,
                        listener,
                        method,
                        &args,
                    );
                }
            }
            // Execute the stack frame (if any).
            let _ = avm.run_stack_till_empty(context);
        }
    }

    fn build_matrices(&mut self) {
        // Create  view matrix to scale stage into viewport area.
        let (movie_width, movie_height) = (self.movie_width as f32, self.movie_height as f32);
        let (viewport_width, viewport_height) =
            (self.viewport_width as f32, self.viewport_height as f32);
        let movie_aspect = movie_width / movie_height;
        let viewport_aspect = viewport_width / viewport_height;
        let (scale, margin_width, margin_height) = if viewport_aspect > movie_aspect {
            let scale = viewport_height / movie_height;
            (scale, (viewport_width - movie_width * scale) / 2.0, 0.0)
        } else {
            let scale = viewport_width / movie_width;
            (scale, 0.0, (viewport_height - movie_height * scale) / 2.0)
        };
        self.view_matrix = Matrix {
            a: scale,
            b: 0.0,
            c: 0.0,
            d: scale,
            tx: Twips::from_pixels(margin_width.into()),
            ty: Twips::from_pixels(margin_height.into()),
        };
        self.inverse_view_matrix = self.view_matrix;
        self.inverse_view_matrix.invert();

        // Calculate letterbox dimensions.
        // TODO: Letterbox should be an option; the original Flash Player defaults to showing content
        // in the extra margins.
        self.letterbox = if margin_width > 0.0 {
            Letterbox::Pillarbox(margin_width)
        } else if margin_height > 0.0 {
            Letterbox::Letterbox(margin_height)
        } else {
            Letterbox::None
        };
    }

    /// Runs the closure `f` with an `UpdateContext`.
    /// This takes cares of populating the `UpdateContext` struct, avoiding borrow issues.
    fn mutate_with_update_context<F, R>(&mut self, f: F) -> R
    where
        F: for<'a, 'gc> FnOnce(&mut Avm1<'gc>, &mut UpdateContext<'a, 'gc, '_>) -> R,
    {
        // We have to do this piecewise borrowing of fields before the closure to avoid
        // completely borrowing `self`.
        let (
            player_version,
            global_time,
            swf,
            background_color,
            renderer,
            audio,
            navigator,
            input,
            rng,
            mouse_position,
            stage_width,
            stage_height,
            player,
            system_properties,
            instance_counter,
            storage,
        ) = (
            self.player_version,
            self.global_time,
            &self.swf,
            &mut self.background_color,
            self.renderer.deref_mut(),
            self.audio.deref_mut(),
            self.navigator.deref_mut(),
            self.input.deref_mut(),
            &mut self.rng,
            &self.mouse_pos,
            Twips::from_pixels(self.movie_width.into()),
            Twips::from_pixels(self.movie_height.into()),
            self.self_reference.clone(),
            &mut self.system,
            &mut self.instance_counter,
            self.storage.deref_mut(),
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut root_data = gc_root.0.write(gc_context);
            let mouse_hovered_object = root_data.mouse_hovered_object;
            let (levels, library, action_queue, avm, drag_object, load_manager, shared_objects) =
                root_data.update_context_params();

            let mut update_context = UpdateContext {
                player_version,
                global_time,
                swf,
                library,
                background_color,
                rng,
                renderer,
                audio,
                navigator,
                input,
                action_queue,
                gc_context,
                levels,
                mouse_hovered_object,
                mouse_position,
                drag_object,
                stage_size: (stage_width, stage_height),
                system_prototypes: avm.prototypes().clone(),
                player,
                load_manager,
                system: system_properties,
                instance_counter,
                storage,
                shared_objects,
            };

            let ret = f(avm, &mut update_context);

            // Hovered object may have been updated; copy it back to the GC root.
            root_data.mouse_hovered_object = update_context.mouse_hovered_object;
            ret
        })
    }

    /// Loads font data from the given buffer.
    /// The buffer should be the `DefineFont3` info for the tag.
    /// The tag header should not be included.
    fn load_device_font<'gc>(
        gc_context: gc_arena::MutationContext<'gc, '_>,
        data: &[u8],
        renderer: &mut Renderer,
    ) -> Result<crate::font::Font<'gc>, Error> {
        let mut reader = swf::read::Reader::new(data, 8);
        let device_font = crate::font::Font::from_swf_tag(
            gc_context,
            renderer.deref_mut(),
            &reader.read_define_font_2(3)?,
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
        F: for<'a, 'gc> FnOnce(&mut Avm1<'gc>, &mut UpdateContext<'a, 'gc, '_>) -> R,
    {
        let rval = self.mutate_with_update_context(|avm, context| {
            let rval = func(avm, context);

            Self::run_actions(avm, context);

            rval
        });

        // Update mouse state (check for new hovered button, etc.)
        self.update_drag();
        self.update_roll_over();

        // GC
        self.gc_arena.collect_debt();

        rval
    }

    pub fn flush_shared_objects(&mut self) {
        self.update(|avm, update_context| {
            let shared_objects = update_context.shared_objects.clone();
            for so in shared_objects.values() {
                let _ = crate::avm1::globals::shared_object::flush(avm, update_context, *so, &[]);
            }
        });
    }
}

pub struct DragObject<'gc> {
    /// The display object being dragged.
    pub display_object: DisplayObject<'gc>,

    /// The offset from the mouse position to the center of the clip.
    pub offset: (Twips, Twips),

    /// The bounding rectangle where the clip will be maintained.
    pub constraint: BoundingBox,
}

unsafe impl<'gc> gc_arena::Collect for DragObject<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.display_object.trace(cc);
    }
}
