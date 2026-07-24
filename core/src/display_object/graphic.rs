use crate::avm1::Object as Avm1Object;
use crate::avm2::{
    Activation as Avm2Activation, Avm2, ClassObject as Avm2ClassObject,
    StageObject as Avm2StageObject,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{BoundsMode, DisplayObjectBase};
use crate::drawing::Drawing;
use crate::library::MovieLibrarySource;
use crate::prelude::*;
use crate::scale9_cache::{Scale9Cache, Scale9Key};
use crate::tag_utils::SwfMovie;
use crate::tessellation_cache::TessellationCache;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::backend::ShapeHandle;
use ruffle_render::commands::CommandHandler;
use std::cell::{OnceCell, RefCell, RefMut};
use std::sync::Arc;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Graphic<'gc>(Gc<'gc, GraphicData<'gc>>);

impl fmt::Debug for Graphic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Graphic")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct GraphicData<'gc> {
    base: DisplayObjectBase<'gc>,
    shared: Lock<Gc<'gc, GraphicShared>>,
    class: Lock<Option<Avm2ClassObject<'gc>>>,
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,
    /// This is lazily allocated on demand, to make `GraphicData` smaller in the common case.
    #[collect(require_static)]
    drawing: OnceCell<Box<RefCell<Drawing>>>,
}

impl<'gc> Graphic<'gc> {
    /// Construct a `Graphic` from it's associated `Shape` tag.
    pub fn from_swf_tag(
        context: &mut UpdateContext<'gc>,
        swf_shape: swf::Shape,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let library = context.library.library_for_movie(movie.clone()).unwrap();
        let shared = GraphicShared {
            id: swf_shape.id,
            bounds: swf_shape.shape_bounds,
            render_handle: Some(
                context
                    .renderer
                    .register_shape((&swf_shape).into(), &MovieLibrarySource { library }),
            ),
            shape: swf_shape,
            movie,
            scaled_handle: RefCell::new(TessellationCache::new()),
            scale9_handles: RefCell::new(Scale9Cache::new()),
        };

        Graphic(Gc::new(
            context.gc(),
            GraphicData {
                base: Default::default(),
                shared: Lock::new(Gc::new(context.gc(), shared)),
                class: Lock::new(None),
                avm2_object: Lock::new(None),
                drawing: OnceCell::new(),
            },
        ))
    }

    /// Construct an empty `Graphic`.
    pub fn empty(context: &mut UpdateContext<'gc>) -> Self {
        let shared = GraphicShared {
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
            movie: context.root_swf.clone(),
            scaled_handle: RefCell::new(TessellationCache::new()),
            scale9_handles: RefCell::new(Scale9Cache::new()),
        };

        Graphic(Gc::new(
            context.gc(),
            GraphicData {
                base: Default::default(),
                shared: Lock::new(Gc::new(context.gc(), shared)),
                class: Lock::new(None),
                avm2_object: Lock::new(None),
                drawing: OnceCell::new(),
            },
        ))
    }

    pub fn drawing_mut(&self) -> RefMut<'_, Drawing> {
        self.0.drawing.get_or_init(Default::default).borrow_mut()
    }

    pub fn set_avm2_class(self, mc: &Mutation<'gc>, class: Avm2ClassObject<'gc>) {
        unlock!(Gc::write(mc, self.0), GraphicData, class).set(Some(class));
    }

    fn set_shared(self, mc: &Mutation<'gc>, shared: Gc<'gc, GraphicShared>) {
        unlock!(Gc::write(mc, self.0), GraphicData, shared).set(shared);
    }

    /// Drops cached 9-slice transformed handles for both the SWF-source
    /// path and any lazy `Drawing` overlay. Called when the owning
    /// DisplayObject's scaling_grid changes.
    pub fn invalidate_scale9_cache(self) {
        self.0.shared.get().scale9_handles.borrow_mut().clear();
        if let Some(drawing_cell) = self.0.drawing.get() {
            drawing_cell.borrow().invalidate_scale9_cache();
        }
    }

    /// Returns a 9-slice transformed shape handle, building and caching one
    /// on first request. AS3-created shapes (`new Shape(); shape.graphics.*`)
    /// store their geometry in the lazy `Drawing` rather than the SWF
    /// `shape`; route through the Drawing's own scale9 cache in that case.
    pub fn get_or_register_scale9_handle(
        self,
        context: &mut RenderContext<'_, 'gc>,
        grid: Rectangle<Twips>,
        bounds: Rectangle<Twips>,
        world_sx: f32,
        world_sy: f32,
    ) -> Option<ShapeHandle> {
        if let Some(drawing_cell) = self.0.drawing.get() {
            return drawing_cell.borrow().get_or_register_scale9_handle(
                context.renderer,
                grid,
                bounds,
                world_sx,
                world_sy,
            );
        }

        let shared = self.0.shared.get();
        let library = context.library.library_for_movie(shared.movie.clone())?;
        let key = Scale9Key::new(grid, bounds, world_sx, world_sy);
        shared
            .scale9_handles
            .borrow_mut()
            .get_or_try_insert(key, || {
                let source: ruffle_render::shape_utils::DistilledShape = (&shared.shape).into();
                if source.paths.is_empty() {
                    return None;
                }
                // Defer to MASK fallback (caller handles None) only when fills
                // carry source-space UV sampling that PH2 geometry compression
                // would break — non-repeating bitmap fills. Solid-color shapes
                // stay on PH2 because MASK would discontinuously stretch any
                // curve crossing a slice boundary.
                if ruffle_render::shape_utils::shape_needs_source_uv_sampling(&source) {
                    return None;
                }
                let transformed = ruffle_render::shape_utils::transform_for_scale9grid(
                    &source, grid, bounds, world_sx, world_sy,
                );
                Some(
                    context
                        .renderer
                        .register_shape(transformed, &MovieLibrarySource { library }),
                )
            })
    }

    /// Returns the best shape handle for the current scale, retessellating if necessary.
    fn get_or_retessellate_handle(
        self,
        context: &mut RenderContext,
        base_handle: &ShapeHandle,
        current_scale: f32,
    ) -> ShapeHandle {
        // Since graphics are created from a shared shape, we may be able to reuse a
        // cached tessellation from another instance at a similar scale.
        let shared = self.0.shared.get();

        {
            let mut cache = shared.scaled_handle.borrow_mut();
            if let Some(handle) = cache.find_near_and_touch(current_scale) {
                // Found a cached handle at a similar scale; reuse it.
                return handle;
            }
        }

        // Retessellate at the new scale
        let library = context.library.library_for_movie(shared.movie.clone());
        if let Some(library) = library {
            let new_handle = context.renderer.register_shape_with_scale(
                (&shared.shape).into(),
                &MovieLibrarySource { library },
                current_scale,
            );

            {
                let mut cache = shared.scaled_handle.borrow_mut();
                tracing::debug!(
                    "Graphic id={} retessellated: new_scale={:.2}, cache_size={}",
                    shared.id,
                    current_scale,
                    cache.len()
                );
                cache.insert(current_scale, new_handle.clone());
            }

            new_handle
        } else {
            base_handle.clone()
        }
    }
}

impl<'gc> TDisplayObject<'gc> for Graphic<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(gc_context, self.0.as_ref().clone())).into()
    }

    fn id(self) -> CharacterId {
        self.0.shared.get().id
    }

    fn self_bounds(self, _mode: BoundsMode) -> Rectangle<Twips> {
        if let Some(drawing) = self.0.drawing.get() {
            drawing.borrow().self_bounds()
        } else {
            self.0.shared.get().bounds
        }
    }

    fn construct_frame(self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() && self.object2().is_none() {
            let class_object = self
                .0
                .class
                .get()
                .unwrap_or_else(|| context.avm2.classes().shape);

            let mut activation = Avm2Activation::from_nothing(context);

            match Avm2StageObject::for_display_object_childless(
                &mut activation,
                self.into(),
                class_object,
            ) {
                Ok(object) => self.set_object2(activation.context, object),
                Err(err) => {
                    Avm2::uncaught_error(
                        &mut activation,
                        Some(self.into()),
                        err,
                        "Error running AVM2 construction for shape",
                    );
                }
            }

            self.on_construction_complete(context);
        }
    }

    fn replace_with(self, context: &mut UpdateContext<'gc>, id: CharacterId) {
        // Static assets like Graphics can replace themselves via a PlaceObject tag with PlaceObjectAction::Replace.
        // This does not create a new instance, but instead swaps out the underlying static data to point to the new art.
        if let Some(new_graphic) = context
            .library
            .library_for_movie_mut(self.movie())
            .get_graphic(id)
        {
            self.set_shared(context.gc(), new_graphic.0.shared.get());
        } else {
            tracing::warn!("PlaceObject: expected Graphic at character ID {}", id);
        }
        self.invalidate_cached_bitmap();
    }

    fn render_self(self, context: &mut RenderContext) {
        if !context.is_offscreen
            && !self
                .world_bounds(BoundsMode::Engine)
                .intersects(&context.stage.view_bounds())
        {
            // Off-screen; culled
            return;
        }

        if let Some(drawing) = self.0.drawing.get() {
            drawing.borrow().render(context);
        } else if let Some(base_handle) = self.0.shared.get().render_handle.clone() {
            let transform = context.transform_stack.transform();

            // Calculate the current scale from the transform, to determine if
            // we can reuse a cached tessellation or need to retessellate.
            let matrix = &transform.matrix;
            let scale_x = f32::abs(matrix.a + matrix.c);
            let scale_y = f32::abs(matrix.b + matrix.d);
            let current_scale = ((scale_x * scale_x + scale_y * scale_y) / 2.0).sqrt();

            let handle = self.get_or_retessellate_handle(context, &base_handle, current_scale);

            context.commands.render_shape(handle, transform)
        }
    }

    fn hit_test_shape(
        self,
        _context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        // Transform point to local coordinates and test.
        if (!options.contains(HitTestOptions::SKIP_INVISIBLE) || self.visible())
            && self.world_bounds(BoundsMode::Engine).contains(point)
        {
            let Some(local_matrix) = self.global_to_local_matrix() else {
                return false;
            };
            let point = local_matrix * point;
            if let Some(drawing) = self.0.drawing.get() {
                if drawing.borrow().hit_test(point, &local_matrix) {
                    return true;
                }
            } else {
                let shape = &self.0.shared.get().shape;
                return ruffle_render::shape_utils::shape_hit_test(shape, point, &local_matrix);
            }
        }

        false
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        if self.movie().is_action_script_3() {
            self.set_default_instance_name(context);
        }
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.shared.get().movie.clone()
    }

    fn object1(self) -> Option<Avm1Object<'gc>> {
        None
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.avm2_object.get()
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), GraphicData, avm2_object).set(Some(to));
    }

    fn as_drawing(&self) -> Option<RefMut<'_, Drawing>> {
        Some(self.drawing_mut())
    }
}

/// Data shared between all instances of a Graphic.
#[derive(Collect)]
#[collect(require_static)]
struct GraphicShared {
    id: CharacterId,
    shape: swf::Shape,
    render_handle: Option<ShapeHandle>,
    bounds: Rectangle<Twips>,
    movie: Arc<SwfMovie>,
    #[collect(require_static)]
    scaled_handle: RefCell<TessellationCache>,
    /// Cache of 9-slice transformed shape handles, keyed by grid + world scale.
    /// Populated lazily by `get_or_register_scale9_handle`.
    #[collect(require_static)]
    scale9_handles: RefCell<Scale9Cache>,
}
