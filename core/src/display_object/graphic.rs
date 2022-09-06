use crate::avm1::Object as Avm1Object;
use crate::avm2::{
    Activation as Avm2Activation, Object as Avm2Object, StageObject as Avm2StageObject,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::drawing::Drawing;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::backend::ShapeHandle;
use std::cell::{Ref, RefMut};
use std::sync::Arc;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Graphic<'gc>(GcCell<'gc, GraphicData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct GraphicData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, GraphicStatic>,
    avm2_object: Option<Avm2Object<'gc>>,
    drawing: Option<Drawing>,
}

impl<'gc> Graphic<'gc> {
    /// Construct a `Graphic` from it's associated `Shape` tag.
    pub fn from_swf_tag(
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_shape: swf::Shape,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let library = context.library.library_for_movie(movie.clone()).unwrap();
        let static_data = GraphicStatic {
            id: swf_shape.id,
            bounds: (&swf_shape.shape_bounds).into(),
            render_handle: Some(
                context
                    .renderer
                    .register_shape((&swf_shape).into(), library),
            ),
            shape: swf_shape,
            movie: Some(movie),
        };

        Graphic(GcCell::allocate(
            context.gc_context,
            GraphicData {
                base: Default::default(),
                static_data: gc_arena::Gc::allocate(context.gc_context, static_data),
                avm2_object: None,
                drawing: None,
            },
        ))
    }

    /// Construct an empty `Graphic`.
    pub fn new_with_avm2(
        context: &mut UpdateContext<'_, 'gc, '_>,
        avm2_object: Avm2Object<'gc>,
    ) -> Self {
        let static_data = GraphicStatic {
            id: 0,
            bounds: Default::default(),
            render_handle: None,
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
            movie: None,
        };
        let drawing = Drawing::new();

        Graphic(GcCell::allocate(
            context.gc_context,
            GraphicData {
                base: Default::default(),
                static_data: gc_arena::Gc::allocate(context.gc_context, static_data),
                avm2_object: Some(avm2_object),
                drawing: Some(drawing),
            },
        ))
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

    fn self_bounds(&self) -> BoundingBox {
        if let Some(drawing) = &self.0.read().drawing {
            drawing.self_bounds()
        } else {
            self.0.read().static_data.bounds.clone()
        }
    }

    fn construct_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if context.is_action_script_3() && matches!(self.object2(), Avm2Value::Undefined) {
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
                Err(e) => log::error!("Got {} when constructing AVM2 side of display object", e),
            }
        }
    }

    fn replace_with(&self, context: &mut UpdateContext<'_, 'gc, '_>, id: CharacterId) {
        // Static assets like Graphics can replace themselves via a PlaceObject tag with PlaceObjectAction::Replace.
        // This does not create a new instance, but instead swaps out the underlying static data to point to the new art.
        if let Some(new_graphic) = context
            .library
            .library_for_movie_mut(self.movie().unwrap())
            .get_graphic(id)
        {
            self.0.write(context.gc_context).static_data = new_graphic.0.read().static_data;
        } else {
            log::warn!("PlaceObject: expected Graphic at character ID {}", id);
        }
    }

    fn run_frame(&self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render_self(&self, context: &mut RenderContext) {
        if !context.is_offscreen && !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        if let Some(drawing) = &self.0.read().drawing {
            drawing.render(context);
        } else if let Some(render_handle) = self.0.read().static_data.render_handle {
            context
                .renderer
                .render_shape(render_handle, context.transform_stack.transform())
        }
    }

    fn hit_test_shape(
        &self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
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
        context: &mut UpdateContext<'_, 'gc, '_>,
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
        }

        if run_frame {
            self.run_frame(context);
        }
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        self.0.read().static_data.movie.clone()
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .avm2_object
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Undefined)
    }

    fn set_object2(&mut self, mc: MutationContext<'gc, '_>, to: Avm2Object<'gc>) {
        self.0.write(mc).avm2_object = Some(to);
    }

    fn as_drawing(&self, gc_context: MutationContext<'gc, '_>) -> Option<RefMut<'_, Drawing>> {
        let mut write = self.0.write(gc_context);
        if write.drawing.is_none() {
            write.drawing = Some(Drawing::new());
        }

        Some(RefMut::map(write, |m| m.drawing.as_mut().unwrap()))
    }
}

/// Static data shared between all instances of a Graphic.
#[allow(dead_code)]
#[derive(Collect)]
#[collect(require_static)]
struct GraphicStatic {
    id: CharacterId,
    shape: swf::Shape,
    render_handle: Option<ShapeHandle>,
    bounds: BoundingBox,
    movie: Option<Arc<SwfMovie>>,
}
