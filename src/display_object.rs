use crate::Library;
use crate::RenderContext;
use crate::{graphic::Graphic, MovieClip};
use bacon_rajan_cc::{Trace, Tracer};

pub trait DisplayObject {
    //fn children_gc_mut(&self) -> std::slice::Iter<&mut DisplayObjectNode>;
    fn run_frame(&mut self, library: &Library);
    fn render(&self, context: &mut RenderContext);
}

pub enum DisplayObjectNode {
    Graphic(Graphic),
    MovieClip(MovieClip),
}

impl DisplayObject for DisplayObjectNode {
    fn run_frame(&mut self, library: &Library) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.run_frame(library),
            DisplayObjectNode::MovieClip(movie_clip) => movie_clip.run_frame(library),
        }
    }

    fn render(&self, context: &mut RenderContext) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.render(context),
            DisplayObjectNode::MovieClip(movie_clip) => movie_clip.render(context),
        }
    }
}

impl Trace for DisplayObjectNode {
    fn trace(&mut self, tracer: &mut Tracer) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.trace(tracer),
            DisplayObjectNode::MovieClip(movie_clip) => movie_clip.trace(tracer),
        }
    }
}
