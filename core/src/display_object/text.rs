use crate::avm2::{
    Activation as Avm2Activation, Object as Avm2Object, StageObject as Avm2StageObject,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr};
use crate::font::TextRenderSettings;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::{Collect, GcCell, Mutation};
use ruffle_render::commands::CommandHandler;
use ruffle_render::transform::Transform;
use std::cell::{Ref, RefMut};
use std::sync::Arc;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Text<'gc>(GcCell<'gc, TextData<'gc>>);

impl fmt::Debug for Text<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Text")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct TextData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, TextStatic>,
    #[collect(require_static)]
    render_settings: TextRenderSettings,
    avm2_object: Option<Avm2Object<'gc>>,
}

impl<'gc> Text<'gc> {
    pub fn from_swf_tag(
        context: &mut UpdateContext<'gc>,
        swf: Arc<SwfMovie>,
        tag: &swf::Text,
    ) -> Self {
        Text(GcCell::new(
            context.gc_context,
            TextData {
                base: Default::default(),
                static_data: gc_arena::Gc::new(
                    context.gc_context,
                    TextStatic {
                        swf,
                        id: tag.id,
                        bounds: tag.bounds.clone(),
                        text_transform: tag.matrix.into(),
                        text_blocks: tag.records.clone(),
                    },
                ),
                render_settings: Default::default(),
                avm2_object: None,
            },
        ))
    }

    pub fn set_render_settings(self, gc_context: &Mutation<'gc>, settings: TextRenderSettings) {
        self.0.write(gc_context).render_settings = settings;
        self.invalidate_cached_bitmap(gc_context);
    }
}

impl<'gc> TDisplayObject<'gc> for Text<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn base_mut<'a>(&'a self, mc: &Mutation<'gc>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn instantiate(&self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(GcCell::new(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.read().static_data.swf.clone()
    }

    fn replace_with(&self, context: &mut UpdateContext<'gc>, id: CharacterId) {
        if let Some(new_text) = context
            .library
            .library_for_movie_mut(self.movie())
            .get_text(id)
        {
            self.0.write(context.gc_context).static_data = new_text.0.read().static_data;
        } else {
            tracing::warn!("PlaceObject: expected text at character ID {}", id);
        }
        self.invalidate_cached_bitmap(context.gc_context);
    }

    fn run_frame_avm1(&self, _context: &mut UpdateContext) {
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
            color = block.color.unwrap_or(color);
            font_id = block.font_id.unwrap_or(font_id);
            height = block.height.unwrap_or(height);
            if let Some(font) = context
                .library
                .library_for_movie(self.movie())
                .unwrap()
                .get_font(font_id)
            {
                let scale = (height.get() as f32) / font.scale();
                transform.matrix.a = scale;
                transform.matrix.d = scale;
                transform.color_transform.set_mult_color(&color);
                for c in &block.glyphs {
                    if let Some(glyph) = font.get_glyph(c.index as usize) {
                        if let Some(glyph_shape_handle) = glyph.shape_handle(context.renderer) {
                            context.transform_stack.push(&transform);
                            context.commands.render_shape(
                                glyph_shape_handle,
                                context.transform_stack.transform(),
                            );
                            context.transform_stack.pop();
                        }

                        transform.matrix.tx += Twips::new(c.advance);
                    }
                }
            }
        }
        context.transform_stack.pop();
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        self.0.read().static_data.bounds.clone()
    }

    fn hit_test_shape(
        &self,
        context: &mut UpdateContext<'gc>,
        mut point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        if (!options.contains(HitTestOptions::SKIP_INVISIBLE) || self.visible())
            && self.world_bounds().contains(point)
        {
            // Texts using the "Advanced text rendering" always hit test using their bounding box.
            if self.0.read().render_settings.is_advanced() {
                return true;
            }

            // Transform the point into the text's local space.
            let Some(local_matrix) = self.global_to_local_matrix() else {
                return false;
            };
            let tf = self.0.read();
            let Some(text_matrix) = tf.static_data.text_transform.inverse() else {
                return false;
            };
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
                    .library_for_movie(self.movie())
                    .unwrap()
                    .get_font(font_id)
                {
                    let scale = (height.get() as f32) / font.scale();
                    glyph_matrix.a = scale;
                    glyph_matrix.d = scale;
                    for c in &block.glyphs {
                        if let Some(glyph) = font.get_glyph(c.index as usize) {
                            // Transform the point into glyph space and test.
                            let Some(matrix) = glyph_matrix.inverse() else {
                                return false;
                            };
                            let point = matrix * point;
                            if glyph.hit_test(point, &local_matrix) {
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
        context: &mut UpdateContext<'gc>,
        _init_object: Option<crate::avm1::Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        if self.movie().is_action_script_3() {
            let domain = context
                .library
                .library_for_movie(self.movie())
                .unwrap()
                .avm2_domain();
            let mut activation = Avm2Activation::from_domain(context, domain);
            let statictext = activation.avm2().classes().statictext;
            match Avm2StageObject::for_display_object_childless(
                &mut activation,
                (*self).into(),
                statictext,
            ) {
                Ok(object) => {
                    self.0.write(activation.context.gc_context).avm2_object = Some(object.into())
                }
                Err(e) => tracing::error!("Got error when creating AVM2 side of Text: {}", e),
            }

            self.on_construction_complete(context);
        }
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .avm2_object
            .map(|o| o.into())
            .unwrap_or(Avm2Value::Null)
    }

    fn set_object2(&self, context: &mut UpdateContext<'gc>, to: Avm2Object<'gc>) {
        self.0.write(context.gc_context).avm2_object = Some(to);
    }
}

/// Static data shared between all instances of a text object.
#[allow(dead_code)]
#[derive(Debug, Clone, Collect)]
#[collect(require_static)]
struct TextStatic {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    bounds: Rectangle<Twips>,
    text_transform: Matrix,
    text_blocks: Vec<swf::TextRecord>,
}
