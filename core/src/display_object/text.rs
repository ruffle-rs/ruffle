use crate::avm1::Avm1;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::transform::Transform;
use gc_arena::{Collect, GcCell};
use std::sync::Arc;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Text<'gc>(GcCell<'gc, TextData<'gc>>);

#[derive(Clone, Debug)]
pub struct TextData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, TextStatic>,
}

impl<'gc> Text<'gc> {
    pub fn from_swf_tag(
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf: Arc<SwfMovie>,
        tag: &swf::Text,
    ) -> Self {
        Text(GcCell::allocate(
            context.gc_context,
            TextData {
                base: Default::default(),
                static_data: gc_arena::Gc::allocate(
                    context.gc_context,
                    TextStatic {
                        swf,
                        id: tag.id,
                        bounds: tag.bounds.clone().into(),
                        text_transform: tag.matrix,
                        text_blocks: tag.records.clone(),
                    },
                ),
            },
        ))
    }
}

impl<'gc> TDisplayObject<'gc> for Text<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        Some(self.0.read().static_data.swf.clone())
    }

    fn run_frame(&mut self, _avm: &mut Avm1<'gc>, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        let tf = self.0.read();
        context.transform_stack.push(&*self.transform());
        context.transform_stack.push(&Transform {
            matrix: tf.static_data.text_transform,
            ..Default::default()
        });

        let mut color = swf::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        let mut font_id = 0;
        let mut height = Twips::new(0);
        let mut transform: Transform = Default::default();
        for block in &tf.static_data.text_blocks {
            if let Some(x) = block.x_offset {
                transform.matrix.tx = x;
            }
            if let Some(y) = block.y_offset {
                transform.matrix.ty = y;
            }
            color = block.color.as_ref().unwrap_or(&color).clone();
            font_id = block.font_id.unwrap_or(font_id);
            height = block.height.unwrap_or(height);
            if let Some(font) = context
                .library
                .library_for_movie(self.movie().unwrap())
                .unwrap()
                .get_font(font_id)
            {
                let scale = (height.get() as f32) / font.scale();
                transform.matrix.a = scale;
                transform.matrix.d = scale;
                transform.color_transform.r_mult = f32::from(color.r) / 255.0;
                transform.color_transform.g_mult = f32::from(color.g) / 255.0;
                transform.color_transform.b_mult = f32::from(color.b) / 255.0;
                transform.color_transform.a_mult = f32::from(color.a) / 255.0;
                for c in &block.glyphs {
                    if let Some(glyph) = font.get_glyph(c.index as usize) {
                        context.transform_stack.push(&transform);
                        context
                            .renderer
                            .render_shape(glyph.shape, context.transform_stack.transform());
                        context.transform_stack.pop();
                        transform.matrix.tx += Twips::new(c.advance);
                    }
                }
            }
        }
        context.transform_stack.pop();
        context.transform_stack.pop();
    }

    fn self_bounds(&self) -> BoundingBox {
        self.0.read().static_data.bounds.clone()
    }
}

unsafe impl<'gc> gc_arena::Collect for TextData<'gc> {
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
    swf: Arc<SwfMovie>,
    id: CharacterId,
    bounds: BoundingBox,
    text_transform: Matrix,
    text_blocks: Vec<swf::TextRecord>,
}

unsafe impl<'gc> gc_arena::Collect for TextStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
