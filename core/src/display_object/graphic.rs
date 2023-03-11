use crate::avm1::Object as Avm1Object;
use crate::avm2::{
    Activation as Avm2Activation, Object as Avm2Object, StageObject as Avm2StageObject,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::drawing::Drawing;
use crate::library::MovieLibrarySource;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::backend::ShapeHandle;
use ruffle_render::commands::CommandHandler;
use ruffle_render::shape_utils::{DistilledShape, ShapeStrokes};
use ruffle_render::transform::Transform;
use std::cell::{Ref, RefMut};
use std::sync::Arc;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Graphic<'gc>(GcCell<'gc, GraphicData<'gc>>);

impl fmt::Debug for Graphic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Graphic")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct GraphicData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, GraphicStatic>,
    avm2_object: Option<Avm2Object<'gc>>,
    drawing: Option<Drawing>,
    #[collect(require_static)]
    strokes_handle: Option<ShapeHandle>,
    #[collect(require_static)]
    last_scale: (f32, f32),
}

impl<'gc> Graphic<'gc> {
    /// Construct a `Graphic` from it's associated `Shape` tag.
    pub fn from_swf_tag(
        context: &mut UpdateContext<'_, 'gc>,
        swf_shape: swf::Shape,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let library = context.library.library_for_movie(movie.clone()).unwrap();
        let bitmap_source = MovieLibrarySource {
            library,
            gc_context: context.gc_context,
        };
        let shape = DistilledShape::from_shape(&swf_shape, &bitmap_source, context.renderer);

        let static_data = GraphicStatic {
            id: swf_shape.id,
            bounds: swf_shape.shape_bounds.clone(),
            fills_handle: Some(
                context
                    .renderer
                    .register_shape_fills(&shape.fills, shape.id),
            ),
            strokes: Some(shape.strokes),
            shape: swf_shape,
            movie,
        };

        Graphic(GcCell::allocate(
            context.gc_context,
            GraphicData {
                base: Default::default(),
                static_data: gc_arena::Gc::allocate(context.gc_context, static_data),
                avm2_object: None,
                drawing: None,
                strokes_handle: None,
                last_scale: (0.0, 0.0),
            },
        ))
    }

    /// Construct an empty `Graphic`.
    pub fn new_with_avm2(
        context: &mut UpdateContext<'_, 'gc>,
        avm2_object: Avm2Object<'gc>,
    ) -> Self {
        let static_data = GraphicStatic {
            id: 0,
            bounds: Default::default(),
            fills_handle: None,
            strokes: None,
            shape: swf::Shape {
                version: 32,
                id: 0,
                shape_bounds: Default::default(),
                edge_bounds: Default::default(),
                flags: swf::ShapeFlag::empty(),
                styles: swf::ShapeStyles {
                    fill_styles: Vec::new(),
                    line_styles: Vec::new(),
                },
                shape: Vec::new(),
            },
            movie: context.swf.clone(),
        };
        let drawing = Drawing::new();

        Graphic(GcCell::allocate(
            context.gc_context,
            GraphicData {
                base: Default::default(),
                static_data: gc_arena::Gc::allocate(context.gc_context, static_data),
                avm2_object: Some(avm2_object),
                drawing: Some(drawing),
                strokes_handle: None,
                last_scale: (0.0, 0.0),
            },
        ))
    }

    pub fn drawing(&self, gc_context: MutationContext<'gc, '_>) -> RefMut<'_, Drawing> {
        RefMut::map(self.0.write(gc_context), |w| {
            w.drawing.get_or_insert_with(Drawing::new)
        })
    }
}

impl<'gc> TDisplayObject<'gc> for Graphic<'gc> {
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

    fn self_bounds(&self) -> Rectangle<Twips> {
        if let Some(drawing) = &self.0.read().drawing {
            drawing.self_bounds().clone()
        } else {
            self.0.read().static_data.bounds.clone()
        }
    }

    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc>) {
        if context.is_action_script_3() && matches!(self.object2(), Avm2Value::Null) {
            let shape_constr = context.avm2.classes().shape;
            let mut activation = Avm2Activation::from_nothing(context.reborrow());

            match Avm2StageObject::for_display_object_childless(
                &mut activation,
                (*self).into(),
                shape_constr,
            ) {
                Ok(object) => {
                    self.0.write(activation.context.gc_context).avm2_object = Some(object.into())
                }
                Err(e) => {
                    tracing::error!("Got {} when constructing AVM2 side of display object", e)
                }
            }

            self.on_construction_complete(context);
        }
    }

    fn replace_with(&self, context: &mut UpdateContext<'_, 'gc>, id: CharacterId) {
        // Static assets like Graphics can replace themselves via a PlaceObject tag with PlaceObjectAction::Replace.
        // This does not create a new instance, but instead swaps out the underlying static data to point to the new art.
        if let Some(new_graphic) = context
            .library
            .library_for_movie_mut(self.movie())
            .get_graphic(id)
        {
            let mut write = self.0.write(context.gc_context);
            write.static_data = new_graphic.0.read().static_data;
            write.last_scale = (0.0, 0.0); // Force recreation of stroke
        } else {
            tracing::warn!("PlaceObject: expected Graphic at character ID {}", id);
        }
    }

    fn run_frame_avm1(&self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        if let Some(drawing) = &self.0.read().drawing {
            drawing.render(context);
            return;
        }

        if let Some(render_handle) = self.0.read().static_data.fills_handle {
            context
                .commands
                .render_shape(render_handle, context.transform_stack.transform(), false)
        }

        // Update the stroke if we're drawing it at a different scale than last time
        let old_scale = self.0.read().last_scale;
        let cur_matrix = context.transform_stack.transform().matrix;
        let render_stroke_matrix = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: cur_matrix.tx,
            ty: cur_matrix.ty,
        };
        let cur_scale = (
            f32::abs(cur_matrix.a + cur_matrix.c),
            f32::abs(cur_matrix.b + cur_matrix.d),
        );
        if old_scale != cur_scale {
            let mut write = self.0.write(context.gc_context);
            if let Some(strokes) = &write.static_data.strokes {
                let build_stroke_matrix = Matrix {
                    a: cur_matrix.a,
                    b: cur_matrix.b,
                    c: cur_matrix.c,
                    d: cur_matrix.d,
                    tx: Default::default(),
                    ty: Default::default(),
                };
                if let Some(handle) = write.strokes_handle {
                    context.renderer.replace_shape_strokes(
                        strokes,
                        write.static_data.id,
                        build_stroke_matrix,
                        handle,
                    );
                } else {
                    write.strokes_handle = Some(context.renderer.register_shape_strokes(
                        strokes,
                        write.static_data.id,
                        build_stroke_matrix,
                    ));
                }
            }
            write.last_scale = cur_scale;
        }

        if let Some(render_handle) = self.0.read().strokes_handle {
            context.commands.render_shape(
                render_handle,
                Transform {
                    matrix: render_stroke_matrix,
                    color_transform: context.transform_stack.transform().color_transform,
                },
                true,
            );
        }
    }

    fn hit_test_shape(
        &self,
        _context: &mut UpdateContext<'_, 'gc>,
        point: (Twips, Twips),
        _options: HitTestOptions,
    ) -> bool {
        // Transform point to local coordinates and test.
        if self.world_bounds().contains(point) {
            let local_matrix = self.global_to_local_matrix();
            let point = local_matrix * point;
            if let Some(drawing) = &self.0.read().drawing {
                if drawing.hit_test(point, &local_matrix) {
                    return true;
                }
            } else {
                let shape = &self.0.read().static_data.shape;
                return ruffle_render::shape_utils::shape_hit_test(shape, point, &local_matrix);
            }
        }

        false
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if context.is_action_script_3() {
            self.set_default_instance_name(context);
        } else {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());

            if run_frame {
                self.run_frame_avm1(context);
            }
        }
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.read().static_data.movie.clone()
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .avm2_object
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Null)
    }

    fn set_object2(&self, context: &mut UpdateContext<'_, 'gc>, to: Avm2Object<'gc>) {
        self.0.write(context.gc_context).avm2_object = Some(to);
    }

    fn as_drawing(&self, gc_context: MutationContext<'gc, '_>) -> Option<RefMut<'_, Drawing>> {
        Some(self.drawing(gc_context))
    }
}

/// Static data shared between all instances of a Graphic.
#[allow(dead_code)]
#[derive(Collect)]
#[collect(require_static)]
struct GraphicStatic {
    id: CharacterId,
    shape: swf::Shape,
    fills_handle: Option<ShapeHandle>,
    strokes: Option<ShapeStrokes>,
    bounds: Rectangle<Twips>,
    movie: Arc<SwfMovie>,
}
