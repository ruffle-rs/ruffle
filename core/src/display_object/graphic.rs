use crate::avm1::Object as Avm1Object;
use crate::avm2::{
    Activation as Avm2Activation, Error as Avm2Error, Namespace as Avm2Namespace,
    Object as Avm2Object, QName as Avm2QName, StageObject as Avm2StageObject,
    TObject as Avm2TObject,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::drawing::Drawing;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::types::{Degrees, Percent};
use crate::vminterface::{AvmType, Instantiator};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::RefMut;
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
    drawing: Drawing,
}

impl<'gc> Graphic<'gc> {
    /// Construct a `Graphic` from it's associated `Shape` tag.
    pub fn from_swf_tag(
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_shape: swf::Shape,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let drawing = Drawing::from_swf_shape(&swf_shape);
        let static_data = GraphicStatic {
            id: swf_shape.id,
            shape: swf_shape,
            movie: Some(movie),
        };

        Graphic(GcCell::allocate(
            context.gc_context,
            GraphicData {
                base: Default::default(),
                static_data: gc_arena::Gc::allocate(context.gc_context, static_data),
                avm2_object: None,
                drawing,
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
            shape: swf::Shape {
                version: 32,
                id: 0,
                shape_bounds: Default::default(),
                edge_bounds: Default::default(),
                has_fill_winding_rule: false,
                has_non_scaling_strokes: false,
                has_scaling_strokes: false,
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
                drawing,
            },
        ))
    }
}

impl<'gc> TDisplayObject<'gc> for Graphic<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
    }

    fn self_bounds(&self) -> BoundingBox {
        self.0.read().drawing.self_bounds()
    }

    fn run_frame(&self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render_self(&self, context: &mut RenderContext) {
        if !self.world_bounds().intersects(&context.view_bounds) {
            // Off-screen; culled
            return;
        }

        self.0
            .read()
            .drawing
            .render(context, self.0.read().static_data.movie.clone());
    }

    fn hit_test_shape(
        &self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        point: (Twips, Twips),
    ) -> bool {
        // Transform point to local coordinates and test.
        if self.world_bounds().contains(point) {
            let local_matrix = self.global_to_local_matrix();
            let point = local_matrix * point;
            if self.0.read().drawing.hit_test(point, &local_matrix) {
                return true;
            }
        }

        false
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        let shape = self.0.read().static_data.shape.clone();
        self.0.write(context.gc_context).drawing = Drawing::from_swf_shape(&shape);

        if self.vm_type(context) == AvmType::Avm2 {
            let mut allocator = || {
                let mut activation = Avm2Activation::from_nothing(context.reborrow());
                let mut proto = activation.context.avm2.prototypes().shape;
                let constr = proto
                    .get_property(
                        proto,
                        &Avm2QName::new(Avm2Namespace::public(), "constructor"),
                        &mut activation,
                    )?
                    .coerce_to_object(&mut activation)?;

                let object = Avm2StageObject::for_display_object(
                    activation.context.gc_context,
                    display_object,
                    proto,
                )
                .into();
                constr.call(Some(object), &[], &mut activation, Some(proto))?;

                Ok(object)
            };
            let result: Result<Avm2Object<'gc>, Avm2Error> = allocator();

            match result {
                Ok(object) => self.0.write(context.gc_context).avm2_object = Some(object),
                Err(e) => log::error!("Got {} when constructing AVM2 side of display object", e),
            }
        }

        if run_frame {
            self.run_frame(context);
        }
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .avm2_object
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Undefined)
    }

    fn as_drawing(&self, gc_context: MutationContext<'gc, '_>) -> Option<RefMut<'_, Drawing>> {
        Some(RefMut::map(self.0.write(gc_context), |m| &mut m.drawing))
    }
}

/// Static data shared between all instances of a graphic.
#[allow(dead_code)]
#[derive(Collect)]
#[collect(require_static)]
struct GraphicStatic {
    id: CharacterId,
    shape: swf::Shape,
    movie: Option<Arc<SwfMovie>>,
}
