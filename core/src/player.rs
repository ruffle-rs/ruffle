use crate::avm1::{self, Avm1};
use crate::backend::{
    audio::AudioBackend, navigator::NavigatorBackend, render::Letterbox, render::RenderBackend,
};
use crate::context::{ActionQueue, RenderContext, UpdateContext};
use crate::display_object::{MorphShape, MovieClip};
use crate::events::{ButtonEvent, PlayerEvent};
use crate::library::Library;
use crate::prelude::*;
use crate::transform::TransformStack;
use gc_arena::{make_arena, ArenaParameters, Collect, GcCell};
use log::info;
use rand::{rngs::SmallRng, SeedableRng};
use std::sync::Arc;

static DEVICE_FONT_TAG: &[u8] = include_bytes!("../assets/noto-sans-definefont3.bin");

/// The newest known Flash Player version, serves as a default to
/// `player_version`.
pub const NEWEST_PLAYER_VERSION: u8 = 32;

#[derive(Collect)]
#[collect(empty_drop)]
struct GcRoot<'gc> {
    library: GcCell<'gc, Library<'gc>>,
    root: DisplayNode<'gc>,
    mouse_hover_node: GcCell<'gc, Option<DisplayNode<'gc>>>, // TODO: Remove GcCell wrapped inside GcCell.
    avm: GcCell<'gc, Avm1<'gc>>,
    action_queue: GcCell<'gc, ActionQueue<'gc>>,
}

type Error = Box<dyn std::error::Error>;

make_arena!(GcArena, GcRoot);

pub struct Player<Audio: AudioBackend, Renderer: RenderBackend, Navigator: NavigatorBackend> {
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

    swf_data: Arc<Vec<u8>>,
    swf_version: u8,

    is_playing: bool,

    audio: Audio,
    renderer: Renderer,
    navigator: Navigator,
    transform_stack: TransformStack,
    view_matrix: Matrix,
    inverse_view_matrix: Matrix,

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
}

impl<Audio: AudioBackend, Renderer: RenderBackend, Navigator: NavigatorBackend>
    Player<Audio, Renderer, Navigator>
{
    pub fn new(
        mut renderer: Renderer,
        audio: Audio,
        navigator: Navigator,
        swf_data: Vec<u8>,
    ) -> Result<Self, Error> {
        let swf_stream = swf::read::read_swf_header(&swf_data[..]).unwrap();
        let header = swf_stream.header;
        let mut reader = swf_stream.reader;

        // Decompress the entire SWF in memory.
        // Sometimes SWFs will have an incorrectly compressed stream,
        // but will otherwise decompress fine up to the End tag.
        // So just warn on this case and try to continue gracefully.
        let data = if header.compression == swf::Compression::Lzma {
            // TODO: The LZMA decoder is still funky.
            // It always errors, and doesn't return all the data if you use read_to_end,
            // but read_exact at least returns the data... why?
            // Does the decoder need to be flushed somehow?
            let mut data = vec![0u8; swf_stream.uncompressed_length];
            let _ = reader.get_mut().read_exact(&mut data);
            data
        } else {
            let mut data = Vec::with_capacity(swf_stream.uncompressed_length);
            if let Err(e) = reader.get_mut().read_to_end(&mut data) {
                log::error!("Error decompressing SWF, may be corrupt: {}", e);
            }
            data
        };

        let swf_len = data.len();

        info!("{}x{}", header.stage_size.x_max, header.stage_size.y_max);

        let movie_width = (header.stage_size.x_max - header.stage_size.x_min).to_pixels() as u32;
        let movie_height = (header.stage_size.y_max - header.stage_size.y_min).to_pixels() as u32;

        // Load and parse the device font.
        // TODO: We could use lazy_static here.
        let device_font = match Self::load_device_font(DEVICE_FONT_TAG, &mut renderer) {
            Ok(font) => Some(font),
            Err(e) => {
                log::error!("Unable to load device font: {}", e);
                None
            }
        };

        let mut player = Player {
            player_version: NEWEST_PLAYER_VERSION,

            swf_data: Arc::new(data),
            swf_version: header.version,

            is_playing: false,

            renderer,
            audio,
            navigator,

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
                let mut library = Library::new();
                library.set_device_font(device_font);
                GcRoot {
                    library: GcCell::allocate(gc_context, library),
                    root: GcCell::allocate(
                        gc_context,
                        Box::new(MovieClip::new_with_data(
                            header.version,
                            gc_context,
                            0,
                            0,
                            swf_len,
                            header.num_frames,
                        )),
                    ),
                    mouse_hover_node: GcCell::allocate(gc_context, None),
                    avm: GcCell::allocate(gc_context, Avm1::new(gc_context, NEWEST_PLAYER_VERSION)),
                    action_queue: GcCell::allocate(gc_context, ActionQueue::new()),
                }
            }),

            frame_rate: header.frame_rate.into(),
            frame_accumulator: 0.0,
            global_time: 0,

            movie_width,
            movie_height,
            viewport_width: movie_width,
            viewport_height: movie_height,
            letterbox: Letterbox::None,

            mouse_pos: (Twips::new(0), Twips::new(0)),
            is_mouse_down: false,
        };

        player.gc_arena.mutate(|gc_context, gc_root| {
            gc_root
                .root
                .write(gc_context)
                .post_instantiation(gc_context, gc_root.root)
        });

        player.build_matrices();
        player.preload();

        Ok(player)
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

            let needs_render = self.frame_accumulator >= frame_time;

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

            if needs_render {
                self.render();
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
        let mut needs_render = false;
        let player_version = self.player_version;

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

        let (
            global_time,
            swf_data,
            swf_version,
            background_color,
            renderer,
            audio,
            navigator,
            rng,
            is_mouse_down,
        ) = (
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.navigator,
            &mut self.rng,
            &mut self.is_mouse_down,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut update_context = UpdateContext {
                player_version,
                global_time,
                swf_data,
                swf_version,
                library: &mut *gc_root.library.write(gc_context),
                background_color,
                rng,
                renderer,
                audio,
                navigator,
                action_queue: &mut *gc_root.action_queue.write(gc_context),
                gc_context,
                active_clip: gc_root.root,
                start_clip: gc_root.root,
                target_clip: Some(gc_root.root),
                root: gc_root.root,
                target_path: avm1::Value::Undefined,
            };

            if let Some(node) = &*gc_root.mouse_hover_node.read() {
                if let Some(button) = node.write(gc_context).as_button_mut() {
                    match event {
                        PlayerEvent::MouseDown { .. } => {
                            *is_mouse_down = true;
                            needs_render = true;
                            update_context.active_clip = *node;
                            button.handle_button_event(&mut update_context, ButtonEvent::Press);
                        }

                        PlayerEvent::MouseUp { .. } => {
                            *is_mouse_down = false;
                            needs_render = true;
                            update_context.active_clip = *node;
                            button.handle_button_event(&mut update_context, ButtonEvent::Release);
                        }

                        _ => (),
                    }
                }
            }

            Self::run_actions(&mut *gc_root.avm.write(gc_context), &mut update_context);
        });

        if needs_render {
            // Update display after mouse events.
            self.render();
        }
    }

    fn update_roll_over(&mut self) -> bool {
        let player_version = self.player_version;
        // TODO: While the mouse is down, maintain the hovered node.
        if self.is_mouse_down {
            return false;
        }

        let (global_time, swf_data, swf_version, background_color, renderer, audio, navigator, rng) = (
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.navigator,
            &mut self.rng,
        );

        let mouse_pos = &self.mouse_pos;
        // Check hovered object.
        self.gc_arena.mutate(|gc_context, gc_root| {
            let new_hover_node = gc_root
                .root
                .read()
                .mouse_pick(gc_root.root, (mouse_pos.0, mouse_pos.1));
            let mut cur_hover_node = gc_root.mouse_hover_node.write(gc_context);
            if cur_hover_node.map(GcCell::as_ptr) != new_hover_node.map(GcCell::as_ptr) {
                let mut update_context = UpdateContext {
                    player_version,
                    global_time,
                    swf_data,
                    swf_version,
                    library: &mut *gc_root.library.write(gc_context),
                    background_color,
                    rng,
                    renderer,
                    audio,
                    navigator,
                    action_queue: &mut *gc_root.action_queue.write(gc_context),
                    gc_context,
                    active_clip: gc_root.root,
                    start_clip: gc_root.root,
                    target_clip: Some(gc_root.root),
                    root: gc_root.root,
                    target_path: avm1::Value::Undefined,
                };

                // RollOut of previous node.
                if let Some(node) = &*cur_hover_node {
                    if let Some(button) = node.write(gc_context).as_button_mut() {
                        update_context.active_clip = *node;
                        button.handle_button_event(&mut update_context, ButtonEvent::RollOut);
                    }
                }

                // RollOver on new node.
                if let Some(node) = new_hover_node {
                    if let Some(button) = node.write(gc_context).as_button_mut() {
                        update_context.active_clip = node;
                        button.handle_button_event(&mut update_context, ButtonEvent::RollOver);
                    }
                }

                *cur_hover_node = new_hover_node;

                Self::run_actions(&mut *gc_root.avm.write(gc_context), &mut update_context);
                true
            } else {
                false
            }
        })
    }

    fn preload(&mut self) {
        let (
            player_version,
            global_time,
            swf_data,
            swf_version,
            background_color,
            renderer,
            audio,
            navigator,
            rng,
        ) = (
            self.player_version,
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.navigator,
            &mut self.rng,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut update_context = UpdateContext {
                player_version,
                global_time,
                swf_data,
                swf_version,
                library: &mut *gc_root.library.write(gc_context),
                background_color,
                rng,
                renderer,
                audio,
                navigator,
                action_queue: &mut *gc_root.action_queue.write(gc_context),
                gc_context,
                active_clip: gc_root.root,
                start_clip: gc_root.root,
                target_clip: Some(gc_root.root),
                root: gc_root.root,
                target_path: avm1::Value::Undefined,
            };

            let mut morph_shapes = fnv::FnvHashMap::default();
            gc_root
                .root
                .write(gc_context)
                .as_movie_clip_mut()
                .unwrap()
                .preload(&mut update_context, &mut morph_shapes);

            // Finalize morph shapes.
            for (id, static_data) in morph_shapes {
                let morph_shape = MorphShape::new(gc_context, static_data);
                update_context.library.register_character(
                    id,
                    crate::character::Character::MorphShape(Box::new(morph_shape)),
                );
            }
        });
    }

    pub fn run_frame(&mut self) {
        let (
            player_version,
            global_time,
            swf_data,
            swf_version,
            background_color,
            renderer,
            audio,
            navigator,
            rng,
        ) = (
            self.player_version,
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.navigator,
            &mut self.rng,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut update_context = UpdateContext {
                player_version,
                global_time,
                swf_data,
                swf_version,
                library: &mut *gc_root.library.write(gc_context),
                background_color,
                rng,
                renderer,
                audio,
                navigator,
                action_queue: &mut *gc_root.action_queue.write(gc_context),
                gc_context,
                active_clip: gc_root.root,
                start_clip: gc_root.root,
                target_clip: Some(gc_root.root),
                root: gc_root.root,
                target_path: avm1::Value::Undefined,
            };

            gc_root
                .root
                .write(gc_context)
                .run_frame(&mut update_context);

            Self::run_actions(&mut *gc_root.avm.write(gc_context), &mut update_context);
        });

        // Update mouse state (check for new hovered button, etc.)
        self.update_roll_over();

        // GC
        self.gc_arena.collect_debt();
    }

    pub fn render(&mut self) {
        let view_bounds = BoundingBox {
            x_min: Twips::new(0),
            y_min: Twips::new(0),
            x_max: Twips::from_pixels(self.movie_width.into()),
            y_max: Twips::from_pixels(self.movie_height.into()),
            valid: true,
        };

        self.renderer.begin_frame();

        self.renderer.clear(self.background_color.clone());

        let (renderer, transform_stack) = (&mut self.renderer, &mut self.transform_stack);

        transform_stack.push(&crate::transform::Transform {
            matrix: self.view_matrix,
            ..Default::default()
        });
        self.gc_arena.mutate(|_gc_context, gc_root| {
            let mut render_context = RenderContext {
                renderer,
                library: &*gc_root.library.read(),
                transform_stack,
                view_bounds,
                clip_depth_stack: vec![],
            };
            gc_root.root.read().render(&mut render_context);
        });
        transform_stack.pop();

        if !self.is_playing() {
            self.renderer.draw_pause_overlay();
        }

        self.renderer.draw_letterbox(self.letterbox);
        self.renderer.end_frame();
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    fn run_actions<'gc>(avm: &mut Avm1<'gc>, context: &mut UpdateContext<'_, 'gc, '_>) {
        while let Some(actions) = context.action_queue.pop() {
            // We don't run the action f the clip was removed after it queued the action.
            if actions.clip.read().removed() {
                continue;
            }

            context.start_clip = actions.clip;
            context.active_clip = actions.clip;
            context.target_clip = Some(actions.clip);
            avm.insert_stack_frame_for_action(context.swf_version, actions.actions, context);
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
            tx: margin_width * 20.0,
            ty: margin_height * 20.0,
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

    /// Loads font data from the given buffer.
    /// The buffer should be the `DefineFont3` info for the tag.
    /// The tag header should not be included.
    fn load_device_font(
        data: &[u8],
        renderer: &mut Renderer,
    ) -> Result<Box<crate::font::Font>, Error> {
        let mut reader = swf::read::Reader::new(data, 8);
        let device_font = Box::new(crate::font::Font::from_swf_tag(
            renderer,
            &reader.read_define_font_2(3)?,
        )?);
        Ok(device_font)
    }
}
