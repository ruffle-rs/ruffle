use crate::audio::Audio;
//use crate::avm1::Avm1;
use crate::backend::{audio::AudioBackend, render::RenderBackend};
use crate::display_object::DisplayObject;
use crate::library::Library;
use crate::movie_clip::MovieClip;
use crate::prelude::*;
use crate::transform::TransformStack;
use gc_arena::{make_arena, ArenaParameters, Collect, GcCell, MutationContext};
use log::info;
use std::io::Cursor;

#[derive(Collect)]
#[collect(empty_drop)]
struct GcRoot<'gc> {
    library: GcCell<'gc, Library<'gc>>,
    root: GcCell<'gc, MovieClip<'gc>>,
}

make_arena!(GcArena, GcRoot);

pub struct Player {
    tag_stream: swf::read::Reader<Cursor<Vec<u8>>>,

    //avm: Avm1,
    audio: Audio,
    renderer: Box<RenderBackend>,
    transform_stack: TransformStack,

    gc_arena: GcArena,
    background_color: Color,

    frame_rate: f64,
    frame_accumulator: f64,
    global_time: u64,

    movie_width: u32,
    movie_height: u32,

    mouse_pos: (f32, f32),
}

impl Player {
    pub fn new(
        mut renderer: Box<RenderBackend>,
        audio: Box<AudioBackend>,
        swf_data: Vec<u8>,
    ) -> Result<Self, Box<std::error::Error>> {
        let (header, mut reader) = swf::read::read_swf_header(&swf_data[..]).unwrap();
        // Decompress the entire SWF in memory.
        let mut data = Vec::new();
        reader.get_mut().read_to_end(&mut data)?;
        let tag_stream = swf::read::Reader::new(Cursor::new(data), header.version);

        info!("{}x{}", header.stage_size.x_max, header.stage_size.y_max);

        let movie_width = (header.stage_size.x_max - header.stage_size.x_min).to_pixels() as u32;
        let movie_height = (header.stage_size.y_max - header.stage_size.y_min).to_pixels() as u32;
        renderer.set_dimensions(movie_width, movie_height);

        let mut player = Player {
            tag_stream,

            // avm: Avm1::new(header.version),
            renderer,
            audio: Audio::new(audio),

            background_color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            transform_stack: TransformStack::new(),

            gc_arena: GcArena::new(ArenaParameters::default(), |gc_context| GcRoot {
                library: GcCell::allocate(gc_context, Library::new()),
                root: GcCell::allocate(gc_context, MovieClip::new_with_data(0, header.num_frames)),
            }),

            frame_rate: header.frame_rate.into(),
            frame_accumulator: 0.0,
            global_time: 0,

            movie_width,
            movie_height,

            mouse_pos: (0.0, 0.0),
        };

        player.preload();

        Ok(player)
    }

    pub fn tick(&mut self, dt: f64) {
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

    pub fn movie_width(&self) -> u32 {
        self.movie_width
    }

    pub fn movie_height(&self) -> u32 {
        self.movie_height
    }

    pub fn mouse_move(&mut self, pos: (f32, f32)) {
        self.mouse_pos = pos;
    }

    pub fn mouse_down(&mut self) {}

    pub fn mouse_up(&mut self) {
        //self.stage.handle_click(self.mouse_pos);
    }

    fn preload(&mut self) {
        let (global_time, mouse_pos, tag_stream, background_color, renderer, audio) = (
            self.global_time,
            self.mouse_pos,
            &mut self.tag_stream,
            &mut self.background_color,
            &mut *self.renderer,
            &mut self.audio,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut update_context = UpdateContext {
                global_time,
                mouse_pos,
                tag_stream,
                position_stack: vec![],
                library: gc_root.library.write(gc_context),
                background_color,
                //avm1: &mut self.avm,
                renderer,
                audio,
                action: None,
                gc_context,
            };

            gc_root.root.write(gc_context).preload(&mut update_context);
        });
    }

    fn run_frame(&mut self) {
        let (global_time, mouse_pos, tag_stream, background_color, renderer, audio) = (
            self.global_time,
            self.mouse_pos,
            &mut self.tag_stream,
            &mut self.background_color,
            &mut *self.renderer,
            &mut self.audio,
        );

        self.gc_arena.mutate(|gc_context, gc_root| {
            let mut update_context = UpdateContext {
                global_time,
                mouse_pos,
                tag_stream,
                position_stack: vec![],
                library: gc_root.library.write(gc_context),
                background_color,
                //avm1: &mut self.avm,
                renderer,
                audio,
                action: None,
                gc_context,
            };

            gc_root
                .root
                .write(gc_context)
                .run_frame(&mut update_context);
            gc_root
                .root
                .write(gc_context)
                .run_post_frame(&mut update_context)
        });
    }

    fn render(&mut self) {
        self.renderer.begin_frame();

        self.renderer.clear(self.background_color.clone());

        let (renderer, transform_stack) = (&mut *self.renderer, &mut self.transform_stack);
        self.gc_arena.mutate(|_gc_context, gc_root| {
            let mut render_context = RenderContext {
                renderer,
                library: gc_root.library.read(),
                transform_stack,
            };
            gc_root.root.read().render(&mut render_context);
        });

        self.renderer.end_frame();
    }
}

pub struct UpdateContext<'a, 'gc, 'gc_context> {
    pub global_time: u64,
    pub mouse_pos: (f32, f32),
    pub tag_stream: &'a mut swf::read::Reader<Cursor<Vec<u8>>>,
    pub position_stack: Vec<u64>,
    pub library: std::cell::RefMut<'a, Library<'gc>>,
    pub gc_context: MutationContext<'gc, 'gc_context>,
    pub background_color: &'a mut Color,
    //pub avm1: &'a mut Avm1,
    pub renderer: &'a mut RenderBackend,
    pub audio: &'a mut Audio,
    pub action: Option<(usize, usize)>,
}

pub struct RenderContext<'a, 'gc> {
    pub renderer: &'a mut RenderBackend,
    pub library: std::cell::Ref<'a, Library<'gc>>,
    pub transform_stack: &'a mut TransformStack,
}
