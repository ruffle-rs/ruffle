use crate::character::Character;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObject, DisplayObjectNode};
use crate::matrix::Matrix;
use crate::movie_clip::MovieClip;
use crate::Library;
use crate::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Cc, Trace, Tracer};
use log::{info, trace, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use swf::Color;
use web_sys::HtmlImageElement;

type Depth = i16;
type FrameNumber = u16;

pub struct Stage {
    tag_stream_start: Option<u64>,
    tag_stream_pos: u64,
    is_playing: bool,
    matrix: Matrix,
    color_transform: ColorTransform,
    current_frame: FrameNumber,
    next_frame: FrameNumber,
    total_frames: FrameNumber,
    children: HashMap<Depth, Cc<RefCell<DisplayObjectNode>>>,
    background_color: Color,
}

impl Stage {
    pub fn new(num_frames: u16) -> Cc<RefCell<Stage>> {
        let clip = Stage {
            tag_stream_start: Some(0),
            tag_stream_pos: 0,
            is_playing: true,
            matrix: Matrix::default(),
            color_transform: Default::default(),
            current_frame: 0,
            next_frame: 1,
            total_frames: num_frames,
            children: HashMap::new(),
            background_color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        };
        Cc::new(RefCell::new(clip))
    }

    pub fn background_color(&self) -> &Color {
        &self.background_color
    }
}

impl DisplayObject for Stage {
    fn run_frame(&mut self, context: &mut UpdateContext) {
        use swf::{read::SwfRead, Tag};
        if self.tag_stream_start.is_some() {
            context
                .position_stack
                .push(context.tag_stream.get_ref().position());
            context
                .tag_stream
                .get_inner()
                .set_position(self.tag_stream_pos);

            let mut start_pos = self.tag_stream_pos;
            while let Ok(Some(tag)) = context.tag_stream.read_tag() {
                //trace!("{:?}", tag);
                match tag {
                    Tag::FileAttributes(file_attributes) => (),

                    Tag::SetBackgroundColor(color) => self.background_color = color,

                    Tag::ShowFrame => break,

                    Tag::DefineSceneAndFrameLabelData {
                        scenes,
                        frame_labels,
                    } => (), // TODO(Herschel)

                    Tag::DefineShape(shape) => {
                        if !context.library.contains_character(shape.id) {
                            let svg = crate::shape_utils::swf_shape_to_svg(&shape);
                            use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
                            let url_encoded_svg = format!(
                                "data:image/svg+xml,{}",
                                utf8_percent_encode(&svg, DEFAULT_ENCODE_SET)
                            );

                            let mut image = HtmlImageElement::new().unwrap();
                            image.set_src(&url_encoded_svg);
                            context.library.register_character(
                                shape.id,
                                Character::Graphic {
                                    image,
                                    x_min: shape.shape_bounds.x_min,
                                    y_min: shape.shape_bounds.y_min,
                                },
                            );
                        }
                    }
                    Tag::DefineSprite(sprite) => {
                        let pos = context.tag_stream.get_ref().position();
                        context.tag_stream.get_inner().set_position(start_pos);
                        context.tag_stream.read_tag_code_and_length().unwrap();
                        context.tag_stream.read_u32().unwrap();
                        let mc_start_pos = context.tag_stream.get_ref().position();
                        context.tag_stream.get_inner().set_position(pos);
                        if !context.library.contains_character(sprite.id) {
                            context.library.register_character(
                                sprite.id,
                                Character::MovieClip {
                                    num_frames: sprite.num_frames,
                                    tag_stream_start: mc_start_pos,
                                },
                            );
                        }
                    }

                    Tag::ShowFrame => break,
                    Tag::PlaceObject(place_object) => {
                        MovieClip::run_place_object(&mut self.children, &*place_object, context)
                    }
                    Tag::RemoveObject {
                        depth,
                        character_id,
                    } => {
                        // TODO(Herschel): How does the character ID work for RemoveObject?
                        self.children.remove(&depth).is_some();
                        info!("REMOVE {} {}", depth, self.children.len());
                        //panic!();
                    }

                    _ => info!("Umimplemented tag: {:?}", tag),
                }
                start_pos = context.tag_stream.get_ref().position();
            }
            self.tag_stream_pos = context.tag_stream.get_ref().position();
            context
                .tag_stream
                .get_inner()
                .set_position(context.position_stack.pop().unwrap());

            // Advance frame number.
            if self.next_frame < self.total_frames {
                self.next_frame += 1;
            } else {
                self.next_frame = 1;
                if let Some(start) = self.tag_stream_start {
                    self.tag_stream_pos = start;
                }
            }
        }

        // TODO(Herschel): Verify order of execution for parent/children.
        // Parent first? Children first? Sorted by depth?
        for child in self.children.values() {
            child.borrow_mut().run_frame(context);
        }
    }

    fn update_frame_number(&mut self) {
        self.current_frame = self.next_frame;
        for child in self.children.values() {
            child.borrow_mut().update_frame_number();
        }
    }

    fn render(&self, context: &mut RenderContext) {
        context.matrix_stack.push(&self.matrix);
        context.color_transform_stack.push(&self.color_transform);

        let mut sorted_children: Vec<_> = self.children.iter().collect();
        sorted_children.sort_by_key(|(depth, _)| *depth);

        for child in sorted_children {
            child.1.borrow_mut().render(context);
        }

        context.color_transform_stack.pop();
        context.matrix_stack.pop();
    }

    fn set_matrix(&mut self, matrix: Matrix) {
        self.matrix = matrix;
    }

    fn set_color_transform(&mut self, color_transform: ColorTransform) {
        self.color_transform = color_transform;
    }
}

impl Trace for Stage {
    fn trace(&mut self, tracer: &mut Tracer) {
        for child in self.children.values_mut() {
            child.trace(tracer);
        }
    }
}
