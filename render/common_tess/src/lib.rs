use lyon::path::Path;
use lyon::tessellation::{
    self,
    geometry_builder::{BuffersBuilder, FillVertexConstructor, VertexBuffers},
    FillTessellator, FillVertex, StrokeTessellator, StrokeVertex, StrokeVertexConstructor,
};
use lyon::tessellation::{FillOptions, StrokeOptions};
use ruffle_core::backend::render::{
    swf::{self, FillStyle, GradientInterpolation, Twips},
    BitmapHandle,
};
use ruffle_core::shape_utils::{DistilledShape, DrawCommand, DrawPath};

pub struct ShapeTessellator {
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
}

impl ShapeTessellator {
    pub fn new() -> Self {
        Self {
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
        }
    }

    pub fn tessellate_shape<F>(&mut self, shape: DistilledShape, get_bitmap: F) -> Mesh
    where
        F: Fn(swf::CharacterId) -> Option<(u32, u32, BitmapHandle)>,
    {
        let mut mesh = Vec::new();

        let mut lyon_mesh: VertexBuffers<_, u32> = VertexBuffers::new();

        fn flush_draw(draw: DrawType, mesh: &mut Mesh, lyon_mesh: &mut VertexBuffers<Vertex, u32>) {
            if lyon_mesh.vertices.is_empty() {
                return;
            }

            let draw_mesh = std::mem::replace(lyon_mesh, VertexBuffers::new());
            mesh.push(Draw {
                draw_type: draw,
                vertices: draw_mesh.vertices,
                indices: draw_mesh.indices,
            });
        }

        for path in shape.paths {
            match path {
                DrawPath::Fill { style, commands } => match style {
                    FillStyle::Color(color) => {
                        let color = ((color.a as u32) << 24)
                            | ((color.b as u32) << 16)
                            | ((color.g as u32) << 8)
                            | (color.r as u32);

                        let mut buffers_builder =
                            BuffersBuilder::new(&mut lyon_mesh, RuffleVertexCtor { color });

                        if let Err(e) = self.fill_tess.tessellate_path(
                            &ruffle_path_to_lyon_path(commands, true),
                            &FillOptions::even_odd(),
                            &mut buffers_builder,
                        ) {
                            // This may just be a degenerate path; skip it.
                            log::error!("Tessellation failure: {:?}", e);
                            continue;
                        }
                    }
                    FillStyle::LinearGradient(gradient) => {
                        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh);

                        let mut buffers_builder = BuffersBuilder::new(
                            &mut lyon_mesh,
                            RuffleVertexCtor { color: 0xffff_ffff },
                        );

                        if let Err(e) = self.fill_tess.tessellate_path(
                            &ruffle_path_to_lyon_path(commands, true),
                            &FillOptions::even_odd(),
                            &mut buffers_builder,
                        ) {
                            // This may just be a degenerate path; skip it.
                            log::error!("Tessellation failure: {:?}", e);
                            continue;
                        }

                        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(8);
                        let mut ratios: Vec<f32> = Vec::with_capacity(8);
                        for record in &gradient.records {
                            colors.push([
                                f32::from(record.color.r) / 255.0,
                                f32::from(record.color.g) / 255.0,
                                f32::from(record.color.b) / 255.0,
                                f32::from(record.color.a) / 255.0,
                            ]);
                            ratios.push(f32::from(record.ratio) / 255.0);
                        }

                        let gradient = Gradient {
                            gradient_type: GradientType::Linear,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            matrix: swf_to_gl_matrix(gradient.matrix),
                            repeat_mode: gradient.spread,
                            focal_point: 0.0,
                            interpolation: gradient.interpolation,
                        };

                        flush_draw(DrawType::Gradient(gradient), &mut mesh, &mut lyon_mesh);
                    }
                    FillStyle::RadialGradient(gradient) => {
                        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh);

                        let mut buffers_builder = BuffersBuilder::new(
                            &mut lyon_mesh,
                            RuffleVertexCtor { color: 0xffff_ffff },
                        );

                        if let Err(e) = self.fill_tess.tessellate_path(
                            &ruffle_path_to_lyon_path(commands, true),
                            &FillOptions::even_odd(),
                            &mut buffers_builder,
                        ) {
                            // This may just be a degenerate path; skip it.
                            log::error!("Tessellation failure: {:?}", e);
                            continue;
                        }

                        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(8);
                        let mut ratios: Vec<f32> = Vec::with_capacity(8);
                        for record in &gradient.records {
                            colors.push([
                                f32::from(record.color.r) / 255.0,
                                f32::from(record.color.g) / 255.0,
                                f32::from(record.color.b) / 255.0,
                                f32::from(record.color.a) / 255.0,
                            ]);
                            ratios.push(f32::from(record.ratio) / 255.0);
                        }

                        let gradient = Gradient {
                            gradient_type: GradientType::Radial,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            matrix: swf_to_gl_matrix(gradient.matrix),
                            repeat_mode: gradient.spread,
                            focal_point: 0.0,
                            interpolation: gradient.interpolation,
                        };

                        flush_draw(DrawType::Gradient(gradient), &mut mesh, &mut lyon_mesh);
                    }
                    FillStyle::FocalGradient {
                        gradient,
                        focal_point,
                    } => {
                        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh);

                        let mut buffers_builder = BuffersBuilder::new(
                            &mut lyon_mesh,
                            RuffleVertexCtor { color: 0xffff_ffff },
                        );

                        if let Err(e) = self.fill_tess.tessellate_path(
                            &ruffle_path_to_lyon_path(commands, true),
                            &FillOptions::even_odd(),
                            &mut buffers_builder,
                        ) {
                            // This may just be a degenerate path; skip it.
                            log::error!("Tessellation failure: {:?}", e);
                            continue;
                        }

                        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(8);
                        let mut ratios: Vec<f32> = Vec::with_capacity(8);
                        for record in &gradient.records {
                            colors.push([
                                f32::from(record.color.r) / 255.0,
                                f32::from(record.color.g) / 255.0,
                                f32::from(record.color.b) / 255.0,
                                f32::from(record.color.a) / 255.0,
                            ]);
                            ratios.push(f32::from(record.ratio) / 255.0);
                        }

                        let gradient = Gradient {
                            gradient_type: GradientType::Focal,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            matrix: swf_to_gl_matrix(gradient.matrix),
                            repeat_mode: gradient.spread,
                            focal_point: *focal_point,
                            interpolation: gradient.interpolation,
                        };

                        flush_draw(DrawType::Gradient(gradient), &mut mesh, &mut lyon_mesh);
                    }
                    FillStyle::Bitmap {
                        id,
                        matrix,
                        is_smoothed,
                        is_repeating,
                    } => {
                        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh);

                        let mut buffers_builder = BuffersBuilder::new(
                            &mut lyon_mesh,
                            RuffleVertexCtor { color: 0xffff_ffff },
                        );

                        if let Err(e) = self.fill_tess.tessellate_path(
                            &ruffle_path_to_lyon_path(commands, true),
                            &FillOptions::even_odd(),
                            &mut buffers_builder,
                        ) {
                            // This may just be a degenerate path; skip it.
                            log::error!("Tessellation failure: {:?}", e);
                            continue;
                        }

                        if let Some((bitmap_width, bitmap_height, bitmap)) = get_bitmap(*id) {
                            let bitmap = Bitmap {
                                matrix: swf_bitmap_to_gl_matrix(
                                    *matrix,
                                    bitmap_width,
                                    bitmap_height,
                                ),
                                bitmap,
                                is_smoothed: *is_smoothed,
                                is_repeating: *is_repeating,
                            };

                            flush_draw(DrawType::Bitmap(bitmap), &mut mesh, &mut lyon_mesh);
                        }
                    }
                },
                DrawPath::Stroke {
                    style,
                    commands,
                    is_closed,
                } => {
                    let color = ((style.color.a as u32) << 24)
                        | ((style.color.b as u32) << 16)
                        | ((style.color.g as u32) << 8)
                        | (style.color.r as u32);

                    let mut buffers_builder =
                        BuffersBuilder::new(&mut lyon_mesh, RuffleVertexCtor { color });

                    // TODO(Herschel): 0 width indicates "hairline".
                    let width = if style.width.to_pixels() >= 1.0 {
                        style.width.to_pixels() as f32
                    } else {
                        1.0
                    };

                    let mut options = StrokeOptions::default()
                        .with_line_width(width)
                        .with_start_cap(match style.start_cap {
                            swf::LineCapStyle::None => tessellation::LineCap::Butt,
                            swf::LineCapStyle::Round => tessellation::LineCap::Round,
                            swf::LineCapStyle::Square => tessellation::LineCap::Square,
                        })
                        .with_end_cap(match style.end_cap {
                            swf::LineCapStyle::None => tessellation::LineCap::Butt,
                            swf::LineCapStyle::Round => tessellation::LineCap::Round,
                            swf::LineCapStyle::Square => tessellation::LineCap::Square,
                        });

                    let line_join = match style.join_style {
                        swf::LineJoinStyle::Round => tessellation::LineJoin::Round,
                        swf::LineJoinStyle::Bevel => tessellation::LineJoin::Bevel,
                        swf::LineJoinStyle::Miter(limit) => {
                            // Avoid lyon assert with small miter limits.
                            if limit >= StrokeOptions::MINIMUM_MITER_LIMIT {
                                options = options.with_miter_limit(limit);
                                tessellation::LineJoin::MiterClip
                            } else {
                                tessellation::LineJoin::Bevel
                            }
                        }
                    };
                    options = options.with_line_join(line_join);

                    if let Err(e) = self.stroke_tess.tessellate_path(
                        &ruffle_path_to_lyon_path(commands, is_closed),
                        &options,
                        &mut buffers_builder,
                    ) {
                        // This may just be a degenerate path; skip it.
                        log::error!("Tessellation failure: {:?}", e);
                        continue;
                    }
                }
            }
        }

        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh);

        mesh
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
}

pub enum DrawType {
    Color,
    Gradient(Gradient),
    Bitmap(Bitmap),
}

#[derive(Clone, Debug)]
pub struct Gradient {
    pub matrix: [[f32; 3]; 3],
    pub gradient_type: GradientType,
    pub ratios: Vec<f32>,
    pub colors: Vec<[f32; 4]>,
    pub num_colors: u32,
    pub repeat_mode: GradientSpread,
    pub focal_point: f32,
    pub interpolation: GradientInterpolation,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: u32,
}

#[derive(Clone, Debug)]
pub struct Bitmap {
    pub matrix: [[f32; 3]; 3],
    pub bitmap: BitmapHandle,
    pub is_smoothed: bool,
    pub is_repeating: bool,
}

#[allow(clippy::many_single_char_names)]
fn swf_to_gl_matrix(m: swf::Matrix) -> [[f32; 3]; 3] {
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
fn swf_bitmap_to_gl_matrix(m: swf::Matrix, bitmap_width: u32, bitmap_height: u32) -> [[f32; 3]; 3] {
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

fn ruffle_path_to_lyon_path(commands: Vec<DrawCommand>, is_closed: bool) -> Path {
    fn point(x: Twips, y: Twips) -> lyon::math::Point {
        lyon::math::Point::new(x.to_pixels() as f32, y.to_pixels() as f32)
    }

    let mut builder = Path::builder();
    let mut move_to = Some((Twips::default(), Twips::default()));
    for cmd in commands {
        match cmd {
            DrawCommand::MoveTo { x, y } => {
                if move_to.is_none() {
                    builder.end(false);
                }
                move_to = Some((x, y));
            }
            DrawCommand::LineTo { x, y } => {
                if let Some((x, y)) = move_to.take() {
                    builder.begin(point(x, y));
                }
                builder.line_to(point(x, y));
            }
            DrawCommand::CurveTo { x1, y1, x2, y2 } => {
                if let Some((x, y)) = move_to.take() {
                    builder.begin(point(x, y));
                }
                builder.quadratic_bezier_to(point(x1, y1), point(x2, y2));
            }
        }
    }

    if move_to.is_none() {
        if is_closed {
            builder.close();
        } else {
            builder.end(false);
        }
    }

    builder.build()
}

struct RuffleVertexCtor {
    color: u32,
}

impl FillVertexConstructor<Vertex> for RuffleVertexCtor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y],
            color: self.color,
        }
    }
}

impl StrokeVertexConstructor<Vertex> for RuffleVertexCtor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        Vertex {
            position: [vertex.position().x, vertex.position().y],
            color: self.color,
        }
    }
}

pub use swf::GradientSpread;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GradientType {
    Linear,
    Radial,
    Focal,
}
