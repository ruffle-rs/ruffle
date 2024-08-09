use crate::avm2::{
    Activation as Avm2Activation, Object as Avm2Object, StageObject as Avm2StageObject,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr};
use crate::library::{Library, MovieLibrarySource};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use core::fmt;
use gc_arena::{Collect, Gc, GcCell, Mutation};
use ruffle_render::backend::ShapeHandle;
use ruffle_render::commands::CommandHandler;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use swf::{Fixed16, Fixed8};

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct MorphShape<'gc>(GcCell<'gc, MorphShapeData<'gc>>);

impl fmt::Debug for MorphShape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MorphShape")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct MorphShapeData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: Gc<'gc, MorphShapeStatic>,
    ratio: u16,
    /// The AVM2 representation of this MorphShape.
    object: Option<Avm2Object<'gc>>,
}

impl<'gc> MorphShape<'gc> {
    pub fn from_swf_tag(
        gc_context: &Mutation<'gc>,
        tag: swf::DefineMorphShape,
        movie: Arc<SwfMovie>,
    ) -> Self {
        let static_data = MorphShapeStatic::from_swf_tag(&tag, movie);
        MorphShape(GcCell::new(
            gc_context,
            MorphShapeData {
                base: Default::default(),
                static_data: Gc::new(gc_context, static_data),
                ratio: 0,
                object: None,
            },
        ))
    }

    pub fn ratio(self) -> u16 {
        self.0.read().ratio
    }

    pub fn set_ratio(&mut self, gc_context: &Mutation<'gc>, ratio: u16) {
        self.0.write(gc_context).ratio = ratio;
        self.invalidate_cached_bitmap(gc_context);
    }
}

impl<'gc> TDisplayObject<'gc> for MorphShape<'gc> {
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

    fn as_morph_shape(&self) -> Option<Self> {
        Some(*self)
    }

    fn replace_with(&self, context: &mut UpdateContext<'gc>, id: CharacterId) {
        if let Some(new_morph_shape) = context
            .library
            .library_for_movie_mut(self.movie())
            .get_morph_shape(id)
        {
            self.0.write(context.gc_context).static_data = new_morph_shape.0.read().static_data;
        } else {
            tracing::warn!("PlaceObject: expected morph shape at character ID {}", id);
        }
        self.invalidate_cached_bitmap(context.gc_context);
    }

    fn run_frame_avm1(&self, _context: &mut UpdateContext) {
        // Noop
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .object
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Null)
    }

    fn set_object2(&self, context: &mut UpdateContext<'gc>, to: Avm2Object<'gc>) {
        self.0.write(context.gc_context).object = Some(to);
    }

    /// Construct objects placed on this frame.
    fn construct_frame(&self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() && matches!(self.object2(), Avm2Value::Null) {
            let class = context.avm2.classes().morphshape;
            let mut activation = Avm2Activation::from_nothing(context);
            match Avm2StageObject::for_display_object_childless(
                &mut activation,
                (*self).into(),
                class,
            ) {
                Ok(object) => self.0.write(context.gc_context).object = Some(object.into()),
                Err(e) => tracing::error!("Got {} when constructing AVM2 side of MorphShape", e),
            };
            self.on_construction_complete(context);
        }
    }

    fn render_self(&self, context: &mut RenderContext) {
        let this = self.0.read();
        let ratio = this.ratio;
        let static_data = this.static_data;
        let shape_handle = static_data.get_shape(context, context.library, ratio);
        context
            .commands
            .render_shape(shape_handle, context.transform_stack.transform());
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        let this = self.0.read();
        let ratio = this.ratio;
        let static_data = this.static_data;
        let frame = static_data.get_frame(ratio);
        frame.bounds.clone()
    }

    fn hit_test_shape(
        &self,
        _context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        if (!options.contains(HitTestOptions::SKIP_INVISIBLE) || self.visible())
            && self.world_bounds().contains(point)
        {
            if let Some(frame) = self.0.read().static_data.frames.borrow().get(&self.ratio()) {
                let Some(local_matrix) = self.global_to_local_matrix() else {
                    return false;
                };
                return ruffle_render::shape_utils::shape_hit_test(
                    &frame.shape,
                    local_matrix * point,
                    &local_matrix,
                );
            } else {
                tracing::warn!("Missing ratio for morph shape");
            }
        }

        false
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.read().static_data.movie.clone()
    }
}

/// A precalculated intermediate frame for a morph shape.
struct Frame {
    shape_handle: Option<ShapeHandle>,
    shape: swf::Shape,
    bounds: Rectangle<Twips>,
}

/// Static data shared between all instances of a morph shape.
#[allow(dead_code)]
#[derive(Collect)]
#[collect(require_static)]
pub struct MorphShapeStatic {
    id: CharacterId,
    start: swf::MorphShape,
    end: swf::MorphShape,
    frames: RefCell<fnv::FnvHashMap<u16, Frame>>,
    movie: Arc<SwfMovie>,
}

impl MorphShapeStatic {
    pub fn from_swf_tag(swf_tag: &swf::DefineMorphShape, movie: Arc<SwfMovie>) -> Self {
        Self {
            id: swf_tag.id,
            start: swf_tag.start.clone(),
            end: swf_tag.end.clone(),
            frames: RefCell::new(fnv::FnvHashMap::default()),
            movie,
        }
    }

    /// Retrieves the `Frame` for the given ratio.
    /// Lazily initializes the frame if it does not yet exist.
    fn get_frame(&self, ratio: u16) -> RefMut<'_, Frame> {
        let frames = self.frames.borrow_mut();
        RefMut::map(frames, |frames| {
            frames
                .entry(ratio)
                .or_insert_with(|| self.build_morph_frame(ratio))
        })
    }

    /// Retrieves the `ShapeHandle` for the given ratio.
    /// Lazily initializes and tessellates the shape if it does not yet exist.
    fn get_shape<'gc>(
        &self,
        context: &mut RenderContext<'_, 'gc>,
        library: &Library<'gc>,
        ratio: u16,
    ) -> ShapeHandle {
        let mut frame = self.get_frame(ratio);
        if let Some(handle) = frame.shape_handle.clone() {
            handle
        } else {
            let library = library.library_for_movie(self.movie.clone()).unwrap();
            let handle = context
                .renderer
                .register_shape((&frame.shape).into(), &MovieLibrarySource { library });
            frame.shape_handle = Some(handle.clone());
            handle
        }
    }

    fn build_morph_frame(&self, ratio: u16) -> Frame {
        // Interpolate MorphShapes into a Shape.
        use swf::{FillStyle, LineStyle, ShapeRecord, ShapeStyles};
        // Start shape is ratio 65535, end shape is ratio 0.
        let b = f32::from(ratio) / 65535.0;
        let a = 1.0 - b;
        let fill_styles: Vec<FillStyle> = self
            .start
            .fill_styles
            .iter()
            .zip(self.end.fill_styles.iter())
            .map(|(start, end)| lerp_fill(start, end, a, b))
            .collect();
        let line_styles: Vec<LineStyle> = self
            .start
            .line_styles
            .iter()
            .zip(self.end.line_styles.iter())
            .map(|(start, end)| {
                start
                    .clone()
                    .with_width(lerp_twips(start.width(), end.width(), a, b))
                    .with_fill_style(lerp_fill(start.fill_style(), end.fill_style(), a, b))
            })
            .collect();

        let mut shape = Vec::with_capacity(self.start.shape.len());
        let mut start_iter = self.start.shape.iter();
        let mut end_iter = self.end.shape.iter();
        let mut start = start_iter.next();
        let mut end = end_iter.next();
        let mut start_x = Twips::ZERO;
        let mut start_y = Twips::ZERO;
        let mut end_x = Twips::ZERO;
        let mut end_y = Twips::ZERO;
        // TODO: Feels like this could be cleaned up a bit.
        // We step through both the start records and end records, interpolating edges pairwise.
        // Fill style/line style changes should only appear in the start records.
        // However, StyleChangeRecord move_to can appear it both start and end records,
        // and not necessarily in matching pairs; therefore, we have to keep track of the pen position
        // in case one side is missing a move_to; it will implicitly use the last pen position.
        while let (Some(s), Some(e)) = (start, end) {
            match (s, e) {
                (ShapeRecord::StyleChange(start_change), ShapeRecord::StyleChange(end_change)) => {
                    let mut style_change = start_change.clone();
                    if start_change.move_to.is_some() || end_change.move_to.is_some() {
                        if let Some(move_to) = &start_change.move_to {
                            start_x = move_to.x;
                            start_y = move_to.y;
                        }
                        if let Some(move_to) = &end_change.move_to {
                            end_x = move_to.x;
                            end_y = move_to.y;
                        }
                        style_change.move_to = Some(Point::new(
                            lerp_twips(start_x, end_x, a, b),
                            lerp_twips(start_y, end_y, a, b),
                        ));
                    }
                    shape.push(ShapeRecord::StyleChange(style_change));
                    start = start_iter.next();
                    end = end_iter.next();
                }
                (ShapeRecord::StyleChange(start_change), _) => {
                    let mut style_change = start_change.clone();
                    if let Some(move_to) = &start_change.move_to {
                        start_x = move_to.x;
                        start_y = move_to.y;
                        style_change.move_to = Some(Point::new(
                            lerp_twips(start_x, end_x, a, b),
                            lerp_twips(start_y, end_y, a, b),
                        ));
                    }
                    shape.push(ShapeRecord::StyleChange(style_change));
                    Self::update_pos(&mut start_x, &mut start_y, s);
                    start = start_iter.next();
                }
                (_, ShapeRecord::StyleChange(end_change)) => {
                    let mut style_change = end_change.clone();
                    if let Some(move_to) = &end_change.move_to {
                        end_x = move_to.x;
                        end_y = move_to.y;
                        style_change.move_to = Some(Point::new(
                            lerp_twips(start_x, end_x, a, b),
                            lerp_twips(start_y, end_y, a, b),
                        ));
                    }
                    shape.push(ShapeRecord::StyleChange(style_change));
                    Self::update_pos(&mut end_x, &mut end_y, s);
                    end = end_iter.next();
                    continue;
                }
                _ => {
                    shape.push(lerp_edges(
                        Point::new(start_x, start_y),
                        Point::new(end_x, end_y),
                        s,
                        e,
                        a,
                        b,
                    ));
                    Self::update_pos(&mut start_x, &mut start_y, s);
                    Self::update_pos(&mut end_x, &mut end_y, e);
                    start = start_iter.next();
                    end = end_iter.next();
                }
            }
        }

        let styles = ShapeStyles {
            fill_styles,
            line_styles,
        };

        let bounds = ruffle_render::shape_utils::calculate_shape_bounds(&shape);
        let shape = swf::Shape {
            version: 4,
            id: 0,
            shape_bounds: bounds.clone(),
            edge_bounds: bounds.clone(),
            flags: swf::ShapeFlag::HAS_SCALING_STROKES,
            styles,
            shape,
        };

        Frame {
            shape_handle: None,
            shape,
            bounds,
        }
    }

    fn update_pos(x: &mut Twips, y: &mut Twips, record: &swf::ShapeRecord) {
        use swf::ShapeRecord;
        match record {
            ShapeRecord::StraightEdge { delta } => {
                *x += delta.dx;
                *y += delta.dy;
            }
            ShapeRecord::CurvedEdge {
                control_delta,
                anchor_delta,
            } => {
                *x += control_delta.dx + anchor_delta.dx;
                *y += control_delta.dy + anchor_delta.dy;
            }
            ShapeRecord::StyleChange(ref style_change) => {
                if let Some(move_to) = &style_change.move_to {
                    *x = move_to.x;
                    *y = move_to.y;
                }
            }
        }
    }
}

// Interpolation functions
// These interpolate between two SWF shape structures.
// a + b should = 1.0

fn lerp_color(start: &Color, end: &Color, a: f32, b: f32) -> Color {
    // f32 -> u8 cast is defined to saturate for out of bounds values,
    // so we don't have to worry about clamping.
    Color {
        r: (a * f32::from(start.r) + b * f32::from(end.r)) as u8,
        g: (a * f32::from(start.g) + b * f32::from(end.g)) as u8,
        b: (a * f32::from(start.b) + b * f32::from(end.b)) as u8,
        a: (a * f32::from(start.a) + b * f32::from(end.a)) as u8,
    }
}

fn lerp_twips(start: Twips, end: Twips, a: f32, b: f32) -> Twips {
    Twips::new((start.get() as f32 * a + end.get() as f32 * b).round() as i32)
}

fn lerp_point_twips(start: Point<Twips>, end: Point<Twips>, a: f32, b: f32) -> Point<Twips> {
    Point::new(
        lerp_twips(start.x, end.x, a, b),
        lerp_twips(start.y, end.y, a, b),
    )
}

fn lerp_fill(start: &swf::FillStyle, end: &swf::FillStyle, a: f32, b: f32) -> swf::FillStyle {
    use swf::FillStyle;
    match (start, end) {
        // Color-to-color
        (FillStyle::Color(start), FillStyle::Color(end)) => {
            FillStyle::Color(lerp_color(start, end, a, b))
        }

        // Bitmap-to-bitmap
        // ID should be the same.
        (
            FillStyle::Bitmap {
                id: start_id,
                matrix: start,
                is_smoothed,
                is_repeating,
            },
            FillStyle::Bitmap { matrix: end, .. },
        ) => FillStyle::Bitmap {
            id: *start_id,
            matrix: lerp_matrix(start, end, a, b),
            is_smoothed: *is_smoothed,
            is_repeating: *is_repeating,
        },

        // Linear-to-linear
        (FillStyle::LinearGradient(start), FillStyle::LinearGradient(end)) => {
            FillStyle::LinearGradient(lerp_gradient(start, end, a, b))
        }

        // Radial-to-radial
        (FillStyle::RadialGradient(start), FillStyle::RadialGradient(end)) => {
            FillStyle::RadialGradient(lerp_gradient(start, end, a, b))
        }

        // Focal gradients also interpolate focal point.
        (
            FillStyle::FocalGradient {
                gradient: start,
                focal_point: start_focal,
            },
            FillStyle::FocalGradient {
                gradient: end,
                focal_point: end_focal,
            },
        ) => FillStyle::FocalGradient {
            gradient: lerp_gradient(start, end, a, b),
            focal_point: *start_focal * Fixed8::from_f32(a) + *end_focal * Fixed8::from_f32(b),
        },

        // All other combinations should not occur, because SWF stores the start/end fill as the same type, always.
        // If you happened to make, say, a solid color-to-radial gradient tween in the IDE, this would get baked down into
        // a radial-to-radial gradient on export.
        _ => {
            tracing::warn!(
                "Unexpected morph shape fill style combination: {:#?}, {:#?}",
                start,
                end
            );
            start.clone()
        }
    }
}

fn lerp_edges(
    start_pen: Point<Twips>,
    end_pen: Point<Twips>,
    start: &swf::ShapeRecord,
    end: &swf::ShapeRecord,
    a: f32,
    b: f32,
) -> swf::ShapeRecord {
    use swf::ShapeRecord;
    let pen = lerp_point_twips(start_pen, end_pen, a, b);
    match (start, end) {
        (
            ShapeRecord::StraightEdge { delta: start_delta },
            ShapeRecord::StraightEdge { delta: end_delta },
        ) => {
            let start_anchor = start_pen + *start_delta;
            let end_anchor = end_pen + *end_delta;

            let anchor = lerp_point_twips(start_anchor, end_anchor, a, b);

            ShapeRecord::StraightEdge {
                delta: anchor - pen,
            }
        }

        (
            ShapeRecord::CurvedEdge {
                control_delta: start_control_delta,
                anchor_delta: start_anchor_delta,
            },
            ShapeRecord::CurvedEdge {
                control_delta: end_control_delta,
                anchor_delta: end_anchor_delta,
            },
        ) => {
            let start_control = start_pen + *start_control_delta;
            let start_anchor = start_control + *start_anchor_delta;

            let end_control = end_pen + *end_control_delta;
            let end_anchor = end_control + *end_anchor_delta;

            let control = lerp_point_twips(start_control, end_control, a, b);
            let anchor = lerp_point_twips(start_anchor, end_anchor, a, b);

            ShapeRecord::CurvedEdge {
                control_delta: control - pen,
                anchor_delta: anchor - control,
            }
        }

        (
            ShapeRecord::StraightEdge { delta: start_delta },
            ShapeRecord::CurvedEdge {
                control_delta: end_control_delta,
                anchor_delta: end_anchor_delta,
            },
        ) => {
            let start_control = start_pen + *start_delta / 2;
            let start_anchor = start_pen + *start_delta;

            let end_control = end_pen + *end_control_delta;
            let end_anchor = end_control + *end_anchor_delta;

            let control = lerp_point_twips(start_control, end_control, a, b);
            let anchor = lerp_point_twips(start_anchor, end_anchor, a, b);

            ShapeRecord::CurvedEdge {
                control_delta: control - pen,
                anchor_delta: anchor - control,
            }
        }

        (
            ShapeRecord::CurvedEdge {
                control_delta: start_control_delta,
                anchor_delta: start_anchor_delta,
            },
            ShapeRecord::StraightEdge { delta: end_delta },
        ) => {
            let start_control = start_pen + *start_control_delta;
            let start_anchor = start_control + *start_anchor_delta;

            let end_control = end_pen + *end_delta / 2;
            let end_anchor = end_pen + *end_delta;

            let control = lerp_point_twips(start_control, end_control, a, b);
            let anchor = lerp_point_twips(start_anchor, end_anchor, a, b);

            ShapeRecord::CurvedEdge {
                control_delta: control - pen,
                anchor_delta: anchor - control,
            }
        }
        _ => unreachable!("{:?} {:?}", start, end),
    }
}

fn lerp_matrix(start: &swf::Matrix, end: &swf::Matrix, a: f32, b: f32) -> swf::Matrix {
    // TODO: Lerping a matrix element-wise is geometrically wrong,
    // but I doubt Flash is decomposing the matrix into scale-rotate-translate?
    let af = Fixed16::from_f32(a);
    let bf = Fixed16::from_f32(b);
    swf::Matrix {
        a: start.a * af + end.a * bf,
        b: start.b * af + end.b * bf,
        c: start.c * af + end.c * bf,
        d: start.d * af + end.d * bf,
        tx: lerp_twips(start.tx, end.tx, a, b),
        ty: lerp_twips(start.ty, end.ty, a, b),
    }
}

fn lerp_gradient(start: &swf::Gradient, end: &swf::Gradient, a: f32, b: f32) -> swf::Gradient {
    use swf::{Gradient, GradientRecord};
    // Morph gradients are guaranteed to have the same number of records in the start/end gradient.
    debug_assert_eq!(start.records.len(), end.records.len());
    let records: Vec<GradientRecord> = start
        .records
        .iter()
        .zip(end.records.iter())
        .map(|(start, end)| swf::GradientRecord {
            ratio: (f32::from(start.ratio) * a + f32::from(end.ratio) * b) as u8,
            color: lerp_color(&start.color, &end.color, a, b),
        })
        .collect();

    Gradient {
        matrix: lerp_matrix(&start.matrix, &end.matrix, a, b),
        spread: start.spread,
        interpolation: start.interpolation,
        records,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Regression test for #14074
    #[test]
    fn test_lerp_rounding() {
        let ratio: u16 = 17246;
        let b = f32::from(ratio) / 65535.0;
        let a = 1.0 - b;

        assert_eq!(
            lerp_twips(Twips::new(-7), Twips::new(-7), a, b),
            Twips::new(-7)
        );
    }
}
