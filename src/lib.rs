mod character;
mod display_object;
mod graphic;
mod library;
mod matrix;
mod movie_clip;
mod shape_utils;

use self::character::Character;
use self::display_object::DisplayObject;
use self::library::Library;
use self::matrix::{Matrix, MatrixStack};
use self::movie_clip::MovieClip;
use bacon_rajan_cc::Cc;
use js_sys::{ArrayBuffer, Uint8Array};
use log::{info, trace, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;
use swf::Color;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

type CharacterId = swf::CharacterId;

#[wasm_bindgen]
pub struct Player {
    tag_stream: swf::read::Reader<Cursor<Vec<u8>>>,
    canvas: HtmlCanvasElement,
    render_context: RenderContext,

    library: Library,
    root: Cc<RefCell<MovieClip>>,

    frame_rate: f64,
    frame_accumulator: f64,
    cur_timestamp: f64,

    background_color: Color,
}

#[wasm_bindgen]
impl Player {
    pub fn new(data: ArrayBuffer, canvas: HtmlCanvasElement) -> Player {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Trace).expect("error initializing log");

        let data = Uint8Array::new(data.as_ref());
        let mut swf_data = vec![0; data.byte_length() as usize];
        data.copy_to(&mut swf_data[..]);

        let (swf, tag_stream) = swf::read::read_swf_header_decompressed(&swf_data[..]).unwrap();
        info!("{}x{}", swf.stage_size.x_max, swf.stage_size.y_max);

        canvas.set_width(swf.stage_size.x_max as u32);
        canvas.set_height(swf.stage_size.y_max as u32);

        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .expect("Expected canvas")
            .expect("Expected canvas")
            .dyn_into()
            .expect("Expected CanvasRenderingContext2d");

        Player {
            tag_stream,

            canvas,
            render_context: RenderContext {
                context_2d: context,
                matrix_stack: MatrixStack::new(),
            },

            library: Library::new(),
            root: MovieClip::new(),

            frame_rate: swf.frame_rate.into(),
            frame_accumulator: 0.0,
            cur_timestamp: web_sys::window()
                .expect("Expected window")
                .performance()
                .expect("Expected performance")
                .now(),
            background_color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        }
    }

    pub fn tick(&mut self, timestamp: f64) {
        let dt = timestamp - self.cur_timestamp;
        self.cur_timestamp = timestamp;

        self.frame_accumulator += dt;
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

impl Player {
    fn run_frame(&mut self) {
        use swf::Tag;

        while let Ok(Some(tag)) = self.tag_stream.read_tag() {
            trace!("{:?}", tag);
            match tag {
                Tag::FileAttributes(file_attributes) => {}

                Tag::SetBackgroundColor(color) => self.background_color = color,

                Tag::ShowFrame => break,

                Tag::DefineSceneAndFrameLabelData {
                    scenes,
                    frame_labels,
                } => (), // TODO(Herschel)

                Tag::DefineShape(shape) => {
                    if !self.library.contains_character(shape.id) {
                        let svg = shape_utils::swf_shape_to_svg(&shape);
                        info!("{}", svg);
                        let mut image = HtmlImageElement::new().unwrap();
                        image.set_src(&format!("data:image/svg+xml;utf8;{}", svg));
                        self.library
                            .register_character(shape.id, Character::Graphic { image });
                    }
                }

                //tag => self.root.borrow_mut().run_tag(tag)
                _ => (),
            }
        }
    }

    fn render(&mut self) {
        let background_color = format!(
            "rgb({}, {}, {})",
            self.background_color.r, self.background_color.g, self.background_color.b
        );
        info!("{:?}", background_color);
        self.render_context
            .context_2d
            .set_fill_style(&background_color.into());

        let width: f64 = self.canvas.width().into();
        let height: f64 = self.canvas.height().into();

        self.render_context
            .context_2d
            .fill_rect(0.0, 0.0, width, height);

        self.root.borrow_mut().render(&mut self.render_context);
    }
}

pub struct RenderContext {
    context_2d: CanvasRenderingContext2d,
    matrix_stack: MatrixStack,
}
