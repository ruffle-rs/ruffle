use crate::avm1::Avm1;
use crate::backend::{audio::AudioBackend, render::RenderBackend};
use crate::library::Library;
use crate::movie_clip::MovieClip;
use crate::prelude::*;
use crate::transform::TransformStack;
use crate::Event;
use gc_arena::{make_arena, ArenaParameters, Collect, GcCell, MutationContext};
use log::info;
use std::sync::Arc;

#[derive(Collect)]
#[collect(empty_drop)]
struct GcRoot<'gc> {
    library: GcCell<'gc, Library<'gc>>,
    root: DisplayNode<'gc>,
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
                    Box::new(MovieClip::new_with_data(gc_context, 0, 0, swf_len, header.num_frames)),
                ),
            }),

            frame_rate: header.frame_rate.into(),
            frame_accumulator: 0.0,
            global_time: 0,

            movie_width,
            movie_height,
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

    pub fn handle_event(&mut self, event: Event) {
        let (global_time, swf_data, swf_version, background_color, renderer, audio, avm) = (
            self.global_time,
            &mut self.swf_data,
            self.swf_version,
            &mut self.background_color,
            &mut self.renderer,
            &mut self.audio,
            &mut self.avm,
        );

        self.gc_arena.mutate(move |gc_context, gc_root| {
            let actions = {
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

                match event {
                    Event::MouseMove { x, y } => {
                        if let Some(node) = gc_root.root.read().pick((x, y)) {
                            update_context.active_clip = node;
                            node.write(gc_context).handle_event(
                                &mut update_context,
                                crate::event::PlayerEvent::RollOver,
                            );
                        };
                    }
                    Event::MouseDown { x, y } => {
                        if let Some(node) = gc_root.root.read().pick((x, y)) {
                            update_context.active_clip = node;
                            node.write(gc_context).handle_event(
                                &mut update_context,
                                crate::event::PlayerEvent::Click,
                            );
                        };
                    }
                    _ => (),
                }
                update_context.actions
            };

            if !actions.is_empty() {
                let mut action_context = crate::avm1::ActionContext {
                    gc_context,
                    global_time,
                    root: gc_root.root,
                    start_clip: gc_root.root,
                    active_clip: gc_root.root,
                    audio,
                };
                for (active_clip, action) in actions {
                    action_context.start_clip = active_clip;
                    action_context.active_clip = active_clip;
                    let _ = avm.do_action(&mut action_context, action.as_ref());
                }
            }
        });
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
            let mut actions = {
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
                update_context.actions
            };

            // TODO: Loop here because goto-ing a frame can queue up for actions.
            // Need to figure out the proper order of operations between ticking a clip
            // and running the actions.
            loop {
                {
                    let mut action_context = crate::avm1::ActionContext {
                        gc_context,
                        global_time,
                        root: gc_root.root,
                        start_clip: gc_root.root,
                        active_clip: gc_root.root,
                        audio,
                    };
                    for (active_clip, action) in actions {
                        action_context.start_clip = active_clip;
                        action_context.active_clip = active_clip;
                        let _ = avm.do_action(&mut action_context, action.as_ref());
                    }
                }

                actions = {
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
                        .run_post_frame(&mut update_context);

                    update_context.actions
                };

                if actions.is_empty() {
                    break;
                }
            }
        });
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
