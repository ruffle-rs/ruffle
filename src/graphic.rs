use crate::display_object::DisplayObject;
use crate::library::Library;
use crate::Matrix;
use crate::RenderContext;
use bacon_rajan_cc::{Trace, Tracer};
use web_sys::HtmlImageElement;

pub struct Graphic {
    matrix: Matrix,
    image: HtmlImageElement,
}

impl Graphic {
    pub fn new(image: HtmlImageElement) -> Graphic {
        Graphic {
            image,
            matrix: std::default::Default::default(),
        }
    }
}

impl DisplayObject for Graphic {
    fn run_frame(&mut self, _library: &Library) {
        // Noop
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
        context
            .context_2d
            .draw_image_with_html_image_element(&self.image, 0.0, 0.0)
            .unwrap();
        context.matrix_stack.pop();
    }
}

impl Trace for Graphic {
    fn trace(&mut self, _tracer: &mut Tracer) {
        // Noop

    }
}
