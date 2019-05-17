use crate::backend::render::{RenderBackend, ShapeHandle};
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObjectBase, DisplayObjectImpl};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use std::collections::HashMap;
use swf::Twips;

#[derive(Clone, Trace, Finalize)]
pub struct MorphShape {
    base: DisplayObjectBase,

    #[unsafe_ignore_trace]
    start: swf::MorphShape,

    #[unsafe_ignore_trace]
    end: swf::MorphShape,

    #[unsafe_ignore_trace]
    frames: HashMap<u16, ShapeHandle>,

    ratio: u16,
}

impl MorphShape {
    pub fn from_swf_tag(swf_tag: &swf::DefineMorphShape, context: &mut UpdateContext) -> Self {
        // Convert the MorphShape into a normal Shape.
        // TODO(Herschel): impl From in swf crate?
        let mut morph_shape = Self {
            start: swf_tag.start.clone(),
            end: swf_tag.end.clone(),
            base: Default::default(),
            frames: HashMap::new(),
            ratio: 0,
        };

        morph_shape.register_ratio(context.renderer, 0);
        morph_shape.register_ratio(context.renderer, 65535);

        morph_shape
    }

    pub fn register_ratio(&mut self, renderer: &mut RenderBackend, ratio: u16) {
        if self.frames.contains_key(&ratio) {
            // Already registered.
            return;
        }

        info!("Registered ratio {}", ratio);

        // Interpolate MorphShapes into a Shape.
        use swf::{Color, FillStyle, Gradient, LineStyle, ShapeRecord, ShapeStyles};
        let a = f32::from(ratio) / 65535.0;
        let b = 1.0 - a;
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
        while let Some(ref start) = start_iter.next() {
            match start {
                &ShapeRecord::StraightEdge { .. } | ShapeRecord::CurvedEdge { .. } => {
                    let end = end_iter.next().unwrap();
                    shape.push(Self::interpolate_edges(&start, &end, a));
                }
                &ShapeRecord::StyleChange(style_change) => {
                    let mut style_change = style_change.clone();
                    if let Some((start_x, start_y)) = style_change.move_to {
                        let end = end_iter.next().unwrap();
                        if let ShapeRecord::StyleChange(swf::StyleChangeData {
                            move_to: Some((end_x, end_y)),
                            ..
                        }) = end
                        {
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
            _ => unreachable!(),
        }
    }

    pub fn ratio(&self) -> u16 {
        self.ratio
    }

    pub fn set_ratio(&mut self, ratio: u16) {
        self.ratio = ratio;
    }
}

impl DisplayObjectImpl for MorphShape {
    impl_display_object!(base);

    fn as_morph_shape(&self) -> Option<&crate::morph_shape::MorphShape> {
        Some(self)
    }

    fn as_morph_shape_mut(&mut self) -> Option<&mut crate::morph_shape::MorphShape> {
        Some(self)
    }

    fn run_frame(&mut self, context: &mut UpdateContext) {
        if !self.frames.contains_key(&self.ratio) {
            self.register_ratio(context.renderer, self.ratio);
        }
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());

        if let Some(shape) = self.frames.get(&self.ratio) {
            context
                .renderer
                .render_shape(*shape, context.transform_stack.transform());
        } else {
            warn!("Missing ratio for morph shape");
        }

        context.transform_stack.pop();
    }
}
