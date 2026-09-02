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
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::tessellation_cache::TessellationCache;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Finalization, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::backend::ShapeHandle;
use ruffle_render::backend::null::NullBitmapSource;
use ruffle_render::commands::CommandHandler;
use std::cell::{Cell, OnceCell, RefCell, RefMut};
use std::sync::Arc;
use web_time::Instant;

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
    /// Whether the definition data this object shares with every other
    /// instance of the same character was reached from the root by the
    /// marking phase that this finalization concludes.
    /// See [`crate::display_object::MovieClip::shared_data_is_reachable`].
    pub fn shared_data_is_reachable(self, fc: &Finalization<'gc>) -> bool {
        !Gc::is_dead(fc, self.0.shared.get())
    }

    /// Construct a `Graphic` from it's associated `Shape` tag.
    ///
    /// `source` is the `DefineShape` tag body the shape was read from, with
    /// the tag version. When given, the parsed shape can be dropped while
    /// the graphic is not in use and read again from the movie's data when
    /// it is; see [`Self::evict_stale_tessellation`].
    pub fn from_swf_tag(
        context: &mut UpdateContext<'gc>,
        swf_shape: swf::Shape,
        source: Option<(SwfSlice, u8)>,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let shared = GraphicShared {
            id: swf_shape.id,
            shape_bounds: swf_shape.shape_bounds,
            edge_bounds: swf_shape.edge_bounds,
            shape: RefCell::new(Some(Box::new(swf_shape))),
            shape_source: source,
            movie,
            scaled_handle: RefCell::new(TessellationCache::new()),
            last_drawn: Cell::new(None),
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
            shape_bounds: Default::default(),
            edge_bounds: Default::default(),
            shape: RefCell::new(Some(Box::new(swf::Shape {
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
            }))),
            shape_source: None,
            movie: context.root_swf.clone(),
            scaled_handle: RefCell::new(TessellationCache::new()),
            last_drawn: Cell::new(None),
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

    pub fn instantiate(self, mc: &Mutation<'gc>) -> Self {
        Self(Gc::new(mc, (*self.0).clone()))
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

    /// Returns the best shape handle for the current scale, tessellating if necessary.
    ///
    /// Shapes are tessellated on first draw rather than when their movie is
    /// loaded. A loaded SWF's shapes stay resident for as long as anything can
    /// still reach the SWF (see `MovieLibrary`), and a game that keeps assets
    /// resolvable in an `ApplicationDomain` keeps hundreds of them; their
    /// meshes are by far the largest thing per shape, and most of them are
    /// never on screen at once. Meshes of shapes that have not been drawn for
    /// a while are dropped again by [`Self::evict_stale_tessellation`].
    fn get_or_retessellate_handle(
        self,
        context: &mut RenderContext,
        current_scale: f32,
    ) -> ShapeHandle {
        // Since graphics are created from a shared shape, we may be able to reuse a
        // cached tessellation from another instance at a similar scale.
        let shared = self.0.shared.get();
        shared.touch();

        {
            let mut cache = shared.scaled_handle.borrow_mut();
            if let Some(handle) = cache.find_near_and_touch(current_scale) {
                // Found a cached handle at a similar scale; reuse it.
                return handle;
            }
        }

        // Tessellate at the current scale. Bitmap fills come from the movie's
        // library; a shape whose library is gone can only draw its solid fills.
        let library = context.library.library_for_movie(shared.movie.clone());
        let new_handle = shared.with_shape(|shape| match library {
            Some(library) => context.renderer.register_shape_with_scale(
                shape.into(),
                &MovieLibrarySource { library },
                current_scale,
            ),
            None => context.renderer.register_shape_with_scale(
                shape.into(),
                &NullBitmapSource,
                current_scale,
            ),
        });

        {
            let mut cache = shared.scaled_handle.borrow_mut();
            tracing::debug!(
                "Graphic id={} tessellated: scale={:.2}, cache_size={}",
                shared.id,
                current_scale,
                cache.len()
            );
            cache.insert(current_scale, new_handle.clone());
        }

        new_handle
    }

    /// Drops this shape's tessellations, and its parsed shape records if
    /// they can be read back from the movie, if it has not been drawn or
    /// hit-tested since `before`. Both are rebuilt on the next use.
    pub fn evict_stale_tessellation(self, before: Instant) -> bool {
        let shared = self.0.shared.get();
        let stale = match shared.last_drawn.get() {
            Some(last) => last < before,
            None => true,
        };
        if !stale {
            return false;
        }
        let mut evicted = false;
        let mut cache = shared.scaled_handle.borrow_mut();
        if cache.len() > 0 {
            cache.clear();
            evicted = true;
        }
        if shared.shape_source.is_some() && shared.shape.borrow_mut().take().is_some() {
            evicted = true;
        }
        evicted
    }
}

impl GraphicShared {
    fn touch(&self) {
        self.last_drawn.set(Some(Instant::now()));
    }

    /// Whether this graphic has any shape to draw or hit-test at all.
    fn has_shape(&self) -> bool {
        self.shape_source.is_some()
            || self
                .shape
                .borrow()
                .as_ref()
                .is_some_and(|shape| !shape.shape.is_empty())
    }

    /// Runs `f` on the parsed shape, reading it back from the movie's data
    /// first if it has been dropped.
    fn with_shape<R>(&self, f: impl FnOnce(&swf::Shape) -> R) -> R {
        let mut slot = self.shape.borrow_mut();
        if slot.is_none() {
            let parsed = self.shape_source.as_ref().and_then(|(source, version)| {
                match source.read_from(0).read_define_shape(*version) {
                    Ok(shape) => Some(shape),
                    Err(e) => {
                        tracing::error!("Could not re-read shape {}: {e}", self.id);
                        None
                    }
                }
            });
            *slot = Some(Box::new(parsed.unwrap_or_else(|| swf::Shape {
                version: 32,
                id: self.id,
                shape_bounds: self.shape_bounds,
                edge_bounds: self.edge_bounds,
                flags: swf::ShapeFlag::empty(),
                styles: swf::ShapeStyles {
                    fill_styles: Vec::new(),
                    line_styles: Vec::new(),
                },
                shape: Vec::new(),
            })));
        }
        f(slot.as_ref().expect("shape was just parsed"))
    }
}

impl<'gc> TDisplayObject<'gc> for Graphic<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn id(self) -> CharacterId {
        self.0.shared.get().id
    }

    fn self_bounds(self, mode: BoundsMode) -> Rectangle<Twips> {
        let include_strokes = mode.includes_strokes();

        if let Some(drawing) = self.0.drawing.get() {
            drawing.borrow().self_bounds(include_strokes)
        } else if include_strokes {
            self.0.shared.get().shape_bounds
        } else {
            self.0.shared.get().edge_bounds
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
        } else if self.0.shared.get().has_shape() {
            let transform = context.transform_stack.transform();

            // Calculate the current scale from the transform, to determine if
            // we can reuse a cached tessellation or need to retessellate.
            let matrix = &transform.matrix;
            let scale_x = f32::abs(matrix.a + matrix.c);
            let scale_y = f32::abs(matrix.b + matrix.d);
            let current_scale = ((scale_x * scale_x + scale_y * scale_y) / 2.0).sqrt();

            let handle = self.get_or_retessellate_handle(context, current_scale);

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
                let shared = self.0.shared.get();
                shared.touch();
                return shared.with_shape(|shape| {
                    ruffle_render::shape_utils::shape_hit_test(shape, point, &local_matrix)
                });
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
    /// The parsed shape. `None` while it has been dropped to save memory;
    /// see `with_shape`. Never `None` for a graphic without `shape_source`.
    #[collect(require_static)]
    shape: RefCell<Option<Box<swf::Shape>>>,
    /// The `DefineShape` tag this shape was read from, and its version, so
    /// that the parsed shape can be dropped and read again on demand.
    #[collect(require_static)]
    shape_source: Option<(SwfSlice, u8)>,
    shape_bounds: Rectangle<Twips>,
    edge_bounds: Rectangle<Twips>,
    movie: Arc<SwfMovie>,
    /// Tessellations of `shape` at the scales it has been drawn at. Built on
    /// first draw and dropped again when the shape goes undrawn for a while;
    /// see `Graphic::get_or_retessellate_handle`.
    #[collect(require_static)]
    scaled_handle: RefCell<TessellationCache>,
    /// When any instance of this shape was last drawn or hit-tested.
    #[collect(require_static)]
    last_drawn: Cell<Option<Instant>>,
}
