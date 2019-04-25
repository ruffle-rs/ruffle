use crate::display_object::{DisplayObject, DisplayObjectNode};
use crate::matrix::Matrix;
use crate::Library;
use crate::RenderContext;
use bacon_rajan_cc::{Cc, Trace, Tracer};
use log::{info, trace, warn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;

type Depth = i16;
type FrameNumber = u16;

pub struct MovieClip {
    tag_stream: Option<swf::read::Reader<Cursor<Vec<u8>>>>,
    is_playing: bool,
    matrix: Matrix,
    current_frame: FrameNumber,
    total_frames: FrameNumber,
    children: HashMap<Depth, Cc<RefCell<DisplayObjectNode>>>,
}

impl MovieClip {
    pub fn new() -> Cc<RefCell<MovieClip>> {
        let clip = MovieClip {
            tag_stream: None,
            is_playing: true,
            matrix: Matrix::default(),
            current_frame: 1,
            total_frames: 1,
            children: HashMap::new(),
        };
        Cc::new(RefCell::new(clip))
    }

    pub fn new_with_data(
        tag_stream: swf::read::Reader<Cursor<Vec<u8>>>,
        num_frames: u16,
    ) -> MovieClip {
        MovieClip {
            tag_stream: Some(tag_stream),
            is_playing: true,
            matrix: Matrix::default(),
            current_frame: 1,
            total_frames: num_frames,
            children: HashMap::new(),
        }
    }

    fn run_place_object(&mut self, place_object: &swf::PlaceObject, library: &Library) {
        use swf::PlaceObjectAction;
        match place_object.action {
            PlaceObjectAction::Place(id) => {
                // TODO(Herschel): Behavior when character doesn't exist/isn't a DisplayObject?
                let mut character = library.instantiate_display_object(id).unwrap();

                // TODO(Herschel): Behavior when depth is occupied? (I think it replaces)
                self.children
                    .insert(place_object.depth, Cc::new(RefCell::new(character)));
            }
            PlaceObjectAction::Modify => (),
            PlaceObjectAction::Replace(id) => {
                let mut character = library.instantiate_display_object(id).unwrap();

                // TODO(Herschel): Behavior when depth is occupied? (I think it replaces)
                self.children
                    .insert(place_object.depth, Cc::new(RefCell::new(character)));
            }
        }
    }
}

impl DisplayObject for MovieClip {
    fn run_frame(&mut self, library: &Library) {
        use swf::Tag;
        if self.tag_stream.is_some() {
            while let Ok(Some(tag)) = self.tag_stream.as_mut().unwrap().read_tag() {
                trace!("{:?}", tag);
                match tag {
                    Tag::ShowFrame => break,
                    Tag::PlaceObject(place_object) => {
                        self.run_place_object(&*place_object, library)
                    }
                    _ => unimplemented!("Umimplemented tag: {:?}", tag),
                }
            }

            // Advance frame number.
            self.current_frame += 1;
            if self.current_frame < self.total_frames {
                self.current_frame += 1;
            } else {
                self.current_frame = 1;
                //tag_stream.to_inner().set_position(0);
            }
        }

        // TODO(Herschel): Verify order of execution for parent/children.
        // Parent first? Children first? Sorted by depth?
        for child in self.children.values() {
            child.borrow_mut().run_frame(library);
        }
    }

    fn render(&self, context: &mut RenderContext) {
        context.matrix_stack.push(&self.matrix);
        let world_matrix = context.matrix_stack.matrix();
        context
            .context_2d
            .transform(
                world_matrix.a.into(),
                world_matrix.b.into(),
                world_matrix.c.into(),
                world_matrix.d.into(),
                world_matrix.tx.into(),
                world_matrix.ty.into(),
            )
            .unwrap();

        for child in self.children.values() {
            child.borrow_mut().render(context);
        }

        context.matrix_stack.pop();
    }
}

impl Trace for MovieClip {
    fn trace(&mut self, tracer: &mut Tracer) {
        for child in self.children.values_mut() {
            child.trace(tracer);
        }
    }
}
