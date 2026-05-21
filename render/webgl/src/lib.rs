#![deny(clippy::unwrap_used)]
// Remove this when we start using `Rc` when compiling for wasm
#![allow(clippy::arc_with_non_send_sync)]
#![allow(unsafe_code)]

use bytemuck::{Pod, Zeroable};
pub use glow;
use glow::HasContext;
use ruffle_render::backend::{
    BitmapCacheEntry, Context3D, Context3DProfile, PixelBenderOutput, PixelBenderTarget,
    RenderBackend, ShapeHandle, ShapeHandleImpl, ViewportDimensions,
};
use ruffle_render::bitmap::{
    Bitmap, BitmapFormat, BitmapHandle, BitmapHandleImpl, BitmapSource, PixelRegion, PixelSnapping,
    RgbaBufRead, SyncHandle,
};
use ruffle_render::commands::{CommandHandler, CommandList, RenderBlendMode};
use ruffle_render::error::Error as BitmapError;
use ruffle_render::matrix::Matrix;
use ruffle_render::quality::StageQuality;
use ruffle_render::shape_utils::{DistilledShape, GradientType};
use ruffle_render::tessellator::{
    Gradient as TessGradient, ShapeTessellator, Vertex as TessVertex,
};
use ruffle_render::transform::Transform;
use std::any::Any;
use std::borrow::Cow;
use std::num::NonZeroU32;
use std::sync::Arc;
use swf::{BlendMode, Color, Twips};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't create GL context")]
    CantCreateGLContext,

    #[error("Couldn't create frame buffer")]
    UnableToCreateFrameBuffer,

    #[error("Couldn't create program")]
    UnableToCreateProgram,

    #[error("Couldn't create texture")]
    UnableToCreateTexture,

    #[error("Couldn't compile shader")]
    UnableToCreateShader,

    #[error("Couldn't create render buffer")]
    UnableToCreateRenderBuffer,

    #[error("Couldn't create vertex array object")]
    UnableToCreateVAO,

    #[error("Couldn't create buffer")]
    UnableToCreateBuffer,

    #[error("OES_element_index_uint extension not available")]
    OESExtensionNotFound,

    #[error("VAO extension not found")]
    VAOExtensionNotFound,

    #[error("Couldn't link shader program: {0}")]
    LinkingShaderProgram(String),

    #[error("GL Error in {0}: {1}")]
    GLError(&'static str, u32),
}

const COLOR_VERTEX_GLSL: &str = include_str!("../shaders/color.vert");
const COLOR_FRAGMENT_GLSL: &str = include_str!("../shaders/color.frag");
const TEXTURE_VERTEX_GLSL: &str = include_str!("../shaders/texture.vert");
const GRADIENT_FRAGMENT_GLSL: &str = include_str!("../shaders/gradient.frag");
const BITMAP_FRAGMENT_GLSL: &str = include_str!("../shaders/bitmap.frag");
const NUM_VERTEX_ATTRIBUTES: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaskState {
    NoMask,
    DrawMaskStencil,
    DrawMaskedContent,
    ClearMaskStencil,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: u32,
}

impl From<TessVertex> for Vertex {
    fn from(vertex: TessVertex) -> Self {
        Self {
            position: [vertex.x, vertex.y],
            color: u32::from_le_bytes([
                vertex.color.r,
                vertex.color.g,
                vertex.color.b,
                vertex.color.a,
            ]),
        }
    }
}

pub struct WebGlRenderBackend {
    /// glow context
    gl: Arc<glow::Context>,

    // Says if a WebGL2/GLES3 Context is Available
    gl2: bool,

    // The frame buffers used for resolving MSAA.
    msaa_buffers: Option<MsaaBuffers>,
    msaa_sample_count: u32,

    color_program: ShaderProgram,
    bitmap_program: ShaderProgram,
    gradient_program: ShaderProgram,

    shape_tessellator: ShapeTessellator,

    color_quad_draws: Vec<Draw>,
    bitmap_quad_draws: Vec<Draw>,

    mask_state: MaskState,
    num_masks: u32,
    mask_state_dirty: bool,
    is_transparent: bool,

    active_program: *const ShaderProgram,
    blend_modes: Vec<RenderBlendMode>,
    mult_color: Option<[f32; 4]>,
    add_color: Option<[f32; 4]>,

    renderbuffer_width: i32,
    renderbuffer_height: i32,
    view_matrix: [[f32; 4]; 4],

    // This is currently unused - we just hold on to it
    // to expose via `get_viewport_dimensions`
    viewport_scale_factor: f64,
}

#[derive(Debug)]
struct RegistryData {
    gl: Arc<glow::Context>,
    width: u32,
    height: u32,
    texture: glow::Texture,
}

impl Drop for RegistryData {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
        }
    }
}

impl BitmapHandleImpl for RegistryData {}

fn as_registry_data(handle: &BitmapHandle) -> &RegistryData {
    <dyn Any>::downcast_ref(&*handle.0).expect("Bitmap handle must be webgl RegistryData")
}

const MAX_GRADIENT_COLORS: usize = 15;

impl WebGlRenderBackend {
    pub fn new(
        glow_context: Arc<glow::Context>,
        is_transparent: bool,
        quality: StageQuality,
    ) -> Result<Self, Error> {
        unsafe {
            let gl = glow_context;
            let gl2 = gl.version().major >= 3;

            // Determine MSAA sample count.
            let mut msaa_sample_count = quality.sample_count().min(4);

            // Ensure that we don't exceed the max MSAA of this device.
            if gl2 {
                let max_samples = gl.get_parameter_i32(glow::MAX_SAMPLES) as u32;
                if max_samples > 0 && max_samples < msaa_sample_count {
                    log::info!("Device only supports {max_samples}xMSAA");
                    msaa_sample_count = max_samples;
                }
            }

            let color_vertex = Self::compile_shader(&gl, glow::VERTEX_SHADER, COLOR_VERTEX_GLSL)?;
            let texture_vertex =
                Self::compile_shader(&gl, glow::VERTEX_SHADER, TEXTURE_VERTEX_GLSL)?;
            let color_fragment =
                Self::compile_shader(&gl, glow::FRAGMENT_SHADER, COLOR_FRAGMENT_GLSL)?;
            let bitmap_fragment =
                Self::compile_shader(&gl, glow::FRAGMENT_SHADER, BITMAP_FRAGMENT_GLSL)?;
            let gradient_fragment =
                Self::compile_shader(&gl, glow::FRAGMENT_SHADER, GRADIENT_FRAGMENT_GLSL)?;

            let color_program = ShaderProgram::new(&gl, color_vertex, color_fragment)?;
            let bitmap_program = ShaderProgram::new(&gl, texture_vertex, bitmap_fragment)?;
            let gradient_program = ShaderProgram::new(&gl, texture_vertex, gradient_fragment)?;

            gl.enable(glow::BLEND);

            // Necessary to load RGB textures (alignment defaults to 4).
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

            let mut renderer = Self {
                gl,
                gl2,

                msaa_buffers: None,
                msaa_sample_count,

                color_program,
                gradient_program,
                bitmap_program,

                shape_tessellator: ShapeTessellator::new(),

                color_quad_draws: vec![],
                bitmap_quad_draws: vec![],
                renderbuffer_width: 1,
                renderbuffer_height: 1,
                view_matrix: [[0.0; 4]; 4],

                mask_state: MaskState::NoMask,
                num_masks: 0,
                mask_state_dirty: true,
                is_transparent,

                active_program: std::ptr::null(),
                blend_modes: vec![],
                mult_color: None,
                add_color: None,

                viewport_scale_factor: 1.0,
            };

            renderer.push_blend_mode(RenderBlendMode::Builtin(BlendMode::Normal));

            let mut color_quad_mesh = renderer.build_quad_mesh(&renderer.color_program)?;
            let mut bitmap_quad_mesh = renderer.build_quad_mesh(&renderer.bitmap_program)?;
            renderer.color_quad_draws.append(&mut color_quad_mesh);
            renderer.bitmap_quad_draws.append(&mut bitmap_quad_mesh);

            renderer.set_viewport_dimensions(ViewportDimensions {
                width: 1,
                height: 1,
                scale_factor: 1.0,
            });

            Ok(renderer)
        }
    }

    fn build_quad_mesh(&self, program: &ShaderProgram) -> Result<Vec<Draw>, Error> {
        let vao = self.create_vertex_array()?;

        unsafe {
            let vertex_buffer = self
                .gl
                .create_buffer()
                .map_err(|_| Error::UnableToCreateBuffer)?;
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&[
                    Vertex {
                        position: [0.0, 0.0],
                        color: 0xffff_ffff,
                    },
                    Vertex {
                        position: [1.0, 0.0],
                        color: 0xffff_ffff,
                    },
                    Vertex {
                        position: [1.0, 1.0],
                        color: 0xffff_ffff,
                    },
                    Vertex {
                        position: [0.0, 1.0],
                        color: 0xffff_ffff,
                    },
                ]),
                glow::STATIC_DRAW,
            );

            let index_buffer = self
                .gl
                .create_buffer()
                .map_err(|_| Error::UnableToCreateBuffer)?;
            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            self.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&[0u32, 1, 2, 3]),
                glow::STATIC_DRAW,
            );

            if program.vertex_position_location != 0xffff_ffff {
                self.gl.vertex_attrib_pointer_f32(
                    program.vertex_position_location,
                    2,
                    glow::FLOAT,
                    false,
                    12,
                    0,
                );
                self.gl
                    .enable_vertex_attrib_array(program.vertex_position_location);
            }

            if program.vertex_color_location != 0xffff_ffff {
                self.gl.vertex_attrib_pointer_f32(
                    program.vertex_color_location,
                    4,
                    glow::UNSIGNED_BYTE,
                    true,
                    12,
                    8,
                );
                self.gl
                    .enable_vertex_attrib_array(program.vertex_color_location);
            }
            self.bind_vertex_array(None);
            for i in program.num_vertex_attributes..NUM_VERTEX_ATTRIBUTES {
                self.gl.disable_vertex_attrib_array(i);
            }

            let mut draws = vec![];
            draws.push(Draw {
                draw_type: if program.program == self.bitmap_program.program {
                    DrawType::Bitmap(BitmapDraw {
                        matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                        handle: None,
                        is_smoothed: true,
                        is_repeating: false,
                    })
                } else {
                    DrawType::Color
                },
                vao,
                vertex_buffer: Buffer {
                    gl: self.gl.clone(),
                    buffer: vertex_buffer,
                },
                index_buffer: Buffer {
                    gl: self.gl.clone(),
                    buffer: index_buffer,
                },
                num_indices: 4,
                num_mask_indices: 4,
            });
            Ok(draws)
        }
    }

    fn compile_shader(
        gl: &glow::Context,
        shader_type: u32,
        glsl_src: &str,
    ) -> Result<glow::Shader, Error> {
        unsafe {
            let shader = gl
                .create_shader(shader_type)
                .map_err(|_| Error::UnableToCreateShader)?;
            gl.shader_source(shader, glsl_src);
            gl.compile_shader(shader);
            if log::log_enabled!(log::Level::Error) {
                let log = gl.get_shader_info_log(shader);
                if !log.is_empty() {
                    log::error!("{log}");
                }
            }
            Ok(shader)
        }
    }

    fn build_msaa_buffers(&mut self) -> Result<(), Error> {
        if !self.gl2 || self.msaa_sample_count <= 1 {
            unsafe {
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                self.gl.bind_renderbuffer(glow::RENDERBUFFER, None);
                return Ok(());
            }
        }
        unsafe {
            let gl = self.gl.as_ref();

            // Delete previous buffers, if they exist.
            if let Some(msaa_buffers) = self.msaa_buffers.take() {
                gl.delete_renderbuffer(msaa_buffers.color_renderbuffer);
                gl.delete_renderbuffer(msaa_buffers.stencil_renderbuffer);
                gl.delete_framebuffer(msaa_buffers.render_framebuffer);
                gl.delete_framebuffer(msaa_buffers.color_framebuffer);
                gl.delete_texture(msaa_buffers.framebuffer_texture);
            }

            // Create frame and render buffers.
            let render_framebuffer = gl
                .create_framebuffer()
                .map_err(|_| Error::UnableToCreateFrameBuffer)?;
            let color_framebuffer = gl
                .create_framebuffer()
                .map_err(|_| Error::UnableToCreateFrameBuffer)?;

            // Note for future self:
            // Whenever we support playing transparent movies,
            // switch this to RGBA and probably need to change shaders to all
            // be premultiplied alpha.
            let color_renderbuffer = gl
                .create_renderbuffer()
                .map_err(|_| Error::UnableToCreateRenderBuffer)?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(color_renderbuffer));
            gl.renderbuffer_storage_multisample(
                glow::RENDERBUFFER,
                self.msaa_sample_count as i32,
                glow::RGBA8,
                self.renderbuffer_width,
                self.renderbuffer_height,
            );
            gl.check_error("renderbuffer_storage_multisample (color)")?;

            let stencil_renderbuffer = gl
                .create_renderbuffer()
                .map_err(|_| Error::UnableToCreateFrameBuffer)?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(stencil_renderbuffer));
            gl.renderbuffer_storage_multisample(
                glow::RENDERBUFFER,
                self.msaa_sample_count as i32,
                glow::STENCIL_INDEX8,
                self.renderbuffer_width,
                self.renderbuffer_height,
            );
            gl.check_error("renderbuffer_storage_multisample (stencil)")?;

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(render_framebuffer));
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(color_renderbuffer),
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::STENCIL_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(stencil_renderbuffer),
            );

            let framebuffer_texture = gl
                .create_texture()
                .map_err(|_| Error::UnableToCreateTexture)?;
            gl.bind_texture(glow::TEXTURE_2D, Some(framebuffer_texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                self.renderbuffer_width,
                self.renderbuffer_height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(color_framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(framebuffer_texture),
                0,
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            self.msaa_buffers = Some(MsaaBuffers {
                color_renderbuffer,
                stencil_renderbuffer,
                render_framebuffer,
                color_framebuffer,
                framebuffer_texture,
            });

            Ok(())
        }
    }

    fn register_shape_internal(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
        scale: f32,
    ) -> Result<Vec<Draw>, Error> {
        unsafe {
            use ruffle_render::tessellator::DrawType as TessDrawType;

            let lyon_mesh =
                self.shape_tessellator
                    .tessellate_shape_with_scale(shape, bitmap_source, scale);

            let mut draws = Vec::with_capacity(lyon_mesh.draws.len());
            for draw in lyon_mesh.draws {
                let num_indices = draw.indices.len() as i32;
                let num_mask_indices = draw.mask_index_count as i32;

                let vao = self.create_vertex_array()?;
                let vertex_buffer = self
                    .gl
                    .create_buffer()
                    .map_err(|_| Error::UnableToCreateBuffer)?;
                self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));

                let vertices: Vec<_> = draw.vertices.into_iter().map(Vertex::from).collect();
                self.gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&vertices),
                    glow::STATIC_DRAW,
                );

                let index_buffer = self
                    .gl
                    .create_buffer()
                    .map_err(|_| Error::UnableToCreateBuffer)?;
                self.gl
                    .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
                self.gl.buffer_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    bytemuck::cast_slice(&draw.indices),
                    glow::STATIC_DRAW,
                );

                let program = match draw.draw_type {
                    TessDrawType::Color => &self.color_program,
                    TessDrawType::Gradient { .. } => &self.gradient_program,
                    TessDrawType::Bitmap(_) => &self.bitmap_program,
                };

                // Unfortunately it doesn't seem to be possible to ensure that vertex attributes will be in
                // a guaranteed position between shaders in WebGL1 (no layout qualifiers in GLSL in OpenGL ES 1.0).
                // Attributes can change between shaders, even if the vertex layout is otherwise "the same".
                // This varies between platforms based on what the GLSL compiler decides to do.
                if program.vertex_position_location != 0xffff_ffff {
                    self.gl.vertex_attrib_pointer_f32(
                        program.vertex_position_location,
                        2,
                        glow::FLOAT,
                        false,
                        12,
                        0,
                    );
                    self.gl
                        .enable_vertex_attrib_array(program.vertex_position_location);
                }

                if program.vertex_color_location != 0xffff_ffff {
                    self.gl.vertex_attrib_pointer_f32(
                        program.vertex_color_location,
                        4,
                        glow::UNSIGNED_BYTE,
                        true,
                        12,
                        8,
                    );
                    self.gl
                        .enable_vertex_attrib_array(program.vertex_color_location);
                }

                let num_vertex_attributes = program.num_vertex_attributes;

                draws.push(match draw.draw_type {
                    TessDrawType::Color => Draw {
                        draw_type: DrawType::Color,
                        vao,
                        vertex_buffer: Buffer {
                            gl: self.gl.clone(),
                            buffer: vertex_buffer,
                        },
                        index_buffer: Buffer {
                            gl: self.gl.clone(),
                            buffer: index_buffer,
                        },
                        num_indices,
                        num_mask_indices,
                    },
                    TessDrawType::Gradient { matrix, gradient } => Draw {
                        draw_type: DrawType::Gradient(Box::new(Gradient::new(
                            lyon_mesh.gradients[gradient].clone(), // TODO: Gradient deduplication
                            matrix,
                        ))),
                        vao,
                        vertex_buffer: Buffer {
                            gl: self.gl.clone(),
                            buffer: vertex_buffer,
                        },
                        index_buffer: Buffer {
                            gl: self.gl.clone(),
                            buffer: index_buffer,
                        },
                        num_indices,
                        num_mask_indices,
                    },
                    TessDrawType::Bitmap(bitmap) => Draw {
                        draw_type: DrawType::Bitmap(BitmapDraw {
                            matrix: bitmap.matrix,
                            handle: bitmap_source.bitmap_handle(bitmap.bitmap_id, self),
                            is_smoothed: bitmap.is_smoothed,
                            is_repeating: bitmap.is_repeating,
                        }),
                        vao,
                        vertex_buffer: Buffer {
                            gl: self.gl.clone(),
                            buffer: vertex_buffer,
                        },
                        index_buffer: Buffer {
                            gl: self.gl.clone(),
                            buffer: index_buffer,
                        },
                        num_indices,
                        num_mask_indices,
                    },
                });

                self.bind_vertex_array(None);

                // Don't use 'program' here in order to satisfy the borrow checker
                for i in num_vertex_attributes..NUM_VERTEX_ATTRIBUTES {
                    self.gl.disable_vertex_attrib_array(i);
                }
            }

            Ok(draws)
        }
    }

    /// Creates and binds a new VAO.
    fn create_vertex_array(&self) -> Result<glow::VertexArray, Error> {
        unsafe {
            let vao = self
                .gl
                .create_vertex_array()
                .map_err(|_| Error::UnableToCreateVAO)?;
            self.gl.bind_vertex_array(Some(vao));
            Ok(vao)
        }
    }

    /// Binds a VAO.
    fn bind_vertex_array(&self, vao: Option<glow::VertexArray>) {
        unsafe {
            self.gl.bind_vertex_array(vao);
        }
    }

    fn set_stencil_state(&self) {
        unsafe {
            // Set stencil state for masking, if necessary.
            if self.mask_state_dirty {
                match self.mask_state {
                    MaskState::NoMask => {
                        self.gl.disable(glow::STENCIL_TEST);
                        self.gl.color_mask(true, true, true, true);
                    }
                    MaskState::DrawMaskStencil => {
                        self.gl.enable(glow::STENCIL_TEST);
                        self.gl
                            .stencil_func(glow::EQUAL, (self.num_masks - 1) as i32, 0xff);
                        self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::INCR);
                        self.gl.color_mask(false, false, false, false);
                    }
                    MaskState::DrawMaskedContent => {
                        self.gl.enable(glow::STENCIL_TEST);
                        self.gl
                            .stencil_func(glow::EQUAL, self.num_masks as i32, 0xff);
                        self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
                        self.gl.color_mask(true, true, true, true);
                    }
                    MaskState::ClearMaskStencil => {
                        self.gl.enable(glow::STENCIL_TEST);
                        self.gl
                            .stencil_func(glow::EQUAL, self.num_masks as i32, 0xff);
                        self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::DECR);
                        self.gl.color_mask(false, false, false, false);
                    }
                }
            }
        }
    }

    fn apply_blend_mode(&self, mode: RenderBlendMode) {
        unsafe {
            let (blend_op, src_rgb, dst_rgb) = match mode {
                RenderBlendMode::Builtin(BlendMode::Normal) => {
                    // src + (1-a)
                    (glow::FUNC_ADD, glow::ONE, glow::ONE_MINUS_SRC_ALPHA)
                }
                RenderBlendMode::Builtin(BlendMode::Add) => {
                    // src + dst
                    (glow::FUNC_ADD, glow::ONE, glow::ONE)
                }
                RenderBlendMode::Builtin(BlendMode::Subtract) => {
                    // dst - src
                    (glow::FUNC_REVERSE_SUBTRACT, glow::ONE, glow::ONE)
                }
                _ => {
                    // TODO: Unsupported blend mode. Default to normal for now.
                    (glow::FUNC_ADD, glow::ONE, glow::ONE_MINUS_SRC_ALPHA)
                }
            };
            self.gl.blend_equation_separate(blend_op, glow::FUNC_ADD);
            self.gl
                .blend_func_separate(src_rgb, dst_rgb, glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
        }
    }

    fn begin_frame(&mut self, clear: Color) {
        unsafe {
            self.active_program = std::ptr::null();
            self.mask_state = MaskState::NoMask;
            self.num_masks = 0;
            self.mask_state_dirty = true;

            self.mult_color = None;
            self.add_color = None;

            // Bind to MSAA render buffer if using MSAA.
            if let Some(msaa_buffers) = &self.msaa_buffers {
                self.gl
                    .bind_framebuffer(glow::FRAMEBUFFER, Some(msaa_buffers.render_framebuffer));
            }

            self.gl
                .viewport(0, 0, self.renderbuffer_width, self.renderbuffer_height);

            self.set_stencil_state();
            if self.is_transparent {
                self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            } else {
                self.gl.clear_color(
                    clear.r as f32 / 255.0,
                    clear.g as f32 / 255.0,
                    clear.b as f32 / 255.0,
                    clear.a as f32 / 255.0,
                );
            }
            self.gl.stencil_mask(0xff);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        }
    }

    fn end_frame(&self) {
        unsafe {
            // Resolve MSAA, if we're using it (WebGL2).
            if self.gl2
                && let Some(msaa_buffers) = &self.msaa_buffers
            {
                // Disable any remaining masking state.
                self.gl.disable(glow::STENCIL_TEST);
                self.gl.color_mask(true, true, true, true);

                // Resolve the MSAA in the render buffer.
                self.gl.bind_framebuffer(
                    glow::READ_FRAMEBUFFER,
                    Some(msaa_buffers.render_framebuffer),
                );
                self.gl
                    .bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(msaa_buffers.color_framebuffer));
                self.gl.blit_framebuffer(
                    0,
                    0,
                    self.renderbuffer_width,
                    self.renderbuffer_height,
                    0,
                    0,
                    self.renderbuffer_width,
                    self.renderbuffer_height,
                    glow::COLOR_BUFFER_BIT,
                    glow::NEAREST,
                );

                // Render the resolved framebuffer texture to a quad on the screen.
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);

                self.gl
                    .viewport(0, 0, self.renderbuffer_width, self.renderbuffer_height);

                let program = &self.bitmap_program;
                self.gl.use_program(Some(program.program));

                // Scale to fill screen.
                program.uniform_matrix4fv(
                    &self.gl,
                    ShaderUniform::WorldMatrix,
                    &[
                        [2.0, 0.0, 0.0, 0.0],
                        [0.0, 2.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [-1.0, -1.0, 0.0, 1.0],
                    ],
                );
                program.uniform_matrix4fv(
                    &self.gl,
                    ShaderUniform::ViewMatrix,
                    &[
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                );
                program.uniform4fv(&self.gl, ShaderUniform::MultColor, &[1.0, 1.0, 1.0, 1.0]);
                program.uniform4fv(&self.gl, ShaderUniform::AddColor, &[0.0, 0.0, 0.0, 0.0]);

                program.uniform_matrix3fv(
                    &self.gl,
                    ShaderUniform::TextureMatrix,
                    &[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                );

                // Bind the framebuffer texture.
                self.gl.active_texture(glow::TEXTURE0);
                self.gl
                    .bind_texture(glow::TEXTURE_2D, Some(msaa_buffers.framebuffer_texture));
                program.uniform1i(&self.gl, ShaderUniform::BitmapTexture, 0);

                // Render the quad.
                let quad = &self.bitmap_quad_draws;
                self.bind_vertex_array(Some(quad[0].vao));
                self.gl.draw_elements(
                    glow::TRIANGLE_FAN,
                    quad[0].num_indices,
                    glow::UNSIGNED_INT,
                    0,
                );
            }
        }
    }

    fn push_blend_mode(&mut self, blend: RenderBlendMode) {
        if !same_blend_mode(self.blend_modes.last(), &blend) {
            self.apply_blend_mode(blend.clone());
        }
        self.blend_modes.push(blend);
    }
    fn pop_blend_mode(&mut self) {
        let old = self.blend_modes.pop();
        // We never pop our base 'BlendMode::Normal'
        let current = self
            .blend_modes
            .last()
            .unwrap_or(&RenderBlendMode::Builtin(BlendMode::Normal));
        if !same_blend_mode(old.as_ref(), current) {
            self.apply_blend_mode(current.clone());
        }
    }

    fn draw_quad<const MODE: u32, const COUNT: i32>(&mut self, color: Color, matrix: Matrix) {
        let world_matrix = [
            [matrix.a, matrix.b, 0.0, 0.0],
            [matrix.c, matrix.d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                matrix.tx.to_pixels() as f32,
                matrix.ty.to_pixels() as f32,
                0.0,
                1.0,
            ],
        ];

        let mult_color = [
            color.r as f32 * 255.0,
            color.g as f32 * 255.0,
            color.b as f32 * 255.0,
            color.a as f32 * 255.0,
        ];
        let add_color = [0.0; 4];

        self.set_stencil_state();

        let program = &self.color_program;

        // Set common render state, while minimizing unnecessary state changes.
        // TODO: Using designated layout specifiers in WebGL2/OpenGL ES 3, we could guarantee that uniforms
        // are in the same location between shaders, and avoid changing them unless necessary.
        if !std::ptr::eq(program, self.active_program) {
            unsafe {
                self.gl.use_program(Some(program.program));
            }
            self.active_program = program as *const ShaderProgram;

            program.uniform_matrix4fv(&self.gl, ShaderUniform::ViewMatrix, &self.view_matrix);

            self.mult_color = None;
            self.add_color = None;
        };

        self.color_program
            .uniform_matrix4fv(&self.gl, ShaderUniform::WorldMatrix, &world_matrix);
        if Some(mult_color) != self.mult_color {
            self.color_program
                .uniform4fv(&self.gl, ShaderUniform::MultColor, &mult_color);
            self.mult_color = Some(mult_color);
        }
        if Some(add_color) != self.add_color {
            self.color_program
                .uniform4fv(&self.gl, ShaderUniform::AddColor, &add_color);
            self.add_color = Some(add_color);
        }

        let quad = &self.color_quad_draws;
        self.bind_vertex_array(Some(quad[0].vao));
        unsafe {
            let count = if COUNT < 0 {
                quad[0].num_indices
            } else {
                COUNT
            };
            self.gl.draw_elements(MODE, count, glow::UNSIGNED_INT, 0);
        }
    }
}

fn same_blend_mode(first: Option<&RenderBlendMode>, second: &RenderBlendMode) -> bool {
    match (first, second) {
        (Some(RenderBlendMode::Builtin(old)), RenderBlendMode::Builtin(new)) => old == new,
        _ => false,
    }
}

impl RenderBackend for WebGlRenderBackend {
    fn render_offscreen(
        &mut self,
        _handle: BitmapHandle,
        _commands: CommandList,
        _quality: StageQuality,
        _bounds: PixelRegion,
    ) -> Option<Box<dyn SyncHandle>> {
        None
    }

    fn viewport_dimensions(&self) -> ViewportDimensions {
        ViewportDimensions {
            width: self.renderbuffer_width as u32,
            height: self.renderbuffer_height as u32,
            scale_factor: self.viewport_scale_factor,
        }
    }

    fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions) {
        // Build view matrix based on canvas size.
        self.view_matrix = [
            [1.0 / (dimensions.width as f32 / 2.0), 0.0, 0.0, 0.0],
            [0.0, -1.0 / (dimensions.height as f32 / 2.0), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        // Setup GL viewport and renderbuffers clamped to reasonable sizes.
        // We don't use `.clamp()` here because `self.gl.drawing_buffer_width()` and
        // `self.gl.drawing_buffer_height()` return zero when the WebGL context is lost,
        // then an assertion error would be triggered.
        self.renderbuffer_width = (dimensions.width.max(1) as i32).min(dimensions.width as i32);
        self.renderbuffer_height = (dimensions.height.max(1) as i32).min(dimensions.height as i32);

        // Recreate framebuffers with the new size.
        let _ = self.build_msaa_buffers();
        unsafe {
            self.gl
                .viewport(0, 0, self.renderbuffer_width, self.renderbuffer_height);
        }
        self.viewport_scale_factor = dimensions.scale_factor
    }

    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        self.register_shape_with_scale(shape, bitmap_source, 1.0)
    }

    fn register_shape_with_scale(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
        scale: f32,
    ) -> ShapeHandle {
        let mesh = match self.register_shape_internal(shape, bitmap_source, scale) {
            Ok(draws) => Mesh {
                draws,
                gl2: self.gl.clone(),
            },
            Err(e) => {
                log::error!("Couldn't register shape: {e:?}");
                Mesh {
                    draws: vec![],
                    gl2: self.gl.clone(),
                }
            }
        };
        ShapeHandle(Arc::new(mesh))
    }

    fn submit_frame(
        &mut self,
        clear: Color,
        commands: CommandList,
        cache_entries: Vec<BitmapCacheEntry>,
    ) {
        if !cache_entries.is_empty() {
            panic!("Bitmap caching is unavailable on the webgl backend");
        }
        self.begin_frame(clear);
        commands.execute(self);
        self.end_frame();
    }

    fn register_bitmap(&mut self, bitmap: Bitmap<'_>) -> Result<BitmapHandle, BitmapError> {
        unsafe {
            let (format, bitmap) = match bitmap.format() {
                BitmapFormat::Rgb | BitmapFormat::Yuv420p => (glow::RGB, bitmap.to_rgb()),
                BitmapFormat::Rgba | BitmapFormat::Yuva420p => (glow::RGBA, bitmap.to_rgba()),
            };

            let texture = self
                .gl
                .create_texture()
                .map_err(|_| BitmapError::UnableToCreateTexture)?;
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format as i32,
                bitmap.width() as i32,
                bitmap.height() as i32,
                0,
                format,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(bitmap.data())),
            );

            // You must set the texture parameters for non-power-of-2 textures to function in WebGL1.
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            Ok(BitmapHandle(Arc::new(RegistryData {
                gl: self.gl.clone(),
                width: bitmap.width(),
                height: bitmap.height(),
                texture,
            })))
        }
    }

    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap<'_>,
        _region: PixelRegion,
    ) -> Result<(), BitmapError> {
        unsafe {
            let texture = as_registry_data(handle).texture;

            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            let (format, bitmap) = match bitmap.format() {
                BitmapFormat::Rgb | BitmapFormat::Yuv420p => (glow::RGB, bitmap.to_rgb()),
                BitmapFormat::Rgba | BitmapFormat::Yuva420p => (glow::RGBA, bitmap.to_rgba()),
            };

            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format as i32,
                bitmap.width() as i32,
                bitmap.height() as i32,
                0,
                format,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(bitmap.data())),
            );

            Ok(())
        }
    }

    fn create_context3d(
        &mut self,
        _profile: Context3DProfile,
    ) -> Result<Box<dyn Context3D>, BitmapError> {
        Err(BitmapError::Unimplemented("createContext3D".into()))
    }

    fn debug_info(&self) -> Cow<'static, str> {
        let mut result = vec![];

        if self.gl2 {
            result.push("Renderer: WebGL 2.0".to_string());
        } else {
            result.push("Renderer: WebGL 1.0".to_string());
        }

        let mut add_line = |name, val: String| result.push(format!("{name}: {}", val));
        unsafe {
            add_line("Adapter Vendor", self.gl.get_parameter_string(glow::VENDOR));
            add_line(
                "Adapter Renderer",
                self.gl.get_parameter_string(glow::RENDERER),
            );
            add_line(
                "Adapter Version",
                self.gl.get_parameter_string(glow::VERSION),
            );
        }

        result.push(format!("Surface samples: {} x ", self.msaa_sample_count));
        result.push(format!(
            "Surface size: {} x {}",
            self.renderbuffer_width, self.renderbuffer_height
        ));

        Cow::Owned(result.join("\n"))
    }

    fn name(&self) -> &'static str {
        "webgl"
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

    fn resolve_sync_handle(
        &mut self,
        _handle: Box<dyn SyncHandle>,
        _with_rgba: RgbaBufRead,
    ) -> Result<(), ruffle_render::error::Error> {
        Err(ruffle_render::error::Error::Unimplemented(
            "Sync handle resolution".into(),
        ))
    }

    fn run_pixelbender_shader(
        &mut self,
        _handle: ruffle_render::pixel_bender::PixelBenderShaderHandle,
        _arguments: &[ruffle_render::pixel_bender_support::PixelBenderShaderArgument],
        _target: &PixelBenderTarget,
    ) -> Result<PixelBenderOutput, BitmapError> {
        Err(BitmapError::Unimplemented("run_pixelbender_shader".into()))
    }

    fn create_empty_texture(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<BitmapHandle, BitmapError> {
        unsafe {
            let texture = self
                .gl
                .create_texture()
                .map_err(|_| BitmapError::UnableToCreateTexture)?;
            self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            // You must set the texture parameters for non-power-of-2 textures to function in WebGL1.
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            self.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            Ok(BitmapHandle(Arc::new(RegistryData {
                gl: self.gl.clone(),
                width: width.get(),
                height: height.get(),
                texture,
            })))
        }
    }
}

impl CommandHandler for WebGlRenderBackend {
    fn render_bitmap(
        &mut self,
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        pixel_snapping: PixelSnapping,
    ) {
        unsafe {
            self.set_stencil_state();
            let entry = as_registry_data(&bitmap);
            // Adjust the quad draw to use the target bitmap.
            let quad = &self.bitmap_quad_draws;
            let draw = &quad[0];
            let bitmap_matrix = if let DrawType::Bitmap(BitmapDraw { matrix, .. }) = &draw.draw_type
            {
                matrix
            } else {
                unreachable!()
            };

            // Scale the quad to the bitmap's dimensions.
            let mut matrix = transform.matrix;
            pixel_snapping.apply(&mut matrix);
            matrix *= Matrix::scale(entry.width as f32, entry.height as f32);

            let world_matrix = [
                [matrix.a, matrix.b, 0.0, 0.0],
                [matrix.c, matrix.d, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [
                    matrix.tx.to_pixels() as f32,
                    matrix.ty.to_pixels() as f32,
                    0.0,
                    1.0,
                ],
            ];

            let mult_color = transform.color_transform.mult_rgba_normalized();
            let add_color = transform.color_transform.add_rgba_normalized();

            self.bind_vertex_array(Some(draw.vao));

            let program = &self.bitmap_program;

            // Set common render state, while minimizing unnecessary state changes.
            // TODO: Using designated layout specifiers in WebGL2/OpenGL ES 3, we could guarantee that uniforms
            // are in the same location between shaders, and avoid changing them unless necessary.
            if !std::ptr::eq(program, self.active_program) {
                self.gl.use_program(Some(program.program));
                self.active_program = program as *const ShaderProgram;

                program.uniform_matrix4fv(&self.gl, ShaderUniform::ViewMatrix, &self.view_matrix);

                self.mult_color = None;
                self.add_color = None;
            }

            program.uniform_matrix4fv(&self.gl, ShaderUniform::WorldMatrix, &world_matrix);
            if Some(mult_color) != self.mult_color {
                program.uniform4fv(&self.gl, ShaderUniform::MultColor, &mult_color);
                self.mult_color = Some(mult_color);
            }
            if Some(add_color) != self.add_color {
                program.uniform4fv(&self.gl, ShaderUniform::AddColor, &add_color);
                self.add_color = Some(add_color);
            }

            program.uniform_matrix3fv(&self.gl, ShaderUniform::TextureMatrix, bitmap_matrix);

            // Bind texture.
            self.gl.active_texture(glow::TEXTURE0);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(entry.texture));
            program.uniform1i(&self.gl, ShaderUniform::BitmapTexture, 0);

            // Set texture parameters.
            let filter = if smoothing {
                glow::LINEAR as i32
            } else {
                glow::NEAREST as i32
            };
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, filter);
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, filter);

            let wrap = glow::CLAMP_TO_EDGE as i32;
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap);
            self.gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap);

            // Draw the triangles.
            self.gl
                .draw_elements(glow::TRIANGLE_FAN, draw.num_indices, glow::UNSIGNED_INT, 0);
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform) {
        unsafe {
            let world_matrix = [
                [transform.matrix.a, transform.matrix.b, 0.0, 0.0],
                [transform.matrix.c, transform.matrix.d, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [
                    transform.matrix.tx.to_pixels() as f32,
                    transform.matrix.ty.to_pixels() as f32,
                    0.0,
                    1.0,
                ],
            ];

            let mult_color = transform.color_transform.mult_rgba_normalized();
            let add_color = transform.color_transform.add_rgba_normalized();

            self.set_stencil_state();

            let mesh = as_mesh(&shape);
            for draw in &mesh.draws {
                // Ignore strokes when drawing a mask stencil.
                let num_indices = if self.mask_state != MaskState::DrawMaskStencil
                    && self.mask_state != MaskState::ClearMaskStencil
                {
                    draw.num_indices
                } else {
                    draw.num_mask_indices
                };
                if num_indices == 0 {
                    continue;
                }

                self.bind_vertex_array(Some(draw.vao));

                let program = match &draw.draw_type {
                    DrawType::Color => &self.color_program,
                    DrawType::Gradient(_) => &self.gradient_program,
                    DrawType::Bitmap { .. } => &self.bitmap_program,
                };

                // Set common render state, while minimizing unnecessary state changes.
                // TODO: Using designated layout specifiers in WebGL2/OpenGL ES 3, we could guarantee that uniforms
                // are in the same location between shaders, and avoid changing them unless necessary.
                if !std::ptr::eq(program, self.active_program) {
                    self.gl.use_program(Some(program.program));
                    self.active_program = program as *const ShaderProgram;

                    program.uniform_matrix4fv(
                        &self.gl,
                        ShaderUniform::ViewMatrix,
                        &self.view_matrix,
                    );

                    self.mult_color = None;
                    self.add_color = None;
                }

                program.uniform_matrix4fv(&self.gl, ShaderUniform::WorldMatrix, &world_matrix);
                if Some(mult_color) != self.mult_color {
                    program.uniform4fv(&self.gl, ShaderUniform::MultColor, &mult_color);
                    self.mult_color = Some(mult_color);
                }
                if Some(add_color) != self.add_color {
                    program.uniform4fv(&self.gl, ShaderUniform::AddColor, &add_color);
                    self.add_color = Some(add_color);
                }

                // Set shader specific uniforms.
                match &draw.draw_type {
                    DrawType::Color => (),
                    DrawType::Gradient(gradient) => {
                        program.uniform_matrix3fv(
                            &self.gl,
                            ShaderUniform::TextureMatrix,
                            &gradient.matrix,
                        );
                        program.uniform1i(
                            &self.gl,
                            ShaderUniform::GradientType,
                            gradient.gradient_type,
                        );
                        program.uniform1fv(
                            &self.gl,
                            ShaderUniform::GradientRatios,
                            &gradient.ratios,
                        );
                        program.uniform4fv(
                            &self.gl,
                            ShaderUniform::GradientColors,
                            bytemuck::cast_slice(&gradient.colors),
                        );
                        program.uniform1i(
                            &self.gl,
                            ShaderUniform::GradientRepeatMode,
                            gradient.repeat_mode,
                        );
                        program.uniform1f(
                            &self.gl,
                            ShaderUniform::GradientFocalPoint,
                            gradient.focal_point,
                        );
                        program.uniform1i(
                            &self.gl,
                            ShaderUniform::GradientInterpolation,
                            (gradient.interpolation == swf::GradientInterpolation::LinearRgb)
                                as i32,
                        );
                    }
                    DrawType::Bitmap(bitmap) => {
                        let texture = match &bitmap.handle {
                            Some(handle) => &as_registry_data(handle).texture,
                            None => {
                                log::warn!("Tried to render a handleless bitmap");
                                continue;
                            }
                        };

                        program.uniform_matrix3fv(
                            &self.gl,
                            ShaderUniform::TextureMatrix,
                            &bitmap.matrix,
                        );

                        // Bind texture.
                        self.gl.active_texture(glow::TEXTURE0);
                        self.gl.bind_texture(glow::TEXTURE_2D, Some(*texture));
                        program.uniform1i(&self.gl, ShaderUniform::BitmapTexture, 0);

                        // Set texture parameters.
                        let filter = if bitmap.is_smoothed {
                            glow::LINEAR as i32
                        } else {
                            glow::NEAREST as i32
                        };
                        self.gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_MAG_FILTER,
                            filter,
                        );
                        self.gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_MIN_FILTER,
                            filter,
                        );
                        // On WebGL1, you are unable to change the wrapping parameter of non-power-of-2 textures.
                        let wrap = if self.gl2 && bitmap.is_repeating {
                            glow::REPEAT as i32
                        } else {
                            glow::CLAMP_TO_EDGE as i32
                        };
                        self.gl
                            .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap);
                        self.gl
                            .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap);
                    }
                }

                // Draw the triangles.
                self.gl
                    .draw_elements(glow::TRIANGLES, num_indices, glow::UNSIGNED_INT, 0);
            }
        }
    }

    fn render_stage3d(&mut self, _bitmap: BitmapHandle, _transform: Transform) {
        panic!("Stage3D should not have been created on WebGL backend")
    }

    fn draw_rect(&mut self, color: Color, matrix: Matrix) {
        self.draw_quad::<{ glow::TRIANGLE_FAN }, -1>(color, matrix)
    }

    fn draw_line(&mut self, color: Color, mut matrix: Matrix) {
        matrix.tx += Twips::HALF_PX;
        matrix.ty += Twips::HALF_PX;
        self.draw_quad::<{ glow::LINE_STRIP }, 2>(color, matrix)
    }

    fn draw_line_rect(&mut self, color: Color, mut matrix: Matrix) {
        matrix.tx += Twips::HALF_PX;
        matrix.ty += Twips::HALF_PX;
        self.draw_quad::<{ glow::LINE_LOOP }, -1>(color, matrix)
    }

    fn push_mask(&mut self) {
        debug_assert!(
            self.mask_state == MaskState::NoMask || self.mask_state == MaskState::DrawMaskedContent
        );
        self.num_masks += 1;
        self.mask_state = MaskState::DrawMaskStencil;
        self.mask_state_dirty = true;
    }

    fn activate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskStencil);
        self.mask_state = MaskState::DrawMaskedContent;
        self.mask_state_dirty = true;
    }

    fn deactivate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskedContent);
        self.mask_state = MaskState::ClearMaskStencil;
        self.mask_state_dirty = true;
    }

    fn pop_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::ClearMaskStencil);
        self.num_masks -= 1;
        self.mask_state = if self.num_masks == 0 {
            MaskState::NoMask
        } else {
            MaskState::DrawMaskedContent
        };
        self.mask_state_dirty = true;
    }

    fn blend(&mut self, commands: CommandList, blend: RenderBlendMode) {
        self.push_blend_mode(blend);
        commands.execute(self);
        self.pop_blend_mode();
    }

    fn render_alpha_mask(&mut self, maskee_commands: CommandList, _mask_commands: CommandList) {
        // TODO Add support for alpha masks
        maskee_commands.execute(self);
    }
}

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

impl Gradient {
    fn new(gradient: TessGradient, matrix: [[f32; 3]; 3]) -> Self {
        // TODO: Support more than MAX_GRADIENT_COLORS.
        let num_colors = gradient.records.len().min(MAX_GRADIENT_COLORS);
        let mut ratios = [0.0; MAX_GRADIENT_COLORS];
        let mut colors = [[0.0; 4]; MAX_GRADIENT_COLORS];
        for i in 0..num_colors {
            let record = &gradient.records[i];
            let mut color = [
                f32::from(record.color.r) / 255.0,
                f32::from(record.color.g) / 255.0,
                f32::from(record.color.b) / 255.0,
                f32::from(record.color.a) / 255.0,
            ];
            // Convert to linear color space if this is a linear-interpolated gradient.
            match gradient.interpolation {
                swf::GradientInterpolation::Rgb => {}
                swf::GradientInterpolation::LinearRgb => srgb_to_linear(&mut color),
            }

            colors[i] = color;
            ratios[i] = f32::from(record.ratio) / 255.0;
        }

        for i in num_colors..MAX_GRADIENT_COLORS {
            ratios[i] = ratios[i - 1];
            colors[i] = colors[i - 1];
        }

        Self {
            matrix,
            gradient_type: match gradient.gradient_type {
                GradientType::Linear => 0,
                GradientType::Radial => 1,
                GradientType::Focal => 2,
            },
            ratios,
            colors,
            repeat_mode: match gradient.repeat_mode {
                swf::GradientSpread::Pad => 0,
                swf::GradientSpread::Repeat => 1,
                swf::GradientSpread::Reflect => 2,
            },
            focal_point: gradient.focal_point.to_f32().clamp(-0.98, 0.98),
            interpolation: gradient.interpolation,
        }
    }
}

#[derive(Clone, Debug)]
struct BitmapDraw {
    matrix: [[f32; 3]; 3],
    handle: Option<BitmapHandle>,
    is_repeating: bool,
    is_smoothed: bool,
}

#[derive(Debug)]
struct Mesh {
    gl2: Arc<glow::Context>,
    draws: Vec<Draw>,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            for draw in &self.draws {
                self.gl2.delete_vertex_array(draw.vao);
            }
        }
    }
}

impl ShapeHandleImpl for Mesh {}

fn as_mesh(handle: &ShapeHandle) -> &Mesh {
    <dyn Any>::downcast_ref(&*handle.0).expect("Shape handle must be a WebGL ShapeData")
}

#[derive(Debug)]
struct Buffer {
    gl: Arc<glow::Context>,
    buffer: glow::Buffer,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.buffer);
        }
    }
}

#[derive(Debug)]
struct Draw {
    draw_type: DrawType,
    #[expect(dead_code)]
    vertex_buffer: Buffer,
    #[expect(dead_code)]
    index_buffer: Buffer,
    vao: glow::VertexArray,
    num_indices: i32,
    num_mask_indices: i32,
}

#[derive(Debug)]
enum DrawType {
    Color,
    Gradient(Box<Gradient>),
    Bitmap(BitmapDraw),
}

struct MsaaBuffers {
    color_renderbuffer: glow::Renderbuffer,
    stencil_renderbuffer: glow::Renderbuffer,
    render_framebuffer: glow::Framebuffer,
    color_framebuffer: glow::Framebuffer,
    framebuffer_texture: glow::Texture,
}

// Because the shaders are currently simple and few in number, we are using a
// straightforward shader model. We maintain an enum of every possible uniform,
// and each shader tries to grab the location of each uniform.
struct ShaderProgram {
    program: glow::Program,
    uniforms: [Option<glow::UniformLocation>; NUM_UNIFORMS],
    vertex_position_location: u32,
    vertex_color_location: u32,
    num_vertex_attributes: u32,
}

// These should match the uniform names in the shaders.
const NUM_UNIFORMS: usize = 12;
const UNIFORM_NAMES: [&str; NUM_UNIFORMS] = [
    "world_matrix",
    "view_matrix",
    "mult_color",
    "add_color",
    "u_matrix",
    "u_gradient_type",
    "u_ratios",
    "u_colors",
    "u_repeat_mode",
    "u_focal_point",
    "u_interpolation",
    "u_texture",
];

enum ShaderUniform {
    WorldMatrix = 0,
    ViewMatrix,
    MultColor,
    AddColor,
    TextureMatrix,
    GradientType,
    GradientRatios,
    GradientColors,
    GradientRepeatMode,
    GradientFocalPoint,
    GradientInterpolation,
    BitmapTexture,
}

impl ShaderProgram {
    fn new(
        gl: &glow::Context,
        vertex_shader: glow::Shader,
        fragment_shader: glow::Shader,
    ) -> Result<Self, Error> {
        unsafe {
            let program = gl
                .create_program()
                .map_err(|_| Error::UnableToCreateProgram)?;
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);

            gl.link_program(program);
            if !gl.get_program_parameter_i32(program, glow::LINK_STATUS) != 0 {
                let msg = format!(
                    "Error linking shader program: {:?}",
                    gl.get_program_info_log(program)
                );
                log::error!("{msg}");
                return Err(Error::LinkingShaderProgram(msg));
            }

            // Find uniforms.
            let mut uniforms: [Option<glow::UniformLocation>; NUM_UNIFORMS] = Default::default();
            for i in 0..NUM_UNIFORMS {
                uniforms[i] = gl.get_uniform_location(program, UNIFORM_NAMES[i]);
            }

            let vertex_position_location = gl
                .get_attrib_location(program, "position")
                .unwrap_or(0xffff_ffff);
            let vertex_color_location = gl
                .get_attrib_location(program, "color")
                .unwrap_or(0xffff_ffff);
            let num_vertex_attributes = if vertex_position_location != 0xffff_ffff {
                1
            } else {
                0
            } + if vertex_color_location != 0xffff_ffff {
                1
            } else {
                0
            };

            Ok(ShaderProgram {
                program,
                uniforms,
                vertex_position_location,
                vertex_color_location,
                num_vertex_attributes,
            })
        }
    }

    fn uniform1f(&self, gl: &glow::Context, uniform: ShaderUniform, value: f32) {
        unsafe {
            gl.uniform_1_f32(self.uniforms[uniform as usize].as_ref(), value);
        }
    }

    fn uniform1fv(&self, gl: &glow::Context, uniform: ShaderUniform, values: &[f32]) {
        unsafe {
            gl.uniform_1_f32_slice(self.uniforms[uniform as usize].as_ref(), values);
        }
    }

    fn uniform1i(&self, gl: &glow::Context, uniform: ShaderUniform, value: i32) {
        unsafe {
            gl.uniform_1_i32(self.uniforms[uniform as usize].as_ref(), value);
        }
    }

    fn uniform4fv(&self, gl: &glow::Context, uniform: ShaderUniform, values: &[f32]) {
        unsafe {
            gl.uniform_4_f32_slice(self.uniforms[uniform as usize].as_ref(), values);
        }
    }

    fn uniform_matrix3fv(
        &self,
        gl: &glow::Context,
        uniform: ShaderUniform,
        values: &[[f32; 3]; 3],
    ) {
        unsafe {
            gl.uniform_matrix_3_f32_slice(
                self.uniforms[uniform as usize].as_ref(),
                false,
                bytemuck::cast_slice(values),
            );
        }
    }

    fn uniform_matrix4fv(
        &self,
        gl: &glow::Context,
        uniform: ShaderUniform,
        values: &[[f32; 4]; 4],
    ) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(
                self.uniforms[uniform as usize].as_ref(),
                false,
                bytemuck::cast_slice(values),
            );
        }
    }
}

trait GlExt {
    fn check_error(&self, error_msg: &'static str) -> Result<(), Error>;
}

impl GlExt for glow::Context {
    /// Check if GL returned an error for the previous operation.
    fn check_error(&self, error_msg: &'static str) -> Result<(), Error> {
        unsafe {
            match self.get_error() {
                glow::NO_ERROR => Ok(()),
                error => Err(Error::GLError(error_msg, error)),
            }
        }
    }
}

/// Converts an RGBA color from sRGB space to linear color space.
fn srgb_to_linear(color: &mut [f32; 4]) {
    for n in &mut color[..3] {
        *n = if *n <= 0.04045 {
            *n / 12.92
        } else {
            f32::powf((*n + 0.055) / 1.055, 2.4)
        };
    }
}
