use crate::ColorTransform;
use crate::Matrix;
use crate::{graphic::Graphic, MovieClip, Stage};
use crate::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Trace, Tracer};

pub trait DisplayObject {
    //fn children_gc_mut(&self) -> std::slice::Iter<&mut DisplayObjectNode>;
    fn run_frame(&mut self, context: &mut UpdateContext);
    fn update_frame_number(&mut self) {}
    fn render(&self, context: &mut RenderContext);
    fn set_matrix(&mut self, matrix: Matrix);
    fn set_color_transform(&mut self, color_transform: ColorTransform);
}

pub enum DisplayObjectNode {
    Graphic(Graphic),
    MovieClip(MovieClip),
    Stage(Stage),
}

impl DisplayObject for DisplayObjectNode {
    fn run_frame(&mut self, context: &mut UpdateContext) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.run_frame(context),
            DisplayObjectNode::MovieClip(movie_clip) => movie_clip.run_frame(context),
            DisplayObjectNode::Stage(stage) => stage.run_frame(context),
        }
    }

    fn update_frame_number(&mut self) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.update_frame_number(),
            DisplayObjectNode::MovieClip(movie_clip) => movie_clip.update_frame_number(),
            DisplayObjectNode::Stage(stage) => stage.update_frame_number(),
        }
    }

    fn render(&self, context: &mut RenderContext) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.render(context),
            DisplayObjectNode::MovieClip(movie_clip) => movie_clip.render(context),
            DisplayObjectNode::Stage(stage) => stage.render(context),
        }
    }

    fn set_matrix(&mut self, matrix: Matrix) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.set_matrix(matrix),
            DisplayObjectNode::MovieClip(movie_clip) => movie_clip.set_matrix(matrix),
            DisplayObjectNode::Stage(stage) => stage.set_matrix(matrix),
        }
    }

    fn set_color_transform(&mut self, color_transform: ColorTransform) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.set_color_transform(color_transform),
            DisplayObjectNode::MovieClip(movie_clip) => {
                movie_clip.set_color_transform(color_transform)
            }
            DisplayObjectNode::Stage(stage) => stage.set_color_transform(color_transform),
        }
    }
}

impl Trace for DisplayObjectNode {
    fn trace(&mut self, tracer: &mut Tracer) {
        match self {
            DisplayObjectNode::Graphic(graphic) => graphic.trace(tracer),
            DisplayObjectNode::MovieClip(movie_clip) => movie_clip.trace(tracer),
            DisplayObjectNode::Stage(stage) => stage.trace(tracer),
        }
    }
}
