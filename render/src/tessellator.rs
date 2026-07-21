use crate::bitmap::BitmapSource;
use crate::shape_utils::{DistilledShape, DrawCommand, DrawPath, GradientType};
use indexmap::IndexSet;
use lyon::path::Path;
use lyon::tessellation::{
    self, FillTessellator, FillVertex, StrokeTessellator, StrokeVertex, StrokeVertexConstructor,
    geometry_builder::{BuffersBuilder, FillVertexConstructor, VertexBuffers},
};
use lyon::tessellation::{FillOptions, StrokeOptions};
use swf::GradientRecord;
use tracing::instrument;

pub struct ShapeTessellator {
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
    mesh: Vec<Draw>,
    gradients: IndexSet<Gradient>,
    lyon_mesh: VertexBuffers<Vertex, u32>,
    mask_index_count: Option<u32>,
    is_stroke: bool,
}

const TESSELLATION_EPSILON: f32 = 0.0000001;

impl ShapeTessellator {
    pub fn new() -> Self {
        Self {
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
            mesh: Vec::new(),
            gradients: IndexSet::new(),
            lyon_mesh: VertexBuffers::new(),
            mask_index_count: None,
            is_stroke: false,
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub fn tessellate_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> Mesh {
        self.tessellate_shape_with_scale(shape, bitmap_source, 1.0)
    }

    #[instrument(level = "debug", skip_all)]
    pub fn tessellate_shape_with_scale(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
        scale: f32,
    ) -> Mesh {
        self.mesh = Vec::new();
        self.gradients = IndexSet::new();
        self.lyon_mesh = VertexBuffers::new();

        for path in shape.paths {
            let path = match path {
                DrawPath::Triangles {
                    style,
                    vertices,
                    indices,
                    texture_coords,
                } => {
                    self.flush_draw(DrawType::Color);
                    self.is_stroke = false;

                    if let Some((draw_type, color, _)) =
                        self.draw_type_for_style(style, bitmap_source, texture_coords.is_some())
                    {
                        let texture_coords = if matches!(draw_type, DrawType::Bitmap(_)) {
                            texture_coords
                        } else {
                            None
                        };
                        self.mesh.push(Draw {
                            draw_type,
                            vertices: vertices
                                .into_iter()
                                .map(|position| Vertex {
                                    x: position.x.to_pixels() as f32,
                                    y: position.y.to_pixels() as f32,
                                    color,
                                })
                                .collect(),
                            mask_index_count: indices.len() as u32,
                            indices,
                            texture_coords,
                        });
                    }
                    continue;
                }
                path => path,
            };

            let (fill_style, lyon_path, next_is_stroke) = match &path {
                DrawPath::Fill {
                    style,
                    commands,
                    winding_rule: _,
                } => (*style, ruffle_path_to_lyon_path(commands, true), false),
                DrawPath::Stroke {
                    style,
                    commands,
                    is_closed,
                } => (
                    style.fill_style(),
                    ruffle_path_to_lyon_path(commands, *is_closed),
                    true,
                ),
                DrawPath::Triangles { .. } => unreachable!(),
            };

            let Some((draw, color, needs_flush)) =
                self.draw_type_for_style(fill_style, bitmap_source, false)
            else {
                continue;
            };

            if needs_flush || (self.is_stroke && !next_is_stroke) {
                // We flush separate draw calls in these cases:
                // * Non-solid color fills which require their own shader.
                // * Strokes followed by fills, because strokes need to be omitted
                //   when using this shape as a mask.
                self.flush_draw(DrawType::Color);
            } else if !self.is_stroke && next_is_stroke {
                // Bake solid color fills followed by strokes into a single draw call, and adjust
                // the index count to omit the strokes when rendering this shape as a mask.
                assert!(self.mask_index_count.is_none());
                self.mask_index_count = Some(self.lyon_mesh.indices.len() as u32);
            }
            self.is_stroke = next_is_stroke;

            let mut buffers_builder =
                BuffersBuilder::new(&mut self.lyon_mesh, RuffleVertexCtor { color });
            let result = match path {
                DrawPath::Fill { winding_rule, .. } => {
                    // Larger scales require more precise tessellation to avoid artifacts
                    let tolerance = FillOptions::DEFAULT_TOLERANCE / scale;
                    self.fill_tess.tessellate_path(
                        &lyon_path,
                        &FillOptions::default()
                            .with_fill_rule(winding_rule.into())
                            .with_tolerance(tolerance),
                        &mut buffers_builder,
                    )
                }
                DrawPath::Stroke { style, .. } => {
                    // This calculation ensures that hairline strokes are rendered with
                    // a minimum width of 1 pixel, while still allowing for proper scaling
                    let width = style.width().to_pixels() as f32;
                    let min_screen_width = if f32::abs(scale) > TESSELLATION_EPSILON {
                        1.0 / scale
                    } else {
                        1.0
                    };
                    let width = width.max(min_screen_width);

                    // Larger scales require more precise tessellation to avoid artifacts
                    let tolerance = StrokeOptions::DEFAULT_TOLERANCE / scale;
                    let mut stroke_options = StrokeOptions::default()
                        .with_line_width(width)
                        .with_tolerance(tolerance)
                        .with_start_cap(match style.start_cap() {
                            swf::LineCapStyle::None => tessellation::LineCap::Butt,
                            swf::LineCapStyle::Round => tessellation::LineCap::Round,
                            swf::LineCapStyle::Square => tessellation::LineCap::Square,
                        })
                        .with_end_cap(match style.end_cap() {
                            swf::LineCapStyle::None => tessellation::LineCap::Butt,
                            swf::LineCapStyle::Round => tessellation::LineCap::Round,
                            swf::LineCapStyle::Square => tessellation::LineCap::Square,
                        });

                    let line_join = match style.join_style() {
                        swf::LineJoinStyle::Round => tessellation::LineJoin::Round,
                        swf::LineJoinStyle::Bevel => tessellation::LineJoin::Bevel,
                        swf::LineJoinStyle::Miter(limit) => {
                            // Avoid lyon assert with small miter limits.
                            let limit = limit.to_f32();
                            if limit >= StrokeOptions::MINIMUM_MITER_LIMIT {
                                stroke_options = stroke_options.with_miter_limit(limit);
                                tessellation::LineJoin::MiterClip
                            } else {
                                tessellation::LineJoin::Bevel
                            }
                        }
                    };
                    stroke_options = stroke_options.with_line_join(line_join);
                    self.stroke_tess.tessellate_path(
                        &lyon_path,
                        &stroke_options,
                        &mut buffers_builder,
                    )
                }
                DrawPath::Triangles { .. } => unreachable!(),
            };
            match result {
                Ok(_) => {
                    if needs_flush {
                        // Non-solid color fills are isolated draw calls; flush immediately.
                        self.flush_draw(draw);
                    }
                }
                Err(e) => {
                    // This may simply be a degenerate path.
                    tracing::error!("Tessellation failure: {:?}", e);
                }
            }
        }

        // Flush the final pending draw.
        self.flush_draw(DrawType::Color);

        self.lyon_mesh = VertexBuffers::new();
        Mesh {
            draws: std::mem::take(&mut self.mesh),
            gradients: std::mem::take(&mut self.gradients).into_iter().collect(),
        }
    }

    fn flush_draw(&mut self, draw: DrawType) {
        if self.lyon_mesh.vertices.is_empty() || self.lyon_mesh.indices.len() < 3 {
            // Ignore degenerate fills
            self.lyon_mesh = VertexBuffers::new();
            self.mask_index_count = None;
            return;
        }
        let draw_mesh = std::mem::replace(&mut self.lyon_mesh, VertexBuffers::new());
        self.mesh.push(Draw {
            draw_type: draw,
            mask_index_count: self
                .mask_index_count
                .unwrap_or(draw_mesh.indices.len() as u32),
            vertices: draw_mesh.vertices,
            indices: draw_mesh.indices,
            texture_coords: None,
        });
        self.mask_index_count = None;
    }

    fn draw_type_for_style(
        &mut self,
        fill_style: &swf::FillStyle,
        bitmap_source: &dyn BitmapSource,
        use_texture_coords: bool,
    ) -> Option<(DrawType, swf::Color, bool)> {
        Some(match fill_style {
            swf::FillStyle::Color(color) => (DrawType::Color, *color, false),
            swf::FillStyle::LinearGradient(gradient) => {
                let uniform =
                    swf_gradient_to_uniforms(GradientType::Linear, gradient, swf::Fixed8::ZERO);
                let (gradient_index, _) = self.gradients.insert_full(uniform);
                (
                    DrawType::Gradient {
                        matrix: swf_to_gl_matrix(gradient.matrix.into()),
                        gradient: gradient_index,
                    },
                    swf::Color::WHITE,
                    true,
                )
            }
            swf::FillStyle::RadialGradient(gradient) => {
                let uniform =
                    swf_gradient_to_uniforms(GradientType::Radial, gradient, swf::Fixed8::ZERO);
                let (gradient_index, _) = self.gradients.insert_full(uniform);
                (
                    DrawType::Gradient {
                        matrix: swf_to_gl_matrix(gradient.matrix.into()),
                        gradient: gradient_index,
                    },
                    swf::Color::WHITE,
                    true,
                )
            }
            swf::FillStyle::FocalGradient {
                gradient,
                focal_point,
            } => {
                let uniform = swf_gradient_to_uniforms(GradientType::Focal, gradient, *focal_point);
                let (gradient_index, _) = self.gradients.insert_full(uniform);
                (
                    DrawType::Gradient {
                        matrix: swf_to_gl_matrix(gradient.matrix.into()),
                        gradient: gradient_index,
                    },
                    swf::Color::WHITE,
                    true,
                )
            }
            swf::FillStyle::Bitmap {
                id,
                matrix,
                is_smoothed,
                is_repeating,
            } => {
                let bitmap = bitmap_source.bitmap_size(*id)?;
                let matrix = if use_texture_coords {
                    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
                } else {
                    swf_bitmap_to_gl_matrix((*matrix).into(), bitmap.width, bitmap.height)
                };
                (
                    DrawType::Bitmap(Bitmap {
                        matrix,
                        bitmap_id: *id,
                        is_smoothed: *is_smoothed,
                        is_repeating: *is_repeating,
                    }),
                    swf::Color::WHITE,
                    true,
                )
            }
        })
    }
}

impl Default for ShapeTessellator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Mesh {
    pub draws: Vec<Draw>,
    pub gradients: Vec<Gradient>,
}

pub struct Draw {
    pub draw_type: DrawType,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub mask_index_count: u32,
    pub texture_coords: Option<Vec<[f32; 3]>>,
}

pub enum DrawType {
    Color,
    Gradient {
        matrix: [[f32; 3]; 3],
        gradient: usize,
    },
    Bitmap(Bitmap),
}

impl DrawType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Color => "Color",
            Self::Gradient { .. } => "Gradient",
            Self::Bitmap { .. } => "Bitmap",
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Gradient {
    pub gradient_type: GradientType,
    pub repeat_mode: swf::GradientSpread,
    pub focal_point: swf::Fixed8,
    pub interpolation: swf::GradientInterpolation,
    pub records: Vec<GradientRecord>,
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub color: swf::Color,
}

#[derive(Clone, Debug)]
pub struct Bitmap {
    pub matrix: [[f32; 3]; 3],
    pub bitmap_id: u16,
    pub is_smoothed: bool,
    pub is_repeating: bool,
}

#[expect(clippy::many_single_char_names)]
fn swf_to_gl_matrix(m: crate::matrix::Matrix) -> [[f32; 3]; 3] {
    let tx = m.tx.get() as f32;
    let ty = m.ty.get() as f32;
    let det = m.a * m.d - m.c * m.b;
    let mut a = m.d / det;
    let mut b = -m.c / det;
    let mut c = -(tx * m.d - m.c * ty) / det;
    let mut d = -m.b / det;
    let mut e = m.a / det;
    let mut f = (tx * m.b - m.a * ty) / det;

    a *= 20.0 / 32768.0;
    b *= 20.0 / 32768.0;
    d *= 20.0 / 32768.0;
    e *= 20.0 / 32768.0;

    c /= 32768.0;
    f /= 32768.0;
    c += 0.5;
    f += 0.5;
    [[a, d, 0.0], [b, e, 0.0], [c, f, 1.0]]
}

#[expect(clippy::many_single_char_names)]
fn swf_bitmap_to_gl_matrix(
    m: crate::matrix::Matrix,
    bitmap_width: u32,
    bitmap_height: u32,
) -> [[f32; 3]; 3] {
    let bitmap_width = bitmap_width as f32;
    let bitmap_height = bitmap_height as f32;

    let tx = m.tx.get() as f32;
    let ty = m.ty.get() as f32;
    let det = m.a * m.d - m.c * m.b;
    let mut a = m.d / det;
    let mut b = -m.c / det;
    let mut c = -(tx * m.d - m.c * ty) / det;
    let mut d = -m.b / det;
    let mut e = m.a / det;
    let mut f = (tx * m.b - m.a * ty) / det;

    a *= 20.0 / bitmap_width;
    b *= 20.0 / bitmap_width;
    d *= 20.0 / bitmap_height;
    e *= 20.0 / bitmap_height;

    c /= bitmap_width;
    f /= bitmap_height;

    [[a, d, 0.0], [b, e, 0.0], [c, f, 1.0]]
}

fn ruffle_path_to_lyon_path(commands: &[DrawCommand], is_closed: bool) -> Path {
    fn point(point: swf::Point<swf::Twips>) -> lyon::math::Point {
        lyon::math::Point::new(point.x.to_pixels() as f32, point.y.to_pixels() as f32)
    }

    let mut builder = Path::builder();
    let mut cursor = Some(swf::Point::ZERO);
    for command in commands {
        match command {
            DrawCommand::MoveTo(move_to) => {
                if cursor.is_none() {
                    builder.end(false);
                }
                cursor = Some(*move_to);
            }
            DrawCommand::LineTo(line_to) => {
                if let Some(cursor) = cursor.take() {
                    builder.begin(point(cursor));
                }
                builder.line_to(point(*line_to));
            }
            DrawCommand::QuadraticCurveTo { control, anchor } => {
                if let Some(cursor) = cursor.take() {
                    builder.begin(point(cursor));
                }
                builder.quadratic_bezier_to(point(*control), point(*anchor));
            }
            DrawCommand::CubicCurveTo {
                control_a,
                control_b,
                anchor,
            } => {
                if let Some(cursor) = cursor.take() {
                    builder.begin(point(cursor));
                }
                builder.cubic_bezier_to(point(*control_a), point(*control_b), point(*anchor));
            }
        }
    }

    if cursor.is_none() {
        if is_closed {
            builder.close();
        } else {
            builder.end(false);
        }
    }

    builder.build()
}

/// Converts a gradient to the uniforms used by the shader.
fn swf_gradient_to_uniforms(
    gradient_type: GradientType,
    gradient: &swf::Gradient,
    focal_point: swf::Fixed8,
) -> Gradient {
    Gradient {
        records: gradient.records.clone(),
        gradient_type,
        repeat_mode: gradient.spread,
        focal_point,
        interpolation: gradient.interpolation,
    }
}

struct RuffleVertexCtor {
    color: swf::Color,
}

impl FillVertexConstructor<Vertex> for RuffleVertexCtor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            x: vertex.position().x,
            y: vertex.position().y,
            color: self.color,
        }
    }
}

impl StrokeVertexConstructor<Vertex> for RuffleVertexCtor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        Vertex {
            x: vertex.position().x,
            y: vertex.position().y,
            color: self.color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bitmap, DrawType, ShapeTessellator};
    use crate::backend::RenderBackend;
    use crate::bitmap::{BitmapHandle, BitmapSize, BitmapSource};
    use crate::shape_utils::{DistilledShape, DrawPath};
    use swf::{Color, FillStyle, Matrix, Point, Rectangle, Twips};

    struct TestBitmapSource;

    impl BitmapSource for TestBitmapSource {
        fn bitmap_size(&self, id: u16) -> Option<BitmapSize> {
            (id == 7).then_some(BitmapSize {
                width: 64,
                height: 32,
            })
        }

        fn bitmap_handle(
            &self,
            _id: u16,
            _renderer: &mut dyn RenderBackend,
        ) -> Option<BitmapHandle> {
            None
        }
    }

    #[test]
    fn preserves_bitmap_triangle_texture_coordinates() {
        let vertices = vec![
            Point::from_pixels(0.0, 0.0),
            Point::from_pixels(20.0, 0.0),
            Point::from_pixels(0.0, 20.0),
        ];
        let texture_coords = vec![[0.0, 0.0, 1.0], [0.5, 0.0, 0.5], [0.0, 0.25, 0.25]];
        let style = FillStyle::Bitmap {
            id: 7,
            matrix: Matrix::IDENTITY,
            is_smoothed: true,
            is_repeating: false,
        };
        let shape = DistilledShape {
            paths: vec![DrawPath::Triangles {
                style: &style,
                vertices,
                indices: vec![0, 1, 2],
                texture_coords: Some(texture_coords.clone()),
            }],
            shape_bounds: Rectangle::<Twips>::default(),
            edge_bounds: Rectangle::<Twips>::default(),
            id: 0,
        };

        let mesh = ShapeTessellator::new().tessellate_shape(shape, &TestBitmapSource);
        assert_eq!(mesh.draws.len(), 1);
        let draw = &mesh.draws[0];
        assert_eq!(draw.indices, [0, 1, 2]);
        assert_eq!(draw.mask_index_count, 3);
        assert_eq!(
            draw.texture_coords.as_deref(),
            Some(texture_coords.as_slice())
        );
        assert_eq!(
            draw.vertices
                .iter()
                .map(|vertex| (vertex.x, vertex.y))
                .collect::<Vec<_>>(),
            [(0.0, 0.0), (20.0, 0.0), (0.0, 20.0)]
        );
        match &draw.draw_type {
            DrawType::Bitmap(Bitmap {
                matrix,
                bitmap_id,
                is_smoothed,
                is_repeating,
            }) => {
                assert_eq!(*matrix, [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
                assert_eq!(*bitmap_id, 7);
                assert!(*is_smoothed);
                assert!(!*is_repeating);
            }
            _ => panic!("Expected a bitmap triangle draw"),
        }
    }

    #[test]
    fn injects_color_triangles_without_tessellation() {
        let style = FillStyle::Color(Color::RED);
        let shape = DistilledShape {
            paths: vec![DrawPath::Triangles {
                style: &style,
                vertices: vec![
                    Point::from_pixels(0.0, 0.0),
                    Point::from_pixels(10.0, 0.0),
                    Point::from_pixels(0.0, 10.0),
                    Point::from_pixels(10.0, 10.0),
                ],
                indices: vec![0, 1, 2, 1, 3, 2],
                texture_coords: None,
            }],
            shape_bounds: Rectangle::<Twips>::default(),
            edge_bounds: Rectangle::<Twips>::default(),
            id: 0,
        };

        let mesh = ShapeTessellator::new().tessellate_shape(shape, &TestBitmapSource);
        let [draw] = mesh.draws.as_slice() else {
            panic!("Expected one direct triangle draw");
        };
        assert!(matches!(draw.draw_type, DrawType::Color));
        assert_eq!(draw.indices, [0, 1, 2, 1, 3, 2]);
        assert_eq!(draw.vertices.len(), 4);
        assert!(
            draw.vertices
                .iter()
                .all(|vertex| vertex.color == Color::RED)
        );
        assert!(draw.texture_coords.is_none());
    }
}
