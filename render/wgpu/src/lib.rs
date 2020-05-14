use lyon::tessellation::{
    self,
    geometry_builder::{BuffersBuilder, FillVertexConstructor, VertexBuffers},
    FillAttributes, FillTessellator, StrokeAttributes, StrokeTessellator, StrokeVertexConstructor,
};
use ruffle_core::backend::render::swf::{self, FillStyle};
use ruffle_core::backend::render::{
    Bitmap, BitmapFormat, BitmapHandle, BitmapInfo, Color, Letterbox, RenderBackend, ShapeHandle,
    Transform,
};
use ruffle_core::shape_utils::{DistilledShape, DrawPath};
use std::convert::TryInto;
use swf::{CharacterId, DefineBitsLossless, Glyph};

use bytemuck::{Pod, Zeroable};
use futures::executor::block_on;
use raw_window_handle::HasRawWindowHandle;

use crate::pipelines::Pipelines;
use crate::shapes::{Draw, DrawType, GradientUniforms, IncompleteDrawType, Mesh};
use crate::target::{RenderTarget, RenderTargetFrame, SwapChainTarget};
use crate::utils::{
    build_view_matrix, create_buffer_with_data, ruffle_path_to_lyon_path, swf_bitmap_to_gl_matrix,
    swf_to_gl_matrix,
};
use ruffle_core::color_transform::ColorTransform;
use std::mem::replace;
use std::rc::Rc;

type Error = Box<dyn std::error::Error>;

#[macro_use]
mod utils;

mod pipelines;
mod shapes;
pub mod target;

pub struct WgpuRenderBackend<T: RenderTarget> {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    target: T,
    msaa_sample_count: u32,
    pipelines: Pipelines,
    frame_buffer_view: wgpu::TextureView,
    depth_texture_view: wgpu::TextureView,
    current_frame: Option<(T::Frame, wgpu::CommandEncoder)>,
    register_encoder: wgpu::CommandEncoder,
    meshes: Vec<Mesh>,
    viewport_width: f32,
    viewport_height: f32,
    view_matrix: [[f32; 4]; 4],
    textures: Vec<(swf::CharacterId, Texture)>,
    num_masks: u32,
    num_masks_active: u32,
    write_stencil_mask: u32,
    test_stencil_mask: u32,
    next_stencil_mask: u32,
    mask_stack: Vec<(u32, u32)>,
    quad_vbo: wgpu::Buffer,
    quad_ibo: wgpu::Buffer,
    quad_tex_transforms: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Transforms {
    view_matrix: [[f32; 4]; 4],
    world_matrix: [[f32; 4]; 4],
}

unsafe impl Pod for Transforms {}
unsafe impl Zeroable for Transforms {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct TextureTransforms {
    u_matrix: [[f32; 4]; 4],
}

unsafe impl Pod for TextureTransforms {}
unsafe impl Zeroable for TextureTransforms {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct ColorAdjustments {
    mult_color: [f32; 4],
    add_color: [f32; 4],
}

impl From<ColorTransform> for ColorAdjustments {
    fn from(transform: ColorTransform) -> Self {
        Self {
            mult_color: [
                transform.r_mult,
                transform.g_mult,
                transform.b_mult,
                transform.a_mult,
            ],
            add_color: [
                transform.r_add,
                transform.g_add,
                transform.b_add,
                transform.a_add,
            ],
        }
    }
}

unsafe impl Pod for ColorAdjustments {}
unsafe impl Zeroable for ColorAdjustments {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GPUVertex {
    position: [f32; 2],
    color: [f32; 4],
}

unsafe impl Pod for GPUVertex {}
unsafe impl Zeroable for GPUVertex {}

impl WgpuRenderBackend<SwapChainTarget> {
    pub fn for_window<W: HasRawWindowHandle>(window: &W, size: (u32, u32)) -> Result<Self, Error> {
        let surface = wgpu::Surface::create(window);

        let adapter = block_on(wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        ))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        }));

        let target = SwapChainTarget::new(surface, size, &device);
        Self::new(Rc::new(device), Rc::new(queue), target)
    }
}

impl<T: RenderTarget> WgpuRenderBackend<T> {
    pub fn new(device: Rc<wgpu::Device>, queue: Rc<wgpu::Queue>, target: T) -> Result<Self, Error> {
        // TODO: Allow this to be set from command line/settings file.
        let msaa_sample_count = 4;

        let pipelines = Pipelines::new(&device, msaa_sample_count)?;

        let extent = wgpu::Extent3d {
            width: target.width(),
            height: target.height(),
            depth: 1,
        };

        let frame_buffer_label = create_debug_label!("Framebuffer texture");
        let frame_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: frame_buffer_label.as_deref(),
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: target.format(),
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let frame_buffer_view = frame_buffer.create_default_view();

        let depth_label = create_debug_label!("Depth texture");
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: depth_label.as_deref(),
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let register_encoder_label = create_debug_label!("Register encoder");
        let register_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: register_encoder_label.as_deref(),
        });

        let depth_texture_view = depth_texture.create_default_view();

        let (quad_vbo, quad_ibo, quad_tex_transforms) = create_quad_buffers(&device);

        let viewport_width = target.width() as f32;
        let viewport_height = target.height() as f32;
        let view_matrix = build_view_matrix(target.width(), target.height());

        Ok(Self {
            device,
            queue,
            target,
            msaa_sample_count,
            pipelines,
            frame_buffer_view,
            depth_texture_view,
            current_frame: None,
            register_encoder,
            meshes: Vec::new(),
            viewport_width,
            viewport_height,
            view_matrix,
            textures: Vec::new(),
            num_masks: 0,
            num_masks_active: 0,
            write_stencil_mask: 0,
            test_stencil_mask: 0,
            next_stencil_mask: 1,
            mask_stack: Vec::new(),
            quad_vbo,
            quad_ibo,
            quad_tex_transforms,
        })
    }

    #[allow(clippy::cognitive_complexity)]
    fn register_shape_internal(&mut self, shape: DistilledShape) -> ShapeHandle {
        let handle = ShapeHandle(self.meshes.len());

        use lyon::tessellation::{FillOptions, StrokeOptions};

        let transforms_label = create_debug_label!("Shape {} transforms ubo", shape.id);
        let transforms_ubo = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: transforms_label.as_deref(),
            size: std::mem::size_of::<Transforms>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let colors_ubo = create_buffer_with_data(
            &self.device,
            bytemuck::cast_slice(&[ColorAdjustments {
                mult_color: [1.0, 1.0, 1.0, 1.0],
                add_color: [0.0, 0.0, 0.0, 0.0],
            }]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            create_debug_label!("Shape {} colors ubo", shape.id),
        );

        let mut draws = Vec::new();

        let mut fill_tess = FillTessellator::new();
        let mut stroke_tess = StrokeTessellator::new();
        let mut lyon_mesh: VertexBuffers<_, u16> = VertexBuffers::new();

        #[allow(clippy::too_many_arguments)]
        fn flush_draw(
            shape_id: CharacterId,
            draw: IncompleteDrawType,
            draws: &mut Vec<Draw>,
            lyon_mesh: &mut VertexBuffers<GPUVertex, u16>,
            device: &wgpu::Device,
            transforms_ubo: &wgpu::Buffer,
            colors_ubo: &wgpu::Buffer,
            pipelines: &Pipelines,
        ) {
            if lyon_mesh.vertices.is_empty() || lyon_mesh.indices.len() < 3 {
                return;
            }

            let vbo = create_buffer_with_data(
                device,
                bytemuck::cast_slice(&lyon_mesh.vertices),
                wgpu::BufferUsage::VERTEX,
                create_debug_label!("Shape {} ({}) vbo", shape_id, draw.name()),
            );

            let ibo = create_buffer_with_data(
                device,
                bytemuck::cast_slice(&lyon_mesh.indices),
                wgpu::BufferUsage::INDEX,
                create_debug_label!("Shape {} ({}) ibo", shape_id, draw.name()),
            );

            let draw_id = draws.len();

            draws.push(draw.build(
                device,
                transforms_ubo,
                colors_ubo,
                vbo,
                ibo,
                lyon_mesh.indices.len() as u32,
                pipelines,
                shape_id,
                draw_id,
            ));

            *lyon_mesh = VertexBuffers::new();
        }

        for path in shape.paths {
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
                        flush_draw(
                            shape.id,
                            IncompleteDrawType::Color,
                            &mut draws,
                            &mut lyon_mesh,
                            &self.device,
                            &transforms_ubo,
                            &colors_ubo,
                            &self.pipelines,
                        );

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

                        let mut colors: [[f32; 4]; 16] = Default::default();
                        let mut ratios: [f32; 16] = Default::default();
                        for (i, record) in gradient.records.iter().enumerate() {
                            if i >= 16 {
                                // TODO: we need to support these!
                                break;
                            }
                            colors[i] = [
                                f32::from(record.color.r) / 255.0,
                                f32::from(record.color.g) / 255.0,
                                f32::from(record.color.b) / 255.0,
                                f32::from(record.color.a) / 255.0,
                            ];
                            ratios[i] = f32::from(record.ratio) / 255.0;
                        }

                        let uniforms = GradientUniforms {
                            gradient_type: 0,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            repeat_mode: 0,
                            focal_point: 0.0,
                        };
                        let matrix = swf_to_gl_matrix(gradient.matrix.clone());

                        flush_draw(
                            shape.id,
                            IncompleteDrawType::Gradient {
                                texture_transform: matrix,
                                gradient: uniforms,
                            },
                            &mut draws,
                            &mut lyon_mesh,
                            &self.device,
                            &transforms_ubo,
                            &colors_ubo,
                            &self.pipelines,
                        );
                    }
                    FillStyle::RadialGradient(gradient) => {
                        flush_draw(
                            shape.id,
                            IncompleteDrawType::Color,
                            &mut draws,
                            &mut lyon_mesh,
                            &self.device,
                            &transforms_ubo,
                            &colors_ubo,
                            &self.pipelines,
                        );

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

                        let mut colors: [[f32; 4]; 16] = Default::default();
                        let mut ratios: [f32; 16] = Default::default();
                        for (i, record) in gradient.records.iter().enumerate() {
                            if i >= 16 {
                                // TODO: we need to support these!
                                break;
                            }
                            colors[i] = [
                                f32::from(record.color.r) / 255.0,
                                f32::from(record.color.g) / 255.0,
                                f32::from(record.color.b) / 255.0,
                                f32::from(record.color.a) / 255.0,
                            ];
                            ratios[i] = f32::from(record.ratio) / 255.0;
                        }

                        let uniforms = GradientUniforms {
                            gradient_type: 1,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            repeat_mode: 0,
                            focal_point: 0.0,
                        };
                        let matrix = swf_to_gl_matrix(gradient.matrix.clone());

                        flush_draw(
                            shape.id,
                            IncompleteDrawType::Gradient {
                                texture_transform: matrix,
                                gradient: uniforms,
                            },
                            &mut draws,
                            &mut lyon_mesh,
                            &self.device,
                            &transforms_ubo,
                            &colors_ubo,
                            &self.pipelines,
                        );
                    }
                    FillStyle::FocalGradient {
                        gradient,
                        focal_point,
                    } => {
                        flush_draw(
                            shape.id,
                            IncompleteDrawType::Color,
                            &mut draws,
                            &mut lyon_mesh,
                            &self.device,
                            &transforms_ubo,
                            &colors_ubo,
                            &self.pipelines,
                        );

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

                        let mut colors: [[f32; 4]; 16] = Default::default();
                        let mut ratios: [f32; 16] = Default::default();
                        for (i, record) in gradient.records.iter().enumerate() {
                            if i >= 16 {
                                // TODO: we need to support these!
                                break;
                            }
                            colors[i] = [
                                f32::from(record.color.r) / 255.0,
                                f32::from(record.color.g) / 255.0,
                                f32::from(record.color.b) / 255.0,
                                f32::from(record.color.a) / 255.0,
                            ];
                            ratios[i] = f32::from(record.ratio) / 255.0;
                        }

                        let uniforms = GradientUniforms {
                            gradient_type: 1,
                            ratios,
                            colors,
                            num_colors: gradient.records.len() as u32,
                            repeat_mode: 0,
                            focal_point: *focal_point,
                        };
                        let matrix = swf_to_gl_matrix(gradient.matrix.clone());

                        flush_draw(
                            shape.id,
                            IncompleteDrawType::Gradient {
                                texture_transform: matrix,
                                gradient: uniforms,
                            },
                            &mut draws,
                            &mut lyon_mesh,
                            &self.device,
                            &transforms_ubo,
                            &colors_ubo,
                            &self.pipelines,
                        );
                    }
                    FillStyle::Bitmap {
                        id,
                        matrix,
                        is_smoothed,
                        is_repeating,
                    } => {
                        flush_draw(
                            shape.id,
                            IncompleteDrawType::Color,
                            &mut draws,
                            &mut lyon_mesh,
                            &self.device,
                            &transforms_ubo,
                            &colors_ubo,
                            &self.pipelines,
                        );

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

                        let texture = match self
                            .textures
                            .iter()
                            .find(|(other_id, _tex)| *other_id == *id)
                        {
                            None => {
                                log::error!("Couldn't fill shape with unknown bitmap {}", id);
                                continue;
                            }
                            Some(t) => &t.1,
                        };
                        let texture_view = texture.texture.create_default_view();

                        flush_draw(
                            shape.id,
                            IncompleteDrawType::Bitmap {
                                texture_transform: swf_bitmap_to_gl_matrix(
                                    matrix.clone(),
                                    texture.width,
                                    texture.height,
                                ),
                                is_smoothed: *is_smoothed,
                                is_repeating: *is_repeating,
                                texture_view,
                                id: *id,
                            },
                            &mut draws,
                            &mut lyon_mesh,
                            &self.device,
                            &transforms_ubo,
                            &colors_ubo,
                            &self.pipelines,
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

        flush_draw(
            shape.id,
            IncompleteDrawType::Color,
            &mut draws,
            &mut lyon_mesh,
            &self.device,
            &transforms_ubo,
            &colors_ubo,
            &self.pipelines,
        );

        self.meshes.push(Mesh {
            draws,
            transforms: transforms_ubo,
            colors_buffer: colors_ubo,
            colors_last: ColorTransform::default(),
            shape_id: shape.id,
        });

        handle
    }

    fn register_bitmap(
        &mut self,
        id: swf::CharacterId,
        bitmap: Bitmap,
        debug_str: &str,
    ) -> Result<BitmapInfo, Error> {
        let extent = wgpu::Extent3d {
            width: bitmap.width,
            height: bitmap.height,
            depth: 1,
        };

        let data = match bitmap.data {
            BitmapFormat::Rgba(data) => data,
            BitmapFormat::Rgb(data) => {
                // Expand to RGBA.
                let mut as_rgba =
                    Vec::with_capacity(extent.width as usize * extent.height as usize * 4);
                for i in (0..data.len()).step_by(3) {
                    as_rgba.push(data[i]);
                    as_rgba.push(data[i + 1]);
                    as_rgba.push(data[i + 2]);
                    as_rgba.push(255);
                }
                as_rgba
            }
        };

        let texture_label = create_debug_label!("{} Texture {}", debug_str, id);
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: texture_label.as_deref(),
            size: extent,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let buffer = create_buffer_with_data(
            &self.device,
            &data,
            wgpu::BufferUsage::COPY_SRC,
            create_debug_label!("{} transfer buffer {}", debug_str, id),
        );

        self.register_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * extent.width,
                rows_per_image: 0,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            extent,
        );

        let handle = BitmapHandle(self.textures.len());
        self.textures.push((
            id,
            Texture {
                texture,
                width: bitmap.width,
                height: bitmap.height,
            },
        ));

        Ok(BitmapInfo {
            handle,
            width: bitmap.width.try_into().unwrap(),
            height: bitmap.height.try_into().unwrap(),
        })
    }

    pub fn target(&self) -> &T {
        &self.target
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        let (frame_output, encoder) = if let Some((frame_output, encoder)) = &mut self.current_frame
        {
            (frame_output, encoder)
        } else {
            return;
        };

        let world_matrix = [
            [width, 0.0, 0.0, 0.0],
            [0.0, height, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [x, y, 0.0, 1.0],
        ];

        let mult_color = [
            f32::from(color.r) / 255.0,
            f32::from(color.g) / 255.0,
            f32::from(color.b) / 255.0,
            f32::from(color.a) / 255.0,
        ];

        let add_color = [0.0, 0.0, 0.0, 0.0];

        let transforms_ubo = create_buffer_with_data(
            &self.device,
            bytemuck::cast_slice(&[Transforms {
                view_matrix: self.view_matrix,
                world_matrix,
            }]),
            wgpu::BufferUsage::UNIFORM,
            create_debug_label!("Rectangle transfer buffer"),
        );

        let colors_ubo = create_buffer_with_data(
            &self.device,
            bytemuck::cast_slice(&[ColorAdjustments {
                mult_color,
                add_color,
            }]),
            wgpu::BufferUsage::UNIFORM,
            create_debug_label!("Rectangle colors transfer buffer"),
        );

        let bind_group_label = create_debug_label!("Rectangle bind group");
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipelines.color.bind_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &transforms_ubo,
                        range: 0..std::mem::size_of::<Transforms>() as u64,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &colors_ubo,
                        range: 0..std::mem::size_of::<ColorAdjustments>() as u64,
                    },
                },
            ],
            label: bind_group_label.as_deref(),
        });

        let (color_attachment, resolve_target) = if self.msaa_sample_count >= 2 {
            (&self.frame_buffer_view, Some(frame_output.view()))
        } else {
            (frame_output.view(), None)
        };
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: color_attachment,
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::WHITE,
                resolve_target,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture_view,
                depth_load_op: wgpu::LoadOp::Load,
                depth_store_op: wgpu::StoreOp::Store,
                stencil_load_op: wgpu::LoadOp::Load,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_depth: 0.0,
                clear_stencil: 0,
            }),
        });

        render_pass.set_pipeline(&self.pipelines.color.pipeline_for(
            self.num_masks,
            self.num_masks_active,
            self.test_stencil_mask,
            self.write_stencil_mask,
        ));
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, &self.quad_vbo, 0, 0);
        render_pass.set_index_buffer(&self.quad_ibo, 0, 0);

        if self.num_masks_active < self.num_masks {
            render_pass.set_stencil_reference(self.write_stencil_mask);
        } else {
            render_pass.set_stencil_reference(self.test_stencil_mask);
        }

        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}

impl<T: RenderTarget + 'static> RenderBackend for WgpuRenderBackend<T> {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        // Avoid panics from creating 0-sized framebuffers.
        let width = std::cmp::max(width, 1);
        let height = std::cmp::max(height, 1);

        self.target.resize(&self.device, width, height);

        let label = create_debug_label!("Framebuffer texture");
        let frame_buffer = self.device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: self.msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: self.target.format(),
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        self.frame_buffer_view = frame_buffer.create_default_view();

        let label = create_debug_label!("Depth texture");
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: self.msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        self.depth_texture_view = depth_texture.create_default_view();

        self.viewport_width = width as f32;
        self.viewport_height = height as f32;
        self.view_matrix = build_view_matrix(width, height);
    }

    fn register_shape(&mut self, shape: DistilledShape) -> ShapeHandle {
        self.register_shape_internal(shape)
    }

    fn register_glyph_shape(&mut self, glyph: &Glyph) -> ShapeHandle {
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
        self.register_shape_internal((&shape).into())
    }

    fn register_bitmap_jpeg(
        &mut self,
        id: u16,
        data: &[u8],
        jpeg_tables: Option<&[u8]>,
    ) -> BitmapInfo {
        let data = ruffle_core::backend::render::glue_tables_to_jpeg(data, jpeg_tables);
        self.register_bitmap_jpeg_2(id, &data[..])
    }

    fn register_bitmap_jpeg_2(&mut self, id: u16, data: &[u8]) -> BitmapInfo {
        let bitmap = ruffle_core::backend::render::decode_define_bits_jpeg(data, None)
            .expect("Invalid DefineBitsJpeg2 data");
        self.register_bitmap(id, bitmap, "JPEG2")
            .expect("Unable to register bitmap")
    }

    fn register_bitmap_jpeg_3(
        &mut self,
        id: u16,
        jpeg_data: &[u8],
        alpha_data: &[u8],
    ) -> BitmapInfo {
        let bitmap =
            ruffle_core::backend::render::decode_define_bits_jpeg(jpeg_data, Some(alpha_data))
                .expect("Invalid DefineBitsJpeg3 data");
        self.register_bitmap(id, bitmap, "JPEG3")
            .expect("Unable to register bitmap")
    }

    fn register_bitmap_png(&mut self, swf_tag: &DefineBitsLossless) -> BitmapInfo {
        let bitmap = ruffle_core::backend::render::decode_define_bits_lossless(swf_tag)
            .expect("Invalid DefineBitsJpeg2 data");
        self.register_bitmap(swf_tag.id, bitmap, "PNG")
            .expect("Unable to register bitmap")
    }

    fn begin_frame(&mut self, clear: Color) {
        assert!(self.current_frame.is_none());
        self.current_frame = match self.target.get_next_texture() {
            Ok(frame) => {
                let label = create_debug_label!("Frame encoder");
                Some((
                    frame,
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: label.as_deref(),
                        }),
                ))
            }
            Err(wgpu::TimeOut) => {
                log::warn!("Couldn't begin new render frame: timed out whilst aquiring new swapchain output");
                None
            }
        };
        self.num_masks = 0;
        self.num_masks_active = 0;
        self.write_stencil_mask = 0;
        self.test_stencil_mask = 0;
        self.next_stencil_mask = 1;

        if let Some((frame_output, encoder)) = &mut self.current_frame {
            let (color_attachment, resolve_target) = if self.msaa_sample_count >= 2 {
                (&self.frame_buffer_view, Some(frame_output.view()))
            } else {
                (frame_output.view(), None)
            };
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: color_attachment,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: f64::from(clear.r) / 255.0,
                        g: f64::from(clear.g) / 255.0,
                        b: f64::from(clear.b) / 255.0,
                        a: f64::from(clear.a) / 255.0,
                    },
                    resolve_target,
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_depth: 0.0,
                    clear_stencil: 0,
                }),
            });
        }
    }

    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform) {
        if let Some((_id, texture)) = self.textures.get(bitmap.0) {
            let (frame_output, encoder) =
                if let Some((frame_output, encoder)) = &mut self.current_frame {
                    (frame_output, encoder)
                } else {
                    return;
                };

            use ruffle_core::matrix::Matrix;
            let transform = Transform {
                matrix: transform.matrix
                    * Matrix {
                        a: texture.width as f32,
                        d: texture.height as f32,
                        ..Default::default()
                    },
                ..*transform
            };

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

            let transforms_ubo = create_buffer_with_data(
                &self.device,
                bytemuck::cast_slice(&[Transforms {
                    view_matrix: self.view_matrix,
                    world_matrix,
                }]),
                wgpu::BufferUsage::UNIFORM,
                create_debug_label!("Bitmap {} transforms transfer buffer", bitmap.0),
            );

            let colors_ubo = create_buffer_with_data(
                &self.device,
                bytemuck::cast_slice(&[ColorAdjustments::from(transform.color_transform)]),
                wgpu::BufferUsage::UNIFORM,
                create_debug_label!("Bitmap {} colors transfer buffer", bitmap.0),
            );

            let texture_view = texture.texture.create_default_view();
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                compare: wgpu::CompareFunction::Undefined,
            });

            let bind_group_label = create_debug_label!("Bitmap {} bind group", bitmap.0);
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.pipelines.bitmap.bind_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &transforms_ubo,
                            range: 0..std::mem::size_of::<Transforms>() as u64,
                        },
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &self.quad_tex_transforms,
                            range: 0..std::mem::size_of::<TextureTransforms>() as u64,
                        },
                    },
                    wgpu::Binding {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &colors_ubo,
                            range: 0..std::mem::size_of::<ColorAdjustments>() as u64,
                        },
                    },
                    wgpu::Binding {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::Binding {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: bind_group_label.as_deref(),
            });

            let (color_attachment, resolve_target) = if self.msaa_sample_count >= 2 {
                (&self.frame_buffer_view, Some(frame_output.view()))
            } else {
                (frame_output.view(), None)
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: color_attachment,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::WHITE,
                    resolve_target,
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_load_op: wgpu::LoadOp::Load,
                    depth_store_op: wgpu::StoreOp::Store,
                    stencil_load_op: wgpu::LoadOp::Load,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_depth: 0.0,
                    clear_stencil: 0,
                }),
            });

            render_pass.set_pipeline(&self.pipelines.bitmap.pipeline_for(
                self.num_masks,
                self.num_masks_active,
                self.test_stencil_mask,
                self.write_stencil_mask,
            ));
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, &self.quad_vbo, 0, 0);
            render_pass.set_index_buffer(&self.quad_ibo, 0, 0);

            if self.num_masks_active < self.num_masks {
                render_pass.set_stencil_reference(self.write_stencil_mask);
            } else {
                render_pass.set_stencil_reference(self.test_stencil_mask);
            }

            render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        let (frame_output, encoder) = if let Some((frame_output, encoder)) = &mut self.current_frame
        {
            (frame_output, encoder)
        } else {
            return;
        };

        let mesh = &mut self.meshes[shape.0];

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

        if transform.color_transform != mesh.colors_last {
            let colors_temp = create_buffer_with_data(
                &self.device,
                bytemuck::cast_slice(&[ColorAdjustments::from(transform.color_transform)]),
                wgpu::BufferUsage::COPY_SRC,
                create_debug_label!("Shape {} colors transfer buffer", mesh.shape_id),
            );

            encoder.copy_buffer_to_buffer(
                &colors_temp,
                0,
                &mesh.colors_buffer,
                0,
                std::mem::size_of::<ColorAdjustments>() as u64,
            );

            mesh.colors_last = transform.color_transform;
        }

        let transforms_temp = create_buffer_with_data(
            &self.device,
            bytemuck::cast_slice(&[Transforms {
                view_matrix: self.view_matrix,
                world_matrix,
            }]),
            wgpu::BufferUsage::COPY_SRC,
            create_debug_label!("Shape {} transforms transfer buffer", mesh.shape_id),
        );

        encoder.copy_buffer_to_buffer(
            &transforms_temp,
            0,
            &mesh.transforms,
            0,
            std::mem::size_of::<Transforms>() as u64,
        );

        let (color_attachment, resolve_target) = if self.msaa_sample_count >= 2 {
            (&self.frame_buffer_view, Some(frame_output.view()))
        } else {
            (frame_output.view(), None)
        };
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: color_attachment,
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::WHITE,
                resolve_target,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture_view,
                depth_load_op: wgpu::LoadOp::Load,
                depth_store_op: wgpu::StoreOp::Store,
                stencil_load_op: wgpu::LoadOp::Load,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_depth: 0.0,
                clear_stencil: 0,
            }),
        });

        for draw in &mesh.draws {
            match &draw.draw_type {
                DrawType::Color => {
                    render_pass.set_pipeline(&self.pipelines.color.pipeline_for(
                        self.num_masks,
                        self.num_masks_active,
                        self.test_stencil_mask,
                        self.write_stencil_mask,
                    ));
                }
                DrawType::Gradient { .. } => {
                    render_pass.set_pipeline(&self.pipelines.gradient.pipeline_for(
                        self.num_masks,
                        self.num_masks_active,
                        self.test_stencil_mask,
                        self.write_stencil_mask,
                    ));
                }
                DrawType::Bitmap { .. } => {
                    render_pass.set_pipeline(&self.pipelines.bitmap.pipeline_for(
                        self.num_masks,
                        self.num_masks_active,
                        self.test_stencil_mask,
                        self.write_stencil_mask,
                    ));
                }
            }

            render_pass.set_bind_group(0, &draw.bind_group, &[]);
            render_pass.set_vertex_buffer(0, &draw.vertex_buffer, 0, 0);
            render_pass.set_index_buffer(&draw.index_buffer, 0, 0);

            if self.num_masks_active < self.num_masks {
                render_pass.set_stencil_reference(self.write_stencil_mask);
            } else {
                render_pass.set_stencil_reference(self.test_stencil_mask);
            }

            render_pass.draw_indexed(0..draw.index_count, 0, 0..1);
        }
    }

    fn end_frame(&mut self) {
        if let Some((_frame, encoder)) = self.current_frame.take() {
            let register_encoder_label = create_debug_label!("Register encoder");
            let new_register_encoder =
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: register_encoder_label.as_deref(),
                    });
            let register_buffer =
                replace(&mut self.register_encoder, new_register_encoder).finish();
            self.target.submit(
                &self.device,
                &self.queue,
                &[register_buffer, encoder.finish()],
            );
        }
    }

    fn draw_letterbox(&mut self, letterbox: Letterbox) {
        match letterbox {
            Letterbox::None => {}
            Letterbox::Letterbox(margin) => {
                self.draw_rect(
                    0.0,
                    0.0,
                    self.viewport_width,
                    margin,
                    Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                );
                self.draw_rect(
                    0.0,
                    self.viewport_height - margin,
                    self.viewport_width,
                    margin,
                    Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                );
            }
            Letterbox::Pillarbox(margin) => {
                self.draw_rect(
                    0.0,
                    0.0,
                    margin,
                    self.viewport_height,
                    Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                );
                self.draw_rect(
                    self.viewport_width - margin,
                    0.0,
                    margin,
                    self.viewport_height,
                    Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                );
            }
        }
    }

    fn push_mask(&mut self) {
        // Desktop draws the masker to the stencil buffer, one bit per mask.
        // Masks-within-masks are handled as a bitmask.
        // This does unfortunately mean we are limited in the number of masks at once (8 bits).
        if self.next_stencil_mask >= 0x100 {
            // If we've reached the limit of masks, clear the stencil buffer and start over.
            // But this may not be correct if there is still a mask active (mask-within-mask).
            if self.test_stencil_mask != 0 {
                log::warn!(
                    "Too many masks active for stencil buffer; possibly incorrect rendering"
                );
            }
            self.next_stencil_mask = 1;
            if let Some((frame_output, encoder)) = &mut self.current_frame {
                let (color_attachment, resolve_target) = if self.msaa_sample_count >= 2 {
                    (&self.frame_buffer_view, Some(frame_output.view()))
                } else {
                    (frame_output.view(), None)
                };
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: color_attachment,
                        load_op: wgpu::LoadOp::Load,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::WHITE,
                        resolve_target,
                    }],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachmentDescriptor {
                            attachment: &self.depth_texture_view,
                            depth_load_op: wgpu::LoadOp::Load,
                            depth_store_op: wgpu::StoreOp::Store,
                            stencil_load_op: wgpu::LoadOp::Clear,
                            stencil_store_op: wgpu::StoreOp::Store,
                            clear_depth: 0.0,
                            clear_stencil: self.test_stencil_mask,
                        },
                    ),
                });
            }
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
        }
    }
}

fn create_quad_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    let vertices = [
        GPUVertex {
            position: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        GPUVertex {
            position: [1.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        GPUVertex {
            position: [1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
        GPUVertex {
            position: [0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        },
    ];
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    let vbo = create_buffer_with_data(
        device,
        bytemuck::cast_slice(&vertices),
        wgpu::BufferUsage::VERTEX,
        create_debug_label!("Quad vbo"),
    );

    let ibo = create_buffer_with_data(
        device,
        bytemuck::cast_slice(&indices),
        wgpu::BufferUsage::INDEX,
        create_debug_label!("Quad ibo"),
    );

    let tex_transforms = create_buffer_with_data(
        device,
        bytemuck::cast_slice(&[TextureTransforms {
            u_matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }]),
        wgpu::BufferUsage::UNIFORM,
        create_debug_label!("Quad tex transforms"),
    );

    (vbo, ibo, tex_transforms)
}

#[derive(Debug)]
struct Texture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
}

struct RuffleVertexCtor {
    color: [f32; 4],
}

impl FillVertexConstructor<GPUVertex> for RuffleVertexCtor {
    fn new_vertex(&mut self, position: lyon::math::Point, _: FillAttributes) -> GPUVertex {
        GPUVertex {
            position: [position.x, position.y],
            color: self.color,
        }
    }
}

impl StrokeVertexConstructor<GPUVertex> for RuffleVertexCtor {
    fn new_vertex(&mut self, position: lyon::math::Point, _: StrokeAttributes) -> GPUVertex {
        GPUVertex {
            position: [position.x, position.y],
            color: self.color,
        }
    }
}
