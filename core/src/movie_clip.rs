use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObject, DisplayObjectNode};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Cc, Trace, Tracer};
use log::info;
use std::cell::RefCell;
use std::collections::HashMap;

type Depth = i16;
type FrameNumber = u16;

pub struct MovieClip {
    tag_stream_start: Option<u64>,
    tag_stream_pos: u64,
    is_playing: bool,
    matrix: Matrix,
    color_transform: ColorTransform,
    current_frame: FrameNumber,
    next_frame: FrameNumber,
    total_frames: FrameNumber,
    children: HashMap<Depth, Cc<RefCell<DisplayObjectNode>>>,
}

impl MovieClip {
    pub fn new() -> Cc<RefCell<MovieClip>> {
        let clip = MovieClip {
            tag_stream_start: None,
            tag_stream_pos: 0,
            is_playing: true,
            matrix: Matrix::default(),
            color_transform: Default::default(),
            current_frame: 0,
            next_frame: 1,
            total_frames: 1,
            children: HashMap::new(),
        };
        Cc::new(RefCell::new(clip))
    }

    pub fn new_with_data(tag_stream_start: u64, num_frames: u16) -> MovieClip {
        info!("start: {} ", tag_stream_start);
        MovieClip {
            tag_stream_start: Some(tag_stream_start),
            tag_stream_pos: tag_stream_start,
            is_playing: true,
            matrix: Matrix::default(),
            color_transform: Default::default(),
            current_frame: 0,
            next_frame: 1,
            total_frames: num_frames,
            children: HashMap::new(),
        }
    }

    pub fn run_place_object(
        children: &mut HashMap<Depth, Cc<RefCell<DisplayObjectNode>>>,
        place_object: &swf::PlaceObject,
        context: &mut UpdateContext,
    ) {
        use swf::PlaceObjectAction;
        let character = match place_object.action {
            PlaceObjectAction::Place(id) => {
                // TODO(Herschel): Behavior when character doesn't exist/isn't a DisplayObject?
                let character =
                    if let Ok(character) = context.library.instantiate_display_object(id) {
                        Cc::new(RefCell::new(character))
                    } else {
                        return;
                    };

                // TODO(Herschel): Behavior when depth is occupied? (I think it replaces)
                children.insert(place_object.depth, character.clone());
                character
            }
            PlaceObjectAction::Modify => {
                if let Some(child) = children.get(&place_object.depth) {
                    child.clone()
                } else {
                    return;
                }
            }
            PlaceObjectAction::Replace(id) => {
                let character =
                    if let Ok(character) = context.library.instantiate_display_object(id) {
                        Cc::new(RefCell::new(character))
                    } else {
                        return;
                    };

                children.insert(place_object.depth, character.clone());
                character
            }
        };

        let mut character = character.borrow_mut();
        if let Some(matrix) = &place_object.matrix {
            let m = matrix.clone();
            character.set_matrix(Matrix::from(m));
        }
    }
}

impl DisplayObject for MovieClip {
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

            while let Ok(Some(tag)) = context.tag_stream.read_tag() {
                //trace!("mc: {:?}", tag);
                match tag {
                    Tag::ShowFrame => break,
                    Tag::PlaceObject(place_object) => {
                        MovieClip::run_place_object(&mut self.children, &*place_object, context)
                    }
                    Tag::RemoveObject { depth, .. } => {
                        // TODO(Herschel): How does the character ID work for RemoveObject?
                        self.children.remove(&depth);
                        info!("REMOVE {} {}", depth, self.children.len());
                    }

                    Tag::JpegTables(_) => (),
                    Tag::SoundStreamHead(_) => (),
                    Tag::SoundStreamHead2(_) => (),
                    Tag::SoundStreamBlock(_) => (),
                    Tag::DoAction(_) => (),
                    _ => info!("Umimplemented tag: {:?}", tag),
                }
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

        context.matrix_stack.pop();
        context.color_transform_stack.pop();
    }

    fn set_matrix(&mut self, matrix: Matrix) {
        self.matrix = matrix;
    }

    fn set_color_transform(&mut self, color_transform: ColorTransform) {
        self.color_transform = color_transform;
    }
}

impl Trace for MovieClip {
    fn trace(&mut self, tracer: &mut Tracer) {
        for child in self.children.values_mut() {
            child.trace(tracer);
        }
    }
}
