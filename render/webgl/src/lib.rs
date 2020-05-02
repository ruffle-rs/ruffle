use ruffle_core::backend::render::swf::{self, FillStyle};
use ruffle_core::backend::render::{
    BitmapHandle, BitmapInfo, Color, Letterbox, RenderBackend, ShapeHandle, Transform,
};
use ruffle_render_common_tess::{GradientSpread, GradientType, ShapeTessellator, Vertex};
use ruffle_web_common::JsResult;
use std::convert::TryInto;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, OesVertexArrayObject, WebGl2RenderingContext as Gl2, WebGlBuffer,
    WebGlProgram, WebGlRenderingContext as Gl, WebGlShader, WebGlTexture, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

type Error = Box<dyn std::error::Error>;

const COLOR_VERTEX_GLSL: &str = include_str!("../shaders/color.vert");
const COLOR_FRAGMENT_GLSL: &str = include_str!("../shaders/color.frag");
const TEXTURE_VERTEX_GLSL: &str = include_str!("../shaders/texture.vert");
const GRADIENT_FRAGMENT_GLSL: &str = include_str!("../shaders/gradient.frag");
const BITMAP_FRAGMENT_GLSL: &str = include_str!("../shaders/bitmap.frag");

pub struct WebGlRenderBackend {
    /// WebGL1 context
    gl: Gl,

    // WebGL2 context, if available.
    gl2: Option<Gl2>,

    /// In WebGL1, VAOs are only available as an extension.
    vao_ext: OesVertexArrayObject,

    vertex_position_location: u32,
    vertex_color_location: u32,

    color_program: ShaderProgram,
    bitmap_program: ShaderProgram,
    gradient_program: ShaderProgram,

    shape_tessellator: ShapeTessellator,

    textures: Vec<(swf::CharacterId, Texture)>,
    meshes: Vec<Mesh>,

    quad_shape: ShapeHandle,

    num_masks: u32,
    num_masks_active: u32,
    write_stencil_mask: u32,
    test_stencil_mask: u32,
    next_stencil_mask: u32,
    mask_stack: Vec<(u32, u32)>,

    active_program: *const ShaderProgram,
    mask_state_dirty: bool,
    blend_func: (u32, u32),
    mult_color: Option<[f32; 4]>,
    add_color: Option<[f32; 4]>,

    viewport_width: f32,
    viewport_height: f32,
    view_matrix: [[f32; 4]; 4],
}

impl WebGlRenderBackend {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, Error> {
        // Create WebGL context.
        let options = [
            ("stencil", JsValue::TRUE),
            ("alpha", JsValue::FALSE),
            ("antialias", JsValue::TRUE),
            ("depth", JsValue::FALSE),
        ];

        let context_options = js_sys::Object::new();
        for (name, value) in options.iter() {
            js_sys::Reflect::set(&context_options, &JsValue::from(*name), value).warn_on_error();
        }

        // Attempt to create a WebGL2 context, but fall back to WebGL1 if unavailable.
        let (gl, gl2, vao_ext) = if let Ok(Some(gl)) =
            canvas.get_context_with_context_options("webgl2", &context_options)
        {
            log::info!("Creating WebGL2 context.");
            let gl2 = gl.dyn_into::<Gl2>().map_err(|_| "Expected GL context")?;
            // WebGLRenderingContext inherits from WebGL2RenderingContext, so cast it down.
            (
                gl2.clone().unchecked_into::<Gl>(),
                Some(gl2),
                JsValue::UNDEFINED.unchecked_into(),
            )
        } else if let Ok(Some(gl)) =
            canvas.get_context_with_context_options("webgl", &context_options)
        {
            log::info!("Falling back to WebGL1.");
            let gl = gl.dyn_into::<Gl>().map_err(|_| "Expected GL context")?;
            // `dyn_into` doesn't work here; why?
            let vao = gl
                .get_extension("OES_vertex_array_object")
                .into_js_result()?
                .ok_or("VAO extension not found")?
                .unchecked_into::<OesVertexArrayObject>();
            (gl, None, vao)
        } else {
            return Err("Unable to create WebGL rendering context".into());
        };

        let color_vertex = Self::compile_shader(&gl, Gl::VERTEX_SHADER, COLOR_VERTEX_GLSL)?;
        let texture_vertex = Self::compile_shader(&gl, Gl::VERTEX_SHADER, TEXTURE_VERTEX_GLSL)?;
        let color_fragment = Self::compile_shader(&gl, Gl::FRAGMENT_SHADER, COLOR_FRAGMENT_GLSL)?;
        let bitmap_fragment = Self::compile_shader(&gl, Gl::FRAGMENT_SHADER, BITMAP_FRAGMENT_GLSL)?;
        let gradient_fragment =
            Self::compile_shader(&gl, Gl::FRAGMENT_SHADER, GRADIENT_FRAGMENT_GLSL)?;

        let color_program = ShaderProgram::new(&gl, &color_vertex, &color_fragment)?;
        let bitmap_program = ShaderProgram::new(&gl, &texture_vertex, &bitmap_fragment)?;
        let gradient_program = ShaderProgram::new(&gl, &texture_vertex, &gradient_fragment)?;

        // This assumes we are using the same vertex format for all shaders.
        let vertex_position_location =
            gl.get_attrib_location(&color_program.program, "position") as u32;
        let vertex_color_location = gl.get_attrib_location(&color_program.program, "color") as u32;

        // Enable vertex attributes.
        // Because we currently only use on vertex format, we can do this once on init.
        gl.enable_vertex_attrib_array(vertex_position_location as u32);
        gl.enable_vertex_attrib_array(vertex_color_location as u32);
        gl.vertex_attrib_pointer_with_i32(vertex_position_location, 2, Gl::FLOAT, false, 12, 0);
        gl.vertex_attrib_pointer_with_i32(vertex_color_location, 4, Gl::UNSIGNED_BYTE, true, 12, 8);

        gl.enable(Gl::BLEND);
        gl.blend_func(Gl::SRC_ALPHA, Gl::ONE_MINUS_SRC_ALPHA);

        // Necessary to load RGB textures (alignment defaults to 4).
        gl.pixel_storei(Gl::UNPACK_ALIGNMENT, 1);

        let mut renderer = Self {
            gl,
            gl2,
            vao_ext,

            color_program,
            gradient_program,
            bitmap_program,

            shape_tessellator: ShapeTessellator::new(),

            meshes: vec![],
            quad_shape: ShapeHandle(0),
            textures: vec![],
            viewport_width: 500.0,
            viewport_height: 500.0,
            view_matrix: [[0.0; 4]; 4],
            num_masks: 0,
            num_masks_active: 0,
            write_stencil_mask: 0,
            test_stencil_mask: 0,
            next_stencil_mask: 1,
            mask_stack: vec![],

            active_program: std::ptr::null(),
            mask_state_dirty: true,
            blend_func: (Gl::SRC_ALPHA, Gl::ONE_MINUS_SRC_ALPHA),
            mult_color: None,
            add_color: None,

            vertex_position_location,
            vertex_color_location,
        };

        let quad_mesh = renderer.build_quad_mesh()?;
        renderer.meshes.push(quad_mesh);
        renderer.build_matrices();

        Ok(renderer)
    }

    fn build_quad_mesh(&mut self) -> Result<Mesh, Error> {
        let vao = self.create_vertex_array()?;

        let vertex_buffer = self.gl.create_buffer().unwrap();
        self.gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));

        let verts = [
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
        ];
        let (vertex_buffer, index_buffer) = unsafe {
            let verts_bytes = std::slice::from_raw_parts(
                verts.as_ptr() as *const u8,
                std::mem::size_of_val(&verts),
            );
            self.gl
                .buffer_data_with_u8_array(Gl::ARRAY_BUFFER, verts_bytes, Gl::STATIC_DRAW);

            let index_buffer = self.gl.create_buffer().unwrap();
            self.gl
                .bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
            let indices = [0u16, 1, 2, 0, 2, 3];
            let indices_bytes = std::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                std::mem::size_of::<u16>() * indices.len(),
            );
            self.gl.buffer_data_with_u8_array(
                Gl::ELEMENT_ARRAY_BUFFER,
                indices_bytes,
                Gl::STATIC_DRAW,
            );

            (vertex_buffer, index_buffer)
        };

        self.gl.vertex_attrib_pointer_with_i32(
            self.vertex_position_location,
            2,
            Gl::FLOAT,
            false,
            12,
            0,
        );
        self.gl.vertex_attrib_pointer_with_i32(
            self.vertex_color_location,
            4,
            Gl::UNSIGNED_BYTE,
            true,
            12,
            8,
        );
        self.gl
            .enable_vertex_attrib_array(self.vertex_position_location as u32);
        self.gl
            .enable_vertex_attrib_array(self.vertex_color_location as u32);

        let quad_mesh = Mesh {
            draws: vec![Draw {
                draw_type: DrawType::Bitmap(Bitmap {
                    matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                    id: 0,

                    is_smoothed: true,
                    is_repeating: false,
                }),
                vao,
                vertex_buffer,
                index_buffer,
                num_indices: 6,
            }],
        };
        Ok(quad_mesh)
    }

    fn compile_shader(gl: &Gl, shader_type: u32, glsl_src: &str) -> Result<WebGlShader, Error> {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(&shader, glsl_src);
        gl.compile_shader(&shader);
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        if !log.is_empty() {
            log::error!("{}", log);
        }
        Ok(shader)
    }

    fn register_shape_internal(&mut self, shape: &swf::Shape) -> ShapeHandle {
        use ruffle_render_common_tess::DrawType as TessDrawType;

        let handle = ShapeHandle(self.meshes.len());

        let textures = &self.textures;
        let lyon_mesh = self.shape_tessellator.tessellate_shape(shape, |id| {
            textures
                .iter()
                .find(|(other_id, _tex)| *other_id == id)
                .map(|tex| (tex.1.width, tex.1.height))
        });

        let mut draws = Vec::with_capacity(lyon_mesh.len());

        for draw in lyon_mesh {
            let num_indices = draw.indices.len() as i32;

            let vao = self.create_vertex_array().unwrap();
            let vertex_buffer = self.gl.create_buffer().unwrap();
            self.gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertex_buffer));

            let (vertex_buffer, index_buffer) = unsafe {
                let verts_bytes = std::slice::from_raw_parts(
                    draw.vertices.as_ptr() as *const u8,
                    draw.vertices.len() * std::mem::size_of::<Vertex>(),
                );
                self.gl
                    .buffer_data_with_u8_array(Gl::ARRAY_BUFFER, verts_bytes, Gl::STATIC_DRAW);

                let indices: Vec<_> = draw.indices.into_iter().map(|n| n as u16).collect();
                let index_buffer = self.gl.create_buffer().unwrap();
                self.gl
                    .bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
                let indices_bytes = std::slice::from_raw_parts(
                    indices.as_ptr() as *const u8,
                    indices.len() * std::mem::size_of::<u16>(),
                );
                self.gl.buffer_data_with_u8_array(
                    Gl::ELEMENT_ARRAY_BUFFER,
                    indices_bytes,
                    Gl::STATIC_DRAW,
                );
                (vertex_buffer, index_buffer)
            };

            self.gl.vertex_attrib_pointer_with_i32(
                self.vertex_position_location,
                2,
                Gl::FLOAT,
                false,
                12,
                0,
            );
            self.gl.vertex_attrib_pointer_with_i32(
                self.vertex_color_location,
                4,
                Gl::UNSIGNED_BYTE,
                true,
                12,
                8,
            );
            self.gl
                .enable_vertex_attrib_array(self.vertex_position_location as u32);
            self.gl
                .enable_vertex_attrib_array(self.vertex_color_location as u32);

            let out_draw = match draw.draw_type {
                TessDrawType::Color => Draw {
                    draw_type: DrawType::Color,
                    vao,
                    vertex_buffer,
                    index_buffer,
                    num_indices,
                },
                TessDrawType::Gradient(gradient) => {
                    let mut ratios = [0.0; 8];
                    let mut colors = [[0.0; 4]; 8];
                    let num_colors = gradient.num_colors as usize;
                    ratios[..num_colors].copy_from_slice(&gradient.ratios[..num_colors]);
                    colors[..num_colors].copy_from_slice(&gradient.colors[..num_colors]);
                    for i in num_colors..8 {
                        ratios[i] = ratios[i - 1];
                        colors[i] = colors[i - 1];
                    }
                    let out_gradient = Gradient {
                        matrix: gradient.matrix,
                        gradient_type: match gradient.gradient_type {
                            GradientType::Linear => 0,
                            GradientType::Radial => 1,
                            GradientType::Focal => 2,
                        },
                        ratios,
                        colors,
                        num_colors: gradient.num_colors,
                        repeat_mode: match gradient.repeat_mode {
                            GradientSpread::Pad => 0,
                            GradientSpread::Repeat => 1,
                            GradientSpread::Reflect => 2,
                        },
                        focal_point: gradient.focal_point,
                    };
                    Draw {
                        draw_type: DrawType::Gradient(Box::new(out_gradient)),
                        vao,
                        vertex_buffer,
                        index_buffer,
                        num_indices,
                    }
                }
                TessDrawType::Bitmap(bitmap) => Draw {
                    draw_type: DrawType::Bitmap(Bitmap {
                        matrix: bitmap.matrix,
                        id: bitmap.id,
                        is_smoothed: bitmap.is_smoothed,
                        is_repeating: bitmap.is_repeating,
                    }),
                    vao,
                    vertex_buffer,
                    index_buffer,
                    num_indices,
                },
            };

            draws.push(out_draw);
        }

        self.meshes.push(Mesh { draws });

        handle
    }

    fn build_matrices(&mut self) {
        self.view_matrix = [
            [1.0 / (self.viewport_width as f32 / 2.0), 0.0, 0.0, 0.0],
            [0.0, -1.0 / (self.viewport_height as f32 / 2.0), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];
    }

    /// Creates and binds a new VAO.
    fn create_vertex_array(&self) -> Result<WebGlVertexArrayObject, Error> {
        let vao = if let Some(gl2) = &self.gl2 {
            let vao = gl2.create_vertex_array().ok_or("Unable to create VAO")?;
            gl2.bind_vertex_array(Some(&vao));
            vao
        } else {
            let vao = self
                .vao_ext
                .create_vertex_array_oes()
                .ok_or("Unable to create VAO")?;
            self.vao_ext.bind_vertex_array_oes(Some(&vao));
            vao
        };
        Ok(vao)
    }

    /// Binds a VAO.
    fn bind_vertex_array(&self, vao: &WebGlVertexArrayObject) {
        if let Some(gl2) = &self.gl2 {
            gl2.bind_vertex_array(Some(&vao));
        } else {
            self.vao_ext.bind_vertex_array_oes(Some(&vao));
        };
    }
}

impl RenderBackend for WebGlRenderBackend {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.viewport_width = width as f32;
        self.viewport_height = height as f32;
        self.gl.viewport(0, 0, width as i32, height as i32);
        self.build_matrices();
    }

    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle {
        self.register_shape_internal(shape)
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let shape = swf::Shape {
            version: 2,
            id: 0,
            shape_bounds: Default::default(),
            edge_bounds: Default::default(),
            has_fill_winding_rule: false,
            has_non_scaling_strokes: false,
            has_scaling_strokes: true,
            styles: swf::ShapeStyles {
                fill_styles: vec![FillStyle::Color(Color {
                    r: 255,
                    g: 255,
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
        id: swf::CharacterId,
        data: &[u8],
        jpeg_tables: Option<&[u8]>,
    ) -> BitmapInfo {
        let data = ruffle_core::backend::render::glue_tables_to_jpeg(data, jpeg_tables);
        self.register_bitmap_jpeg_2(id, &data[..])
    }

    fn register_bitmap_jpeg_2(&mut self, id: swf::CharacterId, data: &[u8]) -> BitmapInfo {
        let data = ruffle_core::backend::render::remove_invalid_jpeg_data(data);

        let mut decoder = jpeg_decoder::Decoder::new(&data[..]);
        decoder.read_info().unwrap();
        let metadata = decoder.info().unwrap();
        let decoded_data = decoder.decode().expect("failed to decode image");

        let texture = self.gl.create_texture().unwrap();
        self.gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RGB as i32,
                metadata.width.into(),
                metadata.height.into(),
                0,
                Gl::RGB,
                Gl::UNSIGNED_BYTE,
                Some(&decoded_data),
            )
            .warn_on_error();

        let handle = BitmapHandle(self.textures.len());
        self.textures.push((
            id,
            Texture {
                texture,
                width: metadata.width.into(),
                height: metadata.height.into(),
            },
        ));

        BitmapInfo {
            handle,
            width: metadata.width,
            height: metadata.height,
        }
    }

    fn register_bitmap_jpeg_3(
        &mut self,
        id: swf::CharacterId,
        jpeg_data: &[u8],
        alpha_data: &[u8],
    ) -> BitmapInfo {
        let (width, height, rgba) =
            ruffle_core::backend::render::define_bits_jpeg_to_rgba(jpeg_data, alpha_data)
                .expect("Error decoding DefineBitsJPEG3");

        let texture = self.gl.create_texture().unwrap();
        self.gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                Gl::RGBA,
                Gl::UNSIGNED_BYTE,
                Some(&rgba),
            )
            .warn_on_error();

        let handle = BitmapHandle(self.textures.len());
        self.textures.push((
            id,
            Texture {
                texture,
                width,
                height,
            },
        ));

        BitmapInfo {
            handle,
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
        }
    }

    fn register_bitmap_png(&mut self, swf_tag: &swf::DefineBitsLossless) -> BitmapInfo {
        let decoded_data = ruffle_core::backend::render::define_bits_lossless_to_rgba(swf_tag)
            .expect("Error decoding DefineBitsLossless");

        let texture = self.gl.create_texture().unwrap();
        self.gl.bind_texture(Gl::TEXTURE_2D, Some(&texture));
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::RGBA as i32,
                swf_tag.width.into(),
                swf_tag.height.into(),
                0,
                Gl::RGBA,
                Gl::UNSIGNED_BYTE,
                Some(&decoded_data),
            )
            .warn_on_error();

        // You must set the texture parameters for non-power-of-2 textures to function in WebGL.
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR as i32);
        self.gl
            .tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);

        let handle = BitmapHandle(self.textures.len());
        self.textures.push((
            swf_tag.id,
            Texture {
                texture,
                width: swf_tag.width.into(),
                height: swf_tag.height.into(),
            },
        ));

        BitmapInfo {
            handle,
            width: swf_tag.width,
            height: swf_tag.height,
        }
    }

    fn begin_frame(&mut self) {
        self.num_masks = 0;
        self.num_masks_active = 0;
        self.write_stencil_mask = 0;
        self.test_stencil_mask = 0;
        self.next_stencil_mask = 1;

        self.active_program = std::ptr::null();
        self.mask_state_dirty = true;

        self.mult_color = None;
        self.add_color = None;
    }

    fn end_frame(&mut self) {}

    fn clear(&mut self, color: Color) {
        self.gl.clear_color(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        );
        self.gl.clear(Gl::COLOR_BUFFER_BIT | Gl::STENCIL_BUFFER_BIT);
    }

    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform) {
        // TODO: Might be better to make this separate code to render the bitmap
        // instead of going through render_shape. But render_shape already handles
        // masking etc.
        if let Some((id, bitmap)) = self.textures.get(bitmap.0) {
            // Adjust the quad draw to use the target bitmap.
            let mesh = &mut self.meshes[self.quad_shape.0];
            let draw = &mut mesh.draws[0];
            let width = bitmap.width as f32;
            let height = bitmap.height as f32;
            if let DrawType::Bitmap(Bitmap { id: draw_id, .. }) = &mut draw.draw_type {
                *draw_id = *id;
            }

            // Scale the quad to the bitmap's dimensions.
            use ruffle_core::matrix::Matrix;
            let scale_transform = Transform {
                matrix: transform.matrix
                    * Matrix {
                        a: width,
                        d: height,
                        ..Default::default()
                    },
                ..*transform
            };

            // Render the quad.
            self.render_shape(self.quad_shape, &scale_transform);
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        let mesh = &self.meshes[shape.0];

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

        // Set masking state.
        if self.mask_state_dirty {
            if self.num_masks > 0 {
                self.gl.enable(Gl::STENCIL_TEST);
                if self.num_masks_active < self.num_masks {
                    self.gl.stencil_mask(self.write_stencil_mask);
                    self.gl
                        .stencil_func(Gl::ALWAYS, self.write_stencil_mask as i32, 0xff);
                    self.gl.stencil_op(Gl::KEEP, Gl::KEEP, Gl::REPLACE);
                    self.gl.color_mask(false, false, false, false);
                } else {
                    self.gl.stencil_mask(0);
                    self.gl.stencil_func(
                        Gl::EQUAL,
                        self.test_stencil_mask as i32,
                        self.test_stencil_mask,
                    );
                    self.gl.stencil_op(Gl::KEEP, Gl::KEEP, Gl::KEEP);
                    self.gl.color_mask(true, true, true, true);
                }
            } else {
                self.gl.disable(Gl::STENCIL_TEST);
                self.gl.color_mask(true, true, true, true);
            }
            self.mask_state_dirty = false;
        }

        for draw in &mesh.draws {
            self.bind_vertex_array(&draw.vao);

            let (program, src_blend, dst_blend) = match &draw.draw_type {
                DrawType::Color => (&self.color_program, Gl::SRC_ALPHA, Gl::ONE_MINUS_SRC_ALPHA),
                DrawType::Gradient(_) => (
                    &self.gradient_program,
                    Gl::SRC_ALPHA,
                    Gl::ONE_MINUS_SRC_ALPHA,
                ),
                // Bitmaps use pre-multiplied alpha.
                DrawType::Bitmap { .. } => (&self.bitmap_program, Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA),
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

                if (src_blend, dst_blend) != self.blend_func {
                    self.gl.blend_func(src_blend, dst_blend);
                    self.blend_func = (src_blend, dst_blend);
                }
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
                    let colors =
                        unsafe { std::slice::from_raw_parts(gradient.colors[0].as_ptr(), 32) };
                    program.uniform4fv(&self.gl, ShaderUniform::GradientColors, &colors);
                    program.uniform1i(
                        &self.gl,
                        ShaderUniform::GradientNumColors,
                        gradient.num_colors as i32,
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
                }
                DrawType::Bitmap(bitmap) => {
                    let texture = &self
                        .textures
                        .iter()
                        .find(|(id, _tex)| *id == bitmap.id)
                        .unwrap()
                        .1;

                    program.uniform_matrix3fv(
                        &self.gl,
                        ShaderUniform::TextureMatrix,
                        &bitmap.matrix,
                    );

                    // Bind texture.
                    self.gl.active_texture(Gl::TEXTURE0);
                    self.gl.bind_texture(Gl::TEXTURE_2D, Some(&texture.texture));
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
                    let wrap = if bitmap.is_repeating {
                        Gl::MIRRORED_REPEAT as i32
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
                .draw_elements_with_i32(Gl::TRIANGLES, draw.num_indices, Gl::UNSIGNED_SHORT, 0);
        }
    }

    fn draw_letterbox(&mut self, letterbox: Letterbox) {
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);

        match letterbox {
            Letterbox::None => (),
            Letterbox::Letterbox(margin_height) => {
                self.gl.enable(Gl::SCISSOR_TEST);
                self.gl
                    .scissor(0, 0, self.viewport_width as i32, margin_height as i32);
                self.gl.clear(Gl::COLOR_BUFFER_BIT);
                self.gl.scissor(
                    0,
                    (self.viewport_height - margin_height) as i32,
                    self.viewport_width as i32,
                    margin_height as i32 + 1,
                );
                self.gl.clear(Gl::COLOR_BUFFER_BIT);
                self.gl.disable(Gl::SCISSOR_TEST);
            }
            Letterbox::Pillarbox(margin_width) => {
                self.gl.enable(Gl::SCISSOR_TEST);
                self.gl
                    .scissor(0, 0, margin_width as i32, self.viewport_height as i32);
                self.gl.clear(Gl::COLOR_BUFFER_BIT);
                self.gl.scissor(
                    (self.viewport_width - margin_width) as i32,
                    0,
                    margin_width as i32 + 1,
                    self.viewport_height as i32,
                );
                self.gl.clear(Gl::COLOR_BUFFER_BIT);
                self.gl.scissor(
                    0,
                    0,
                    self.viewport_width as i32,
                    self.viewport_height as i32,
                );
                self.gl.disable(Gl::SCISSOR_TEST);
            }
        }
    }

    fn push_mask(&mut self) {
        // Desktop draws the masker to the stencil buffer, one bit per mask.
        // Masks-within-masks are handled as a bitmask.
        // This does unfortunately mean we are limited in the number of masks at once (usually 8 bits).
        if self.next_stencil_mask >= 0x100 {
            // If we've reached the limit of masks, clear the stencil buffer and start over.
            // But this may not be correct if there is still a mask active (mask-within-mask).
            if self.test_stencil_mask != 0 {
                log::warn!(
                    "Too many masks active for stencil buffer; possibly incorrect rendering"
                );
            }
            self.next_stencil_mask = 1;
            self.gl.clear_stencil(self.test_stencil_mask as i32);
        }
        self.num_masks += 1;
        self.mask_stack
            .push((self.write_stencil_mask, self.test_stencil_mask));
        self.write_stencil_mask = self.next_stencil_mask;
        self.test_stencil_mask |= self.next_stencil_mask;
        self.next_stencil_mask <<= 1;
        self.mask_state_dirty = true;
    }

    fn activate_mask(&mut self) {
        self.num_masks_active += 1;
        self.mask_state_dirty = true;
    }

    fn pop_mask(&mut self) {
        if !self.mask_stack.is_empty() {
            self.num_masks -= 1;
            self.num_masks_active -= 1;
            let (write, test) = self.mask_stack.pop().unwrap();
            self.write_stencil_mask = write;
            self.test_stencil_mask = test;
            self.mask_state_dirty = true;
        } else {
            log::warn!("Mask stack underflow\n");
        }
    }
}

struct Texture {
    width: u32,
    height: u32,
    texture: WebGlTexture,
}

#[derive(Clone, Debug)]
struct Gradient {
    matrix: [[f32; 3]; 3],
    gradient_type: i32,
    ratios: [f32; 8],
    colors: [[f32; 4]; 8],
    num_colors: u32,
    repeat_mode: i32,
    focal_point: f32,
}

#[derive(Clone, Debug)]
struct Bitmap {
    matrix: [[f32; 3]; 3],
    id: swf::CharacterId,
    is_repeating: bool,
    is_smoothed: bool,
}

struct Mesh {
    draws: Vec<Draw>,
}

#[allow(dead_code)]
struct Draw {
    draw_type: DrawType,
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    vao: WebGlVertexArrayObject,
    num_indices: i32,
}

enum DrawType {
    Color,
    Gradient(Box<Gradient>),
    Bitmap(Bitmap),
}

// Because the shaders are currently simple and few in number, we are using a
// straightforward shader model. We maintain an enum of every possible uniform,
// and each shader tries to grab the location of each uniform.
struct ShaderProgram {
    program: WebGlProgram,
    uniforms: [Option<WebGlUniformLocation>; NUM_UNIFORMS],
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
    "u_num_colors",
    "u_repeat_mode",
    "u_focal_point",
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
    GradientNumColors,
    GradientRepeatMode,
    GradientFocalPoint,
    BitmapTexture,
}

impl ShaderProgram {
    fn new(
        gl: &Gl,
        vertex_shader: &WebGlShader,
        fragment_shader: &WebGlShader,
    ) -> Result<Self, Error> {
        let program = gl.create_program().ok_or("Unable to create program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);

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
            return Err(msg.into());
        }

        // Find uniforms.
        let mut uniforms: [Option<WebGlUniformLocation>; NUM_UNIFORMS] = Default::default();
        for i in 0..NUM_UNIFORMS {
            uniforms[i] = gl.get_uniform_location(&program, UNIFORM_NAMES[i]);
        }

        Ok(ShaderProgram { program, uniforms })
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
            unsafe { std::slice::from_raw_parts(values[0].as_ptr(), 9) },
        );
    }

    fn uniform_matrix4fv(&self, gl: &Gl, uniform: ShaderUniform, values: &[[f32; 4]; 4]) {
        gl.uniform_matrix4fv_with_f32_array(
            self.uniforms[uniform as usize].as_ref(),
            false,
            unsafe { std::slice::from_raw_parts(values[0].as_ptr(), 16) },
        );
    }
}
