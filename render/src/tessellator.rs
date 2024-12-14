use crate::bitmap::BitmapSource;
use crate::shape_utils::{DistilledShape, DrawCommand, DrawPath, GradientType};
use indexmap::IndexSet;
use lyon::path::Path;
use lyon::tessellation::{
    self,
    geometry_builder::{BuffersBuilder, FillVertexConstructor, VertexBuffers},
    FillTessellator, FillVertex, StrokeTessellator, StrokeVertex, StrokeVertexConstructor,
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
        self.mesh = Vec::new();
        self.gradients = IndexSet::new();
        self.lyon_mesh = VertexBuffers::new();

        for path in shape.paths {
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
            };

            let (draw, color, needs_flush) = match fill_style {
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
                    let uniform =
                        swf_gradient_to_uniforms(GradientType::Focal, gradient, *focal_point);
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
                    if let Some(bitmap) = bitmap_source.bitmap_size(*id) {
                        (
                            DrawType::Bitmap(Bitmap {
                                matrix: swf_bitmap_to_gl_matrix(
                                    (*matrix).into(),
                                    bitmap.width.into(),
                                    bitmap.height.into(),
                                ),
                                bitmap_id: *id,
                                is_smoothed: *is_smoothed,
                                is_repeating: *is_repeating,
                            }),
                            swf::Color::WHITE,
                            true,
                        )
                    } else {
                        // Missing bitmap -- incorrect character ID in SWF?
                        continue;
                    }
                }
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
                DrawPath::Fill { winding_rule, .. } => self.fill_tess.tessellate_path(
                    &lyon_path,
                    &FillOptions::default().with_fill_rule(winding_rule.into()),
                    &mut buffers_builder,
                ),
                DrawPath::Stroke { style, .. } => {
                    // TODO(Herschel): 0 width indicates "hairline".
                    let width = (style.width().to_pixels() as f32).max(1.0);
                    let mut stroke_options = StrokeOptions::default()
                        .with_line_width(width)
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
        });
        self.mask_index_count = None;
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

#[allow(clippy::many_single_char_names)]
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

#[allow(clippy::many_single_char_names)]
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
