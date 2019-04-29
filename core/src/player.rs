use crate::backend::render::RenderBackend;
use crate::color_transform::ColorTransformStack;
use crate::display_object::{DisplayObject, DisplayObjectUpdate};
use crate::library::Library;
use crate::matrix::MatrixStack;
use crate::movie_clip::MovieClip;
use crate::prelude::*;
use bacon_rajan_cc::Cc;
use log::info;
use std::cell::RefCell;
use std::io::Cursor;

#[cfg(target_arch = "wasm32")]
use js_sys::{ArrayBuffer, Uint8Array};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

type CharacterId = swf::CharacterId;

pub struct Player {
    tag_stream: swf::read::Reader<Cursor<Vec<u8>>>,

    render_context: RenderContext,

    library: Library,
    stage: Cc<RefCell<DisplayObject>>,
    background_color: Color,

    frame_rate: f64,
    frame_accumulator: f64,
}

impl Player {
    pub fn new(
        renderer: Box<RenderBackend>,
        swf_data: Vec<u8>,
    ) -> Result<Player, Box<std::error::Error>> {
        let (swf, tag_stream) = swf::read::read_swf_header_decompressed(&swf_data[..]).unwrap();
        info!("{}x{}", swf.stage_size.x_max, swf.stage_size.y_max);

        let stage = DisplayObject::new(Box::new(MovieClip::new_with_data(0, swf.num_frames)));

        Ok(Player {
            tag_stream,

            render_context: RenderContext {
                renderer,
                matrix_stack: MatrixStack::new(),
                color_transform_stack: ColorTransformStack::new(),
            },
            background_color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            library: Library::new(),
            stage: Cc::new(RefCell::new(stage)),

            frame_rate: swf.frame_rate.into(),
            frame_accumulator: 0.0,
        })
    }

    pub fn tick(&mut self, dt: f64) {
        self.frame_accumulator += dt;
        let frame_time = 1000.0 / self.frame_rate;
        info!("{} / {}", self.frame_accumulator, frame_time);
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

impl Player {
    fn run_frame(&mut self) {
        let mut update_context = UpdateContext {
            tag_stream: &mut self.tag_stream,
            position_stack: vec![],
            library: &mut self.library,
            background_color: &mut self.background_color,
            renderer: &mut *self.render_context.renderer,
        };

        let mut stage = self.stage.borrow_mut();
        stage.run_frame(&mut update_context);
        stage.update_frame_number();
    }

    fn render(&mut self) {
        self.render_context.renderer.begin_frame();

        self.render_context
            .renderer
            .clear(self.background_color.clone());

        let stage = self.stage.borrow_mut();
        stage.render(&mut self.render_context);

        self.render_context.renderer.end_frame();
    }
}

pub struct UpdateContext<'a> {
    pub tag_stream: &'a mut swf::read::Reader<Cursor<Vec<u8>>>,
    pub position_stack: Vec<u64>,
    pub library: &'a mut Library,
    pub background_color: &'a mut Color,
    pub renderer: &'a mut RenderBackend,
}

pub struct RenderContext {
    pub renderer: Box<RenderBackend>,
    pub matrix_stack: MatrixStack,
    pub color_transform_stack: ColorTransformStack,
}
