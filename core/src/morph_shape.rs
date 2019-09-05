use crate::backend::render::{RenderBackend, ShapeHandle};
use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use swf::Twips;

#[derive(Clone, Debug)]
pub struct MorphShape<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, MorphShapeStatic>,
    ratio: u16,
}

impl<'gc> MorphShape<'gc> {
    pub fn new(
        gc_context: gc_arena::MutationContext<'gc, '_>,
        static_data: MorphShapeStatic,
    ) -> Self {
        Self {
            base: Default::default(),
            static_data: gc_arena::Gc::allocate(gc_context, static_data),
            ratio: 0,
        }
    }

    pub fn ratio(&self) -> u16 {
        self.ratio
    }

    pub fn set_ratio(&mut self, ratio: u16) {
        self.ratio = ratio;
    }
}

impl<'gc> DisplayObject<'gc> for MorphShape<'gc> {
    impl_display_object!(base);

    fn as_morph_shape(&self) -> Option<&Self> {
        Some(self)
    }

    fn as_morph_shape_mut(&mut self) -> Option<&mut Self> {
        Some(self)
    }

    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());

        if let Some(shape) = self.static_data.frames.get(&self.ratio) {
            context
                .renderer
                .render_shape(*shape, context.transform_stack.transform());
        } else {
            log::warn!("Missing ratio for morph shape");
        }

        context.transform_stack.pop();
    }
}

unsafe impl<'gc> gc_arena::Collect for MorphShape<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.static_data.trace(cc);
    }
}

/// Static data shared between all instances of a morph shape.
#[allow(dead_code)]
pub struct MorphShapeStatic {
    id: CharacterId,
    start: swf::MorphShape,
    end: swf::MorphShape,
    frames: fnv::FnvHashMap<u16, ShapeHandle>,
}

impl MorphShapeStatic {
    pub fn from_swf_tag(renderer: &mut dyn RenderBackend, swf_tag: &swf::DefineMorphShape) -> Self {
        let mut morph_shape = Self {
            id: swf_tag.id,
            start: swf_tag.start.clone(),
            end: swf_tag.end.clone(),
            frames: fnv::FnvHashMap::default(),
        };
        // Pre-register the start and end states.
        morph_shape.register_ratio(renderer, 0);
        morph_shape.register_ratio(renderer, 65535);
        morph_shape
    }

    pub fn register_ratio(&mut self, renderer: &mut dyn RenderBackend, ratio: u16) {
        if self.frames.contains_key(&ratio) {
            // Already registered.
            return;
        }

        // Interpolate MorphShapes into a Shape.
        use swf::{FillStyle, Gradient, LineStyle, ShapeRecord, ShapeStyles};
        // Start shape is ratio 65535, end shape is ratio 0.
        let b = f32::from(ratio) / 65535.0;
        let a = 1.0 - b;
        let fill_styles: Vec<FillStyle> = self
            .start
            .fill_styles
            .iter()
            .zip(self.end.fill_styles.iter())
            .map(|(start, end)| match (start, end) {
                (FillStyle::Color(start), FillStyle::Color(end)) => FillStyle::Color(Color {
                    r: (a * f32::from(start.r) + b * f32::from(end.r)) as u8,
                    g: (a * f32::from(start.g) + b * f32::from(end.g)) as u8,
                    b: (a * f32::from(start.b) + b * f32::from(end.b)) as u8,
                    a: (a * f32::from(start.a) + b * f32::from(end.a)) as u8,
                }),
                (FillStyle::LinearGradient(start), FillStyle::LinearGradient(end)) => {
                    let records: Vec<swf::GradientRecord> = start
                        .records
                        .iter()
                        .zip(end.records.iter())
                        .map(|(start, end)| swf::GradientRecord {
                            ratio: (f32::from(start.ratio) * a + f32::from(end.ratio) * b) as u8,
                            color: Color {
                                r: (a * f32::from(start.color.r) + b * f32::from(end.color.r))
                                    as u8,
                                g: (a * f32::from(start.color.g) + b * f32::from(end.color.g))
                                    as u8,
                                b: (a * f32::from(start.color.b) + b * f32::from(end.color.b))
                                    as u8,
                                a: (a * f32::from(start.color.a) + b * f32::from(end.color.a))
                                    as u8,
                            },
                        })
                        .collect();

                    FillStyle::LinearGradient(Gradient {
                        matrix: start.matrix.clone(),
                        spread: start.spread,
                        interpolation: start.interpolation,
                        records,
                    })
                }
                _ => {
                    log::info!("Unhandled morph shape combination: {:?} {:?}", start, end);
                    start.clone()
                }
            })
            .collect();
        let line_styles: Vec<LineStyle> = self
            .start
            .line_styles
            .iter()
            .zip(self.end.line_styles.iter())
            .map(|(start, end)| LineStyle {
                width: Twips::new(
                    ((start.width.get() as f32) * a + (end.width.get() as f32) * b) as i32,
                ),
                color: Color {
                    r: (a * f32::from(start.color.r) + b * f32::from(end.color.r)) as u8,
                    g: (a * f32::from(start.color.g) + b * f32::from(end.color.g)) as u8,
                    b: (a * f32::from(start.color.b) + b * f32::from(end.color.b)) as u8,
                    a: (a * f32::from(start.color.a) + b * f32::from(end.color.a)) as u8,
                },
                start_cap: start.start_cap,
                end_cap: start.end_cap,
                join_style: start.join_style,
                fill_style: None,
                allow_scale_x: start.allow_scale_x,
                allow_scale_y: start.allow_scale_y,
                is_pixel_hinted: start.is_pixel_hinted,
                allow_close: start.allow_close,
            })
            .collect();

        let mut shape = Vec::with_capacity(self.start.shape.len());
        let mut start_iter = self.start.shape.iter();
        let mut end_iter = self.end.shape.iter();
        let mut start = start_iter.next();
        let mut end = end_iter.next();
        let mut start_x = Twips::new(0);
        let mut start_y = Twips::new(0);
        let mut end_x = Twips::new(0);
        let mut end_y = Twips::new(0);
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
                    if let Some((s_x, s_y)) = start_change.move_to {
                        if let Some((e_x, e_y)) = end_change.move_to {
                            start_x = s_x;
                            start_y = s_y;
                            end_x = e_x;
                            end_y = e_y;
                            style_change.move_to = Some((
                                Twips::new(
                                    (start_x.get() as f32 * a + end_x.get() as f32 * b) as i32,
                                ),
                                Twips::new(
                                    (start_y.get() as f32 * a + end_y.get() as f32 * b) as i32,
                                ),
                            ));
                        } else {
                            panic!("Expected move_to for morph shape")
                        }
                    }
                    shape.push(ShapeRecord::StyleChange(style_change));
                    start = start_iter.next();
                    end = end_iter.next();
                }
                (ShapeRecord::StyleChange(start_change), _) => {
                    let mut style_change = start_change.clone();
                    if let Some((s_x, s_y)) = start_change.move_to {
                        start_x = s_x;
                        start_y = s_y;
                        style_change.move_to = Some((
                            Twips::new((start_x.get() as f32 * a + end_x.get() as f32 * b) as i32),
                            Twips::new((start_y.get() as f32 * a + end_y.get() as f32 * b) as i32),
                        ));
                    }
                    shape.push(ShapeRecord::StyleChange(style_change));
                    Self::update_pos(&mut start_x, &mut start_y, s);
                    start = start_iter.next();
                }
                (_, ShapeRecord::StyleChange(end_change)) => {
                    let mut style_change = end_change.clone();
                    if let Some((e_x, e_y)) = end_change.move_to {
                        end_x = e_x;
                        end_y = e_y;
                        style_change.move_to = Some((
                            Twips::new((start_x.get() as f32 * a + end_x.get() as f32 * b) as i32),
                            Twips::new((start_y.get() as f32 * a + end_y.get() as f32 * b) as i32),
                        ));
                    }
                    shape.push(ShapeRecord::StyleChange(style_change));
                    Self::update_pos(&mut end_x, &mut end_y, s);
                    end = end_iter.next();
                    continue;
                }
                _ => {
                    shape.push(Self::interpolate_edges(s, e, a));
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

        let bounds = crate::shape_utils::calculate_shape_bounds(&shape[..]);
        let shape = swf::Shape {
            version: 4,
            id: 0,
            shape_bounds: bounds.clone(),
            edge_bounds: bounds,
            has_fill_winding_rule: false,
            has_non_scaling_strokes: false,
            has_scaling_strokes: true,
            styles,
            shape,
        };

        let shape_handle = renderer.register_shape(&shape);
        self.frames.insert(ratio, shape_handle);
    }

    fn update_pos(x: &mut Twips, y: &mut Twips, record: &swf::ShapeRecord) {
        use swf::ShapeRecord;
        match record {
            ShapeRecord::StraightEdge { delta_x, delta_y } => {
                *x += *delta_x;
                *y += *delta_y;
            }
            ShapeRecord::CurvedEdge {
                control_delta_x,
                control_delta_y,
                anchor_delta_x,
                anchor_delta_y,
            } => {
                *x += *control_delta_x + *anchor_delta_x;
                *y += *control_delta_y + *anchor_delta_y;
            }
            ShapeRecord::StyleChange(ref style_change) => {
                if let Some((move_x, move_y)) = style_change.move_to {
                    *x = move_x;
                    *y = move_y;
                }
            }
        }
    }

    fn interpolate_edges(
        start: &swf::ShapeRecord,
        end: &swf::ShapeRecord,
        a: f32,
    ) -> swf::ShapeRecord {
        use swf::ShapeRecord;
        let b = 1.0 - a;
        match (start, end) {
            (
                ShapeRecord::StraightEdge {
                    delta_x: start_dx,
                    delta_y: start_dy,
                },
                ShapeRecord::StraightEdge {
                    delta_x: end_dx,
                    delta_y: end_dy,
                },
            ) => ShapeRecord::StraightEdge {
                delta_x: Twips::new((start_dx.get() as f32 * a + end_dx.get() as f32 * b) as i32),
                delta_y: Twips::new((start_dy.get() as f32 * a + end_dy.get() as f32 * b) as i32),
            },

            (
                ShapeRecord::CurvedEdge {
                    control_delta_x: start_cdx,
                    control_delta_y: start_cdy,
                    anchor_delta_x: start_adx,
                    anchor_delta_y: start_ady,
                },
                ShapeRecord::CurvedEdge {
                    control_delta_x: end_cdx,
                    control_delta_y: end_cdy,
                    anchor_delta_x: end_adx,
                    anchor_delta_y: end_ady,
                },
            ) => ShapeRecord::CurvedEdge {
                control_delta_x: Twips::new(
                    (start_cdx.get() as f32 * a + end_cdx.get() as f32 * b) as i32,
                ),
                control_delta_y: Twips::new(
                    (start_cdy.get() as f32 * a + end_cdy.get() as f32 * b) as i32,
                ),
                anchor_delta_x: Twips::new(
                    (start_adx.get() as f32 * a + end_adx.get() as f32 * b) as i32,
                ),
                anchor_delta_y: Twips::new(
                    (start_ady.get() as f32 * a + end_ady.get() as f32 * b) as i32,
                ),
            },

            (
                ShapeRecord::StraightEdge {
                    delta_x: start_dx,
                    delta_y: start_dy,
                },
                ShapeRecord::CurvedEdge {
                    control_delta_x: end_cdx,
                    control_delta_y: end_cdy,
                    anchor_delta_x: end_adx,
                    anchor_delta_y: end_ady,
                },
            ) => {
                let start_cdx = *start_dx / 2;
                let start_cdy = *start_dy / 2;
                let start_adx = start_cdx;
                let start_ady = start_cdy;
                ShapeRecord::CurvedEdge {
                    control_delta_x: Twips::new(
                        (start_cdx.get() as f32 * a + end_cdx.get() as f32 * b) as i32,
                    ),
                    control_delta_y: Twips::new(
                        (start_cdy.get() as f32 * a + end_cdy.get() as f32 * b) as i32,
                    ),
                    anchor_delta_x: Twips::new(
                        (start_adx.get() as f32 * a + end_adx.get() as f32 * b) as i32,
                    ),
                    anchor_delta_y: Twips::new(
                        (start_ady.get() as f32 * a + end_ady.get() as f32 * b) as i32,
                    ),
                }
            }

            (
                ShapeRecord::CurvedEdge {
                    control_delta_x: start_cdx,
                    control_delta_y: start_cdy,
                    anchor_delta_x: start_adx,
                    anchor_delta_y: start_ady,
                },
                ShapeRecord::StraightEdge {
                    delta_x: end_dx,
                    delta_y: end_dy,
                },
            ) => {
                let end_cdx = *end_dx / 2;
                let end_cdy = *end_dy / 2;
                let end_adx = end_cdx;
                let end_ady = end_cdy;
                ShapeRecord::CurvedEdge {
                    control_delta_x: Twips::new(
                        (start_cdx.get() as f32 * a + end_cdx.get() as f32 * b) as i32,
                    ),
                    control_delta_y: Twips::new(
                        (start_cdy.get() as f32 * a + end_cdy.get() as f32 * b) as i32,
                    ),
                    anchor_delta_x: Twips::new(
                        (start_adx.get() as f32 * a + end_adx.get() as f32 * b) as i32,
                    ),
                    anchor_delta_y: Twips::new(
                        (start_ady.get() as f32 * a + end_ady.get() as f32 * b) as i32,
                    ),
                }
            }
            _ => unreachable!("{:?} {:?}", start, end),
        }
    }
}

unsafe impl<'gc> gc_arena::Collect for MorphShapeStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
