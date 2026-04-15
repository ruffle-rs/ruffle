use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};

use citro3d::macros::include_shader;
use citro3d::math::{AspectRatio, ClipPlanes, FVec4, Matrix4, Projection, StereoDisplacement};
use citro3d::render::ClearFlags;
use citro3d::render::Frame;
use citro3d::render::Target as _;
use citro3d::shader::Library;
use citro3d::shader::Program;
use citro3d::texenv::{self, TexEnv};
use citro3d::texture::{self, ColorFormat, Face};
use citro3d::{
    attrib::{self, Permutation, Register},
    buffer::{self, Buffer},
    shader,
    texture::{Texture, TextureParameters},
    uniform,
};
use ctru::linear::LinearAllocator;
use ctru::prelude::Gfx;
use ctru::services::gfx::{RawFrameBuffer, Screen, TopScreen3D, TopScreenLeft};

use ruffle_core::{swf, Color};
use ruffle_render::backend::*;
use ruffle_render::bitmap::*;
use ruffle_render::commands::*;
use ruffle_render::error::Error as BitmapError;
use ruffle_render::matrix::Matrix;
use ruffle_render::quality::StageQuality;
use ruffle_render::shape_utils::DistilledShape;
use ruffle_render::tessellator::DrawType as TessDrawType;
use ruffle_render::tessellator::{ShapeTessellator, Vertex as TessVertex};
use ruffle_render::transform::Transform;

const TOP_WIDTH: u32 = 400;
const TOP_HEIGHT: u32 = 240;

const DIMENSIONS: ViewportDimensions = ViewportDimensions {
    width: TOP_WIDTH,
    height: TOP_HEIGHT,
    scale_factor: 1.0,
};

#[derive(Clone)]
struct Draw {
    draw_type: DrawType,
    vertices: buffer::Info,
    indices: Vec<u16, LinearAllocator>,
}

impl std::fmt::Debug for Draw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Draw")
            .field("draw_type", &self.draw_type)
            .field("vertices", &format!("[len {}]", self.vertices.len()))
            .field("indices", &self.indices)
            .finish()
    }
}

#[derive(Clone, Debug)]
enum DrawType {
    Color,
    Gradient(Box<Gradient>),
    Bitmap(BitmapDraw),
}

#[derive(Clone, Debug)]
struct BitmapDraw {
    matrix: [[f32; 3]; 3],
    handle: Option<BitmapHandle>,
    is_repeating: bool,
    is_smoothed: bool,
}

const MAX_GRADIENT_COLORS: usize = 15;

#[derive(Clone, Debug)]
struct Gradient {
    matrix: [[f32; 3]; 3],
    gradient_type: i32,
    ratios: [f32; MAX_GRADIENT_COLORS],
    colors: [[f32; 4]; MAX_GRADIENT_COLORS],
    repeat_mode: i32,
    focal_point: f32,
    interpolation: swf::GradientInterpolation,
}

struct ShaderProgram {
    program: Program,
    library: Library,
    texenv: TexEnv,
    view_idx: uniform::Index,
    world_idx: uniform::Index,
    multclr_idx: uniform::Index,
    addclr_idx: uniform::Index,
}

impl ShaderProgram {
    pub fn new(program: Program, library: Library, texenv: TexEnv) -> Self {
        let view_idx = program.get_uniform("view").unwrap();
        let world_idx = program.get_uniform("world").unwrap();
        let multclr_idx = program.get_uniform("multclr").unwrap();
        let addclr_idx = program.get_uniform("addclr").unwrap();

        Self {
            program,
            library,
            texenv,
            view_idx,
            world_idx,
            multclr_idx,
            addclr_idx,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl From<TessVertex> for Vertex {
    fn from(vertex: TessVertex) -> Vertex {
        Self {
            position: [vertex.x, vertex.y],
            color: [
                vertex.color.r as f32,
                vertex.color.g as f32,
                vertex.color.b as f32,
                vertex.color.a as f32,
            ],
        }
    }
}

struct CitroTexture {
    texture: Texture,
}

impl Debug for CitroTexture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CitroTexture").finish_non_exhaustive()
    }
}

impl BitmapHandleImpl for CitroTexture {}

// fn as_texture(handle: &mut BitmapHandle) -> &mut CitroTexture {
//     <dyn BitmapHandleImpl>::downcast_mut(&mut *handle.0)
//         .expect("Bitmap handle must be CitroTexture")
// }

#[derive(Clone, Debug)]
struct Mesh {
    draws: Vec<Draw>,
}

impl ShapeHandleImpl for Mesh {}

fn as_mesh(handle: &ShapeHandle) -> &Mesh {
    <dyn ShapeHandleImpl>::downcast_ref(&*handle.0).expect("Shape handle must be citro3d mesh")
}

fn mat4(m: [[f32; 4]; 4]) -> Matrix4 {
    #[rustfmt::skip]
    let flat = [
        m[0][3], m[0][2], m[0][1], m[0][0],
        m[1][3], m[1][2], m[1][1], m[1][0],
        m[2][3], m[2][2], m[2][1], m[2][0],
        m[3][3], m[3][2], m[3][1], m[3][0],
    ];

    Matrix4::from_cells_wzyx(flat)
}

fn vec4(v: [f32; 4]) -> FVec4 {
    FVec4::new(v[0], v[1], v[2], v[3])
}

fn linear_vec<T>(vec: Vec<T>) -> Vec<T, LinearAllocator> {
    let mut linear_vec = Vec::with_capacity_in(vec.len(), LinearAllocator);
    for x in vec {
        linear_vec.push(x);
    }
    linear_vec
}

pub struct Citro3DRenderBackend {
    pub c3d: citro3d::Instance,
    pub inner: BackendInner,
}

pub struct BackendInner {
    gfx: Gfx,

    color_program: ShaderProgram,
    texture_program: ShaderProgram,

    color_quad_draw: Draw,
    bitmap_quad_draw: Draw,

    attr_info: attrib::Info,
    view_matrix: Matrix4,
    shape_tessellator: ShapeTessellator,
    //
    // meshes: Vec<Mesh>,
}

impl std::ops::Deref for Citro3DRenderBackend {
    type Target = BackendInner;

    fn deref(&self) -> &BackendInner {
        &self.inner
    }
}

impl std::ops::DerefMut for Citro3DRenderBackend {
    fn deref_mut(&mut self) -> &mut BackendInner {
        &mut self.inner
    }
}

impl Citro3DRenderBackend {
    pub fn new(gfx: Gfx) -> citro3d::error::Result<Self> {
        let c3d = citro3d::Instance::new().unwrap();

        let color_shader = shader::Library::from_bytes(include_shader!("../shaders/color.v.pica"))
            .expect("failed to load color shader");

        let color_prog = Program::new(color_shader.get(0).unwrap())
            .expect("failed to create color shader program");

        let color_stage0 = TexEnv::new()
            .src(texenv::Mode::BOTH, texenv::Source::PrimaryColor, None, None)
            .func(texenv::Mode::BOTH, texenv::CombineFunc::Replace);
        let color_program = ShaderProgram::new(color_prog, color_shader, color_stage0);

        let texture_shader =
            shader::Library::from_bytes(include_shader!("../shaders/texture.v.pica"))
                .expect("failed to load texture shader");
        let texture_prog = Program::new(texture_shader.get(0).unwrap())
            .expect("failed to create texture shader program")
            .into();
        let texture_stage0 =
            TexEnv::new().src(texenv::Mode::RGB, texenv::Source::Texture0, None, None);
        let texture_program = ShaderProgram::new(texture_prog, texture_shader, texture_stage0);

        // attributes for vertex buffer is always the same for both shaders, so we store this as a field
        let mut attr_info = attrib::Info::new();

        // v0 (position) = Float Vec2
        attr_info.add_loader(Register::V0, attrib::Format::Float, 2)?;
        // v1 (color) = Float Vec4
        attr_info.add_loader(Register::V1, attrib::Format::Float, 4)?;

        let color_quad_draw = Self::quad_draw(attr_info.permutation(), false);
        let bitmap_quad_draw = Self::quad_draw(attr_info.permutation(), true);

        let view_matrix = mat4([
            [1.0 / (TOP_WIDTH as f32 / 2.0), 0.0, 0.0, 0.0],
            [0.0, -1.0 / (TOP_HEIGHT as f32 / 2.0), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ]);

        Ok(Self {
            c3d,
            inner: BackendInner {
                gfx,
                color_program,
                texture_program,
                color_quad_draw,
                bitmap_quad_draw,
                attr_info,
                view_matrix,
                shape_tessellator: ShapeTessellator::new(),
                // meshes: vec![],
            },
        })
    }

    fn quad_draw(permutation: Permutation, is_bitmap: bool) -> Draw {
        const QUAD_VERTICES: &[Vertex] = &[
            Vertex {
                position: [0.0, 0.0],
                color: [255.0, 255.0, 255.0, 255.0],
            },
            Vertex {
                position: [1.0, 0.0],
                color: [255.0, 255.0, 255.0, 255.0],
            },
            Vertex {
                position: [1.0, 1.0],
                color: [255.0, 255.0, 255.0, 255.0],
            },
            Vertex {
                position: [0.0, 1.0],
                color: [255.0, 255.0, 255.0, 255.0],
            },
        ];

        let mut buf_info = buffer::Info::new();
        buf_info
            .add(Buffer::new(QUAD_VERTICES), permutation)
            .unwrap();

        Draw {
            draw_type: if is_bitmap {
                DrawType::Bitmap(BitmapDraw {
                    matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                    handle: None,
                    is_smoothed: false, // TODO: true
                    is_repeating: false,
                })
            } else {
                DrawType::Color
            },
            vertices: buf_info,
            indices: [0, 1, 2, 3].to_vec_in(LinearAllocator),
        }
    }

    fn upload_texture(&mut self, bitmap: Bitmap) -> Texture {
        let (bitmap, alpha) = match bitmap.format() {
            BitmapFormat::Rgb | BitmapFormat::Yuv420p => (bitmap.to_rgb(), false),
            BitmapFormat::Rgba | BitmapFormat::Yuva420p => (bitmap.to_rgba(), true),
        };

        let format = if alpha {
            ColorFormat::Rgba8
        } else {
            ColorFormat::Rgb8
        };
        let mut tex = Texture::new(TextureParameters::new_2d(
            bitmap.width() as u16,
            bitmap.height() as u16,
            format,
        ))
        .expect("failed to create texture");

        tex.load_image(bitmap.data(), Face::default())
            .expect("failed to load texture");

        tex
    }

    // pub fn submit_vbo(
    //     &mut self,
    //     clear: Color,
    //     info: &buffer::Info,
    //     attr: &attrib::Info,
    //     idxs: &Vec<u16, LinearAllocator>,
    //     world: Matrix4,
    //     proj: Matrix4,
    //     m: FVec4,
    //     a: FVec4,
    // ) {
    //     let mut top_screen = self.inner.gfx.top_screen.borrow_mut();
    //     let RawFrameBuffer { width, height, .. } = top_screen.raw_framebuffer();
    //
    //     let mut top_target = self
    //         .c3d
    //         .render_target(width, height, top_screen, None)
    //         .expect("failed to create target");
    //
    //     println!("rendering frame");
    //
    //     // let mut meshes = vec![];
    //     self.c3d.render_frame_with(|mut frame| {
    //         top_target.clear(
    //             ClearFlags::ALL,
    //             0x68_B0_D8_FF, // u32::from_le_bytes([clear.b, clear.g, clear.r, clear.a]),
    //             0,
    //         );
    //
    //         frame.select_render_target(&top_target).unwrap();
    //
    //         println!("rendering shape");
    //         // println!("{:?}", self.meshes);
    //         let program = &self.inner.color_program;
    //
    //         frame.bind_program(&program.program);
    //
    //         println!("world: {:?}", world);
    //         println!("worldView: {:?}", proj * world);
    //
    //         // bind uniforms
    //         frame.bind_vertex_uniform(program.view_idx, proj);
    //         frame.bind_vertex_uniform(program.world_idx, world);
    //         frame.bind_vertex_uniform(program.multclr_idx, m);
    //         frame.bind_vertex_uniform(program.addclr_idx, a);
    //
    //         frame.set_texenvs(&[program.texenv]);
    //         frame.set_attr_info(&attr);
    //
    //         frame.draw_elements(buffer::Primitive::Triangles, info, idxs);
    //         println!("drew elements");
    //
    //         frame
    //     });
    // }
}

impl RenderBackend for Citro3DRenderBackend {
    fn viewport_dimensions(&self) -> ViewportDimensions {
        DIMENSIONS
    }

    fn set_viewport_dimensions(&mut self, dim: ViewportDimensions) {}

    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        let mesh = self
            .shape_tessellator
            .tessellate_shape(shape, bitmap_source);

        let mut draws = vec![];
        for draw in mesh.draws {
            let draw_type = match draw.draw_type {
                TessDrawType::Color => DrawType::Color,
                TessDrawType::Gradient { matrix, gradient } => panic!("ahhh gradient, panic!"),
                // DrawType::Gradient(Box::new(Gradient::new(
                //     mesh.gradients[gradient].clone(), // TODO: Gradient deduplication
                //     matrix,
                // ))),
                TessDrawType::Bitmap(bitmap) => DrawType::Bitmap(BitmapDraw {
                    matrix: bitmap.matrix,
                    handle: bitmap_source.bitmap_handle(bitmap.bitmap_id, self),
                    is_smoothed: bitmap.is_smoothed,
                    is_repeating: bitmap.is_repeating,
                }),
            };

            let vertices = Buffer::new_in_linear(linear_vec(
                draw.vertices.into_iter().map(Vertex::from).collect(),
            ));
            let mut buf_info = buffer::Info::new();
            buf_info.add(vertices, self.attr_info.permutation());

            let mut indices = linear_vec(
                draw.indices
                    .into_iter()
                    .map(u16::try_from)
                    .collect::<Result<_, _>>()
                    .unwrap(),
            );

            draws.push(Draw {
                draw_type,
                vertices: buf_info,
                indices,
            });
        }

        ShapeHandle(Arc::new(Mesh { draws }))
    }

    fn render_offscreen(
        &mut self,
        _handle: BitmapHandle,
        _commands: CommandList,
        _quality: StageQuality,
        _bounds: PixelRegion,
    ) -> Option<Box<dyn SyncHandle>> {
        None
    }

    fn submit_frame(
        &mut self,
        clear: Color,
        commands: CommandList,
        cache_entries: Vec<BitmapCacheEntry>,
    ) {
        if !cache_entries.is_empty() {
            panic!("Bitmap caching unsupported in citro3d backend");
        }

        let mut top_screen = self.inner.gfx.top_screen.borrow_mut();
        let RawFrameBuffer { width, height, .. } = top_screen.raw_framebuffer();

        let mut top_target = self
            .c3d
            .render_target(width, height, top_screen, None)
            .expect("failed to create target");

        println!("rendering frame");

        // let mut meshes = vec![];
        self.c3d.render_frame_with(|mut frame| {
            top_target.clear(
                ClearFlags::ALL,
                u32::from_be_bytes([clear.a, clear.r, clear.g, clear.b]),
                0,
            );

            frame.select_render_target(&top_target).unwrap();

            let mut cmd_handler = Citro3DCommandHandler {
                renderer: &self.inner,
                frame,
                // meshes: &mut meshes,
            };

            commands.execute(&mut cmd_handler);

            let Citro3DCommandHandler { frame, .. } = cmd_handler;
            frame
        });
    }

    fn create_empty_texture(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<BitmapHandle, BitmapError> {
        let tex = Texture::new(TextureParameters::new_2d(
            width as u16,
            height as u16,
            ColorFormat::Rgba8,
        ))
        .expect("failed to create empty texture");

        Ok(BitmapHandle(Arc::new(CitroTexture { texture: tex })))
    }

    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, BitmapError> {
        let texture = self.upload_texture(bitmap);

        Ok(BitmapHandle(Arc::new(CitroTexture { texture })))
    }

    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap,
        region: PixelRegion,
    ) -> Result<(), BitmapError> {
        todo!("i'm not sure how to update textures rn")
    }

    fn create_context3d(
        &mut self,
        _profile: Context3DProfile,
    ) -> Result<Box<dyn Context3D>, BitmapError> {
        Err(BitmapError::Unimplemented("createContext3D".into()))
    }

    fn context3d_present(&mut self, _context: &mut dyn Context3D) -> Result<(), BitmapError> {
        Err(BitmapError::Unimplemented("Context3D.present".into()))
    }

    fn debug_info(&self) -> Cow<'static, str> {
        Cow::Borrowed("Renderer: citro3d")
    }

    fn name(&self) -> &'static str {
        "citro3d"
    }

    fn set_quality(&mut self, _quality: StageQuality) {}

    fn compile_pixelbender_shader(
        &mut self,
        _shader: ruffle_render::pixel_bender::PixelBenderShader,
    ) -> Result<ruffle_render::pixel_bender::PixelBenderShaderHandle, BitmapError> {
        Err(BitmapError::Unimplemented(
            "compile_pixelbender_shader".into(),
        ))
    }

    fn run_pixelbender_shader(
        &mut self,
        _handle: ruffle_render::pixel_bender::PixelBenderShaderHandle,
        _arguments: &[ruffle_render::pixel_bender::PixelBenderShaderArgument],
        _target: &PixelBenderTarget,
    ) -> Result<PixelBenderOutput, BitmapError> {
        Err(BitmapError::Unimplemented("run_pixelbender_shader".into()))
    }

    fn resolve_sync_handle(
        &mut self,
        _handle: Box<dyn SyncHandle>,
        _with_rgba: RgbaBufRead,
    ) -> Result<(), ruffle_render::error::Error> {
        Err(ruffle_render::error::Error::Unimplemented(
            "Sync handle resolution".into(),
        ))
    }
}

struct Citro3DCommandHandler<'a> {
    renderer: &'a BackendInner,
    frame: Frame<'a>,
    // meshes: &'a mut Vec<Mesh>,
}

impl<'a> Citro3DCommandHandler<'a> {
    // fn cache_mesh(&mut self, mesh: &Mesh) -> &'a Mesh {
    //     self.meshes.push(mesh.clone());
    //     // SAFETY: we just pushed, so last() is Some; and meshes lives for 'a
    //     unsafe {
    //         let meshes: *mut Vec<Mesh> = self.meshes;
    //         (*meshes).last().unwrap()
    //     }
    // }
}

static VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

impl<'a> CommandHandler for Citro3DCommandHandler<'a> {
    fn render_bitmap(
        &mut self,
        mut bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        pixel_snapping: PixelSnapping,
    ) {
        panic!("idk how to draw a bitmap");
        // let draw = &self.renderer.bitmap_quad_draw;
        // let bitmap_matrix = match draw.draw_type {
        //     DrawType::Bitmap(BitmapDraw { matrix, .. }) => matrix,
        //     _ => unreachable!(),
        // };
        //
        // let texture = &mut as_texture(&mut bitmap).texture;
        //
        // // Scale the quad to the bitmap's dimensions.
        // let mut matrix = transform.matrix;
        // pixel_snapping.apply(&mut matrix);
        // matrix *= Matrix::scale(texture.width() as f32, texture.height() as f32);
        //
        // let world_matrix = mat4([
        //     [matrix.a, matrix.b, 0.0, 0.0],
        //     [matrix.c, matrix.d, 0.0, 0.0],
        //     [0.0, 0.0, 1.0, 0.0],
        //     [
        //         matrix.tx.to_pixels() as f32,
        //         matrix.ty.to_pixels() as f32,
        //         0.0,
        //         1.0,
        //     ],
        // ]);
        //
        // // TODO: add these
        // let mult_color = transform.color_transform.mult_rgba_normalized();
        // let add_color = transform.color_transform.add_rgba_normalized();
        //
        // let program = &self.renderer.texture_program;
        //
        // self.frame.bind_program(&program.program);
        // self.frame.bind_vertex_uniform(program.view_idx, &self.renderer.view_matrix);
        // self.frame.bind_vertex_uniform(program.world_idx, &world_matrix);
        //
        // let filter = if smoothing {
        //     texture::Filter::Linear
        // } else {
        //     texture::Filter::Nearest
        // };
        // texture.set_filter(filter, filter);
        //
        // self.frame.bind_texture(texture::Index::Texture0, texture);
        //
        // self.frame.set_attr_info(&self.renderer.attr_info);
        // self.frame.draw_elements(buffer::Primitive::TriangleFan, &draw.vertices, &draw.indices)
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform) {
        println!("rendering shape");
        let world_matrix = [
            [transform.matrix.a, transform.matrix.b, 0.0, 0.0],
            [transform.matrix.c, transform.matrix.d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                transform.matrix.tx.to_pixels() as f32,
                transform.matrix.ty.to_pixels() as f32,
                -5.0,
                1.0,
            ],
        ];

        let mult_color = transform.color_transform.mult_rgba_normalized();
        let add_color = transform.color_transform.add_rgba_normalized();

        // let mesh: &'a Mesh = unsafe { &*(as_mesh(&shape) as *const Mesh) };

        let draws = &[&self.renderer.color_quad_draw];

        // println!("mesh with {} draws", mesh.draws.len());
        // println!("{:?}", self.meshes);
        for draw in draws {
            let program = match &draw.draw_type {
                DrawType::Color => &self.renderer.color_program,
                _ => &self.renderer.texture_program,
            };

            // todo: add some optimizations to this function

            // bind shader program
            self.frame.bind_program(&program.program);

            println!("world: {:?}", mat4(world_matrix));
            println!(
                "worldView: {:?}",
                self.renderer.view_matrix * mat4(world_matrix)
            );

            // bind uniforms
            self.frame
                .bind_vertex_uniform(program.view_idx, &self.renderer.view_matrix);
            self.frame
                .bind_vertex_uniform(program.world_idx, &mat4(world_matrix));
            self.frame
                .bind_vertex_uniform(program.multclr_idx, vec4(mult_color));
            self.frame
                .bind_vertex_uniform(program.addclr_idx, vec4(add_color));

            self.frame.set_texenvs(&[program.texenv]);
            self.frame.set_attr_info(&self.renderer.attr_info);

            self.frame
                .draw_elements(buffer::Primitive::Triangles, &draw.vertices, &draw.indices);

            println!("drew elements");
        }
    }

    fn render_stage3d(&mut self, _bitmap: BitmapHandle, _transform: Transform) {
        panic!("Stage3D should not exist on citro3d backend");
    }

    fn draw_rect(&mut self, color: Color, matrix: Matrix) {
        panic!();
    }

    fn draw_line(&mut self, color: Color, matrix: Matrix) {
        panic!();
    }

    fn draw_line_rect(&mut self, color: Color, matrix: Matrix) {
        panic!();
    }

    fn push_mask(&mut self) {
        panic!("masking not implemented for citro3d backend");
    }

    fn activate_mask(&mut self) {
        panic!("masking not implemented for citro3d backend");
    }

    fn deactivate_mask(&mut self) {
        panic!("masking not implemented for citro3d backend");
    }

    fn pop_mask(&mut self) {
        panic!("masking not implemented for citro3d backend");
    }

    fn blend(&mut self, commands: CommandList, _blend: RenderBlendMode) {
        panic!("blend not implemented for citro3d backend");
    }
}
