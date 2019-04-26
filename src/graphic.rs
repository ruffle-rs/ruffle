use crate::color_transform::ColorTransform;
use crate::display_object::DisplayObject;
use crate::library::Library;
use crate::Matrix;
use crate::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Trace, Tracer};
use log::{info, trace, warn};
use web_sys::HtmlImageElement;

pub struct Graphic {
    matrix: Matrix,
    color_transform: ColorTransform,
    x_min: f32,
    y_min: f32,
    image: HtmlImageElement,
}

impl Graphic {
    pub fn new(image: HtmlImageElement, x_min: f32, y_min: f32) -> Graphic {
        Graphic {
            image,
            color_transform: Default::default(),
            x_min,
            y_min,
            matrix: std::default::Default::default(),
        }
    }
}

impl DisplayObject for Graphic {
    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        context.matrix_stack.push(&self.matrix);
        context.color_transform_stack.push(&self.color_transform);

        let world_matrix = context.matrix_stack.matrix();
        let color_transform = context.color_transform_stack.color_transform();

        if !color_transform.is_identity() {
            context
                .context_2d
                .set_global_alpha(color_transform.a_mult.into());
        }

        context
            .context_2d
            .set_transform(
                world_matrix.a.into(),
                world_matrix.b.into(),
                world_matrix.c.into(),
                world_matrix.d.into(),
                world_matrix.tx.into(),
                world_matrix.ty.into(),
            )
            .unwrap();

        if !color_transform.is_identity() {
            context.context_2d.set_global_alpha(1.0);
        }

        context
            .context_2d
            .draw_image_with_html_image_element(&self.image, self.x_min.into(), self.y_min.into())
            .expect("Couldn't render image");

        context.matrix_stack.pop();
        context.color_transform_stack.push(&self.color_transform);
    }

    fn set_matrix(&mut self, matrix: Matrix) {
        self.matrix = matrix;
    }

    fn set_color_transform(&mut self, color_transform: ColorTransform) {
        self.color_transform = color_transform;
    }
}

impl Trace for Graphic {
    fn trace(&mut self, _tracer: &mut Tracer) {
        // Noop

    }
}
