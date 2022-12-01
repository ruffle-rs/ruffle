use crate::avm2::{
    Activation as Avm2Activation, Object as Avm2Object, StageObject as Avm2StageObject,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::font::TextRenderSettings;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::commands::CommandHandler;
use ruffle_render::transform::Transform;
use std::cell::{Ref, RefMut};
use std::sync::Arc;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Text<'gc>(GcCell<'gc, TextData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct TextData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, TextStatic>,
    render_settings: TextRenderSettings,
    avm2_object: Option<Avm2Object<'gc>>,
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
                        bounds: (&tag.bounds).into(),
                        text_transform: tag.matrix.into(),
                        text_blocks: tag.records.clone(),
                    },
                ),
                render_settings: Default::default(),
                avm2_object: None,
            },
        ))
    }

    pub fn set_render_settings(
        self,
        gc_context: MutationContext<'gc, '_>,
        settings: TextRenderSettings,
    ) {
        self.0.write(gc_context).render_settings = settings
    }
}

impl<'gc> TDisplayObject<'gc> for Text<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        Some(self.0.read().static_data.swf.clone())
    }

    fn replace_with(&self, context: &mut UpdateContext<'_, 'gc, '_>, id: CharacterId) {
        if let Some(new_text) = context
            .library
            .library_for_movie_mut(self.movie().unwrap())
            .get_text(id)
        {
            self.0.write(context.gc_context).static_data = new_text.0.read().static_data;
        } else {
            log::warn!("PlaceObject: expected text at character ID {}", id);
        }
    }

    fn run_frame(&self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render_self(&self, context: &mut RenderContext) {
        let tf = self.0.read();
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
        let mut height = Twips::ZERO;
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
                transform.color_transform.set_mult_color(&color);
                for c in &block.glyphs {
                    if let Some(glyph) = font.get_glyph(c.index as usize) {
                        context.transform_stack.push(&transform);
                        let glyph_shape_handle = glyph.shape_handle(context.renderer);
                        context
                            .commands
                            .render_shape(glyph_shape_handle, context.transform_stack.transform());
                        context.transform_stack.pop();
                        transform.matrix.tx += Twips::new(c.advance);
                    }
                }
            }
        }
        context.transform_stack.pop();
    }

    fn self_bounds(&self) -> BoundingBox {
        self.0.read().static_data.bounds.clone()
    }

    fn hit_test_shape(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        mut point: (Twips, Twips),
        _options: HitTestOptions,
    ) -> bool {
        if self.world_bounds().contains(point) {
            // Texts using the "Advanced text rendering" always hit test using their bounding box.
            if self.0.read().render_settings.is_advanced() {
                return true;
            }

            // Transform the point into the text's local space.
            let local_matrix = self.global_to_local_matrix();
            let tf = self.0.read();
            let mut text_matrix = tf.static_data.text_transform;
            text_matrix.invert();
            point = text_matrix * local_matrix * point;

            let mut font_id = 0;
            let mut height = Twips::ZERO;
            let mut glyph_matrix = Matrix::default();
            for block in &tf.static_data.text_blocks {
                if let Some(x) = block.x_offset {
                    glyph_matrix.tx = x;
                }
                if let Some(y) = block.y_offset {
                    glyph_matrix.ty = y;
                }
                font_id = block.font_id.unwrap_or(font_id);
                height = block.height.unwrap_or(height);

                if let Some(font) = context
                    .library
                    .library_for_movie(self.movie().unwrap())
                    .unwrap()
                    .get_font(font_id)
                {
                    let scale = (height.get() as f32) / font.scale();
                    glyph_matrix.a = scale;
                    glyph_matrix.d = scale;
                    for c in &block.glyphs {
                        if let Some(glyph) = font.get_glyph(c.index as usize) {
                            // Transform the point into glyph space and test.
                            let mut matrix = glyph_matrix;
                            matrix.invert();
                            let point = matrix * point;
                            let glyph_shape = glyph.as_shape();
                            let glyph_bounds: BoundingBox = (&glyph_shape.shape_bounds).into();
                            if glyph_bounds.contains(point)
                                && ruffle_render::shape_utils::shape_hit_test(
                                    &glyph_shape,
                                    point,
                                    &local_matrix,
                                )
                            {
                                return true;
                            }

                            glyph_matrix.tx += Twips::new(c.advance);
                        }
                    }
                }
            }
        }

        false
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _init_object: Option<crate::avm1::Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        if context.is_action_script_3() {
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            let statictext = activation.avm2().classes().statictext;
            match Avm2StageObject::for_display_object_childless(
                &mut activation,
                (*self).into(),
                statictext,
            ) {
                Ok(object) => {
                    self.0.write(activation.context.gc_context).avm2_object = Some(object.into())
                }
                Err(e) => log::error!("Got error when creating AVM2 side of Text: {}", e),
            }
        }
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .avm2_object
            .map(|o| o.into())
            .unwrap_or(Avm2Value::Undefined)
    }

    fn set_object2(&mut self, mc: MutationContext<'gc, '_>, to: Avm2Object<'gc>) {
        self.0.write(mc).avm2_object = Some(to);
    }
}

/// Static data shared between all instances of a text object.
#[allow(dead_code)]
#[derive(Debug, Clone, Collect)]
#[collect(require_static)]
struct TextStatic {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    bounds: BoundingBox,
    text_transform: Matrix,
    text_blocks: Vec<swf::TextRecord>,
}
