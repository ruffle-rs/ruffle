use crate::bitmap::BitmapSource;
use crate::shape_utils::{DistilledShape, DrawCommand, DrawPath, GradientType};
use lyon::path::Path as LyonPath;
use lyon::tessellation::FillOptions;
use lyon::tessellation::{
    geometry_builder::{BuffersBuilder, FillVertexConstructor, VertexBuffers},
    FillTessellator, FillVertex,
};
use swf::GradientRecord;
use tiny_skia_path::{LineCap, LineJoin, Path as TinySkiaPath, PathBuilder, PathStroker, Stroke};
use tracing::instrument;

pub struct ShapeTessellator {
    tess: FillTessellator,
    stroker: PathStroker,
    mesh: Vec<Draw>,
    lyon_mesh: VertexBuffers<Vertex, u32>,
    mask_index_count: Option<u32>,
    is_stroke: bool,
}

impl ShapeTessellator {
    pub fn new() -> Self {
        Self {
            tess: FillTessellator::new(),
            stroker: PathStroker::new(),
            mesh: Vec::new(),
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
                } => {
                    let mut stroke_options = Stroke::default();
                    stroke_options.width = (style.width().to_pixels() as f32).max(1.0);
                    stroke_options.line_cap = match style.start_cap() {
                        swf::LineCapStyle::None => LineCap::Butt,
                        swf::LineCapStyle::Round => LineCap::Round,
                        swf::LineCapStyle::Square => LineCap::Square,
                    }; // no separate start and end cap styles :/

                    stroke_options.line_join = match style.join_style() {
                        swf::LineJoinStyle::Round => LineJoin::Round,
                        swf::LineJoinStyle::Bevel => LineJoin::Bevel,
                        swf::LineJoinStyle::Miter(limit) => {
                            stroke_options.miter_limit = limit.to_f32();
                            LineJoin::MiterClip
                        }
                    };

                    let skia_path = &ruffle_path_to_skia_path(commands, *is_closed);

                    let lyon_path = if let Some(skia_path) = skia_path {
                        self.stroker
                            .stroke(skia_path, &stroke_options, 1.0)
                            .map_or_else(LyonPath::new, |stroked| skia_path_to_lyon_path(&stroked))
                    } else {
                        LyonPath::new()
                    };

                    (style.fill_style(), lyon_path, true)
                }
            };

            let (draw, color, needs_flush) = match fill_style {
                swf::FillStyle::Color(color) => (DrawType::Color, *color, false),
                swf::FillStyle::LinearGradient(gradient) => (
                    DrawType::Gradient(swf_gradient_to_uniforms(
                        GradientType::Linear,
                        gradient,
                        swf::Fixed8::ZERO,
                    )),
                    swf::Color::WHITE,
                    true,
                ),
                swf::FillStyle::RadialGradient(gradient) => (
                    DrawType::Gradient(swf_gradient_to_uniforms(
                        GradientType::Radial,
                        gradient,
                        swf::Fixed8::ZERO,
                    )),
                    swf::Color::WHITE,
                    true,
                ),
                swf::FillStyle::FocalGradient {
                    gradient,
                    focal_point,
                } => (
                    DrawType::Gradient(swf_gradient_to_uniforms(
                        GradientType::Focal,
                        gradient,
                        *focal_point,
                    )),
                    swf::Color::WHITE,
                    true,
                ),
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
                DrawPath::Fill { winding_rule, .. } => self.tess.tessellate_path(
                    &lyon_path,
                    &FillOptions::default().with_fill_rule(winding_rule.into()),
                    &mut buffers_builder,
                ),
                DrawPath::Stroke { .. } => self.tess.tessellate_path(
                    &lyon_path,
                    &FillOptions::default().with_fill_rule(lyon::path::FillRule::NonZero),
                    &mut buffers_builder,
                ),
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
        std::mem::take(&mut self.mesh)
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

type Mesh = Vec<Draw>;

pub struct Draw {
    pub draw_type: DrawType,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub mask_index_count: u32,
}

pub enum DrawType {
    Color,
    Gradient(Gradient),
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

#[derive(Clone, Debug)]
pub struct Gradient {
    pub matrix: [[f32; 3]; 3],
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

fn ruffle_path_to_lyon_path(commands: &[DrawCommand], is_closed: bool) -> LyonPath {
    fn point(point: swf::Point<swf::Twips>) -> lyon::math::Point {
        lyon::math::Point::new(point.x.to_pixels() as f32, point.y.to_pixels() as f32)
    }

    let mut builder = LyonPath::builder();
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

fn ruffle_path_to_skia_path(commands: &[DrawCommand], is_closed: bool) -> Option<TinySkiaPath> {
    let mut builder = PathBuilder::new();
    let mut only_a_move = true;
    for command in commands {
        match command {
            DrawCommand::MoveTo(move_to) => {
                builder.move_to(move_to.x.to_pixels() as f32, move_to.y.to_pixels() as f32);
            }
            DrawCommand::LineTo(line_to) => {
                only_a_move = false;
                builder.line_to(line_to.x.to_pixels() as f32, line_to.y.to_pixels() as f32);
            }
            DrawCommand::QuadraticCurveTo { control, anchor } => {
                only_a_move = false;
                builder.quad_to(
                    control.x.to_pixels() as f32,
                    control.y.to_pixels() as f32,
                    anchor.x.to_pixels() as f32,
                    anchor.y.to_pixels() as f32,
                );
            }
            DrawCommand::CubicCurveTo {
                control_a,
                control_b,
                anchor,
            } => {
                only_a_move = false;
                builder.cubic_to(
                    control_a.x.to_pixels() as f32,
                    control_a.y.to_pixels() as f32,
                    control_b.x.to_pixels() as f32,
                    control_b.y.to_pixels() as f32,
                    anchor.x.to_pixels() as f32,
                    anchor.y.to_pixels() as f32,
                );
            }
        }
    }

    if !only_a_move && is_closed {
        builder.close();
    }

    builder.finish()
}

fn skia_path_to_lyon_path(path: &TinySkiaPath) -> LyonPath {
    fn p(point: tiny_skia_path::Point) -> lyon::math::Point {
        lyon::math::Point::new(point.x, point.y)
    }

    let mut builder = LyonPath::builder();
    for segment in path.segments() {
        match segment {
            tiny_skia_path::PathSegment::MoveTo(point) => {
                builder.begin(p(point));
            }
            tiny_skia_path::PathSegment::LineTo(point) => {
                builder.line_to(p(point));
            }
            tiny_skia_path::PathSegment::QuadTo(control, anchor) => {
                builder.quadratic_bezier_to(p(control), p(anchor));
            }
            tiny_skia_path::PathSegment::CubicTo(control1, control2, anchor) => {
                builder.cubic_bezier_to(p(control1), p(control2), p(anchor));
            }
            tiny_skia_path::PathSegment::Close => {
                builder.close();
            }
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
        matrix: swf_to_gl_matrix(gradient.matrix.into()),
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
