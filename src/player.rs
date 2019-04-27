use crate::backend::{render::RenderBackend, ui::UiBackend};
use crate::color_transform::ColorTransformStack;
use crate::display_object::DisplayObject;
use crate::library::Library;
use crate::matrix::MatrixStack;
use crate::stage::Stage;
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Player {
    tag_stream: swf::read::Reader<Cursor<Vec<u8>>>,

    render_context: RenderContext,

    ui: Box<UiBackend>,

    library: Library,
    stage: Cc<RefCell<Stage>>,

    frame_rate: f64,
    frame_accumulator: f64,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Player {
    #[cfg(target_arch = "wasm32")]
    pub fn new(swf_data: js_sys::Uint8Array) -> Result<Player, JsValue> {
        console_error_panic_hook::set_once();
        let mut data = vec![0; swf_data.length() as usize];
        swf_data.copy_to(&mut data[..]);
        Self::new_internal(data).map_err(|_| JsValue::null())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(swf_data: Vec<u8>) -> Result<Player, Box<std::error::Error>> {
        Self::new_internal(swf_data)
    }

    fn new_internal(swf_data: Vec<u8>) -> Result<Player, Box<std::error::Error>> {
        let (swf, tag_stream) = swf::read::read_swf_header_decompressed(&swf_data[..]).unwrap();
        info!("{}x{}", swf.stage_size.x_max, swf.stage_size.y_max);

        let (ui, renderer) = crate::backend::build()?;

        Ok(Player {
            tag_stream,

            ui,

            render_context: RenderContext {
                renderer,
                matrix_stack: MatrixStack::new(),
                color_transform_stack: ColorTransformStack::new(),
            },

            library: Library::new(),
            stage: Stage::new(swf.num_frames),

            frame_rate: swf.frame_rate.into(),
            frame_accumulator: 0.0,
        })
    }

    pub fn play(&mut self) {
        use std::time::{Duration, Instant};

        let mut time = Instant::now();

        while self.ui.poll_events() {
            let new_time = Instant::now();
            let dt = new_time.duration_since(time).as_millis();
            time = new_time;

            self.tick(dt as f64);

            std::thread::sleep(Duration::from_millis(1000 / 60));
        }
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
            renderer: &mut *self.render_context.renderer,
        };

        let mut stage = self.stage.borrow_mut();
        stage.run_frame(&mut update_context);
        stage.update_frame_number();
    }

    fn render(&mut self) {
        self.render_context.renderer.begin_frame();

        let stage = self.stage.borrow_mut();
        stage.render(&mut self.render_context);

        self.render_context.renderer.end_frame();
    }
}

pub struct UpdateContext<'a> {
    pub tag_stream: &'a mut swf::read::Reader<Cursor<Vec<u8>>>,
    pub position_stack: Vec<u64>,
    pub library: &'a mut Library,
    pub renderer: &'a mut RenderBackend,
}

pub struct RenderContext {
    pub renderer: Box<RenderBackend>,
    pub matrix_stack: MatrixStack,
    pub color_transform_stack: ColorTransformStack,
}
