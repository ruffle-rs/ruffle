#![allow(clippy::invalid_ref)]

use fluster_core::backend::render::swf::{self, FillStyle, LineStyle};
use fluster_core::backend::render::{BitmapHandle, Color, RenderBackend, ShapeHandle, Transform};
use glium::texture::texture2d::Texture2d;
use glium::uniforms::{UniformValue, Uniforms};
use glium::{
    draw_parameters::DrawParameters, implement_uniform_block, implement_vertex, uniform, Display,
    Frame, Surface,
};
use glutin::WindowedContext;
use lyon::tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon::{path::PathEvent, tessellation, tessellation::FillTessellator};
use std::collections::{HashMap, VecDeque};

pub struct GliumRenderBackend {
    display: Display,
    target: Option<Frame>,
    shader_program: glium::Program,
    gradient_shader_program: glium::Program,
    meshes: Vec<Mesh>,
    textures: Vec<Texture2d>,
    movie_width: f32,
    movie_height: f32,
}

impl GliumRenderBackend {
    pub fn new(
        windowed_context: WindowedContext,
    ) -> Result<GliumRenderBackend, Box<std::error::Error>> {
        let display = Display::from_gl_window(windowed_context)?;

        use glium::program::ProgramCreationInput;
        let shader_program = glium::Program::new(
            &display,
            ProgramCreationInput::SourceCode {
                vertex_shader: VERTEX_SHADER,
                fragment_shader: FRAGMENT_SHADER,
                geometry_shader: None,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        )?;

        let gradient_shader_program = glium::Program::new(
            &display,
            ProgramCreationInput::SourceCode {
                vertex_shader: TEXTURE_VERTEX_SHADER,
                fragment_shader: GRADIENT_FRAGMENT_SHADER,
                geometry_shader: None,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        )?;

        Ok(GliumRenderBackend {
            display,
            shader_program,
            gradient_shader_program,
            target: None,
            meshes: vec![],
            textures: vec![],
            movie_width: 500.0,
            movie_height: 500.0,
        })
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    fn register_shape_internal(&mut self, shape: &swf::Shape) -> ShapeHandle {
        let handle = ShapeHandle(self.meshes.len());

        use lyon::tessellation::FillOptions;

        let mut mesh = Mesh { draws: vec![] };

        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];

        let paths = swf_shape_to_lyon_paths(shape);
        let mut fill_tess = FillTessellator::new();

        let mut lyon_mesh: VertexBuffers<_, u32> = VertexBuffers::new();
        for (cmd, path) in paths.clone() {
            let color = match cmd {
                PathCommandType::Fill(FillStyle::Color(color)) => [
                    f32::from(color.r) / 255.0,
                    f32::from(color.g) / 255.0,
                    f32::from(color.b) / 255.0,
                    f32::from(color.a) / 255.0,
                ],
                PathCommandType::Fill(_) => continue,
                PathCommandType::Stroke(_) => continue,
            };
            let vertex_ctor = move |vertex: tessellation::FillVertex| Vertex {
                position: [vertex.position.x, vertex.position.y],
                color,
            };

            let mut buffers_builder = BuffersBuilder::new(&mut lyon_mesh, vertex_ctor);
            fill_tess
                .tessellate_path(
                    path.into_iter(),
                    &FillOptions::even_odd(),
                    &mut buffers_builder,
                )
                .expect("Tessellation error");

            let vert_offset = vertices.len() as u32;
            vertices.extend(lyon_mesh.vertices.iter());
            indices.extend(lyon_mesh.indices.iter().map(|&n| n + vert_offset));
        }

        let vertex_buffer = glium::VertexBuffer::new(&self.display, &vertices[..]).unwrap();
        let index_buffer = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            &indices[..],
        )
        .unwrap();

        mesh.draws.push(Draw {
            draw_type: DrawType::Color,
            vertex_buffer,
            index_buffer,
        });

        fn swf_to_gl_matrix(m: swf::Matrix) -> [[f32; 3]; 3] {
            let tx = m.translate_x * 20.0;
            let ty = m.translate_y * 20.0;
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

        for (cmd, path) in paths {
            let mut lyon_mesh: VertexBuffers<_, u32> = VertexBuffers::new();
            if let PathCommandType::Stroke(_) = cmd {
                continue;
            }
            let gradient_uniforms = match cmd {
                PathCommandType::Fill(FillStyle::LinearGradient(gradient)) => {
                    let mut colors = [[0.0; 4]; 8];
                    let mut ratios = [0.0; 8];
                    for (i, record) in gradient.records.iter().enumerate() {
                        colors[i] = [
                            record.color.r as f32 / 255.0,
                            record.color.g as f32 / 255.0,
                            record.color.b as f32 / 255.0,
                            record.color.a as f32 / 255.0,
                        ];
                        ratios[i] = record.ratio as f32 / 255.0;
                    }

                    GradientUniforms {
                        gradient_type: 0,
                        ratios,
                        colors,
                        num_colors: gradient.records.len() as u32,
                        matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                        repeat_mode: 0,
                        focal_point: 0.0,
                    }
                }
                PathCommandType::Fill(FillStyle::RadialGradient(gradient)) => {
                    let mut colors = [[0.0; 4]; 8];
                    let mut ratios = [0.0; 8];
                    for (i, record) in gradient.records.iter().enumerate() {
                        colors[i] = [
                            record.color.r as f32 / 255.0,
                            record.color.g as f32 / 255.0,
                            record.color.b as f32 / 255.0,
                            record.color.a as f32 / 255.0,
                        ];
                        ratios[i] = record.ratio as f32 / 255.0;
                    }

                    GradientUniforms {
                        gradient_type: 1,
                        ratios,
                        colors,
                        num_colors: gradient.records.len() as u32,
                        matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                        repeat_mode: 0,
                        focal_point: 0.0,
                    }
                }
                PathCommandType::Fill(FillStyle::FocalGradient {
                    gradient,
                    focal_point,
                }) => {
                    let mut colors = [[0.0; 4]; 8];
                    let mut ratios = [0.0; 8];
                    for (i, record) in gradient.records.iter().enumerate() {
                        colors[i] = [
                            record.color.r as f32 / 255.0,
                            record.color.g as f32 / 255.0,
                            record.color.b as f32 / 255.0,
                            record.color.a as f32 / 255.0,
                        ];
                        ratios[i] = record.ratio as f32 / 255.0;
                    }

                    GradientUniforms {
                        gradient_type: 2,
                        ratios,
                        colors,
                        num_colors: gradient.records.len() as u32,
                        matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                        repeat_mode: 0,
                        focal_point,
                    }
                }
                // PathCommandType::Fill(FillStyle::Bitmap {
                //     id,
                //     matrix,
                //     is_repeating,
                //     is_smoothed,
                // }) => {
                //     let mut colors = [[0.0; 4]; 8];
                //     let mut ratios = [0.0; 8];
                //     for (i, record) in gradient.records.iter().enumerate() {
                //         colors[i] = [
                //             record.color.r as f32 / 255.0,
                //             record.color.g as f32 / 255.0,
                //             record.color.b as f32 / 255.0,
                //             record.color.a as f32 / 255.0,
                //         ];
                //         ratios[i] = record.ratio as f32 / 255.0;
                //     }

                //     GradientUniforms {
                //         gradient_type: 0,
                //         ratios,
                //         colors,
                //         num_colors: gradient.records.len() as u32,
                //         matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                //         repeat_mode: 0,
                //         focal_point: 0.0,
                //     }
                // }
                PathCommandType::Fill(_) => continue,
                PathCommandType::Stroke(_) => continue,
            };

            let vertex_ctor = move |vertex: tessellation::FillVertex| Vertex {
                position: [vertex.position.x, vertex.position.y],
                color: [0.0, 0.0, 0.0, 0.0],
            };

            let mut buffers_builder = BuffersBuilder::new(&mut lyon_mesh, vertex_ctor);
            fill_tess
                .tessellate_path(
                    path.into_iter(),
                    &FillOptions::even_odd(),
                    &mut buffers_builder,
                )
                .expect("Tessellation error");

            let vertex_buffer =
                glium::VertexBuffer::new(&self.display, &lyon_mesh.vertices[..]).unwrap();
            let index_buffer = glium::IndexBuffer::new(
                &self.display,
                glium::index::PrimitiveType::TrianglesList,
                &lyon_mesh.indices[..],
            )
            .unwrap();

            mesh.draws.push(Draw {
                draw_type: DrawType::LinearGradient(gradient_uniforms),
                vertex_buffer,
                index_buffer,
            });
        }

        self.meshes.push(mesh);

        handle
    }
}

impl RenderBackend for GliumRenderBackend {
    fn set_dimensions(&mut self, width: u32, height: u32) {
        self.movie_width = width as f32;
        self.movie_height = height as f32;
    }

    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle {
        self.register_shape_internal(shape)
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let shape = swf::Shape {
            version: 2,
            id: 0,
            shape_bounds: swf::Rectangle {
                x_min: 0.0,
                x_max: 0.0,
                y_min: 0.0,
                y_max: 0.0,
            },
            edge_bounds: swf::Rectangle {
                x_min: 0.0,
                x_max: 0.0,
                y_min: 0.0,
                y_max: 0.0,
            },
            has_fill_winding_rule: false,
            has_non_scaling_strokes: false,
            has_scaling_strokes: true,
            styles: swf::ShapeStyles {
                fill_styles: vec![FillStyle::Color(Color {
                    r: 255,
                    g: 25,
                    b: 255,
                    a: 255,
                })],
                line_styles: vec![],
            },
            shape: glyph.shape_records.clone(),
        };
        self.register_shape_internal(&shape)
    }

    fn register_bitmap_jpeg(
        &mut self,
        _id: swf::CharacterId,
        mut data: &[u8],
        mut jpeg_tables: &[u8],
    ) -> BitmapHandle {
        // SWF19 p.138:
        // "Before version 8 of the SWF file format, SWF files could contain an erroneous header of 0xFF, 0xD9, 0xFF, 0xD8 before the JPEG SOI marker."
        // Slice off these bytes if necessary.`
        if data[0..4] == [0xFF, 0xD9, 0xFF, 0xD8] {
            data = &data[4..];
        }

        if jpeg_tables[0..4] == [0xFF, 0xD9, 0xFF, 0xD8] {
            jpeg_tables = &jpeg_tables[4..];
        }

        let full_jpeg = fluster_core::backend::render::glue_swf_jpeg_to_tables(jpeg_tables, data);
        let image = image::load(std::io::Cursor::new(&full_jpeg[..]), image::JPEG)
            .unwrap()
            .to_rgba();
        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(&self.display, image).unwrap();

        let handle = BitmapHandle(self.textures.len());
        self.textures.push(texture);
        handle
    }

    fn register_bitmap_jpeg_2(&mut self, id: swf::CharacterId, data: &[u8]) -> BitmapHandle {
        unimplemented!()
    }

    fn register_bitmap_png(&mut self, swf_tag: &swf::DefineBitsLossless) -> BitmapHandle {
        unimplemented!()
    }

    fn begin_frame(&mut self) {
        assert!(self.target.is_none());
        self.target = Some(self.display.draw());
    }

    fn end_frame(&mut self) {
        assert!(self.target.is_some());
        let target = self.target.take().unwrap();
        target.finish().unwrap();
    }

    fn clear(&mut self, color: Color) {
        let target = self.target.as_mut().unwrap();
        target.clear_color_srgb(
            f32::from(color.r) / 255.0,
            f32::from(color.g) / 255.0,
            f32::from(color.b) / 255.0,
            f32::from(color.a) / 255.0,
        );
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        let target = self.target.as_mut().unwrap();

        let mesh = &self.meshes[shape.0];

        let view_matrix = [
            [1.0 / (self.movie_width as f32 / 2.0), 0.0, 0.0, 0.0],
            [0.0, -1.0 / (self.movie_height as f32 / 2.0), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let world_matrix = [
            [transform.matrix.a, transform.matrix.b, 0.0, 0.0],
            [transform.matrix.c, transform.matrix.d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [transform.matrix.tx, transform.matrix.ty, 0.0, 1.0],
        ];

        let mult_color = [
            transform.color_transform.r_mult,
            transform.color_transform.g_mult,
            transform.color_transform.b_mult,
            transform.color_transform.a_mult,
        ];

        let add_color = [
            transform.color_transform.r_add,
            transform.color_transform.g_add,
            transform.color_transform.b_add,
            transform.color_transform.a_add,
        ];

        let draw_parameters = DrawParameters {
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };

        for draw in &mesh.draws {
            match draw.draw_type {
                DrawType::Color => {
                    target
                        .draw(
                            &draw.vertex_buffer,
                            &draw.index_buffer,
                            &self.shader_program,
                            &uniform! { view_matrix: view_matrix, world_matrix: world_matrix, mult_color: mult_color, add_color: add_color },
                            &draw_parameters
                        )
                        .unwrap();
                }
                DrawType::LinearGradient(gradient_uniforms) => {
                    let uniforms = AllUniforms {
                        view_matrix,
                        world_matrix,
                        mult_color,
                        add_color,
                        gradient: gradient_uniforms,
                    };

                    target
                        .draw(
                            &draw.vertex_buffer,
                            &draw.index_buffer,
                            &self.gradient_shader_program,
                            &uniforms,
                            &draw_parameters,
                        )
                        .unwrap();
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

implement_vertex!(Vertex, position, color);

#[derive(Copy, Clone, Debug)]
struct GradientUniforms {
    matrix: [[f32; 3]; 3],
    gradient_type: i32,
    ratios: [f32; 8],
    colors: [[f32; 4]; 8],
    num_colors: u32,
    repeat_mode: i32,
    focal_point: f32,
}

impl Uniforms for GradientUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        visit("u_matrix", UniformValue::Mat3(self.matrix));
        visit(
            "u_gradient_type",
            UniformValue::SignedInt(self.gradient_type),
        );
        for i in 0..8 {
            visit(
                &format!("u_ratios[{}]", i)[..],
                UniformValue::Float(self.ratios[i]),
            );
            visit(
                &format!("u_colors[{}]", i)[..],
                UniformValue::Vec4(self.colors[i]),
            );
        }
        visit("u_num_colors", UniformValue::UnsignedInt(self.num_colors));
        visit("u_repeat_mode", UniformValue::SignedInt(self.repeat_mode));
        visit("u_focal_point", UniformValue::Float(self.focal_point));
    }
}

#[derive(Copy, Clone, Debug)]
struct AllUniforms {
    world_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    mult_color: [f32; 4],
    add_color: [f32; 4],
    gradient: GradientUniforms,
}

impl Uniforms for AllUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        visit("world_matrix", UniformValue::Mat4(self.world_matrix));
        visit("view_matrix", UniformValue::Mat4(self.view_matrix));
        visit("mult_color", UniformValue::Vec4(self.mult_color));
        visit("add_color", UniformValue::Vec4(self.add_color));
        self.gradient.visit_values(visit);
    }
}

const VERTEX_SHADER: &str = r#"
    #version 140

    uniform mat4 view_matrix;
    uniform mat4 world_matrix;
    uniform vec4 mult_color;
    uniform vec4 add_color;
    
    in vec2 position;
    in vec4 color;
    out vec4 frag_color;

    void main() {
        frag_color = color * mult_color + add_color;
        gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 140
    in vec4 frag_color;
    out vec4 out_color;
    void main() {
        out_color = frag_color;
    }
"#;

const TEXTURE_VERTEX_SHADER: &str = r#"
    #version 140

    uniform mat4 view_matrix;
    uniform mat4 world_matrix;

    uniform mat3 u_matrix;

    in vec2 position;
    in vec4 color;
    out vec2 frag_uv;

    void main() {
        frag_uv = vec2(u_matrix * vec3(position, 1.0));
        gl_Position = view_matrix * world_matrix * vec4(position, 0.0, 1.0);
    }
"#;

const GRADIENT_FRAGMENT_SHADER: &str = r#"
#version 140
    uniform vec4 mult_color;
    uniform vec4 add_color;

    uniform int u_gradient_type;
    uniform float u_ratios[8];
    uniform vec4 u_colors[8];
    uniform uint u_num_colors;
    uniform int u_repeat_mode;
    uniform float u_focal_point;

    in vec2 frag_uv;
    out vec4 out_color;

    void main() {
        vec4 color;
        int last = int(int(u_num_colors) - 1);
        float t;
        if( u_gradient_type == 0 )
        {
            t = frag_uv.x;
        }
        else if( u_gradient_type == 1 )
        {
            t = length(frag_uv * 2.0 - 1.0);
        }
        else if( u_gradient_type == 2 )
        {
            vec2 uv = frag_uv * 2.0 - 1.0;
            vec2 d = vec2(u_focal_point, 0.0) - uv;
            float l = length(d);
            d /= l;
            t = l / (sqrt(1.0 -  u_focal_point*u_focal_point*d.y*d.y) + u_focal_point*d.x);
        }
        if( u_repeat_mode == 0 )
        {
            // Clamp
            t = clamp(t, 0.0, 1.0);
        }
        else if( u_repeat_mode == 1 )
        {
            // Repeat
            t = fract(t);
        }
        else
        {
            // Mirror
            if( t < 0.0 )
            {
                t = -t;
            }
            if( (int(t)&1) == 0 ) {
                t = fract(t);
            } else {
                t = 1.0 - fract(t);
            }
        }
        int i = 0;
        int j = 1;
        while( t > u_ratios[j] )
        {
            i = j;
            j++;
        }
        float a = (t - u_ratios[i]) / (u_ratios[j] - u_ratios[i]);
        color = mix(u_colors[i], u_colors[j], a);            
        out_color = mult_color * color + add_color;
    }
"#;

struct Mesh {
    draws: Vec<Draw>,
}

struct Draw {
    draw_type: DrawType,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u32>,
}

enum DrawType {
    Color,
    LinearGradient(GradientUniforms),
}

fn point(x: f32, y: f32) -> lyon::math::Point {
    lyon::math::Point::new(x, y)
}

fn swf_shape_to_lyon_paths(
    shape: &swf::Shape,
) -> Vec<(PathCommandType, Vec<lyon::path::PathEvent>)> {
    let cmds = get_paths(shape);
    let mut out_paths = vec![];
    let mut prev = point(0.0, 0.0);
    use lyon::geom::{LineSegment, QuadraticBezierSegment};
    for cmd in cmds {
        if let PathCommandType::Fill(_fill_style) = &cmd.command_type {
            let mut out_path = vec![];
            for path in cmd.paths {
                out_path.push(PathEvent::MoveTo(point(path.start.0, path.start.1)));
                prev = point(path.start.0, path.start.1);
                for edge in path.edges {
                    let out_cmd = match edge {
                        PathEdge::Straight(x, y) => {
                            let cmd = PathEvent::Line(LineSegment {
                                from: prev,
                                to: point(x, y),
                            });
                            prev = point(x, y);
                            cmd
                        }
                        PathEdge::Bezier(x1, y1, x2, y2) => {
                            let cmd = PathEvent::Quadratic(QuadraticBezierSegment {
                                from: prev,
                                ctrl: point(x1, y1),
                                to: point(x2, y2),
                            });
                            prev = point(x2, y2);
                            cmd
                        }
                    };
                    out_path.push(out_cmd);
                }
            }

            out_path.push(PathEvent::Close(LineSegment {
                from: prev,
                to: prev,
            }));
            out_paths.push((cmd.command_type.clone(), out_path));
        }
    }
    out_paths
}

fn get_paths(shape: &swf::Shape) -> impl Iterator<Item = PathCommand> {
    let mut x = 0.0;
    let mut y = 0.0;

    let mut fill_styles = &shape.styles.fill_styles;
    let mut line_styles = &shape.styles.line_styles;
    let mut fill_style_0 = 0;
    let mut fill_style_1 = 0;
    let mut line_style = 0;

    let mut paths: HashMap<u32, PendingPaths> = HashMap::new();
    let mut strokes: HashMap<u32, PendingPaths> = HashMap::new();

    let mut out = vec![];

    for record in &shape.shape {
        use swf::ShapeRecord::*;
        match record {
            StyleChange(style_change) => {
                if let Some((move_x, move_y)) = style_change.move_to {
                    x = move_x;
                    y = move_y;
                }

                if let Some(i) = style_change.fill_style_0 {
                    fill_style_0 = i;
                }

                if let Some(i) = style_change.fill_style_1 {
                    fill_style_1 = i;
                }

                if let Some(i) = style_change.line_style {
                    line_style = i;
                }

                if let Some(ref new_styles) = style_change.new_styles {
                    for (id, paths) in paths {
                        let mut out_paths = vec![];
                        for path in paths.open_paths {
                            out_paths.push(path)
                        }
                        out.push(PathCommand {
                            command_type: paths.command_type.clone(),
                            paths: out_paths,
                        })
                    }
                    for (id, paths) in strokes {
                        for path in paths.open_paths {
                            out.push(PathCommand {
                                command_type: paths.command_type.clone(),
                                paths: vec![path],
                            })
                        }
                    }
                    paths = HashMap::new();
                    strokes = HashMap::new();
                    fill_styles = &new_styles.fill_styles;
                    line_styles = &new_styles.line_styles;
                }
            }

            StraightEdge { delta_x, delta_y } => {
                if fill_style_0 != 0 {
                    let path = paths.entry(fill_style_0).or_insert_with(|| {
                        PendingPaths::new(PathCommandType::Fill(
                            fill_styles[fill_style_0 as usize - 1].clone(),
                        ))
                    });
                    path.add_edge((x + delta_x, y + delta_y), PathEdge::Straight(x, y));
                }

                if fill_style_1 != 0 {
                    let path = paths.entry(fill_style_1).or_insert_with(|| {
                        PendingPaths::new(PathCommandType::Fill(
                            fill_styles[fill_style_1 as usize - 1].clone(),
                        ))
                    });
                    path.add_edge((x, y), PathEdge::Straight(x + delta_x, y + delta_y));
                }

                if line_style != 0 {
                    let path = strokes.entry(line_style).or_insert_with(|| {
                        PendingPaths::new(PathCommandType::Stroke(
                            line_styles[line_style as usize - 1].clone(),
                        ))
                    });
                    path.add_edge((x, y), PathEdge::Straight(x + delta_x, y + delta_y));
                }

                x += delta_x;
                y += delta_y;
            }

            CurvedEdge {
                control_delta_x,
                control_delta_y,
                anchor_delta_x,
                anchor_delta_y,
            } => {
                if fill_style_0 != 0 {
                    let path = paths.entry(fill_style_0).or_insert_with(|| {
                        PendingPaths::new(PathCommandType::Fill(
                            fill_styles[fill_style_0 as usize - 1].clone(),
                        ))
                    });
                    path.add_edge(
                        (
                            x + control_delta_x + anchor_delta_x,
                            y + control_delta_y + anchor_delta_y,
                        ),
                        PathEdge::Bezier(x + control_delta_x, y + control_delta_y, x, y),
                    );
                }

                if fill_style_1 != 0 {
                    let path = paths.entry(fill_style_1).or_insert_with(|| {
                        PendingPaths::new(PathCommandType::Fill(
                            fill_styles[fill_style_1 as usize - 1].clone(),
                        ))
                    });
                    path.add_edge(
                        (x, y),
                        PathEdge::Bezier(
                            x + control_delta_x,
                            y + control_delta_y,
                            x + control_delta_x + anchor_delta_x,
                            y + control_delta_y + anchor_delta_y,
                        ),
                    );
                }

                if line_style != 0 {
                    let path = strokes.entry(line_style).or_insert_with(|| {
                        PendingPaths::new(PathCommandType::Stroke(
                            line_styles[line_style as usize - 1].clone(),
                        ))
                    });
                    path.add_edge(
                        (x, y),
                        PathEdge::Bezier(
                            x + control_delta_x,
                            y + control_delta_y,
                            x + control_delta_x + anchor_delta_x,
                            y + control_delta_y + anchor_delta_y,
                        ),
                    );
                }

                x += control_delta_x + anchor_delta_x;
                y += control_delta_y + anchor_delta_y;
            }
        }
    }

    for (id, paths) in paths {
        let mut out_paths = vec![];
        for path in paths.open_paths {
            out_paths.push(path)
        }
        out.push(PathCommand {
            command_type: paths.command_type.clone(),
            paths: out_paths,
        })
    }
    for (id, paths) in strokes {
        for path in paths.open_paths {
            out.push(PathCommand {
                command_type: paths.command_type.clone(),
                paths: vec![path],
            })
        }
    }
    out.into_iter()
}

#[derive(Debug)]
pub struct PathCommand {
    command_type: PathCommandType,
    paths: Vec<Path>,
}

#[derive(Clone, Debug)]
enum PathCommandType {
    Fill(FillStyle),
    Stroke(LineStyle),
}

struct PendingPaths {
    command_type: PathCommandType,
    open_paths: Vec<Path>,
}

impl PendingPaths {
    fn new(command_type: PathCommandType) -> PendingPaths {
        Self {
            command_type,
            open_paths: vec![],
        }
    }

    fn add_edge(&mut self, start: (f32, f32), edge: PathEdge) {
        let new_path = Path {
            start,
            end: match edge {
                PathEdge::Straight(x, y) => (x, y),
                PathEdge::Bezier(_cx, _cy, ax, ay) => (ax, ay),
            },

            edges: {
                let mut edges = VecDeque::new();
                edges.push_back(edge);
                edges
            },
        };

        self.merge_subpath(new_path);
    }

    fn merge_subpath(&mut self, mut path: Path) {
        fn approx_eq(a: (f32, f32), b: (f32, f32)) -> bool {
            let dx = a.0 - b.0;
            let dy = a.1 - b.1;
            const EPSILON: f32 = 0.0001;
            dx.abs() < EPSILON && dy.abs() < EPSILON
        }

        let mut path_index = None;
        for (i, other) in self.open_paths.iter_mut().enumerate() {
            if approx_eq(path.end, other.start) {
                other.start = path.start;
                for edge in path.edges.iter().rev() {
                    other.edges.push_front(*edge);
                }
                path_index = Some(i);
                break;
            }

            if approx_eq(other.end, path.start) {
                other.end = path.end;
                other.edges.append(&mut path.edges);

                path_index = Some(i);
                break;
            }
        }

        if let Some(i) = path_index {
            let path = self.open_paths.swap_remove(i);
            self.merge_subpath(path);
        } else {
            self.open_paths.push(path);
        }
    }
}

#[derive(Debug)]
struct Path {
    start: (f32, f32),
    end: (f32, f32),

    edges: VecDeque<PathEdge>,
}

#[derive(Copy, Clone, Debug)]
enum PathEdge {
    Straight(f32, f32),
    Bezier(f32, f32, f32, f32),
}
