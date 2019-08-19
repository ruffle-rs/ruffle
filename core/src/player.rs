use crate::avm1::Avm1;
use crate::backend::{audio::AudioBackend, render::RenderBackend};
use crate::events::{ButtonEvent, PlayerEvent};
use crate::library::Library;
use crate::movie_clip::MovieClip;
use crate::prelude::*;
use crate::transform::TransformStack;
use gc_arena::{make_arena, ArenaParameters, Collect, GcCell, MutationContext};
use log::info;
use std::sync::Arc;

#[derive(Collect)]
#[collect(empty_drop)]
struct GcRoot<'gc> {
    library: GcCell<'gc, Library<'gc>>,
    root: DisplayNode<'gc>,
    mouse_hover_node: GcCell<'gc, Option<DisplayNode<'gc>>>, // TODO: Remove GcCell wrapped inside GcCell.
}

make_arena!(GcArena, GcRoot);

pub struct Player<Audio: AudioBackend, Renderer: RenderBackend> {
    swf_data: Arc<Vec<u8>>,
    swf_version: u8,

    is_playing: bool,

    avm: Avm1,
    audio: Audio,
    renderer: Renderer,
    transform_stack: TransformStack,

    gc_arena: GcArena,
    background_color: Color,

    frame_rate: f64,
    frame_accumulator: f64,
    global_time: u64,

    movie_width: u32,
    movie_height: u32,

    mouse_pos: (Twips, Twips),
    is_mouse_down: bool,
}

impl<Audio: AudioBackend, Renderer: RenderBackend> Player<Audio, Renderer> {
    pub fn new(
        mut renderer: Renderer,
        audio: Audio,
        swf_data: Vec<u8>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (header, mut reader) = swf::read::read_swf_header(&swf_data[..]).unwrap();
        // Decompress the entire SWF in memory.
        let mut data = Vec::new();
        reader.get_mut().read_to_end(&mut data)?;
        let swf_len = data.len();

        info!("{}x{}", header.stage_size.x_max, header.stage_size.y_max);

        let movie_width = (header.stage_size.x_max - header.stage_size.x_min).to_pixels() as u32;
        let movie_height = (header.stage_size.y_max - header.stage_size.y_min).to_pixels() as u32;
        renderer.set_movie_dimensions(movie_width, movie_height);

        let mut player = Player {
            swf_data: Arc::new(data),
            swf_version: header.version,

            is_playing: false,

            avm: Avm1::new(header.version),
            renderer,
            audio,

            background_color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            transform_stack: TransformStack::new(),

            gc_arena: GcArena::new(ArenaParameters::default(), |gc_context| GcRoot {
                library: GcCell::allocate(gc_context, Library::new()),
                root: GcCell::allocate(
                    gc_context,
                    Box::new(MovieClip::new_with_data(
                        gc_context,
                        0,
                        0,
                        swf_len,
                        header.num_frames,
                    )),
                ),
                mouse_hover_node: GcCell::allocate(gc_context, None),
            }),

            frame_rate: header.frame_rate.into(),
            frame_accumulator: 0.0,
            global_time: 0,

            movie_width,
            movie_height,

            mouse_pos: (Twips::new(0), Twips::new(0)),
            is_mouse_down: false,
        };

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
            while self.frame_accumulator >= frame_time {
                self.frame_accumulator -= frame_time;
                self.run_frame();
            }

            if needs_render {
                self.render();
            }

            self.audio.tick();
        }
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

    pub fn handle_event(&mut self, event: PlayerEvent) {
        let mut needs_render = false;

        // Update mouse position from mouse events.
        if let PlayerEvent::MouseMove { x, y }
        | PlayerEvent::MouseDown { x, y }
        | PlayerEvent::MouseUp { x, y } = event
        {
            self.mouse_pos = (x, y);
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
            avm,
            is_mouse_down,
        ) = (
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.avm,
            &mut self.is_mouse_down,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut update_context = UpdateContext {
                global_time,
                swf_data,
                swf_version,
                library: gc_root.library.write(gc_context),
                background_color,
                avm,
                renderer,
                audio,
                actions: vec![],
                gc_context,
                active_clip: gc_root.root,
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

            Self::run_actions(&mut update_context, gc_root.root);
        });

        if needs_render {
            // Update display after mouse events.
            self.render();
        }
    }

    fn update_roll_over(&mut self) -> bool {
        // TODO: While the mouse is down, maintain the hovered node.
        if self.is_mouse_down {
            return false;
        }

        let (global_time, swf_data, swf_version, background_color, renderer, audio, avm) = (
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.avm,
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
                    global_time,
                    swf_data,
                    swf_version,
                    library: gc_root.library.write(gc_context),
                    background_color,
                    avm,
                    renderer,
                    audio,
                    actions: vec![],
                    gc_context,
                    active_clip: gc_root.root,
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

                Self::run_actions(&mut update_context, gc_root.root);
                true
            } else {
                false
            }
        })
    }

    fn preload(&mut self) {
        let (global_time, swf_data, swf_version, background_color, renderer, audio, avm) = (
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.avm,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut update_context = UpdateContext {
                global_time,
                swf_data,
                swf_version,
                library: gc_root.library.write(gc_context),
                background_color,
                avm,
                renderer,
                audio,
                actions: vec![],
                gc_context,
                active_clip: gc_root.root,
            };

            gc_root.root.write(gc_context).preload(&mut update_context);
        });
    }

    fn run_frame(&mut self) {
        let (global_time, swf_data, swf_version, background_color, renderer, audio, avm) = (
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.avm,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut update_context = UpdateContext {
                global_time,
                swf_data,
                swf_version,
                library: gc_root.library.write(gc_context),
                background_color,
                avm,
                renderer,
                audio,
                actions: vec![],
                gc_context,
                active_clip: gc_root.root,
            };

            gc_root
                .root
                .write(gc_context)
                .run_frame(&mut update_context);

            Self::run_actions(&mut update_context, gc_root.root);
        });

        // Update mouse state (check for new hovered button, etc.)
        self.update_roll_over();

        // GC
        //self.gc_arena.collect_all();
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
        self.gc_arena.mutate(|_gc_context, gc_root| {
            let mut render_context = RenderContext {
                renderer,
                library: gc_root.library.read(),
                transform_stack,
                view_bounds,
            };
            gc_root.root.read().render(&mut render_context);
        });

        if !self.is_playing() {
            self.renderer.draw_pause_overlay();
        }

        self.renderer.end_frame();
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    fn run_actions<'gc>(update_context: &mut UpdateContext<'_, 'gc, '_>, root: DisplayNode<'gc>) {
        // TODO: Loop here because goto-ing a frame can queue up for actions.
        // I think this will eventually be cleaned up;
        // Need to figure out the proper order of operations between ticking a clip
        // and running the actions.
        let mut actions = std::mem::replace(&mut update_context.actions, vec![]);
        while !actions.is_empty() {
            {
                let mut action_context = crate::avm1::ActionContext {
                    gc_context: update_context.gc_context,
                    global_time: update_context.global_time,
                    root,
                    start_clip: root,
                    active_clip: root,
                    audio: update_context.audio,
                };
                for (active_clip, action) in actions {
                    action_context.start_clip = active_clip;
                    action_context.active_clip = active_clip;
                    let _ = update_context
                        .avm
                        .do_action(&mut action_context, action.as_ref());
                }
            }

            // Run goto queues.
            update_context.active_clip = root;
            root.write(update_context.gc_context)
                .run_post_frame(update_context);

            actions = std::mem::replace(&mut update_context.actions, vec![]);
        }
    }
}

pub struct UpdateContext<'a, 'gc, 'gc_context> {
    pub swf_version: u8,
    pub swf_data: &'a Arc<Vec<u8>>,
    pub global_time: u64,
    pub library: std::cell::RefMut<'a, Library<'gc>>,
    pub gc_context: MutationContext<'gc, 'gc_context>,
    pub background_color: &'a mut Color,
    pub avm: &'a mut Avm1,
    pub renderer: &'a mut dyn RenderBackend,
    pub audio: &'a mut dyn AudioBackend,
    pub actions: Vec<(DisplayNode<'gc>, crate::tag_utils::SwfSlice)>,
    pub active_clip: DisplayNode<'gc>,
}

pub struct RenderContext<'a, 'gc> {
    pub renderer: &'a mut dyn RenderBackend,
    pub library: std::cell::Ref<'a, Library<'gc>>,
    pub transform_stack: &'a mut TransformStack,
    pub view_bounds: BoundingBox,
}
