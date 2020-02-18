use glium::uniforms::{Sampler, UniformValue, Uniforms};
use glium::{draw_parameters::DrawParameters, implement_vertex, uniform, Display, Frame, Surface};
use glutin::WindowedContext;
use lyon::path::Path;
use lyon::tessellation::{
    self,
    geometry_builder::{BuffersBuilder, FillVertexConstructor, VertexBuffers},
    FillAttributes, FillTessellator, StrokeAttributes, StrokeTessellator, StrokeVertexConstructor,
};
use ruffle_core::backend::render::swf::{self, FillStyle};
use ruffle_core::backend::render::{
    BitmapHandle, BitmapInfo, Color, Letterbox, RenderBackend, ShapeHandle, Transform,
};
use ruffle_core::shape_utils::{DrawCommand, DrawPath};
use std::convert::TryInto;
use swf::Twips;

type Error = Box<dyn std::error::Error>;

pub struct GliumRenderBackend {
    display: Display,
    target: Option<Frame>,
    shader_program: glium::Program,
    gradient_shader_program: glium::Program,
    bitmap_shader_program: glium::Program,
    meshes: Vec<Mesh>,
    quad_shape: ShapeHandle,
    textures: Vec<(swf::CharacterId, Texture)>,
    num_masks: u32,
    num_masks_active: u32,
    write_stencil_mask: u32,
    test_stencil_mask: u32,
    next_stencil_mask: u32,
    mask_stack: Vec<(u32, u32)>,
    viewport_width: f32,
    viewport_height: f32,
    view_matrix: [[f32; 4]; 4],
}

impl GliumRenderBackend {
    pub fn new<T: glutin::ContextCurrentState>(
        windowed_context: WindowedContext<T>,
    ) -> Result<GliumRenderBackend, Error> {
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

        let bitmap_shader_program = glium::Program::new(
            &display,
            ProgramCreationInput::SourceCode {
                vertex_shader: TEXTURE_VERTEX_SHADER,
                fragment_shader: BITMAP_FRAGMENT_SHADER,
                geometry_shader: None,
                tessellation_control_shader: None,
                tessellation_evaluation_shader: None,
                transform_feedback_varyings: None,
                outputs_srgb: true,
                uses_point_size: false,
            },
        )?;

        let quad_mesh = Self::build_quad_mesh(&display)?;
        let quad_shape = ShapeHandle(0);

        let mut renderer = GliumRenderBackend {
            display,
            shader_program,
            gradient_shader_program,
            bitmap_shader_program,
            target: None,
            meshes: vec![quad_mesh],
            quad_shape,
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
        };
        renderer.build_matrices();
        Ok(renderer)
    }

    // Builds the quad mesh that is used for rendering bitmap display objects.
    fn build_quad_mesh(display: &Display) -> Result<Mesh, Error> {
        let vertex_buffer = glium::VertexBuffer::new(
            display,
            &[
                Vertex {
                    position: [0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                Vertex {
                    position: [1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                Vertex {
                    position: [0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ],
        )?;

        let index_buffer = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &[0, 1, 2, 0, 2, 3],
        )?;

        let quad_mesh = Mesh {
            draws: vec![Draw {
                draw_type: DrawType::Bitmap {
                    uniforms: BitmapUniforms {
                        matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                        id: 0,
                    },
                    is_smoothed: true,
                    is_repeating: false,
                },
                vertex_buffer,
                index_buffer,
            }],
        };
        Ok(quad_mesh)
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    fn register_shape_internal(&mut self, shape: &swf::Shape) -> ShapeHandle {
        let handle = ShapeHandle(self.meshes.len());
        let paths = ruffle_core::shape_utils::swf_shape_to_paths(shape);

        use lyon::tessellation::{FillOptions, StrokeOptions};

        let mut mesh = Mesh { draws: vec![] };

        let mut fill_tess = FillTessellator::new();
        let mut stroke_tess = StrokeTessellator::new();
        let mut lyon_mesh: VertexBuffers<_, u32> = VertexBuffers::new();

        fn flush_draw(
            draw: DrawType,
            mesh: &mut Mesh,
            lyon_mesh: &mut VertexBuffers<Vertex, u32>,
            display: &Display,
        ) {
            if lyon_mesh.vertices.is_empty() {
                return;
            }

            let vertex_buffer = glium::VertexBuffer::new(display, &lyon_mesh.vertices[..]).unwrap();

            let index_buffer = glium::IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &lyon_mesh.indices[..],
            )
            .unwrap();

            mesh.draws.push(Draw {
                draw_type: draw,
                vertex_buffer,
                index_buffer,
            });

            *lyon_mesh = VertexBuffers::new();
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
                        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh, &self.display);

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

                        let uniforms = GradientUniforms {
                            gradient_type: 0,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                            repeat_mode: 0,
                            focal_point: 0.0,
                        };

                        flush_draw(
                            DrawType::Gradient(uniforms),
                            &mut mesh,
                            &mut lyon_mesh,
                            &self.display,
                        );
                    }
                    FillStyle::RadialGradient(gradient) => {
                        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh, &self.display);

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

                        let uniforms = GradientUniforms {
                            gradient_type: 1,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                            repeat_mode: 0,
                            focal_point: 0.0,
                        };

                        flush_draw(
                            DrawType::Gradient(uniforms),
                            &mut mesh,
                            &mut lyon_mesh,
                            &self.display,
                        );
                    }
                    FillStyle::FocalGradient {
                        gradient,
                        focal_point,
                    } => {
                        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh, &self.display);

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

                        let uniforms = GradientUniforms {
                            gradient_type: 1,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            matrix: swf_to_gl_matrix(gradient.matrix.clone()),
                            repeat_mode: 0,
                            focal_point: *focal_point,
                        };

                        flush_draw(
                            DrawType::Gradient(uniforms),
                            &mut mesh,
                            &mut lyon_mesh,
                            &self.display,
                        );
                    }
                    FillStyle::Bitmap {
                        id,
                        matrix,
                        is_smoothed,
                        is_repeating,
                    } => {
                        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh, &self.display);

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

                        let texture = &self
                            .textures
                            .iter()
                            .find(|(other_id, _tex)| *other_id == *id)
                            .unwrap()
                            .1;

                        let uniforms = BitmapUniforms {
                            matrix: swf_bitmap_to_gl_matrix(
                                matrix.clone(),
                                texture.width,
                                texture.height,
                            ),
                            id: *id,
                        };

                        flush_draw(
                            DrawType::Bitmap {
                                uniforms,
                                is_smoothed: *is_smoothed,
                                is_repeating: *is_repeating,
                            },
                            &mut mesh,
                            &mut lyon_mesh,
                            &self.display,
                        );
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

        flush_draw(DrawType::Color, &mut mesh, &mut lyon_mesh, &self.display);

        self.meshes.push(mesh);

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
}

impl RenderBackend for GliumRenderBackend {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.viewport_width = width as f32;
        self.viewport_height = height as f32;
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
        let image = glium::texture::RawImage2d::from_raw_rgb(
            decoded_data,
            (metadata.width.into(), metadata.height.into()),
        );

        let texture = glium::texture::Texture2d::new(&self.display, image).unwrap();

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

        let image = glium::texture::RawImage2d::from_raw_rgba(rgba, (width, height));
        let texture = glium::texture::Texture2d::new(&self.display, image).unwrap();
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

        let image = glium::texture::RawImage2d::from_raw_rgba(
            decoded_data,
            (swf_tag.width.into(), swf_tag.height.into()),
        );

        let texture = glium::texture::Texture2d::new(&self.display, image).unwrap();

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
        assert!(self.target.is_none());
        self.target = Some(self.display.draw());
        self.num_masks = 0;
        self.num_masks_active = 0;
        self.write_stencil_mask = 0;
        self.test_stencil_mask = 0;
        self.next_stencil_mask = 1;
    }

    fn end_frame(&mut self) {
        assert!(self.target.is_some());
        let target = self.target.take().unwrap();
        target.finish().unwrap();
    }

    fn clear(&mut self, color: Color) {
        let target = self.target.as_mut().unwrap();
        target.clear_color_srgb_and_stencil(
            (
                f32::from(color.r) / 255.0,
                f32::from(color.g) / 255.0,
                f32::from(color.b) / 255.0,
                f32::from(color.a) / 255.0,
            ),
            0,
        );
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
            if let DrawType::Bitmap {
                uniforms: BitmapUniforms { id: draw_id, .. },
                ..
            } = &mut draw.draw_type
            {
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
        let target = self.target.as_mut().unwrap();

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

        let mut draw_parameters = DrawParameters::default();
        mask_draw_parameters(
            &mut draw_parameters,
            self.num_masks,
            self.num_masks_active,
            self.write_stencil_mask,
            self.test_stencil_mask,
        );

        for draw in &mesh.draws {
            match &draw.draw_type {
                DrawType::Color => {
                    draw_parameters.blend = color_blend();

                    target
                        .draw(
                            &draw.vertex_buffer,
                            &draw.index_buffer,
                            &self.shader_program,
                            &uniform! { view_matrix: self.view_matrix, world_matrix: world_matrix, mult_color: mult_color, add_color: add_color },
                            &draw_parameters
                        )
                        .unwrap();
                }
                DrawType::Gradient(gradient_uniforms) => {
                    let uniforms = GradientUniformsFull {
                        view_matrix: self.view_matrix,
                        world_matrix,
                        mult_color,
                        add_color,
                        gradient: gradient_uniforms.clone(),
                    };

                    draw_parameters.blend = color_blend();

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
                DrawType::Bitmap {
                    uniforms,
                    is_smoothed,
                    is_repeating,
                } => {
                    let texture = &self
                        .textures
                        .iter()
                        .find(|(id, _tex)| *id == uniforms.id)
                        .unwrap()
                        .1;

                    // Set texture sampler smooth/repeat parameters.
                    use glium::uniforms::{
                        MagnifySamplerFilter, MinifySamplerFilter, SamplerWrapFunction,
                    };
                    let texture = &texture
                        .texture
                        .sampled()
                        .magnify_filter(if *is_smoothed {
                            MagnifySamplerFilter::Linear
                        } else {
                            MagnifySamplerFilter::Nearest
                        })
                        .minify_filter(if *is_smoothed {
                            MinifySamplerFilter::LinearMipmapLinear
                        } else {
                            MinifySamplerFilter::Nearest
                        })
                        .wrap_function(if *is_repeating {
                            SamplerWrapFunction::Repeat
                        } else {
                            SamplerWrapFunction::Clamp
                        });

                    let uniforms = BitmapUniformsFull {
                        view_matrix: self.view_matrix,
                        world_matrix,
                        mult_color,
                        add_color,
                        matrix: uniforms.matrix,
                        texture,
                    };

                    draw_parameters.blend = bitmap_blend();

                    target
                        .draw(
                            &draw.vertex_buffer,
                            &draw.index_buffer,
                            &self.bitmap_shader_program,
                            &uniforms,
                            &draw_parameters,
                        )
                        .unwrap();
                }
            }
        }
    }

    fn draw_pause_overlay(&mut self) {}

    fn draw_letterbox(&mut self, letterbox: Letterbox) {
        let target = self.target.as_mut().unwrap();
        let black = Some((0.0, 0.0, 0.0, 1.0));
        match letterbox {
            Letterbox::None => (),
            Letterbox::Letterbox(margin_height) => {
                target.clear(
                    Some(&glium::Rect {
                        left: 0,
                        bottom: 0,
                        width: self.viewport_width as u32,
                        height: margin_height as u32,
                    }),
                    black,
                    true,
                    None,
                    None,
                );
                target.clear(
                    Some(&glium::Rect {
                        left: 0,
                        bottom: (self.viewport_height - margin_height) as u32,
                        width: self.viewport_width as u32,
                        height: margin_height as u32,
                    }),
                    black,
                    true,
                    None,
                    None,
                );
            }
            Letterbox::Pillarbox(margin_width) => {
                target.clear(
                    Some(&glium::Rect {
                        left: 0,
                        bottom: 0,
                        width: margin_width as u32,
                        height: self.viewport_height as u32,
                    }),
                    black,
                    true,
                    None,
                    None,
                );
                target.clear(
                    Some(&glium::Rect {
                        left: (self.viewport_width - margin_width) as u32,
                        bottom: 0,
                        width: margin_width as u32,
                        height: self.viewport_height as u32,
                    }),
                    black,
                    true,
                    None,
                    None,
                );
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
            let target = self.target.as_mut().unwrap();
            if self.test_stencil_mask != 0 {
                log::warn!(
                    "Too many masks active for stencil buffer; possibly incorrect rendering"
                );
            }
            self.next_stencil_mask = 1;
            target.clear_stencil(self.test_stencil_mask as i32);
        }
        self.num_masks += 1;
        self.mask_stack
            .push((self.write_stencil_mask, self.test_stencil_mask));
        self.write_stencil_mask = self.next_stencil_mask;
        self.test_stencil_mask |= self.next_stencil_mask;
        self.next_stencil_mask <<= 1;
    }
    fn activate_mask(&mut self) {
        self.num_masks_active += 1;
    }
    fn pop_mask(&mut self) {
        if !self.mask_stack.is_empty() {
            self.num_masks -= 1;
            self.num_masks_active -= 1;
            let (write, test) = self.mask_stack.pop().unwrap();
            self.write_stencil_mask = write;
            self.test_stencil_mask = test;
        } else {
            log::warn!("Mask stack underflow\n");
        }
    }
}

struct Texture {
    width: u32,
    height: u32,
    texture: glium::Texture2d,
}

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

implement_vertex!(Vertex, position, color);

#[derive(Clone, Debug)]
struct GradientUniforms {
    matrix: [[f32; 3]; 3],
    gradient_type: i32,
    ratios: Vec<f32>,
    colors: Vec<[f32; 4]>,
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
        for i in 0..self.num_colors as usize {
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

#[derive(Clone, Debug)]
struct GradientUniformsFull {
    world_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    mult_color: [f32; 4],
    add_color: [f32; 4],
    gradient: GradientUniforms,
}

impl Uniforms for GradientUniformsFull {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        visit("world_matrix", UniformValue::Mat4(self.world_matrix));
        visit("view_matrix", UniformValue::Mat4(self.view_matrix));
        visit("mult_color", UniformValue::Vec4(self.mult_color));
        visit("add_color", UniformValue::Vec4(self.add_color));
        self.gradient.visit_values(visit);
    }
}

#[derive(Clone, Debug)]
struct BitmapUniforms {
    matrix: [[f32; 3]; 3],
    id: swf::CharacterId,
}

impl Uniforms for BitmapUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut visit: F) {
        visit("u_matrix", UniformValue::Mat3(self.matrix));
    }
}

#[derive(Clone, Debug)]
struct BitmapUniformsFull<'a> {
    world_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    mult_color: [f32; 4],
    add_color: [f32; 4],
    matrix: [[f32; 3]; 3],
    texture: &'a Sampler<'a, glium::Texture2d>,
}

impl<'a> Uniforms for BitmapUniformsFull<'a> {
    fn visit_values<'v, F: FnMut(&str, UniformValue<'v>)>(&'v self, mut visit: F) {
        use glium::uniforms::AsUniformValue;
        visit("world_matrix", UniformValue::Mat4(self.world_matrix));
        visit("view_matrix", UniformValue::Mat4(self.view_matrix));
        visit("mult_color", UniformValue::Vec4(self.mult_color));
        visit("add_color", UniformValue::Vec4(self.add_color));
        visit("u_matrix", UniformValue::Mat3(self.matrix));
        visit("u_texture", self.texture.as_uniform_value());
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
        t = clamp(t, u_ratios[0], u_ratios[last]);
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

const BITMAP_FRAGMENT_SHADER: &str = r#"
#version 140
    uniform vec4 mult_color;
    uniform vec4 add_color;

    in vec2 frag_uv;
    out vec4 out_color;

    uniform sampler2D u_texture;

    void main() {

        vec4 color = texture(u_texture, frag_uv);
        // Unmultiply alpha before apply color transform.
        if( color.a > 0 ) {
            color.rgb /= color.a;
            color = mult_color * color + add_color;
            color.rgb *= color.a;
        }

        out_color = color;
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
    Gradient(GradientUniforms),
    Bitmap {
        uniforms: BitmapUniforms,
        is_smoothed: bool,
        is_repeating: bool,
    },
}

fn point(x: Twips, y: Twips) -> lyon::math::Point {
    lyon::math::Point::new(x.to_pixels() as f32, y.to_pixels() as f32)
}

fn ruffle_path_to_lyon_path(commands: Vec<DrawCommand>, is_closed: bool) -> Path {
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

/// Returns the drawing parameters for masking.
#[inline]
fn mask_draw_parameters(
    params: &mut DrawParameters,
    num_masks: u32,
    num_masks_active: u32,
    write_stencil_mask: u32,
    test_stencil_mask: u32,
) {
    use glium::draw_parameters::{Stencil, StencilOperation, StencilTest};
    if num_masks > 0 {
        let (value, test, pass_op, color_mask, write_mask) = if num_masks_active < num_masks {
            (
                write_stencil_mask as i32,
                StencilTest::AlwaysPass,
                StencilOperation::Replace,
                (false, false, false, false),
                write_stencil_mask,
            )
        } else {
            (
                test_stencil_mask as i32,
                StencilTest::IfEqual {
                    mask: test_stencil_mask,
                },
                StencilOperation::Keep,
                (true, true, true, true),
                test_stencil_mask,
            )
        };
        params.color_mask = color_mask;
        params.stencil = Stencil {
            test_clockwise: test,
            reference_value_clockwise: value,
            write_mask_clockwise: write_mask,
            fail_operation_clockwise: StencilOperation::Keep,
            pass_depth_fail_operation_clockwise: StencilOperation::Keep,
            depth_pass_operation_clockwise: pass_op,
            test_counter_clockwise: test,
            reference_value_counter_clockwise: value,
            write_mask_counter_clockwise: write_mask,
            fail_operation_counter_clockwise: StencilOperation::Keep,
            pass_depth_fail_operation_counter_clockwise: StencilOperation::Keep,
            depth_pass_operation_counter_clockwise: pass_op,
        };
    }
}

/// Returns the drawing parameters for standard color/gradient fills.
#[inline]
fn color_blend() -> glium::Blend {
    glium::Blend::alpha_blending()
}

/// Returns the drawing parameters for bitmaps with pre-multipled alpha.
#[inline]
fn bitmap_blend() -> glium::Blend {
    use glium::{BlendingFunction, LinearBlendingFactor};
    glium::Blend {
        color: BlendingFunction::Addition {
            source: LinearBlendingFactor::One,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        alpha: BlendingFunction::Addition {
            source: LinearBlendingFactor::SourceAlpha,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        ..Default::default()
    }
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
