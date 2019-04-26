use crate::display_object::DisplayObject;
use crate::library::Library;
use crate::Matrix;
use crate::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Trace, Tracer};
use log::{info, trace, warn};
use web_sys::HtmlImageElement;

pub struct Graphic {
    matrix: Matrix,
    x_min: f32,
    y_min: f32,
    image: HtmlImageElement,
}

impl Graphic {
    pub fn new(image: HtmlImageElement, x_min: f32, y_min: f32) -> Graphic {
        Graphic {
            image,
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
        let world_matrix = context.matrix_stack.matrix();
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
        context
            .context_2d
            .draw_image_with_html_image_element(&self.image, self.x_min.into(), self.y_min.into())
            .expect("Couldn't render image");
        context.matrix_stack.pop();
    }

    fn set_matrix(&mut self, matrix: Matrix) {
        self.matrix = matrix;
    }
}

impl Trace for Graphic {
    fn trace(&mut self, _tracer: &mut Tracer) {
        // Noop

    }
}
