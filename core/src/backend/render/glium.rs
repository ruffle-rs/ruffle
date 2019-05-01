use super::{common::ShapeHandle, RenderBackend};
use crate::{matrix::Matrix, Color};
use glium::{implement_vertex, uniform, Display, Frame, Surface};
use glutin::{Window, WindowedContext};
use lyon::tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon::{path::PathEvent, tessellation, tessellation::FillTessellator};
use std::collections::{HashMap, VecDeque};
use swf::{FillStyle, LineStyle};

pub struct GliumRenderBackend {
    display: Display,
    target: Option<Frame>,
    shader_program: glium::Program,
    meshes: Vec<Mesh>,
    movie_width: f32,
    movie_height: f32,
}

impl GliumRenderBackend {
    pub fn new(
        windowed_context: WindowedContext,
    ) -> Result<GliumRenderBackend, Box<std::error::Error>> {
        let display = Display::from_gl_window(windowed_context)?;

        let shader_program =
            glium::Program::from_source(&display, VERTEX_SHADER, FRAGMENT_SHADER, None)?;

        Ok(GliumRenderBackend {
            display,
            shader_program,
            target: None,
            meshes: vec![],
            movie_width: 500.0,
            movie_height: 500.0,
        })
    }

    pub fn display(&self) -> &Display {
        &self.display
    }
}

impl RenderBackend for GliumRenderBackend {
    fn set_dimensions(&mut self, width: u32, height: u32) {
        self.movie_width = width as f32;
        self.movie_height = height as f32;
    }

    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle {
        let handle = ShapeHandle(self.meshes.len());

        use lyon::tessellation::FillOptions;

        let mut mesh = Mesh { draws: vec![] };

        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];

        let mut lyon_mesh: VertexBuffers<_, u32> = VertexBuffers::new();
        let paths = swf_shape_to_lyon_paths(shape);
        let mut fill_tess = FillTessellator::new();

        let mut num_verts = 0;
        let mut num_indices = 0;

        for (cmd, path) in paths {
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

        let vertex_buffer =
            glium::VertexBuffer::new(&self.display, &lyon_mesh.vertices[..]).unwrap();
        let index_buffer = glium::IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            &lyon_mesh.indices[..],
        )
        .unwrap();

        mesh.draws.push(Draw {
            draw_type: DrawType::Color,
            vertex_buffer,
            index_buffer,
        });

        // // Gradient
        // for (cmd, path) in paths {
        //     if let PathCommandType::Stroke(_) = cmd {
        //         continue;
        //     }
        //     let matrix = match cmd {
        //         PathCommandType::Fill(FillStyle::LinearGradient(gradient)) => {
        //             let mut m = gradient.matrix.clone();
        //             let tx = m.translate_x * 20.0;
        //             let ty = m.translate_y * 20.0;
        //             let det = m.scale_x * m.scale_y - m.rotate_skew_1 * m.rotate_skew_0;
        //             let mut a = m.scale_y / det;
        //             let mut b = -m.rotate_skew_1 / det;
        //             let mut c = -(tx * m.scale_y - m.rotate_skew_1 * ty) / det;
        //             let mut d = -m.rotate_skew_0 / det;
        //             let mut e = m.scale_x / det;
        //             let mut f = (tx * m.rotate_skew_0 - m.scale_x * ty) / det;

        //             a *= 20.0 / 32768.0;
        //             b *= 20.0 / 32768.0;
        //             d *= 20.0 / 32768.0;
        //             e *= 20.0 / 32768.0;

        //             c /= 32768.0;
        //             f /= 32768.0;
        //             c += 0.5;
        //             f += 0.5;
        //             [a, b, c, d, e, f]
        //         }
        //         PathCommandType::Fill(_) => continue,
        //         PathCommandType::Stroke(_) => continue,
        //     };

        //     let vertex_ctor = move |vertex: tessellation::FillVertex| Vertex {
        //         position: [vertex.position.x, vertex.position.y],
        //         color: [u, v],
        //     };

        //     let mut buffers_builder = BuffersBuilder::new(&mut lyon_mesh, vertex_ctor);
        //     fill_tess
        //         .tessellate_path(
        //             path.into_iter(),
        //             &FillOptions::even_odd(),
        //             &mut buffers_builder,
        //         )
        //         .expect("Tessellation error");

        //     let vert_offset = vertices.len() as u32;
        //     vertices.extend(lyon_mesh.vertices.iter());
        //     indices.extend(lyon_mesh.indices.iter().map(|&n| n + vert_offset));
        // }

        // let vertex_buffer =
        //     glium::VertexBuffer::new(&self.display, &lyon_mesh.vertices[..]).unwrap();
        // let index_buffer = glium::IndexBuffer::new(
        //     &self.display,
        //     glium::index::PrimitiveType::TrianglesList,
        //     &lyon_mesh.indices[..],
        // )
        // .unwrap();

        // mesh.draws.push(Draw {
        //     draw_type: DrawType::Color,
        //     vertex_buffer,
        //     index_buffer,
        // });

        self.meshes.push(mesh);

        handle
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
        target.clear_color(
            f32::from(color.r) / 255.0,
            f32::from(color.g) / 255.0,
            f32::from(color.b) / 255.0,
            f32::from(color.a) / 255.0,
        );
    }

    fn render_shape(&mut self, shape: ShapeHandle, matrix: &Matrix) {
        let target = self.target.as_mut().unwrap();

        let mesh = &self.meshes[shape.0];

        let view_matrix = [
            [1.0 / (self.movie_width as f32 / 2.0), 0.0, 0.0, 0.0],
            [0.0, -1.0 / (self.movie_height as f32 / 2.0), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let world_matrix = [
            [matrix.a, matrix.b, 0.0, 0.0],
            [matrix.b, matrix.d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [matrix.tx, matrix.ty, 0.0, 1.0],
        ];

        for draw in &mesh.draws {
            match draw.draw_type {
                DrawType::Color => {
                    target
                        .draw(
                            &draw.vertex_buffer,
                            &draw.index_buffer,
                            &self.shader_program,
                            &uniform! { view_matrix: view_matrix, world_matrix: world_matrix },
                            &Default::default(),
                        )
                        .unwrap();
                }
                _ => (),
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

const VERTEX_SHADER: &str = r#"
#version 140

uniform mat4 view_matrix;
uniform mat4 world_matrix;

in vec2 position;
in vec4 color;
out vec4 frag_color;

void main() {
    frag_color = color;
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
    LinearGradient,
}

fn point(x: f32, y: f32) -> lyon::math::Point {
    lyon::math::Point::new(x, y)
}

fn swf_shape_to_lyon_paths(
    shape: &swf::Shape,
) -> Vec<(PathCommandType, Vec<lyon::path::PathEvent>)> {
    let cmds = get_paths(shape);
    let mut out_paths = vec![];
    let mut prev;
    use lyon::geom::{LineSegment, QuadraticBezierSegment};
    for cmd in cmds {
        if let PathCommandType::Fill(_fill_style) = &cmd.command_type {
            let mut out_path = vec![PathEvent::MoveTo(point(cmd.path.start.0, cmd.path.start.1))];
            prev = point(cmd.path.start.0, cmd.path.start.1);
            for edge in cmd.path.edges {
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
            out_path.push(PathEvent::Close(LineSegment {
                from: prev,
                to: prev,
            }));
            out_paths.push((cmd.command_type, out_path));
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
                    for (_id, paths) in paths {
                        for path in paths.open_paths {
                            out.push(PathCommand {
                                command_type: paths.command_type.clone(),
                                path,
                            })
                        }
                    }
                    for (id, paths) in strokes {
                        for path in paths.open_paths {
                            out.push(PathCommand {
                                command_type: paths.command_type.clone(),
                                path,
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
        for path in paths.open_paths {
            out.push(PathCommand {
                command_type: paths.command_type.clone(),
                path,
            })
        }
    }
    for (id, paths) in strokes {
        for path in paths.open_paths {
            out.push(PathCommand {
                command_type: paths.command_type.clone(),
                path,
            })
        }
    }
    out.into_iter()
}

pub struct PathCommand {
    command_type: PathCommandType,
    path: Path,
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

struct Path {
    start: (f32, f32),
    end: (f32, f32),

    edges: VecDeque<PathEdge>,
}

#[derive(Copy, Clone)]
enum PathEdge {
    Straight(f32, f32),
    Bezier(f32, f32, f32, f32),
}
