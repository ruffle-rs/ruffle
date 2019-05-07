use crate::backend::render::ShapeHandle;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObjectBase, DisplayObjectImpl};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Trace, Tracer};

#[derive(Clone)]
pub struct Text {
    base: DisplayObjectBase,
    text_blocks: Vec<swf::TextRecord>,
}

impl Text {
    pub fn from_swf_tag(tag: &swf::Text) -> Self {
        Self {
            base: Default::default(),
            text_blocks: tag.records.clone(),
        }
    }
}

impl DisplayObjectImpl for Text {
    impl_display_object!(base);

    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());

        let mut x = 0.0;
        let mut y = 0.0;
        let mut color = swf::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        let mut font_id = 0;
        let mut height = 0;
        for block in &self.text_blocks {
            x = block.x_offset.unwrap_or(x);
            y = block.y_offset.unwrap_or(y);
            color = block.color.as_ref().unwrap_or_else(|| &color).clone();
            font_id = block.font_id.unwrap_or(font_id);
            height = block.height.unwrap_or(height);
            let mut transform = context.transform_stack.transform().clone();
            transform.matrix.ty += y;
            transform.color_transform.r_mult = f32::from(color.r) / 255.0;
            transform.color_transform.g_mult = f32::from(color.g) / 255.0;
            transform.color_transform.b_mult = f32::from(color.b) / 255.0;
            transform.color_transform.a_mult = f32::from(color.a) / 255.0;
            if let Some(font) = context.library.get_font(font_id) {
                for c in &block.glyphs {
                    if let Some(glyph) = font.get_glyph(c.index as usize) {
                        context.renderer.render_shape(glyph, &transform);
                        x += c.advance as f32 / 20.0;
                        transform.matrix.tx += c.advance as f32 / 20.0;
                    }
                }
            }
        }
        context.transform_stack.pop();
    }
}

impl Trace for Text {
    fn trace(&mut self, _tracer: &mut Tracer) {
        // Noop
    }
}
