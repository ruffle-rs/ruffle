use pathfinder_color::{rgbau, rgbu, ColorF};
use pathfinder_content::gradient::Gradient;
use pathfinder_content::outline::{Contour, Outline};
use pathfinder_content::pattern::{Image, Pattern};
use pathfinder_content::stroke::{LineCap, LineJoin};
use pathfinder_content::stroke::{OutlineStrokeToFill, StrokeStyle};
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_geometry::rect::RectF;
pub use pathfinder_geometry::transform2d::{Matrix2x2F, Transform2F};
use pathfinder_geometry::vector::{vec2f, vec2i, Vector2F};
use pathfinder_gl::{GLDevice, GLVersion};
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_renderer::options::BuildOptions;
use pathfinder_renderer::paint::Paint;
use pathfinder_renderer::scene::{DrawPath, Scene};
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use pathfinder_simd::default::F32x2;
use ruffle_core::backend::render::swf::{self, FillStyle, Twips};
use ruffle_core::backend::render::{
    Bitmap, BitmapFormat, BitmapHandle, BitmapInfo, Color, Letterbox, RenderBackend, ShapeHandle,
    Transform,
};
use ruffle_core::shape_utils::DistilledShape;
use surfman::{
    Connection, ContextAttributeFlags, ContextAttributes, Device, GLVersion as SurfmanGLVersion,
};
use surfman::{SurfaceAccess, SurfaceType};

type Error = Box<dyn std::error::Error>;

pub struct PathfinderRenderBackend {
    pathfinder: Renderer<GLDevice>,
    gl_context: surfman::Context,
    device: Device,

    scene: Scene,

    viewport_width: f32,
    viewport_height: f32,
    view_matrix: [[f32; 4]; 4],

    shapes: Vec<Shape>,
    textures: Vec<Texture>,
}

impl PathfinderRenderBackend {
    pub fn new(
        window: raw_window_handle::RawWindowHandle,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Result<Self, Error> {
        // Create a `surfman` device. On a multi-GPU system, we'll request the low-power integrated
        // GPU.
        let connection = Connection::new().unwrap();
        let native_widget = connection
            .create_native_widget_from_rwh(window.clone())
            .unwrap();
        let adapter = connection.create_hardware_adapter().unwrap();
        let mut device = connection.create_device(&adapter).unwrap();

        // Request an OpenGL 3.x context. Pathfinder requires this.
        let context_attributes = ContextAttributes {
            version: SurfmanGLVersion::new(3, 0),
            flags: ContextAttributeFlags::ALPHA,
        };
        let context_descriptor = device
            .create_context_descriptor(&context_attributes)
            .unwrap();

        // Make the OpenGL context via `surfman`, and load OpenGL functions.
        let surface_type = SurfaceType::Widget { native_widget };
        let mut context = device.create_context(&context_descriptor, None).unwrap();
        let surface = device
            .create_surface(&context, SurfaceAccess::GPUOnly, surface_type)
            .unwrap();
        device
            .bind_surface_to_context(&mut context, surface)
            .unwrap();
        device.make_context_current(&context).unwrap();
        gl::load_with(|symbol_name| device.get_proc_address(&context, symbol_name));

        // Get the real size of the window, taking HiDPI into account.
        //let hidpi_factor = window.get_current_monitor().get_hidpi_factor();
        //let physical_size = logical_size.to_physical(hidpi_factor);
        let framebuffer_size = vec2i(500, 500); //physical_size.width as i32, physical_size.height as i32);

        // Create a Pathfinder GL device.
        let default_framebuffer = device
            .context_surface_info(&context)
            .unwrap()
            .unwrap()
            .framebuffer_object;
        let pathfinder_device = GLDevice::new(GLVersion::GL3, default_framebuffer);

        // Create a Pathfinder renderer.
        let mode = RendererMode::default_for_device(&pathfinder_device);
        let options = RendererOptions {
            dest: DestFramebuffer::full_window(framebuffer_size),
            background_color: Some(ColorF::white()),
            ..RendererOptions::default()
        };
        let resource_loader = EmbeddedResourceLoader::new();
        let pathfinder = Renderer::new(pathfinder_device, &resource_loader, mode, options);

        let mut renderer = Self {
            pathfinder,
            device,
            gl_context: context,

            scene: Scene::new(),

            viewport_width: 500.0,
            viewport_height: 500.0,
            view_matrix: [[0.0; 4]; 4],

            shapes: Vec::new(),
            textures: Vec::new(),
        };

        renderer.set_viewport_dimensions(viewport_width, viewport_height);

        Ok(renderer)
    }

    fn register_shape_internal(&mut self, shape: DistilledShape) -> Shape {
        use ruffle_core::shape_utils::DrawPath as RufflePath;

        let mut out_shape = Shape { draws: Vec::new() };

        for path in shape.paths {
            match path {
                RufflePath::Fill { style, commands } => {
                    let outline = ruffle_commands_to_outline(&commands);
                    let paint = match style {
                        FillStyle::Color(color) => {
                            Paint::from_color(rgbau(color.r, color.g, color.b, color.a))
                        }
                        FillStyle::LinearGradient(swf_gradient) => {
                            let mut gradient =
                                Gradient::linear_from_points(vec2f(-1.0, 0.0), vec2f(1.0, 0.0));
                            for record in &swf_gradient.records {
                                gradient.add_color_stop(
                                    rgbau(
                                        record.color.r,
                                        record.color.g,
                                        record.color.b,
                                        record.color.a,
                                    ),
                                    f32::from(record.ratio) / 255.0,
                                );
                            }
                            let transform = Transform2F {
                                matrix: Matrix2x2F::row_major(
                                    swf_gradient.matrix.a * 16384.0 / 20.0,
                                    swf_gradient.matrix.c * 16384.0 / 20.0,
                                    swf_gradient.matrix.b * 16384.0 / 20.0,
                                    swf_gradient.matrix.d * 16384.0 / 20.0,
                                ),
                                vector: Vector2F::new(
                                    swf_gradient.matrix.tx.to_pixels() as f32,
                                    swf_gradient.matrix.ty.to_pixels() as f32,
                                ),
                            };
                            gradient.apply_transform(transform);
                            Paint::from_gradient(gradient)
                        }
                        FillStyle::RadialGradient(swf_gradient) => {
                            let mut gradient =
                                Gradient::radial(vec2f(0.0, 0.0), F32x2::new(0.0, 1.0));
                            for record in &swf_gradient.records {
                                gradient.add_color_stop(
                                    rgbau(
                                        record.color.r,
                                        record.color.g,
                                        record.color.b,
                                        record.color.a,
                                    ),
                                    f32::from(record.ratio) / 255.0,
                                );
                            }
                            let transform = Transform2F {
                                matrix: Matrix2x2F::row_major(
                                    swf_gradient.matrix.a * 16384.0 / 20.0,
                                    swf_gradient.matrix.c * 16384.0 / 20.0,
                                    swf_gradient.matrix.b * 16384.0 / 20.0,
                                    swf_gradient.matrix.d * 16384.0 / 20.0,
                                ),
                                vector: Vector2F::new(
                                    swf_gradient.matrix.tx.to_pixels() as f32,
                                    swf_gradient.matrix.ty.to_pixels() as f32,
                                ),
                            };
                            gradient.apply_transform(transform);
                            Paint::from_gradient(gradient)
                        }
                        FillStyle::FocalGradient {
                            gradient: swf_gradient,
                            focal_point,
                        } => {
                            let mut gradient = Gradient::radial(
                                LineSegment2F::new(vec2f(*focal_point, 0.0), vec2f(0.0, 0.0)),
                                F32x2::new(0.0, 1.0),
                            );
                            for record in &swf_gradient.records {
                                gradient.add_color_stop(
                                    rgbau(
                                        record.color.r,
                                        record.color.g,
                                        record.color.b,
                                        record.color.a,
                                    ),
                                    f32::from(record.ratio) / 255.0,
                                );
                            }
                            let transform = Transform2F {
                                matrix: Matrix2x2F::row_major(
                                    swf_gradient.matrix.a * 16384.0 / 20.0,
                                    swf_gradient.matrix.c * 16384.0 / 20.0,
                                    swf_gradient.matrix.b * 16384.0 / 20.0,
                                    swf_gradient.matrix.d * 16384.0 / 20.0,
                                ),
                                vector: Vector2F::new(
                                    swf_gradient.matrix.tx.to_pixels() as f32,
                                    swf_gradient.matrix.ty.to_pixels() as f32,
                                ),
                            };
                            gradient.apply_transform(transform);
                            Paint::from_gradient(gradient)
                        }
                        FillStyle::Bitmap {
                            id,
                            matrix,
                            is_smoothed,
                            is_repeating,
                        } => {
                            if let Some(bitmap) =
                                self.textures.iter().find(|bitmap| bitmap.id == *id)
                            {
                                let mut pattern = bitmap.pattern.clone();
                                pattern.set_repeat_x(*is_repeating);
                                pattern.set_repeat_y(*is_repeating);
                                pattern.set_smoothing_enabled(*is_smoothed);
                                let transform = Transform2F {
                                    matrix: Matrix2x2F::row_major(
                                        matrix.a / 20.0,
                                        matrix.c / 20.0,
                                        matrix.b / 20.0,
                                        matrix.d / 20.0,
                                    ),
                                    vector: Vector2F::new(
                                        matrix.tx.to_pixels() as f32,
                                        matrix.ty.to_pixels() as f32,
                                    ),
                                };
                                pattern.apply_transform(transform);
                                let paint = Paint::from_pattern(pattern);

                                paint
                            } else {
                                Paint::transparent_black()
                            }
                        }
                    };
                    out_shape.draws.push(ShapeDraw::Fill { outline, paint });
                }
                RufflePath::Stroke {
                    style,
                    commands,
                    is_closed: _,
                } => {
                    let stroke_style = StrokeStyle {
                        line_width: style.width.to_pixels() as f32,
                        line_cap: match style.start_cap {
                            swf::LineCapStyle::None => LineCap::Butt,
                            swf::LineCapStyle::Square => LineCap::Square,
                            swf::LineCapStyle::Round => LineCap::Round,
                        },
                        line_join: match style.join_style {
                            swf::LineJoinStyle::Round => LineJoin::Round,
                            swf::LineJoinStyle::Bevel => LineJoin::Bevel,
                            swf::LineJoinStyle::Miter(miter_limit) => LineJoin::Miter(miter_limit),
                        },
                    };
                    let outline = ruffle_commands_to_outline(&commands);
                    let paint = Paint::from_color(rgbau(
                        style.color.r,
                        style.color.g,
                        style.color.b,
                        style.color.a,
                    ));
                    out_shape.draws.push(ShapeDraw::Stroke {
                        outline,
                        paint,
                        style: stroke_style,
                    });
                }
            }
        }

        out_shape
    }

    fn register_bitmap(
        &mut self,
        id: swf::CharacterId,
        bitmap: Bitmap,
    ) -> Result<BitmapInfo, Error> {
        let data = match bitmap.data {
            BitmapFormat::Rgb(pixels) => {
                let mut data = Vec::with_capacity(pixels.len() / 3);
                let mut i = 0;
                while i < pixels.len() {
                    data.push(rgbu(pixels[i], pixels[i + 1], pixels[i + 2]));
                    i += 3;
                }
                data
            }
            BitmapFormat::Rgba(mut pixels) => {
                ruffle_core::backend::render::unmultiply_alpha_rgba(&mut pixels);
                let mut data = Vec::with_capacity(pixels.len() / 4);
                let mut i = 0;
                while i < pixels.len() {
                    data.push(rgbau(
                        pixels[i],
                        pixels[i + 1],
                        pixels[i + 2],
                        pixels[i + 3],
                    ));
                    i += 4;
                }
                data
            }
        };
        let image = Image::new(
            vec2i(bitmap.width as i32, bitmap.height as i32),
            std::sync::Arc::new(data),
        );
        let pattern = Pattern::from_image(image);

        let handle = BitmapHandle(self.textures.len());
        self.textures.push(Texture {
            id,
            pattern,
            width: bitmap.width,
            height: bitmap.height,
        });

        Ok(BitmapInfo {
            handle,
            width: bitmap.width as u16,
            height: bitmap.height as u16,
        })
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

impl RenderBackend for PathfinderRenderBackend {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.viewport_width = width as f32;
        self.viewport_height = height as f32;
        self.build_matrices();
        let framebuffer_size = vec2i(width as i32, height as i32); //physical_size.width as i32, physical_size.height as i32);

        // Create a Pathfinder GL device.
        let default_framebuffer = self
            .device
            .context_surface_info(&self.gl_context)
            .unwrap()
            .unwrap()
            .framebuffer_object;
        let pathfinder_device = GLDevice::new(GLVersion::GL3, default_framebuffer);

        // Create a Pathfinder renderer.
        let mode = RendererMode::default_for_device(&pathfinder_device);
        let options = RendererOptions {
            dest: DestFramebuffer::full_window(framebuffer_size),
            background_color: Some(ColorF::white()),
            ..RendererOptions::default()
        };
        let resource_loader = EmbeddedResourceLoader::new();
        let pathfinder = Renderer::new(pathfinder_device, &resource_loader, mode, options);

        self.pathfinder = pathfinder;
    }

    fn register_shape(&mut self, shape: DistilledShape) -> ShapeHandle {
        let shape = self.register_shape_internal(shape);
        let handle = ShapeHandle(self.shapes.len());
        self.shapes.push(shape);
        handle
    }

    fn replace_shape(&mut self, shape: DistilledShape, handle: ShapeHandle) {
        let shape = self.register_shape_internal(shape);
        self.shapes[handle.0] = shape;
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
        self.register_shape((&shape).into())
    }

    fn register_bitmap_jpeg(
        &mut self,
        id: swf::CharacterId,
        data: &[u8],
        jpeg_tables: Option<&[u8]>,
    ) -> Result<BitmapInfo, Error> {
        let data = ruffle_core::backend::render::glue_tables_to_jpeg(data, jpeg_tables);
        self.register_bitmap_jpeg_2(id, &data[..])
    }

    fn register_bitmap_jpeg_2(
        &mut self,
        id: swf::CharacterId,
        data: &[u8],
    ) -> Result<BitmapInfo, Error> {
        let bitmap = ruffle_core::backend::render::decode_define_bits_jpeg(data, None)?;
        self.register_bitmap(id, bitmap)
    }

    fn register_bitmap_jpeg_3(
        &mut self,
        id: swf::CharacterId,
        jpeg_data: &[u8],
        alpha_data: &[u8],
    ) -> Result<BitmapInfo, Error> {
        let bitmap =
            ruffle_core::backend::render::decode_define_bits_jpeg(jpeg_data, Some(alpha_data))?;
        self.register_bitmap(id, bitmap)
    }

    fn register_bitmap_png(
        &mut self,
        swf_tag: &swf::DefineBitsLossless,
    ) -> Result<BitmapInfo, Error> {
        let bitmap = ruffle_core::backend::render::decode_define_bits_lossless(swf_tag)?;
        self.register_bitmap(swf_tag.id, bitmap)
    }

    fn begin_frame(&mut self, clear: Color) {
        self.scene.set_view_box(RectF::new(
            Vector2F::zero(),
            vec2f(self.viewport_width, self.viewport_height),
        ));

        let mut outline = Outline::new();
        let mut contour = Contour::new();
        contour.push_endpoint(vec2f(self.viewport_width, 0.0));
        contour.push_endpoint(vec2f(self.viewport_width, self.viewport_height));
        contour.push_endpoint(vec2f(0.0, self.viewport_height));
        contour.push_endpoint(vec2f(0.0, 0.0));
        outline.push_contour(contour);

        // let out = ruffle_core::backend::render::linear_to_srgb([
        //     clear.r as f32 / 255.0,
        //     clear.g as f32 / 255.0,
        //     clear.b as f32 / 255.0,
        //     clear.a as f32 / 255.0,
        // ]);
        let clear = Paint::from_color(rgbau(clear.r, clear.g, clear.b, clear.a));

        let paint_id = self.scene.push_paint(&clear);
        let draw_path = DrawPath::new(outline, paint_id);

        self.scene.push_draw_path(draw_path);
    }

    fn end_frame(&mut self) {
        self.scene
            .build_and_render(&mut self.pathfinder, BuildOptions::default(), RayonExecutor);
        self.scene = Scene::new();

        let mut surface = self
            .device
            .unbind_surface_from_context(&mut self.gl_context)
            .unwrap()
            .unwrap();
        self.device
            .present_surface(&mut self.gl_context, &mut surface)
            .unwrap();
        self.device
            .bind_surface_to_context(&mut self.gl_context, surface)
            .unwrap();
    }

    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform) {
        if let Some(texture) = self.textures.get(bitmap.0) {
            let mut outline = Outline::new();
            let mut contour = Contour::new();

            let (width, height) = (texture.width as f32, texture.height as f32);
            contour.push_endpoint(vec2f(0.0, 0.0));
            contour.push_endpoint(vec2f(width as f32, 0.0));
            contour.push_endpoint(vec2f(width as f32, height));
            contour.push_endpoint(vec2f(0.0, height));
            outline.push_contour(contour);

            let matrix = transform.matrix;
            let pathfinder_transform = Transform2F {
                matrix: Matrix2x2F::row_major(matrix.a, matrix.c, matrix.b, matrix.d),
                vector: Vector2F::new(matrix.tx.to_pixels() as f32, matrix.ty.to_pixels() as f32),
            };
            outline.transform(&pathfinder_transform);

            let mut paint = Paint::from_pattern(texture.pattern.clone());
            paint.apply_transform(&pathfinder_transform);

            let paint_id = self.scene.push_paint(&paint);
            let draw_path = DrawPath::new(outline, paint_id);
            self.scene.push_draw_path(draw_path);
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        let shape = if let Some(shape) = self.shapes.get(shape.0) {
            shape
        } else {
            return;
        };

        let matrix = transform.matrix;
        let matrix_transform = Transform2F {
            matrix: Matrix2x2F::row_major(matrix.a, matrix.c, matrix.b, matrix.d),
            vector: Vector2F::new(matrix.tx.to_pixels() as f32, matrix.ty.to_pixels() as f32),
        };
        let transform_scales = matrix_transform.extract_scale();
        let transform_scale = f32::min(transform_scales.x(), transform_scales.y());

        for draw in &shape.draws {
            match draw {
                ShapeDraw::Fill { outline, paint } => {
                    let mut outline = outline.clone();
                    outline.transform(&matrix_transform);

                    let mut paint = color_transform_paint(paint, &transform.color_transform);
                    paint.apply_transform(&matrix_transform);
                    let paint_id = self.scene.push_paint(&paint);
                    let draw_path = DrawPath::new(outline, paint_id);

                    self.scene.push_draw_path(draw_path);
                }
                ShapeDraw::Stroke {
                    outline,
                    paint,
                    style,
                } => {
                    let mut outline = outline.clone();
                    outline.transform(&matrix_transform);

                    let mut stroke_style = style.clone();
                    if stroke_style.line_width * transform_scale < 1.0 {
                        stroke_style.line_width = 1.0 / transform_scale;
                    }

                    let mut stroke_to_fill = OutlineStrokeToFill::new(&outline, stroke_style);
                    stroke_to_fill.offset();
                    outline = stroke_to_fill.into_outline();

                    let paint_id = self.scene.push_paint(paint);
                    let draw_path = DrawPath::new(outline, paint_id);

                    self.scene.push_draw_path(draw_path);
                }
            }
        }
    }

    fn draw_letterbox(&mut self, letterbox: Letterbox) {
        match letterbox {
            Letterbox::None => (),
            Letterbox::Letterbox(margin_height) => {
                let mut outline = Outline::new();
                let mut contour = Contour::new();
                contour.push_endpoint(vec2f(0.0, 0.0));
                contour.push_endpoint(vec2f(self.viewport_width, 0.0));
                contour.push_endpoint(vec2f(self.viewport_width, margin_height));
                contour.push_endpoint(vec2f(0.0, margin_height));

                let bottom_margin_y = self.viewport_height - margin_height;
                contour.push_endpoint(vec2f(0.0, bottom_margin_y));
                contour.push_endpoint(vec2f(self.viewport_width, bottom_margin_y));
                contour.push_endpoint(vec2f(self.viewport_width, self.viewport_height));
                contour.push_endpoint(vec2f(0.0, self.viewport_height));
                outline.push_contour(contour);

                let paint_id = self.scene.push_paint(&Paint::black());
                let draw_path = DrawPath::new(outline, paint_id);
                self.scene.push_draw_path(draw_path);
            }
            Letterbox::Pillarbox(margin_width) => {
                let mut outline = Outline::new();
                let mut contour = Contour::new();
                contour.push_endpoint(vec2f(0.0, 0.0));
                contour.push_endpoint(vec2f(0.0, self.viewport_height));
                contour.push_endpoint(vec2f(margin_width, self.viewport_height));
                contour.push_endpoint(vec2f(margin_width, 0.0));

                let margin_x = self.viewport_width - margin_width;
                contour.push_endpoint(vec2f(margin_x, 0.0));
                contour.push_endpoint(vec2f(margin_x, self.viewport_height));
                contour.push_endpoint(vec2f(self.viewport_width, self.viewport_height));
                contour.push_endpoint(vec2f(self.viewport_width, 0.0));
                outline.push_contour(contour);

                let paint_id = self.scene.push_paint(&Paint::black());
                let draw_path = DrawPath::new(outline, paint_id);
                self.scene.push_draw_path(draw_path);
            }
        }
    }

    fn push_mask(&mut self) {
        // TODO
    }

    fn activate_mask(&mut self) {
        // TODO
    }

    fn pop_mask(&mut self) {
        // TODO
    }
}

struct Shape {
    draws: Vec<ShapeDraw>,
}

enum ShapeDraw {
    Fill {
        outline: Outline,
        paint: Paint,
    },
    Stroke {
        outline: Outline,
        paint: Paint,
        style: StrokeStyle,
    },
}

fn ruffle_commands_to_outline(commands: &[ruffle_core::shape_utils::DrawCommand]) -> Outline {
    use ruffle_core::shape_utils::DrawCommand as RuffleCommand;

    let mut outline = Outline::new();
    let mut contour = Contour::new();

    let mut cursor = (Twips::default(), Twips::default());
    for command in commands {
        match command {
            RuffleCommand::MoveTo { x, y } => {
                if !contour.is_empty() {
                    outline.push_contour(std::mem::replace(&mut contour, Contour::new()));
                }
                cursor = (*x, *y);
            }
            RuffleCommand::LineTo { x, y } => {
                if contour.is_empty() {
                    contour.push_endpoint(vec2f(
                        cursor.0.to_pixels() as f32,
                        cursor.1.to_pixels() as f32,
                    ));
                }

                contour.push_endpoint(vec2f(x.to_pixels() as f32, y.to_pixels() as f32));
                cursor = (*x, *y);
            }
            RuffleCommand::CurveTo { x1, y1, x2, y2 } => {
                if contour.is_empty() {
                    contour.push_endpoint(vec2f(
                        cursor.0.to_pixels() as f32,
                        cursor.1.to_pixels() as f32,
                    ));
                }

                contour.push_quadratic(
                    vec2f(x1.to_pixels() as f32, y1.to_pixels() as f32),
                    vec2f(x2.to_pixels() as f32, y2.to_pixels() as f32),
                );
                cursor = (*x2, *y2);
            }
        }
    }

    if !contour.is_empty() {
        outline.push_contour(std::mem::replace(&mut contour, Contour::new()));
    }

    outline
}

struct Texture {
    id: swf::CharacterId,
    width: u32,
    height: u32,
    pattern: Pattern,
}

fn color_transform_paint(
    paint: &Paint,
    color_transform: &ruffle_core::color_transform::ColorTransform,
) -> Paint {
    let r = f32::max(
        0.0,
        f32::min(
            color_transform.r_mult * (paint.base_color().r as f32 / 255.0) + color_transform.r_add,
            1.0,
        ),
    );
    let g = f32::max(
        0.0,
        f32::min(
            color_transform.g_mult * (paint.base_color().g as f32 / 255.0) + color_transform.g_add,
            1.0,
        ),
    );
    let b = f32::max(
        0.0,
        f32::min(
            color_transform.b_mult * (paint.base_color().b as f32 / 255.0) + color_transform.b_add,
            1.0,
        ),
    );
    let a = f32::max(
        0.0,
        f32::min(
            color_transform.a_mult * (paint.base_color().a as f32 / 255.0) + color_transform.a_add,
            1.0,
        ),
    );
    let mut paint = paint.clone();
    paint.set_base_color(rgbau(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    ));
    paint
}
