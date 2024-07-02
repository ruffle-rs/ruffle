#![deny(clippy::unwrap_used)]
// Remove this when we start using `Rc` when compiling for wasm
#![allow(clippy::arc_with_non_send_sync)]

use bytemuck::{Pod, Zeroable};
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
use ruffle_web_common::{JsError, JsResult};
use std::borrow::Cow;
use std::sync::Arc;
use swf::{BlendMode, Color, Twips};
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, OesVertexArrayObject, WebGl2RenderingContext as Gl2, WebGlBuffer,
    WebGlFramebuffer, WebGlProgram, WebGlRenderbuffer, WebGlRenderingContext as Gl, WebGlShader,
    WebGlTexture, WebGlUniformLocation, WebGlVertexArrayObject, WebglDebugRendererInfo,
};

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

    #[error("Javascript error: {0}")]
    JavascriptError(#[from] JsError),

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
    /// WebGL1 context
    gl: Gl,

    // WebGL2 context, if available.
    gl2: Option<Gl2>,

    /// In WebGL1, VAOs are only available as an extension.
    vao_ext: OesVertexArrayObject,

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
    gl: Gl,
    width: u32,
    height: u32,
    texture: WebGlTexture,
}

impl Drop for RegistryData {
    fn drop(&mut self) {
        self.gl.delete_texture(Some(&self.texture));
    }
}

impl BitmapHandleImpl for RegistryData {}

fn as_registry_data(handle: &BitmapHandle) -> &RegistryData {
    <dyn BitmapHandleImpl>::downcast_ref(&*handle.0)
        .expect("Bitmap handle must be webgl RegistryData")
}

const MAX_GRADIENT_COLORS: usize = 15;

impl WebGlRenderBackend {
    pub fn new(
        canvas: &HtmlCanvasElement,
        is_transparent: bool,
        quality: StageQuality,
    ) -> Result<Self, Error> {
        // Create WebGL context.
        let options = [
            ("stencil", JsValue::TRUE),
            ("alpha", JsValue::from_bool(is_transparent)),
            ("antialias", JsValue::FALSE),
            ("depth", JsValue::FALSE),
            ("failIfMajorPerformanceCaveat", JsValue::TRUE), // fail if no GPU available
            ("premultipliedAlpha", JsValue::TRUE),
        ];
        let context_options = js_sys::Object::new();
        for (name, value) in options.into_iter() {
            js_sys::Reflect::set(&context_options, &JsValue::from(name), &value).warn_on_error();
        }

        // Attempt to create a WebGL2 context, but fall back to WebGL1 if unavailable.
        let (gl, gl2, vao_ext, msaa_sample_count) = if let Ok(Some(gl)) =
            canvas.get_context_with_context_options("webgl2", &context_options)
        {
            log::info!("Creating WebGL2 context.");
            let gl2 = gl
                .dyn_into::<Gl2>()
                .map_err(|_| Error::CantCreateGLContext)?;

            // Determine MSAA sample count.
            let mut msaa_sample_count = quality.sample_count().min(4);

            // Ensure that we don't exceed the max MSAA of this device.
            if let Ok(max_samples) = gl2.get_parameter(Gl2::MAX_SAMPLES) {
                let max_samples = max_samples.as_f64().unwrap_or(0.0) as u32;
                if max_samples > 0 && max_samples < msaa_sample_count {
                    log::info!("Device only supports {}xMSAA", max_samples);
                    msaa_sample_count = max_samples;
                }
            }

            // WebGLRenderingContext inherits from WebGL2RenderingContext, so cast it down.
            (
                gl2.clone().unchecked_into::<Gl>(),
                Some(gl2),
                JsValue::UNDEFINED.unchecked_into(),
                msaa_sample_count,
            )
        } else {
            // Fall back to WebGL1.
            // Request antialiasing on WebGL1, because there isn't general MSAA support.
            js_sys::Reflect::set(
                &context_options,
                &JsValue::from("antialias"),
                &JsValue::TRUE,
            )
            .warn_on_error();

            if let Ok(Some(gl)) = canvas.get_context_with_context_options("webgl", &context_options)
            {
                log::info!("Falling back to WebGL1.");

                let gl = gl
                    .dyn_into::<Gl>()
                    .map_err(|_| Error::CantCreateGLContext)?;
                // `dyn_into` doesn't work here; why?
                let vao = gl
                    .get_extension("OES_vertex_array_object")
                    .into_js_result()?
                    .ok_or(Error::VAOExtensionNotFound)?
                    .unchecked_into::<OesVertexArrayObject>();

                // On WebGL1, we need to explicitly request support for u32 index buffers.
                let _ext = gl
                    .get_extension("OES_element_index_uint")
                    .into_js_result()?
                    .ok_or(Error::OESExtensionNotFound)?;
                (gl, None, vao, 1)
            } else {
                return Err(Error::CantCreateGLContext);
            }
        };

        if log::log_enabled!(log::Level::Info) {
            // Get WebGL driver info.
            let driver_info = gl
                .get_extension("WEBGL_debug_renderer_info")
                .and_then(|_| gl.get_parameter(WebglDebugRendererInfo::UNMASKED_RENDERER_WEBGL))
                .ok()
                .and_then(|val| val.as_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            log::info!("WebGL graphics driver: {}", driver_info);
        }

        let color_vertex = Self::compile_shader(&gl, Gl::VERTEX_SHADER, COLOR_VERTEX_GLSL)?;
        let texture_vertex = Self::compile_shader(&gl, Gl::VERTEX_SHADER, TEXTURE_VERTEX_GLSL)?;
        let color_fragment = Self::compile_shader(&gl, Gl::FRAGMENT_SHADER, COLOR_FRAGMENT_GLSL)?;
        let bitmap_fragment = Self::compile_shader(&gl, Gl::FRAGMENT_SHADER, BITMAP_FRAGMENT_GLSL)?;
        let gradient_fragment =
            Self::compile_shader(&gl, Gl::FRAGMENT_SHADER, GRADIENT_FRAGMENT_GLSL)?;

        let color_program = ShaderProgram::new(&gl, &color_vertex, &color_fragment)?;
        let bitmap_program = ShaderProgram::new(&gl, &texture_vertex, &bitmap_fragment)?;
        let gradient_program = ShaderProgram::new(&gl, &texture_vertex, &gradient_fragment)?;

        gl.enable(Gl::BLEND);

        // Necessary to load RGB textures (alignment defaults to 4).
        gl.pixel_storei(Gl::UNPACK_ALIGNMENT, 1);

        let mut renderer = Self {
            gl,
            gl2,
            vao_ext,

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

    fn build_quad_mesh(&self, program: &ShaderProgram) -> Result<Vec<Draw>, Error> {
        let vao = self.create_vertex_array()?;

        let vertex_buffer = self.gl.create_buffer().ok_or(Error::UnableToCreateBuffer)?;
        self.gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));
        self.gl.buffer_data_with_u8_array(
            Gl::ARRAY_BUFFER,
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
            Gl::STATIC_DRAW,
        );

        let index_buffer = self.gl.create_buffer().ok_or(Error::UnableToCreateBuffer)?;
        self.gl
            .bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        self.gl.buffer_data_with_u8_array(
            Gl::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&[0u32, 1, 2, 3]),
            Gl::STATIC_DRAW,
        );

        if program.vertex_position_location != 0xffff_ffff {
            self.gl.vertex_attrib_pointer_with_i32(
                program.vertex_position_location,
                2,
                Gl::FLOAT,
                false,
                12,
                0,
            );
            self.gl
                .enable_vertex_attrib_array(program.vertex_position_location);
        }

        if program.vertex_color_location != 0xffff_ffff {
            self.gl.vertex_attrib_pointer_with_i32(
                program.vertex_color_location,
                4,
                Gl::UNSIGNED_BYTE,
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

    fn compile_shader(gl: &Gl, shader_type: u32, glsl_src: &str) -> Result<WebGlShader, Error> {
        let shader = gl
            .create_shader(shader_type)
            .ok_or(Error::UnableToCreateShader)?;
        gl.shader_source(&shader, glsl_src);
        gl.compile_shader(&shader);
        if log::log_enabled!(log::Level::Error) {
            let log = gl.get_shader_info_log(&shader).unwrap_or_default();
            if !log.is_empty() {
                log::error!("{}", log);
            }
        }
        Ok(shader)
    }

    fn build_msaa_buffers(&mut self) -> Result<(), Error> {
        if self.gl2.is_none() || self.msaa_sample_count <= 1 {
            self.gl.bind_framebuffer(Gl::FRAMEBUFFER, None);
            self.gl.bind_renderbuffer(Gl::RENDERBUFFER, None);
            return Ok(());
        }

        let gl = self.gl2.as_ref().expect("gl2 must exist at this point");

        // Delete previous buffers, if they exist.
        if let Some(msaa_buffers) = self.msaa_buffers.take() {
            gl.delete_renderbuffer(Some(&msaa_buffers.color_renderbuffer));
            gl.delete_renderbuffer(Some(&msaa_buffers.stencil_renderbuffer));
            gl.delete_framebuffer(Some(&msaa_buffers.render_framebuffer));
            gl.delete_framebuffer(Some(&msaa_buffers.color_framebuffer));
            gl.delete_texture(Some(&msaa_buffers.framebuffer_texture));
        }

        // Create frame and render buffers.
        let render_framebuffer = gl
            .create_framebuffer()
            .ok_or(Error::UnableToCreateFrameBuffer)?;
        let color_framebuffer = gl
            .create_framebuffer()
            .ok_or(Error::UnableToCreateFrameBuffer)?;

        // Note for future self:
        // Whenever we support playing transparent movies,
        // switch this to RGBA and probably need to change shaders to all
        // be premultiplied alpha.
        let color_renderbuffer = gl
            .create_renderbuffer()
            .ok_or(Error::UnableToCreateRenderBuffer)?;
        gl.bind_renderbuffer(Gl2::RENDERBUFFER, Some(&color_renderbuffer));
        gl.renderbuffer_storage_multisample(
            Gl2::RENDERBUFFER,
            self.msaa_sample_count as i32,
            Gl2::RGBA8,
            self.renderbuffer_width,
            self.renderbuffer_height,
        );
        gl.check_error("renderbuffer_storage_multisample (color)")?;

        let stencil_renderbuffer = gl
            .create_renderbuffer()
            .ok_or(Error::UnableToCreateFrameBuffer)?;
        gl.bind_renderbuffer(Gl2::RENDERBUFFER, Some(&stencil_renderbuffer));
        gl.renderbuffer_storage_multisample(
            Gl2::RENDERBUFFER,
            self.msaa_sample_count as i32,
            Gl2::STENCIL_INDEX8,
            self.renderbuffer_width,
            self.renderbuffer_height,
        );
        gl.check_error("renderbuffer_storage_multisample (stencil)")?;

        gl.bind_framebuffer(Gl2::FRAMEBUFFER, Some(&render_framebuffer));
        gl.framebuffer_renderbuffer(
            Gl2::FRAMEBUFFER,
            Gl2::COLOR_ATTACHMENT0,
            Gl2::RENDERBUFFER,
            Some(&color_renderbuffer),
        );
        gl.framebuffer_renderbuffer(
            Gl2::FRAMEBUFFER,
            Gl2::STENCIL_ATTACHMENT,
            Gl2::RENDERBUFFER,
            Some(&stencil_renderbuffer),
        );

        let framebuffer_texture = gl.create_texture().ok_or(Error::UnableToCreateTexture)?;
        gl.bind_texture(Gl2::TEXTURE_2D, Some(&framebuffer_texture));
        gl.tex_parameteri(Gl2::TEXTURE_2D, Gl2::TEXTURE_MAG_FILTER, Gl::NEAREST as i32);
        gl.tex_parameteri(Gl2::TEXTURE_2D, Gl2::TEXTURE_MIN_FILTER, Gl::NEAREST as i32);
        gl.tex_parameteri(
            Gl2::TEXTURE_2D,
            Gl2::TEXTURE_WRAP_S,
            Gl::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            Gl2::TEXTURE_2D,
            Gl2::TEXTURE_WRAP_T,
            Gl2::CLAMP_TO_EDGE as i32,
        );
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            Gl2::TEXTURE_2D,
            0,
            Gl2::RGBA as i32,
            self.renderbuffer_width,
            self.renderbuffer_height,
            0,
            Gl2::RGBA,
            Gl2::UNSIGNED_BYTE,
            None,
        )
        .into_js_result()?;
        gl.bind_texture(Gl2::TEXTURE_2D, None);

        gl.bind_framebuffer(Gl2::FRAMEBUFFER, Some(&color_framebuffer));
        gl.framebuffer_texture_2d(
            Gl2::FRAMEBUFFER,
            Gl2::COLOR_ATTACHMENT0,
            Gl2::TEXTURE_2D,
            Some(&framebuffer_texture),
            0,
        );
        gl.bind_framebuffer(Gl2::FRAMEBUFFER, None);

        self.msaa_buffers = Some(MsaaBuffers {
            color_renderbuffer,
            stencil_renderbuffer,
            render_framebuffer,
            color_framebuffer,
            framebuffer_texture,
        });

        Ok(())
    }

    fn register_shape_internal(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> Result<Vec<Draw>, Error> {
        use ruffle_render::tessellator::DrawType as TessDrawType;

        let lyon_mesh = self
            .shape_tessellator
            .tessellate_shape(shape, bitmap_source);

        let mut draws = Vec::with_capacity(lyon_mesh.draws.len());
        for draw in lyon_mesh.draws {
            let num_indices = draw.indices.len() as i32;
            let num_mask_indices = draw.mask_index_count as i32;

            let vao = self.create_vertex_array()?;
            let vertex_buffer = self.gl.create_buffer().ok_or(Error::UnableToCreateBuffer)?;
            self.gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));

            let vertices: Vec<_> = draw.vertices.into_iter().map(Vertex::from).collect();
            self.gl.buffer_data_with_u8_array(
                Gl::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertices),
                Gl::STATIC_DRAW,
            );

            let index_buffer = self.gl.create_buffer().ok_or(Error::UnableToCreateBuffer)?;
            self.gl
                .bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
            self.gl.buffer_data_with_u8_array(
                Gl::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&draw.indices),
                Gl::STATIC_DRAW,
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
                self.gl.vertex_attrib_pointer_with_i32(
                    program.vertex_position_location,
                    2,
                    Gl::FLOAT,
                    false,
                    12,
                    0,
                );
                self.gl
                    .enable_vertex_attrib_array(program.vertex_position_location);
            }

            if program.vertex_color_location != 0xffff_ffff {
                self.gl.vertex_attrib_pointer_with_i32(
                    program.vertex_color_location,
                    4,
                    Gl::UNSIGNED_BYTE,
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

    /// Creates and binds a new VAO.
    fn create_vertex_array(&self) -> Result<WebGlVertexArrayObject, Error> {
        let vao = if let Some(gl2) = &self.gl2 {
            let vao = gl2.create_vertex_array().ok_or(Error::UnableToCreateVAO)?;
            gl2.bind_vertex_array(Some(&vao));
            vao
        } else {
            let vao = self
                .vao_ext
                .create_vertex_array_oes()
                .ok_or(Error::UnableToCreateVAO)?;
            self.vao_ext.bind_vertex_array_oes(Some(&vao));
            vao
        };
        Ok(vao)
    }

    /// Binds a VAO.
    fn bind_vertex_array(&self, vao: Option<&WebGlVertexArrayObject>) {
        if let Some(gl2) = &self.gl2 {
            gl2.bind_vertex_array(vao);
        } else {
            self.vao_ext.bind_vertex_array_oes(vao);
        };
    }

    fn set_stencil_state(&mut self) {
        // Set stencil state for masking, if necessary.
        if self.mask_state_dirty {
            match self.mask_state {
                MaskState::NoMask => {
                    self.gl.disable(Gl::STENCIL_TEST);
                    self.gl.color_mask(true, true, true, true);
                }
                MaskState::DrawMaskStencil => {
                    self.gl.enable(Gl::STENCIL_TEST);
                    self.gl
                        .stencil_func(Gl::EQUAL, (self.num_masks - 1) as i32, 0xff);
                    self.gl.stencil_op(Gl::KEEP, Gl::KEEP, Gl::INCR);
                    self.gl.color_mask(false, false, false, false);
                }
                MaskState::DrawMaskedContent => {
                    self.gl.enable(Gl::STENCIL_TEST);
                    self.gl.stencil_func(Gl::EQUAL, self.num_masks as i32, 0xff);
                    self.gl.stencil_op(Gl::KEEP, Gl::KEEP, Gl::KEEP);
                    self.gl.color_mask(true, true, true, true);
                }
                MaskState::ClearMaskStencil => {
                    self.gl.enable(Gl::STENCIL_TEST);
                    self.gl.stencil_func(Gl::EQUAL, self.num_masks as i32, 0xff);
                    self.gl.stencil_op(Gl::KEEP, Gl::KEEP, Gl::DECR);
                    self.gl.color_mask(false, false, false, false);
                }
            }
        }
    }

    fn apply_blend_mode(&mut self, mode: RenderBlendMode) {
        let (blend_op, src_rgb, dst_rgb) = match mode {
            RenderBlendMode::Builtin(BlendMode::Normal) => {
                // src + (1-a)
                (Gl::FUNC_ADD, Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA)
            }
            RenderBlendMode::Builtin(BlendMode::Add) => {
                // src + dst
                (Gl::FUNC_ADD, Gl::ONE, Gl::ONE)
            }
            RenderBlendMode::Builtin(BlendMode::Subtract) => {
                // dst - src
                (Gl::FUNC_REVERSE_SUBTRACT, Gl::ONE, Gl::ONE)
            }
            _ => {
                // TODO: Unsupported blend mode. Default to normal for now.
                (Gl::FUNC_ADD, Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA)
            }
        };
        self.gl.blend_equation_separate(blend_op, Gl::FUNC_ADD);
        self.gl
            .blend_func_separate(src_rgb, dst_rgb, Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA);
    }

    fn begin_frame(&mut self, clear: Color) {
        self.active_program = std::ptr::null();
        self.mask_state = MaskState::NoMask;
        self.num_masks = 0;
        self.mask_state_dirty = true;

        self.mult_color = None;
        self.add_color = None;

        // Bind to MSAA render buffer if using MSAA.
        if let Some(msaa_buffers) = &self.msaa_buffers {
            let gl = &self.gl;
            gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&msaa_buffers.render_framebuffer));
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
        self.gl.clear(Gl::COLOR_BUFFER_BIT | Gl::STENCIL_BUFFER_BIT);
    }

    fn end_frame(&mut self) {
        // Resolve MSAA, if we're using it (WebGL2).
        if let (Some(ref gl), Some(ref msaa_buffers)) = (&self.gl2, &self.msaa_buffers) {
            // Disable any remaining masking state.
            self.gl.disable(Gl::STENCIL_TEST);
            self.gl.color_mask(true, true, true, true);

            // Resolve the MSAA in the render buffer.
            gl.bind_framebuffer(
                Gl2::READ_FRAMEBUFFER,
                Some(&msaa_buffers.render_framebuffer),
            );
            gl.bind_framebuffer(Gl2::DRAW_FRAMEBUFFER, Some(&msaa_buffers.color_framebuffer));
            gl.blit_framebuffer(
                0,
                0,
                self.renderbuffer_width,
                self.renderbuffer_height,
                0,
                0,
                self.renderbuffer_width,
                self.renderbuffer_height,
                Gl2::COLOR_BUFFER_BIT,
                Gl2::NEAREST,
            );

            // Render the resolved framebuffer texture to a quad on the screen.
            gl.bind_framebuffer(Gl2::FRAMEBUFFER, None);

            self.gl.viewport(
                0,
                0,
                self.gl.drawing_buffer_width(),
                self.gl.drawing_buffer_height(),
            );

            let program = &self.bitmap_program;
            self.gl.use_program(Some(&program.program));

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
            self.gl.active_texture(Gl2::TEXTURE0);
            self.gl
                .bind_texture(Gl2::TEXTURE_2D, Some(&msaa_buffers.framebuffer_texture));
            program.uniform1i(&self.gl, ShaderUniform::BitmapTexture, 0);

            // Render the quad.
            let quad = &self.bitmap_quad_draws;
            self.bind_vertex_array(Some(&quad[0].vao));
            self.gl.draw_elements_with_i32(
                Gl::TRIANGLE_FAN,
                quad[0].num_indices,
                Gl::UNSIGNED_INT,
                0,
            );
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
        if program as *const ShaderProgram != self.active_program {
            self.gl.use_program(Some(&program.program));
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
        self.bind_vertex_array(Some(&quad[0].vao));

        let count = if COUNT < 0 {
            quad[0].num_indices
        } else {
            COUNT
        };
        self.gl
            .draw_elements_with_i32(MODE, count, Gl::UNSIGNED_INT, 0);
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
        self.renderbuffer_width =
            (dimensions.width.max(1) as i32).min(self.gl.drawing_buffer_width());
        self.renderbuffer_height =
            (dimensions.height.max(1) as i32).min(self.gl.drawing_buffer_height());

        // Recreate framebuffers with the new size.
        let _ = self.build_msaa_buffers();
        self.gl
            .viewport(0, 0, self.renderbuffer_width, self.renderbuffer_height);
        self.viewport_scale_factor = dimensions.scale_factor
    }

    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        let mesh = match self.register_shape_internal(shape, bitmap_source) {
            Ok(draws) => Mesh {
                draws,
                gl2: self.gl2.clone(),
                vao_ext: self.vao_ext.clone(),
            },
            Err(e) => {
                log::error!("Couldn't register shape: {:?}", e);
                Mesh {
                    draws: vec![],
                    gl2: self.gl2.clone(),
                    vao_ext: self.vao_ext.clone(),
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

    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, BitmapError> {
        let (format, bitmap) = match bitmap.format() {
            BitmapFormat::Rgb | BitmapFormat::Yuv420p => (Gl::RGB, bitmap.to_rgb()),
            BitmapFormat::Rgba | BitmapFormat::Yuva420p => (Gl::RGBA, bitmap.to_rgba()),
        };

        let texture = self
            .gl
            .create_texture()
            .ok_or_else(|| BitmapError::JavascriptError("Unable to create texture".into()))?;
        self.gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                format as i32,
                bitmap.width() as i32,
                bitmap.height() as i32,
                0,
                format,
                Gl::UNSIGNED_BYTE,
                Some(bitmap.data()),
            )
            .into_js_result()
            .map_err(|e| BitmapError::JavascriptError(e.into()))?;

        // You must set the texture parameters for non-power-of-2 textures to function in WebGL1.
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);

        Ok(BitmapHandle(Arc::new(RegistryData {
            gl: self.gl.clone(),
            width: bitmap.width(),
            height: bitmap.height(),
            texture,
        })))
    }

    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap,
        _region: PixelRegion,
    ) -> Result<(), BitmapError> {
        let texture = &as_registry_data(handle).texture;

        self.gl.bind_texture(Gl::TEXTURE_2D, Some(texture));

        let (format, bitmap) = match bitmap.format() {
            BitmapFormat::Rgb | BitmapFormat::Yuv420p => (Gl::RGB, bitmap.to_rgb()),
            BitmapFormat::Rgba | BitmapFormat::Yuva420p => (Gl::RGBA, bitmap.to_rgba()),
        };

        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                format as i32,
                bitmap.width() as i32,
                bitmap.height() as i32,
                0,
                format,
                Gl::UNSIGNED_BYTE,
                Some(bitmap.data()),
            )
            .into_js_result()
            .map_err(|e| BitmapError::JavascriptError(e.into()))?;

        Ok(())
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
        let mut result = vec![];

        if self.gl2.is_some() {
            result.push("Renderer: WebGL 2.0".to_string());
        } else {
            result.push("Renderer: WebGL 1.0".to_string());
        }

        let mut add_line = |name, val: Result<JsValue, JsValue>| {
            result.push(format!(
                "{name}: {}",
                val.ok()
                    .and_then(|val| val.as_string())
                    .unwrap_or_else(|| "<unknown>".to_string())
            ))
        };

        add_line("Adapter Vendor", self.gl.get_parameter(Gl::VENDOR));
        add_line("Adapter Renderer", self.gl.get_parameter(Gl::RENDERER));
        add_line("Adapter Version", self.gl.get_parameter(Gl::VERSION));

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
        _arguments: &[ruffle_render::pixel_bender::PixelBenderShaderArgument],
        _target: &PixelBenderTarget,
    ) -> Result<PixelBenderOutput, BitmapError> {
        Err(BitmapError::Unimplemented("run_pixelbender_shader".into()))
    }

    fn create_empty_texture(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<BitmapHandle, BitmapError> {
        let texture = self
            .gl
            .create_texture()
            .ok_or_else(|| BitmapError::JavascriptError("Unable to create texture".into()))?;
        self.gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));

        // You must set the texture parameters for non-power-of-2 textures to function in WebGL1.
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);

        Ok(BitmapHandle(Arc::new(RegistryData {
            gl: self.gl.clone(),
            width,
            height,
            texture,
        })))
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
        self.set_stencil_state();
        let entry = as_registry_data(&bitmap);
        // Adjust the quad draw to use the target bitmap.
        let quad = &self.bitmap_quad_draws;
        let draw = &quad[0];
        let bitmap_matrix = if let DrawType::Bitmap(BitmapDraw { matrix, .. }) = &draw.draw_type {
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

        self.bind_vertex_array(Some(&draw.vao));

        let program = &self.bitmap_program;

        // Set common render state, while minimizing unnecessary state changes.
        // TODO: Using designated layout specifiers in WebGL2/OpenGL ES 3, we could guarantee that uniforms
        // are in the same location between shaders, and avoid changing them unless necessary.
        if program as *const ShaderProgram != self.active_program {
            self.gl.use_program(Some(&program.program));
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
        self.gl.active_texture(Gl::TEXTURE0);
        self.gl.bind_texture(Gl::TEXTURE_2D, Some(&entry.texture));
        program.uniform1i(&self.gl, ShaderUniform::BitmapTexture, 0);

        // Set texture parameters.
        let filter = if smoothing {
            Gl::LINEAR as i32
        } else {
            Gl::NEAREST as i32
        };
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, filter);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, filter);

        let wrap = Gl::CLAMP_TO_EDGE as i32;
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, wrap);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, wrap);

        // Draw the triangles.
        self.gl
            .draw_elements_with_i32(Gl::TRIANGLE_FAN, draw.num_indices, Gl::UNSIGNED_INT, 0);
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform) {
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

            self.bind_vertex_array(Some(&draw.vao));

            let program = match &draw.draw_type {
                DrawType::Color => &self.color_program,
                DrawType::Gradient(_) => &self.gradient_program,
                DrawType::Bitmap { .. } => &self.bitmap_program,
            };

            // Set common render state, while minimizing unnecessary state changes.
            // TODO: Using designated layout specifiers in WebGL2/OpenGL ES 3, we could guarantee that uniforms
            // are in the same location between shaders, and avoid changing them unless necessary.
            if program as *const ShaderProgram != self.active_program {
                self.gl.use_program(Some(&program.program));
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
                    program.uniform1fv(&self.gl, ShaderUniform::GradientRatios, &gradient.ratios);
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
                        (gradient.interpolation == swf::GradientInterpolation::LinearRgb) as i32,
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
                    self.gl.active_texture(Gl::TEXTURE0);
                    self.gl.bind_texture(Gl::TEXTURE_2D, Some(texture));
                    program.uniform1i(&self.gl, ShaderUniform::BitmapTexture, 0);

                    // Set texture parameters.
                    let filter = if bitmap.is_smoothed {
                        Gl::LINEAR as i32
                    } else {
                        Gl::NEAREST as i32
                    };
                    self.gl
                        .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, filter);
                    self.gl
                        .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, filter);
                    // On WebGL1, you are unable to change the wrapping parameter of non-power-of-2 textures.
                    let wrap = if self.gl2.is_some() && bitmap.is_repeating {
                        Gl::REPEAT as i32
                    } else {
                        Gl::CLAMP_TO_EDGE as i32
                    };
                    self.gl
                        .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, wrap);
                    self.gl
                        .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, wrap);
                }
            }

            // Draw the triangles.
            self.gl
                .draw_elements_with_i32(Gl::TRIANGLES, num_indices, Gl::UNSIGNED_INT, 0);
        }
    }

    fn render_stage3d(&mut self, _bitmap: BitmapHandle, _transform: Transform) {
        panic!("Stage3D should not have been created on WebGL backend")
    }

    fn draw_rect(&mut self, color: Color, matrix: Matrix) {
        self.draw_quad::<{ Gl::TRIANGLE_FAN }, -1>(color, matrix)
    }

    fn draw_line(&mut self, color: Color, mut matrix: Matrix) {
        matrix.tx += Twips::HALF;
        matrix.ty += Twips::HALF;
        self.draw_quad::<{ Gl::LINE_STRIP }, 2>(color, matrix)
    }

    fn draw_line_rect(&mut self, color: Color, mut matrix: Matrix) {
        matrix.tx += Twips::HALF;
        matrix.ty += Twips::HALF;
        self.draw_quad::<{ Gl::LINE_LOOP }, -1>(color, matrix)
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
    gl2: Option<Gl2>,
    vao_ext: OesVertexArrayObject,
    draws: Vec<Draw>,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        if let Some(gl2) = &self.gl2 {
            for draw in &self.draws {
                gl2.delete_vertex_array(Some(&draw.vao));
            }
        } else {
            for draw in &self.draws {
                self.vao_ext.delete_vertex_array_oes(Some(&draw.vao));
            }
        }
    }
}

impl ShapeHandleImpl for Mesh {}

fn as_mesh(handle: &ShapeHandle) -> &Mesh {
    <dyn ShapeHandleImpl>::downcast_ref(&*handle.0).expect("Shape handle must be a WebGL ShapeData")
}

#[derive(Debug)]
struct Buffer {
    gl: Gl,
    buffer: WebGlBuffer,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.gl.delete_buffer(Some(&self.buffer));
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Draw {
    draw_type: DrawType,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    vao: WebGlVertexArrayObject,
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
    color_renderbuffer: WebGlRenderbuffer,
    stencil_renderbuffer: WebGlRenderbuffer,
    render_framebuffer: WebGlFramebuffer,
    color_framebuffer: WebGlFramebuffer,
    framebuffer_texture: WebGlTexture,
}

// Because the shaders are currently simple and few in number, we are using a
// straightforward shader model. We maintain an enum of every possible uniform,
// and each shader tries to grab the location of each uniform.
struct ShaderProgram {
    program: WebGlProgram,
    uniforms: [Option<WebGlUniformLocation>; NUM_UNIFORMS],
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
        gl: &Gl,
        vertex_shader: &WebGlShader,
        fragment_shader: &WebGlShader,
    ) -> Result<Self, Error> {
        let program = gl.create_program().ok_or(Error::UnableToCreateProgram)?;
        gl.attach_shader(&program, vertex_shader);
        gl.attach_shader(&program, fragment_shader);

        gl.link_program(&program);
        if !gl
            .get_program_parameter(&program, Gl::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let msg = format!(
                "Error linking shader program: {:?}",
                gl.get_program_info_log(&program)
            );
            log::error!("{}", msg);
            return Err(Error::LinkingShaderProgram(msg));
        }

        // Find uniforms.
        let mut uniforms: [Option<WebGlUniformLocation>; NUM_UNIFORMS] = Default::default();
        for i in 0..NUM_UNIFORMS {
            uniforms[i] = gl.get_uniform_location(&program, UNIFORM_NAMES[i]);
        }

        let vertex_position_location = gl.get_attrib_location(&program, "position") as u32;
        let vertex_color_location = gl.get_attrib_location(&program, "color") as u32;
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

    fn uniform1f(&self, gl: &Gl, uniform: ShaderUniform, value: f32) {
        gl.uniform1f(self.uniforms[uniform as usize].as_ref(), value);
    }

    fn uniform1fv(&self, gl: &Gl, uniform: ShaderUniform, values: &[f32]) {
        gl.uniform1fv_with_f32_array(self.uniforms[uniform as usize].as_ref(), values);
    }

    fn uniform1i(&self, gl: &Gl, uniform: ShaderUniform, value: i32) {
        gl.uniform1i(self.uniforms[uniform as usize].as_ref(), value);
    }

    fn uniform4fv(&self, gl: &Gl, uniform: ShaderUniform, values: &[f32]) {
        gl.uniform4fv_with_f32_array(self.uniforms[uniform as usize].as_ref(), values);
    }

    fn uniform_matrix3fv(&self, gl: &Gl, uniform: ShaderUniform, values: &[[f32; 3]; 3]) {
        gl.uniform_matrix3fv_with_f32_array(
            self.uniforms[uniform as usize].as_ref(),
            false,
            bytemuck::cast_slice(values),
        );
    }

    fn uniform_matrix4fv(&self, gl: &Gl, uniform: ShaderUniform, values: &[[f32; 4]; 4]) {
        gl.uniform_matrix4fv_with_f32_array(
            self.uniforms[uniform as usize].as_ref(),
            false,
            bytemuck::cast_slice(values),
        );
    }
}

trait GlExt {
    fn check_error(&self, error_msg: &'static str) -> Result<(), Error>;
}

impl GlExt for Gl {
    /// Check if GL returned an error for the previous operation.
    fn check_error(&self, error_msg: &'static str) -> Result<(), Error> {
        match self.get_error() {
            Self::NO_ERROR => Ok(()),
            error => Err(Error::GLError(error_msg, error)),
        }
    }
}

impl GlExt for Gl2 {
    /// Check if GL returned an error for the previous operation.
    fn check_error(&self, error_msg: &'static str) -> Result<(), Error> {
        match self.get_error() {
            Self::NO_ERROR => Ok(()),
            error => Err(Error::GLError(error_msg, error)),
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
