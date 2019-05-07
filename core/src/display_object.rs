use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use crate::transform::Transform;
use bacon_rajan_cc::{Cc, Trace, Tracer};

#[derive(Clone)]
pub struct DisplayObjectBase {
    depth: Depth,
    transform: Transform,
    name: String,
}

impl Default for DisplayObjectBase {
    fn default() -> Self {
        Self {
            depth: Default::default(),
            transform: Default::default(),
            name: Default::default(),
        }
    }
}

impl DisplayObjectImpl for DisplayObjectBase {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> &Matrix {
        &self.transform.matrix
    }
    fn set_matrix(&mut self, matrix: &Matrix) {
        self.transform.matrix = *matrix;
    }
    fn get_color_transform(&self) -> &ColorTransform {
        &self.transform.color_transform
    }
    fn set_color_transform(&mut self, color_transform: &ColorTransform) {
        self.transform.color_transform = *color_transform;
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl Trace for DisplayObjectBase {
    fn trace(&mut self, _tracer: &mut Tracer) {}
}

pub trait DisplayObjectImpl: Trace {
    fn transform(&self) -> &Transform;
    fn get_matrix(&self) -> &Matrix;
    fn set_matrix(&mut self, matrix: &Matrix);
    fn get_color_transform(&self) -> &ColorTransform;
    fn set_color_transform(&mut self, color_transform: &ColorTransform);
    fn name(&self) -> &str;
    fn set_name(&mut self, name: &str);

    fn preload(&self, _context: &mut UpdateContext) {}
    fn run_frame(&mut self, _context: &mut UpdateContext) {}
    fn run_post_frame(&mut self, _context: &mut UpdateContext) {}
    fn render(&self, _context: &mut RenderContext) {}

    fn handle_click(&mut self, _pos: (f32, f32)) {}
    fn visit_children(&self, queue: &mut VecDeque<Cc<RefCell<DisplayObject>>>) {}
    fn as_movie_clip(&self) -> Option<&crate::movie_clip::MovieClip> {
        None
    }
    fn as_movie_clip_mut(&mut self) -> Option<&mut crate::movie_clip::MovieClip> {
        None
    }
}

macro_rules! impl_display_object {
    ($field:ident) => {
            fn transform(&self) -> &crate::transform::Transform {
                self.$field.transform()
            }
            fn get_matrix(&self) -> &Matrix {
                self.$field.get_matrix()
            }
            fn set_matrix(&mut self, matrix: &Matrix) {
                self.$field.set_matrix(matrix)
            }
            fn get_color_transform(&self) -> &ColorTransform {
                self.$field.get_color_transform()
            }
            fn set_color_transform(&mut self, color_transform: &ColorTransform) {
                self.$field.set_color_transform(color_transform)
            }
            fn name(&self) -> &str {
                self.$field.name()
            }
            fn set_name(&mut self, name: &str) {
                self.$field.set_name(name)
            }
    };
}

// TODO(Herschel): We wrap in a box because using a trait object
// directly with Cc gets hairy.
// Extra heap allocation, though.
// Revisit this eventually, some possibilities:
// - Just use a dumb enum.
// - Some DST magic if we remove the Box below and mark this !Sized?
pub struct DisplayObject {
    inner: Box<DisplayObjectImpl>,
}

impl DisplayObject {
    pub fn new(inner: Box<DisplayObjectImpl>) -> DisplayObject {
        DisplayObject { inner }
    }
}

impl DisplayObjectImpl for DisplayObject {
    impl_display_object!(inner);

    fn preload(&self, context: &mut UpdateContext) {
        self.inner.preload(context);
    }

    fn run_frame(&mut self, context: &mut UpdateContext) {
        self.inner.run_frame(context)
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext) {
        self.inner.run_post_frame(context)
    }

    fn render(&self, context: &mut RenderContext) {
        self.inner.render(context)
    }

    fn handle_click(&mut self, pos: (f32, f32)) {
        self.inner.handle_click(pos)
    }

    fn visit_children(&self, queue: &mut VecDeque<Cc<RefCell<DisplayObject>>>) {
        self.inner.visit_children(queue);
    }

    fn as_movie_clip(&self) -> Option<&crate::movie_clip::MovieClip> {
        self.inner.as_movie_clip()
    }

    fn as_movie_clip_mut(&mut self) -> Option<&mut crate::movie_clip::MovieClip> {
        self.inner.as_movie_clip_mut()
    }
}

impl Trace for DisplayObject {
    fn trace(&mut self, tracer: &mut Tracer) {
        self.inner.trace(tracer)
    }
}

use std::cell::RefCell;
use std::collections::VecDeque;

pub struct DisplayObjectVisitor {
    pub open: VecDeque<Cc<RefCell<DisplayObject>>>,
}

impl DisplayObjectVisitor {
    pub fn run(&mut self, context: &mut crate::player::UpdateContext) {
        let root = self.open[0].clone();
        while let Some(node) = self.open.pop_front() {
            // {
            //     let mut node = node.borrow_mut();
            //     node.run_frame(context);
            // }
            let mut action = None;
            if let Some(clip) = node.borrow().as_movie_clip() {
                action = clip.action();
            }
            if let Some((pos, len)) = action {
                let mut action_context = crate::avm1::ActionContext {
                    global_time: context.global_time,
                    start_clip: node.clone(),
                    active_clip: node.clone(),
                    root: root.clone(),
                    audio: context.audio,
                };
                let data = &context.tag_stream.get_ref().get_ref()[pos..pos + len];
                if let Err(e) = context.avm1.do_action(&mut action_context, &data[..]) {}
            }
            node.borrow_mut().run_post_frame(context);
            node.borrow().visit_children(&mut self.open);
        }
    }
}
