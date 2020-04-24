use lyon::path::Path;
use lyon::tessellation::{
    self,
    geometry_builder::{BuffersBuilder, FillVertexConstructor, VertexBuffers},
    FillAttributes, FillTessellator, StrokeAttributes, StrokeTessellator, StrokeVertexConstructor,
};
use lyon::tessellation::{FillOptions, StrokeOptions};
use ruffle_core::backend::render::swf::{self, FillStyle, Twips};
use ruffle_core::shape_utils::{DrawCommand, DrawPath};

pub fn tessellate_shape<F>(shape: &swf::Shape, get_bitmap_dimenions: F) -> Mesh
where
    F: Fn(swf::CharacterId) -> Option<(u32, u32)>,
{
    let paths = ruffle_core::shape_utils::swf_shape_to_paths(shape);
    let mut mesh = Vec::new();

    let mut fill_tess = FillTessellator::new();
    let mut stroke_tess = StrokeTessellator::new();
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

    for path in paths {
        match path {
            DrawPath::Fill { style, commands } => match style {
                FillStyle::Color(color) => {
                    let color = [
                        f32::from(color.r) / 255.0,
                        f32::from(color.g) / 255.0,
                        f32::from(color.b) / 255.0,
                        f32::from(color.a) / 255.0,
                    ];

                    let mut buffers_builder =
                        BuffersBuilder::new(&mut lyon_mesh, RuffleVertexCtor { color });

                    if let Err(e) = fill_tess.tessellate_path(
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
                        RuffleVertexCtor {
                            color: [1.0, 1.0, 1.0, 1.0],
                        },
                    );

                    if let Err(e) = fill_tess.tessellate_path(
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
                        gradient_type: 0,
                        ratios,
                        colors,
                        num_colors: gradient.records.len() as u32,
                        matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                        repeat_mode: 0,
                        focal_point: 0.0,
                    };

                    flush_draw(DrawType::Gradient(gradient), &mut mesh, &mut lyon_mesh);
                }
                FillStyle::RadialGradient(gradient) => {
                    flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh);

                    let mut buffers_builder = BuffersBuilder::new(
                        &mut lyon_mesh,
                        RuffleVertexCtor {
                            color: [1.0, 1.0, 1.0, 1.0],
                        },
                    );

                    if let Err(e) = fill_tess.tessellate_path(
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
                        gradient_type: 1,
                        ratios,
                        colors,
                        num_colors: gradient.records.len() as u32,
                        matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                        repeat_mode: 0,
                        focal_point: 0.0,
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
                        RuffleVertexCtor {
                            color: [1.0, 1.0, 1.0, 1.0],
                        },
                    );

                    if let Err(e) = fill_tess.tessellate_path(
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
                        gradient_type: 1,
                        ratios,
                        colors,
                        num_colors: gradient.records.len() as u32,
                        matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                        repeat_mode: 0,
                        focal_point: *focal_point,
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
                        RuffleVertexCtor {
                            color: [1.0, 1.0, 1.0, 1.0],
                        },
                    );

                    if let Err(e) = fill_tess.tessellate_path(
                        &ruffle_path_to_lyon_path(commands, true),
                        &FillOptions::even_odd(),
                        &mut buffers_builder,
                    ) {
                        // This may just be a degenerate path; skip it.
                        log::error!("Tessellation failure: {:?}", e);
                        continue;
                    }

                    let (bitmap_width, bitmap_height) = get_bitmap_dimenions(*id).unwrap_or((1, 1));

                    let bitmap = Bitmap {
                        matrix: swf_bitmap_to_gl_matrix(
                            matrix.clone(),
                            bitmap_width,
                            bitmap_height,
                        ),
                        id: *id,
                        is_smoothed: *is_smoothed,
                        is_repeating: *is_repeating,
                    };

                    flush_draw(DrawType::Bitmap(bitmap), &mut mesh, &mut lyon_mesh);
                }
            },
            DrawPath::Stroke {
                style,
                commands,
                is_closed,
            } => {
                let color = [
                    f32::from(style.color.r) / 255.0,
                    f32::from(style.color.g) / 255.0,
                    f32::from(style.color.b) / 255.0,
                    f32::from(style.color.a) / 255.0,
                ];

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
                    .with_line_join(match style.join_style {
                        swf::LineJoinStyle::Round => tessellation::LineJoin::Round,
                        swf::LineJoinStyle::Bevel => tessellation::LineJoin::Bevel,
                        swf::LineJoinStyle::Miter(_) => tessellation::LineJoin::MiterClip,
                    })
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

                if let swf::LineJoinStyle::Miter(limit) = style.join_style {
                    options = options.with_miter_limit(limit);
                }

                if let Err(e) = stroke_tess.tessellate_path(
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
    pub gradient_type: i32,
    pub ratios: Vec<f32>,
    pub colors: Vec<[f32; 4]>,
    pub num_colors: u32,
    pub repeat_mode: i32,
    pub focal_point: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Bitmap {
    pub matrix: [[f32; 3]; 3],
    pub id: swf::CharacterId,
    pub is_smoothed: bool,
    pub is_repeating: bool,
}

#[allow(clippy::many_single_char_names)]
fn swf_to_gl_matrix(m: swf::Matrix) -> [[f32; 3]; 3] {
    let tx = m.translate_x.get() as f32;
    let ty = m.translate_y.get() as f32;
    let det = m.scale_x * m.scale_y - m.rotate_skew_1 * m.rotate_skew_0;
    let mut a = m.scale_y / det;
    let mut b = -m.rotate_skew_1 / det;
    let mut c = -(tx * m.scale_y - m.rotate_skew_1 * ty) / det;
    let mut d = -m.rotate_skew_0 / det;
    let mut e = m.scale_x / det;
    let mut f = (tx * m.rotate_skew_0 - m.scale_x * ty) / det;

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

    let tx = m.translate_x.get() as f32;
    let ty = m.translate_y.get() as f32;
    let det = m.scale_x * m.scale_y - m.rotate_skew_1 * m.rotate_skew_0;
    let mut a = m.scale_y / det;
    let mut b = -m.rotate_skew_1 / det;
    let mut c = -(tx * m.scale_y - m.rotate_skew_1 * ty) / det;
    let mut d = -m.rotate_skew_0 / det;
    let mut e = m.scale_x / det;
    let mut f = (tx * m.rotate_skew_0 - m.scale_x * ty) / det;

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
    for cmd in commands {
        match cmd {
            DrawCommand::MoveTo { x, y } => {
                builder.move_to(point(x, y));
            }
            DrawCommand::LineTo { x, y } => {
                builder.line_to(point(x, y));
            }
            DrawCommand::CurveTo { x1, y1, x2, y2 } => {
                builder.quadratic_bezier_to(point(x1, y1), point(x2, y2));
            }
        }
    }

    if is_closed {
        builder.close();
    }

    builder.build()
}

struct RuffleVertexCtor {
    color: [f32; 4],
}

impl FillVertexConstructor<Vertex> for RuffleVertexCtor {
    fn new_vertex(&mut self, position: lyon::math::Point, _: FillAttributes) -> Vertex {
        Vertex {
            position: [position.x, position.y],
            color: self.color,
        }
    }
}

impl StrokeVertexConstructor<Vertex> for RuffleVertexCtor {
    fn new_vertex(&mut self, position: lyon::math::Point, _: StrokeAttributes) -> Vertex {
        Vertex {
            position: [position.x, position.y],
            color: self.color,
        }
    }
}
