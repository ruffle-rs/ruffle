use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use crate::transform::Transform;

#[derive(Clone, Debug)]
pub struct Text<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, TextStatic>,
}

impl<'gc> Text<'gc> {
    pub fn from_swf_tag(context: &mut UpdateContext<'_, 'gc, '_>, tag: &swf::Text) -> Self {
        Self {
            base: Default::default(),
            static_data: gc_arena::Gc::allocate(
                context.gc_context,
                TextStatic {
                    id: tag.id,
                    text_transform: tag.matrix.clone().into(),
                    text_blocks: tag.records.clone(),
                },
            ),
        }
    }
}

impl<'gc> DisplayObject<'gc> for Text<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.static_data.id
    }

    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());
        context.transform_stack.push(&Transform {
            matrix: self.static_data.text_transform,
            ..Default::default()
        });

        let mut color = swf::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        let mut font_id = 0;
        let mut height = 0;
        let mut transform: Transform = Default::default();
        for block in &self.static_data.text_blocks {
            if let Some(x) = block.x_offset {
                transform.matrix.tx = x.get() as f32;
            }
            if let Some(y) = block.y_offset {
                transform.matrix.ty = y.get() as f32;
            }
            color = block.color.as_ref().unwrap_or(&color).clone();
            font_id = block.font_id.unwrap_or(font_id);
            height = block.height.unwrap_or(height);
            let scale = f32::from(height) / 1024.0;
            transform.matrix.a = scale;
            transform.matrix.d = scale;
            transform.color_transform.r_mult = f32::from(color.r) / 255.0;
            transform.color_transform.g_mult = f32::from(color.g) / 255.0;
            transform.color_transform.b_mult = f32::from(color.b) / 255.0;
            transform.color_transform.a_mult = f32::from(color.a) / 255.0;
            if let Some(font) = context.library.get_font(font_id) {
                for c in &block.glyphs {
                    if let Some(glyph) = font.get_glyph(c.index as usize) {
                        context.transform_stack.push(&transform);
                        context
                            .renderer
                            .render_shape(glyph, context.transform_stack.transform());
                        context.transform_stack.pop();
                        transform.matrix.tx += c.advance as f32;
                    }
                }
            }
        }
        context.transform_stack.pop();
        context.transform_stack.pop();
    }
}

unsafe impl<'gc> gc_arena::Collect for Text<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.static_data.trace(cc);
    }
}

/// Static data shared between all instances of a text object.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct TextStatic {
    id: CharacterId,
    text_transform: Matrix,
    text_blocks: Vec<swf::TextRecord>,
}

unsafe impl<'gc> gc_arena::Collect for TextStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
