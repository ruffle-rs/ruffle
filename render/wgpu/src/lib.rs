use ruffle_core::backend::render::{
    Bitmap, BitmapFormat, BitmapHandle, BitmapInfo, BitmapSource, Color, RenderBackend,
    ShapeHandle, Transform,
};
use ruffle_core::shape_utils::DistilledShape;
use ruffle_core::swf;
use std::{borrow::Cow, num::NonZeroU32};

use bytemuck::{Pod, Zeroable};

use crate::pipelines::Pipelines;
use crate::target::{RenderTarget, RenderTargetFrame, SwapChainTarget};
use crate::utils::{create_buffer_with_data, format_list, get_backend_names};
use enum_map::Enum;
use ruffle_core::color_transform::ColorTransform;
use ruffle_render_common_tess::{
    DrawType as TessDrawType, Gradient as TessGradient, GradientType, ShapeTessellator,
    Vertex as TessVertex,
};

type Error = Box<dyn std::error::Error>;

#[macro_use]
mod utils;

mod bitmaps;
mod globals;
mod pipelines;
pub mod target;
mod uniform_buffer;

#[cfg(feature = "clap")]
pub mod clap;

use crate::bitmaps::BitmapSamplers;
use crate::globals::Globals;
use crate::uniform_buffer::UniformBuffer;
use std::collections::HashMap;
use std::path::Path;
pub use wgpu;

pub struct Descriptors {
    pub device: wgpu::Device,
    pub info: wgpu::AdapterInfo,
    pub limits: wgpu::Limits,
    pub surface_format: wgpu::TextureFormat,
    queue: wgpu::Queue,
    globals: Globals,
    uniform_buffers: UniformBuffer<Transforms>,
    pipelines: Pipelines,
    bitmap_samplers: BitmapSamplers,
    msaa_sample_count: u32,
}

impl Descriptors {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        info: wgpu::AdapterInfo,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, Error> {
        let limits = device.limits();
        // TODO: Allow this to be set from command line/settings file.
        let msaa_sample_count = 4;
        let bitmap_samplers = BitmapSamplers::new(&device);
        let globals = Globals::new(&device);
        let uniform_buffer_layout_label = create_debug_label!("Uniform buffer bind group layout");
        let uniform_buffer_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: uniform_buffer_layout_label.as_deref(),
            });
        let uniform_buffers = UniformBuffer::new(
            uniform_buffer_layout,
            limits.min_uniform_buffer_offset_alignment,
        );
        let pipelines = Pipelines::new(
            &device,
            surface_format,
            msaa_sample_count,
            bitmap_samplers.layout(),
            globals.layout(),
            uniform_buffers.layout(),
        )?;

        Ok(Self {
            device,
            info,
            limits,
            surface_format,
            queue,
            globals,
            uniform_buffers,
            pipelines,
            bitmap_samplers,
            msaa_sample_count,
        })
    }
}

pub struct WgpuRenderBackend<T: RenderTarget> {
    descriptors: Descriptors,
    target: T,
    frame_buffer_view: wgpu::TextureView,
    depth_texture_view: wgpu::TextureView,
    current_frame: Option<Frame<'static, T>>,
    meshes: Vec<Mesh>,
    mask_state: MaskState,
    shape_tessellator: ShapeTessellator,
    textures: Vec<Texture>,
    num_masks: u32,
    quad_vbo: wgpu::Buffer,
    quad_ibo: wgpu::Buffer,
    quad_tex_transforms: wgpu::Buffer,
    bitmap_registry: HashMap<BitmapHandle, Bitmap>,
}

#[allow(dead_code)]
struct Frame<'a, T: RenderTarget> {
    frame_data: Box<(wgpu::CommandEncoder, T::Frame, wgpu::CommandEncoder)>,

    // TODO: This is a self-reference to the above, so we
    // use some unsafe to cast the lifetime away. We know this
    // is safe because the anpve data should live for the
    // entire frame and is boxed to have a stable address.
    // We could clean this up later by adjusting the
    // RenderBackend interface to return a Frame object.
    render_pass: wgpu::RenderPass<'a>,
}

impl<'a, T: RenderTarget> Frame<'static, T> {
    // Get a reference to the render pass with the proper lifetime.
    fn get(&mut self) -> &mut Frame<'a, T> {
        unsafe { std::mem::transmute::<_, &mut Frame<'a, T>>(self) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum MaskState {
    NoMask,
    DrawMaskStencil,
    DrawMaskedContent,
    ClearMaskStencil,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Transforms {
    world_matrix: [[f32; 4]; 4],
    color_adjustments: ColorAdjustments,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TextureTransforms {
    u_matrix: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ColorAdjustments {
    mult_color: [f32; 4],
    add_color: [f32; 4],
}

impl From<ColorTransform> for ColorAdjustments {
    fn from(transform: ColorTransform) -> Self {
        Self {
            mult_color: transform.mult_rgba_normalized(),
            add_color: transform.add_rgba_normalized(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl From<TessVertex> for Vertex {
    fn from(vertex: TessVertex) -> Self {
        Self {
            position: [vertex.x, vertex.y],
            color: [
                f32::from(vertex.color.r) / 255.0,
                f32::from(vertex.color.g) / 255.0,
                f32::from(vertex.color.b) / 255.0,
                f32::from(vertex.color.a) / 255.0,
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GradientUniforms {
    colors: [[f32; 4]; 16],
    ratios: [f32; 16],
    gradient_type: i32,
    num_colors: u32,
    repeat_mode: i32,
    interpolation: i32,
    focal_point: f32,
}

impl From<TessGradient> for GradientUniforms {
    fn from(gradient: TessGradient) -> Self {
        let mut ratios = [0.0; 16];
        let mut colors = [[0.0; 4]; 16];
        ratios[..gradient.num_colors].copy_from_slice(&gradient.ratios[..gradient.num_colors]);
        colors[..gradient.num_colors].copy_from_slice(&gradient.colors[..gradient.num_colors]);

        Self {
            colors,
            ratios,
            gradient_type: match gradient.gradient_type {
                GradientType::Linear => 0,
                GradientType::Radial => 1,
                GradientType::Focal => 2,
            },
            num_colors: gradient.num_colors as u32,
            repeat_mode: match gradient.repeat_mode {
                swf::GradientSpread::Pad => 0,
                swf::GradientSpread::Repeat => 1,
                swf::GradientSpread::Reflect => 2,
            },
            interpolation: (gradient.interpolation == swf::GradientInterpolation::LinearRgb) as i32,
            focal_point: gradient.focal_point.to_f32(),
        }
    }
}

#[derive(Debug)]
struct Mesh {
    draws: Vec<Draw>,
}

#[derive(Debug)]
struct Draw {
    draw_type: DrawType,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
enum DrawType {
    Color,
    Gradient {
        texture_transforms: wgpu::Buffer,
        gradient: wgpu::Buffer,
        bind_group: wgpu::BindGroup,
    },
    Bitmap {
        texture_transforms: wgpu::Buffer,
        texture_view: wgpu::TextureView,
        is_smoothed: bool,
        is_repeating: bool,
        bind_group: wgpu::BindGroup,
    },
}

impl WgpuRenderBackend<SwapChainTarget> {
    #[cfg(target_family = "wasm")]
    pub async fn for_canvas(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::Backends::BROWSER_WEBGPU);
        let surface = unsafe { instance.create_surface_from_canvas(canvas) };
        let descriptors = Self::build_descriptors(
            wgpu::Backends::BROWSER_WEBGPU,
            instance,
            Some(&surface),
            wgpu::PowerPreference::HighPerformance,
            None,
        )
        .await?;
        let target = SwapChainTarget::new(
            surface,
            descriptors.surface_format,
            (1, 1),
            &descriptors.device,
        );
        Self::new(descriptors, target)
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn for_window<W: raw_window_handle::HasRawWindowHandle>(
        window: &W,
        size: (u32, u32),
        backend: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<Self, Error> {
        if wgpu::Backends::SECONDARY.contains(backend) {
            log::warn!(
                "{} graphics backend support may not be fully supported.",
                format_list(&get_backend_names(backend), "and")
            );
        }
        let instance = wgpu::Instance::new(backend);
        let surface = unsafe { instance.create_surface(window) };
        let descriptors = futures::executor::block_on(Self::build_descriptors(
            backend,
            instance,
            Some(&surface),
            power_preference,
            trace_path,
        ))?;
        let target = SwapChainTarget::new(
            surface,
            descriptors.surface_format,
            size,
            &descriptors.device,
        );
        Self::new(descriptors, target)
    }
}

#[cfg(not(target_family = "wasm"))]
impl WgpuRenderBackend<target::TextureTarget> {
    pub fn for_offscreen(
        size: (u32, u32),
        backend: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<Self, Error> {
        if wgpu::Backends::SECONDARY.contains(backend) {
            log::warn!(
                "{} graphics backend support may not be fully supported.",
                format_list(&get_backend_names(backend), "and")
            );
        }
        let instance = wgpu::Instance::new(backend);
        let descriptors = futures::executor::block_on(Self::build_descriptors(
            backend,
            instance,
            None,
            power_preference,
            trace_path,
        ))?;
        let target = target::TextureTarget::new(&descriptors.device, size);
        Self::new(descriptors, target)
    }
}

impl<T: RenderTarget> WgpuRenderBackend<T> {
    pub fn new(mut descriptors: Descriptors, target: T) -> Result<Self, Error> {
        let extent = wgpu::Extent3d {
            width: target.width(),
            height: target.height(),
            depth_or_array_layers: 1,
        };

        let frame_buffer_label = create_debug_label!("Framebuffer texture");
        let frame_buffer = descriptors.device.create_texture(&wgpu::TextureDescriptor {
            label: frame_buffer_label.as_deref(),
            size: extent,
            mip_level_count: 1,
            sample_count: descriptors.msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: target.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });
        let frame_buffer_view = frame_buffer.create_view(&Default::default());

        let depth_label = create_debug_label!("Depth texture");
        let depth_texture = descriptors.device.create_texture(&wgpu::TextureDescriptor {
            label: depth_label.as_deref(),
            size: extent,
            mip_level_count: 1,
            sample_count: descriptors.msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let depth_texture_view = depth_texture.create_view(&Default::default());

        let (quad_vbo, quad_ibo, quad_tex_transforms) = create_quad_buffers(&descriptors.device);

        descriptors
            .globals
            .set_resolution(target.width(), target.height());

        Ok(Self {
            descriptors,
            target,
            frame_buffer_view,
            depth_texture_view,
            current_frame: None,
            meshes: Vec::new(),
            shape_tessellator: ShapeTessellator::new(),
            textures: Vec::new(),

            num_masks: 0,
            mask_state: MaskState::NoMask,

            quad_vbo,
            quad_ibo,
            quad_tex_transforms,
            bitmap_registry: HashMap::new(),
        })
    }

    pub async fn build_descriptors(
        backend: wgpu::Backends,
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface>,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<Descriptors, Error> {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: surface,
            force_fallback_adapter: false,
        }).await
        .ok_or_else(|| {
            let names = get_backend_names(backend);
            if names.is_empty() {
                "Ruffle requires hardware acceleration, but no compatible graphics device was found (no backend provided?)".to_string()
            } else {
                format!("Ruffle requires hardware acceleration, but no compatible graphics device was found supporting {}", format_list(&names, "or"))
            }
        })?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    ..Default::default()
                },
                trace_path,
            )
            .await?;
        let info = adapter.get_info();
        // Prefer a linear surface format, when available.
        let surface_format = if info.backend == wgpu::Backend::Gl {
            // GL often only supports sRGB, so use the adapter's preferred format.
            surface
                .and_then(|surface| surface.get_preferred_format(&adapter))
                .unwrap_or(wgpu::TextureFormat::Bgra8Unorm)
        } else {
            wgpu::TextureFormat::Bgra8Unorm
        };

        Descriptors::new(device, queue, info, surface_format)
    }

    pub fn descriptors(self) -> Descriptors {
        self.descriptors
    }

    fn register_shape_internal(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> Mesh {
        let shape_id = shape.id; // TODO: remove?
        let lyon_mesh = self
            .shape_tessellator
            .tessellate_shape(shape, bitmap_source);

        let mut draws = Vec::with_capacity(lyon_mesh.len());
        for draw in lyon_mesh {
            let vertices: Vec<_> = draw.vertices.into_iter().map(Vertex::from).collect();
            let vertex_buffer = create_buffer_with_data(
                &self.descriptors.device,
                bytemuck::cast_slice(&vertices),
                wgpu::BufferUsages::VERTEX,
                create_debug_label!("Shape {} ({}) vbo", shape_id, draw.draw_type.name()),
            );

            let index_buffer = create_buffer_with_data(
                &self.descriptors.device,
                bytemuck::cast_slice(&draw.indices),
                wgpu::BufferUsages::INDEX,
                create_debug_label!("Shape {} ({}) ibo", shape_id, draw.draw_type.name()),
            );

            let index_count = draw.indices.len() as u32;
            let draw_id = draws.len();

            draws.push(match draw.draw_type {
                TessDrawType::Color => Draw {
                    draw_type: DrawType::Color,
                    vertex_buffer,
                    index_buffer,
                    index_count,
                },
                TessDrawType::Gradient(gradient) => {
                    // TODO: Extract to function?
                    let mut texture_transform = [[0.0; 4]; 4];
                    texture_transform[0][..3].copy_from_slice(&gradient.matrix[0]);
                    texture_transform[1][..3].copy_from_slice(&gradient.matrix[1]);
                    texture_transform[2][..3].copy_from_slice(&gradient.matrix[2]);

                    let tex_transforms_ubo = create_buffer_with_data(
                        &self.descriptors.device,
                        bytemuck::cast_slice(&[texture_transform]),
                        wgpu::BufferUsages::UNIFORM,
                        create_debug_label!(
                            "Shape {} draw {} textransforms ubo transfer buffer",
                            shape_id,
                            draw_id
                        ),
                    );

                    let gradient_ubo = create_buffer_with_data(
                        &self.descriptors.device,
                        bytemuck::cast_slice(&[GradientUniforms::from(gradient)]),
                        wgpu::BufferUsages::STORAGE,
                        create_debug_label!(
                            "Shape {} draw {} gradient ubo transfer buffer",
                            shape_id,
                            draw_id
                        ),
                    );

                    let bind_group_label = create_debug_label!(
                        "Shape {} (gradient) draw {} bindgroup",
                        shape_id,
                        draw_id
                    );
                    let bind_group =
                        self.descriptors
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &self.descriptors.pipelines.gradient_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::Buffer(
                                            wgpu::BufferBinding {
                                                buffer: &tex_transforms_ubo,
                                                offset: 0,
                                                size: wgpu::BufferSize::new(std::mem::size_of::<
                                                    TextureTransforms,
                                                >(
                                                )
                                                    as u64),
                                            },
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::Buffer(
                                            wgpu::BufferBinding {
                                                buffer: &gradient_ubo,
                                                offset: 0,
                                                size: wgpu::BufferSize::new(std::mem::size_of::<
                                                    GradientUniforms,
                                                >(
                                                )
                                                    as u64),
                                            },
                                        ),
                                    },
                                ],
                                label: bind_group_label.as_deref(),
                            });

                    Draw {
                        draw_type: DrawType::Gradient {
                            texture_transforms: tex_transforms_ubo,
                            gradient: gradient_ubo,
                            bind_group,
                        },
                        vertex_buffer,
                        index_buffer,
                        index_count,
                    }
                }
                TessDrawType::Bitmap(bitmap) => {
                    let texture = self.textures.get(bitmap.bitmap.0).unwrap();
                    let texture_view = texture.texture.create_view(&Default::default());

                    // TODO: Extract to function?
                    let mut texture_transform = [[0.0; 4]; 4];
                    texture_transform[0][..3].copy_from_slice(&bitmap.matrix[0]);
                    texture_transform[1][..3].copy_from_slice(&bitmap.matrix[1]);
                    texture_transform[2][..3].copy_from_slice(&bitmap.matrix[2]);

                    let tex_transforms_ubo = create_buffer_with_data(
                        &self.descriptors.device,
                        bytemuck::cast_slice(&[texture_transform]),
                        wgpu::BufferUsages::UNIFORM,
                        create_debug_label!(
                            "Shape {} draw {} textransforms ubo transfer buffer",
                            shape_id,
                            draw_id
                        ),
                    );

                    let bind_group_label = create_debug_label!(
                        "Shape {} (bitmap) draw {} bindgroup",
                        shape_id,
                        draw_id
                    );
                    let bind_group =
                        self.descriptors
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &self.descriptors.pipelines.bitmap_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::Buffer(
                                            wgpu::BufferBinding {
                                                buffer: &tex_transforms_ubo,
                                                offset: 0,
                                                size: wgpu::BufferSize::new(std::mem::size_of::<
                                                    TextureTransforms,
                                                >(
                                                )
                                                    as u64),
                                            },
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::TextureView(&texture_view),
                                    },
                                ],
                                label: bind_group_label.as_deref(),
                            });

                    Draw {
                        draw_type: DrawType::Bitmap {
                            texture_transforms: tex_transforms_ubo,
                            texture_view,
                            is_smoothed: bitmap.is_smoothed,
                            is_repeating: bitmap.is_repeating,
                            bind_group,
                        },
                        vertex_buffer,
                        index_buffer,
                        index_count,
                    }
                }
            });
        }

        Mesh { draws }
    }

    fn register_bitmap(&mut self, bitmap: Bitmap, debug_str: &str) -> BitmapInfo {
        let extent = wgpu::Extent3d {
            width: bitmap.width,
            height: bitmap.height,
            depth_or_array_layers: 1,
        };

        let data: Cow<[u8]> = match &bitmap.data {
            BitmapFormat::Rgba(data) => Cow::Borrowed(data),
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
                Cow::Owned(as_rgba)
            }
        };

        let texture_label = create_debug_label!("{} Texture", debug_str);
        let texture = self
            .descriptors
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: texture_label.as_deref(),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            });

        self.descriptors.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * extent.width),
                rows_per_image: None,
            },
            extent,
        );

        let handle = BitmapHandle(self.textures.len());
        let width = bitmap.width;
        let height = bitmap.height;

        // Make bind group for bitmap quad.
        let texture_view = texture.create_view(&Default::default());
        let bind_group = self
            .descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.descriptors.pipelines.bitmap_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.quad_tex_transforms,
                            offset: 0,
                            size: wgpu::BufferSize::new(
                                std::mem::size_of::<TextureTransforms>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                ],
                label: create_debug_label!("Bitmap {} bind group", handle.0).as_deref(),
            });

        self.bitmap_registry.insert(handle, bitmap);
        self.textures.push(Texture {
            width,
            height,
            texture,
            bind_group,
        });

        BitmapInfo {
            handle,
            width: width as u16,
            height: height as u16,
        }
    }

    pub fn target(&self) -> &T {
        &self.target
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.descriptors.device
    }
}

impl<T: RenderTarget + 'static> RenderBackend for WgpuRenderBackend<T> {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        // Avoid panics from creating 0-sized framebuffers.
        let width = std::cmp::max(width, 1);
        let height = std::cmp::max(height, 1);

        self.target.resize(&self.descriptors.device, width, height);

        let label = create_debug_label!("Framebuffer texture");
        let frame_buffer = self
            .descriptors
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: label.as_deref(),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.descriptors.msaa_sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: self.target.format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });
        self.frame_buffer_view = frame_buffer.create_view(&Default::default());

        let label = create_debug_label!("Depth texture");
        let depth_texture = self
            .descriptors
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: label.as_deref(),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.descriptors.msaa_sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });
        self.depth_texture_view = depth_texture.create_view(&Default::default());
        self.descriptors.globals.set_resolution(width, height);
    }

    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        let handle = ShapeHandle(self.meshes.len());
        let mesh = self.register_shape_internal(shape, bitmap_source);
        self.meshes.push(mesh);
        handle
    }

    fn replace_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
        handle: ShapeHandle,
    ) {
        let mesh = self.register_shape_internal(shape, bitmap_source);
        self.meshes[handle.0] = mesh;
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let shape = ruffle_core::shape_utils::swf_glyph_to_shape(glyph);
        let handle = ShapeHandle(self.meshes.len());
        let mesh = self.register_shape_internal(
            (&shape).into(),
            &ruffle_core::backend::render::NullBitmapSource,
        );
        self.meshes.push(mesh);
        handle
    }

    fn register_bitmap_jpeg(
        &mut self,
        data: &[u8],
        jpeg_tables: Option<&[u8]>,
    ) -> Result<BitmapInfo, Error> {
        let data = ruffle_core::backend::render::glue_tables_to_jpeg(data, jpeg_tables);
        self.register_bitmap_jpeg_2(&data[..])
    }

    fn register_bitmap_jpeg_2(&mut self, data: &[u8]) -> Result<BitmapInfo, Error> {
        let bitmap = ruffle_core::backend::render::decode_define_bits_jpeg(data, None)?;
        Ok(self.register_bitmap(bitmap, "JPEG2"))
    }

    fn register_bitmap_jpeg_3(
        &mut self,
        jpeg_data: &[u8],
        alpha_data: &[u8],
    ) -> Result<BitmapInfo, Error> {
        let bitmap =
            ruffle_core::backend::render::decode_define_bits_jpeg(jpeg_data, Some(alpha_data))?;
        Ok(self.register_bitmap(bitmap, "JPEG3"))
    }

    fn register_bitmap_png(
        &mut self,
        swf_tag: &swf::DefineBitsLossless,
    ) -> Result<BitmapInfo, Error> {
        let bitmap = ruffle_core::backend::render::decode_define_bits_lossless(swf_tag)?;
        Ok(self.register_bitmap(bitmap, "PNG"))
    }

    fn begin_frame(&mut self, clear: Color) {
        self.mask_state = MaskState::NoMask;
        self.num_masks = 0;
        self.descriptors.uniform_buffers.reset();

        let frame_output = match self.target.get_next_texture() {
            Ok(frame) => frame,
            Err(e) => {
                log::warn!("Couldn't begin new render frame: {}", e);
                // Attemp to recreate the swap chain in this case.
                self.target.resize(
                    &self.descriptors.device,
                    self.target.width(),
                    self.target.height(),
                );
                return;
            }
        };

        let label = create_debug_label!("Draw encoder");
        let draw_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: label.as_deref(),
                });
        let uniform_encoder_label = create_debug_label!("Uniform upload command encoder");
        let uniform_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: uniform_encoder_label.as_deref(),
                });
        let mut frame_data = Box::new((draw_encoder, frame_output, uniform_encoder));

        self.descriptors
            .globals
            .update_uniform(&self.descriptors.device, &mut frame_data.0);

        let (color_view, resolve_target) = if self.descriptors.msaa_sample_count >= 2 {
            (&self.frame_buffer_view, Some(frame_data.1.view()))
        } else {
            (frame_data.1.view(), None)
        };

        let render_pass = frame_data.0.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: color_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(clear.r) / 255.0,
                        g: f64::from(clear.g) / 255.0,
                        b: f64::from(clear.b) / 255.0,
                        a: f64::from(clear.a) / 255.0,
                    }),
                    store: true,
                },
                resolve_target,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
            label: None,
        });

        // Since RenderPass holds a reference to the CommandEncoder, we cast the lifetime
        // away to allow for the self-referencing struct. draw_encoder is boxed so its
        // address should remain stable.
        self.current_frame = Some(Frame {
            render_pass: unsafe {
                std::mem::transmute::<_, wgpu::RenderPass<'static>>(render_pass)
            },
            frame_data,
        });
    }

    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform, smoothing: bool) {
        if let Some(texture) = self.textures.get(bitmap.0) {
            let frame = if let Some(frame) = &mut self.current_frame {
                frame.get()
            } else {
                return;
            };

            let transform = Transform {
                matrix: transform.matrix
                    * ruffle_core::matrix::Matrix {
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

            frame.render_pass.set_pipeline(
                self.descriptors
                    .pipelines
                    .bitmap_pipelines
                    .pipeline_for(self.mask_state),
            );
            frame
                .render_pass
                .set_bind_group(0, self.descriptors.globals.bind_group(), &[]);

            self.descriptors.uniform_buffers.write_uniforms(
                &self.descriptors.device,
                &mut frame.frame_data.2,
                &mut frame.render_pass,
                1,
                &Transforms {
                    world_matrix,
                    color_adjustments: ColorAdjustments::from(transform.color_transform),
                },
            );

            frame
                .render_pass
                .set_bind_group(2, &texture.bind_group, &[]);
            frame.render_pass.set_bind_group(
                3,
                self.descriptors
                    .bitmap_samplers
                    .get_bind_group(false, smoothing),
                &[],
            );
            frame
                .render_pass
                .set_vertex_buffer(0, self.quad_vbo.slice(..));
            frame
                .render_pass
                .set_index_buffer(self.quad_ibo.slice(..), wgpu::IndexFormat::Uint32);

            match self.mask_state {
                MaskState::NoMask => (),
                MaskState::DrawMaskStencil => {
                    debug_assert!(self.num_masks > 0);
                    frame.render_pass.set_stencil_reference(self.num_masks - 1);
                }
                MaskState::DrawMaskedContent | MaskState::ClearMaskStencil => {
                    debug_assert!(self.num_masks > 0);
                    frame.render_pass.set_stencil_reference(self.num_masks);
                }
            };

            frame.render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        let frame = if let Some(frame) = &mut self.current_frame {
            frame.get()
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

        frame
            .render_pass
            .set_bind_group(0, self.descriptors.globals.bind_group(), &[]);

        self.descriptors.uniform_buffers.write_uniforms(
            &self.descriptors.device,
            &mut frame.frame_data.2,
            &mut frame.render_pass,
            1,
            &Transforms {
                world_matrix,
                color_adjustments: ColorAdjustments::from(transform.color_transform),
            },
        );

        for draw in &mesh.draws {
            match &draw.draw_type {
                DrawType::Color => {
                    frame.render_pass.set_pipeline(
                        self.descriptors
                            .pipelines
                            .color_pipelines
                            .pipeline_for(self.mask_state),
                    );
                }
                DrawType::Gradient { bind_group, .. } => {
                    frame.render_pass.set_pipeline(
                        self.descriptors
                            .pipelines
                            .gradient_pipelines
                            .pipeline_for(self.mask_state),
                    );
                    frame.render_pass.set_bind_group(2, bind_group, &[]);
                }
                DrawType::Bitmap {
                    is_repeating,
                    is_smoothed,
                    bind_group,
                    ..
                } => {
                    frame.render_pass.set_pipeline(
                        self.descriptors
                            .pipelines
                            .bitmap_pipelines
                            .pipeline_for(self.mask_state),
                    );
                    frame.render_pass.set_bind_group(2, bind_group, &[]);
                    frame.render_pass.set_bind_group(
                        3,
                        self.descriptors
                            .bitmap_samplers
                            .get_bind_group(*is_repeating, *is_smoothed),
                        &[],
                    );
                }
            }

            frame
                .render_pass
                .set_vertex_buffer(0, draw.vertex_buffer.slice(..));
            frame
                .render_pass
                .set_index_buffer(draw.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            match self.mask_state {
                MaskState::NoMask => (),
                MaskState::DrawMaskStencil => {
                    debug_assert!(self.num_masks > 0);
                    frame.render_pass.set_stencil_reference(self.num_masks - 1);
                }
                MaskState::DrawMaskedContent | MaskState::ClearMaskStencil => {
                    debug_assert!(self.num_masks > 0);
                    frame.render_pass.set_stencil_reference(self.num_masks);
                }
            };

            frame.render_pass.draw_indexed(0..draw.index_count, 0, 0..1);
        }
    }

    fn draw_rect(&mut self, color: Color, matrix: &ruffle_core::matrix::Matrix) {
        let frame = if let Some(frame) = &mut self.current_frame {
            frame.get()
        } else {
            return;
        };

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
            f32::from(color.r) / 255.0,
            f32::from(color.g) / 255.0,
            f32::from(color.b) / 255.0,
            f32::from(color.a) / 255.0,
        ];

        let add_color = [0.0, 0.0, 0.0, 0.0];
        frame.render_pass.set_pipeline(
            self.descriptors
                .pipelines
                .color_pipelines
                .pipeline_for(self.mask_state),
        );

        frame
            .render_pass
            .set_bind_group(0, self.descriptors.globals.bind_group(), &[]);

        self.descriptors.uniform_buffers.write_uniforms(
            &self.descriptors.device,
            &mut frame.frame_data.2,
            &mut frame.render_pass,
            1,
            &Transforms {
                world_matrix,
                color_adjustments: ColorAdjustments {
                    mult_color,
                    add_color,
                },
            },
        );

        frame
            .render_pass
            .set_vertex_buffer(0, self.quad_vbo.slice(..));
        frame
            .render_pass
            .set_index_buffer(self.quad_ibo.slice(..), wgpu::IndexFormat::Uint32);

        match self.mask_state {
            MaskState::NoMask => (),
            MaskState::DrawMaskStencil => {
                debug_assert!(self.num_masks > 0);
                frame.render_pass.set_stencil_reference(self.num_masks - 1);
            }
            MaskState::DrawMaskedContent | MaskState::ClearMaskStencil => {
                debug_assert!(self.num_masks > 0);
                frame.render_pass.set_stencil_reference(self.num_masks);
            }
        };

        frame.render_pass.draw_indexed(0..6, 0, 0..1);
    }

    fn end_frame(&mut self) {
        if let Some(frame) = self.current_frame.take() {
            // Finalize render pass.
            drop(frame.render_pass);
            self.descriptors.uniform_buffers.finish();
            let draw_encoder = frame.frame_data.0;
            let uniform_encoder = frame.frame_data.2;
            self.target.submit(
                &self.descriptors.device,
                &self.descriptors.queue,
                vec![uniform_encoder.finish(), draw_encoder.finish()],
                frame.frame_data.1,
            );
        }
    }

    fn push_mask(&mut self) {
        debug_assert!(
            self.mask_state == MaskState::NoMask || self.mask_state == MaskState::DrawMaskedContent
        );
        self.num_masks += 1;
        self.mask_state = MaskState::DrawMaskStencil;
    }

    fn activate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskStencil);
        self.mask_state = MaskState::DrawMaskedContent;
    }

    fn deactivate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskedContent);
        self.mask_state = MaskState::ClearMaskStencil;
    }

    fn pop_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::ClearMaskStencil);
        self.num_masks -= 1;
        self.mask_state = if self.num_masks == 0 {
            MaskState::NoMask
        } else {
            MaskState::DrawMaskedContent
        };
    }

    fn get_bitmap_pixels(&mut self, bitmap: BitmapHandle) -> Option<Bitmap> {
        self.bitmap_registry.get(&bitmap).cloned()
    }

    fn register_bitmap_raw(
        &mut self,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<BitmapHandle, Error> {
        Ok(self
            .register_bitmap(
                Bitmap {
                    height,
                    width,
                    data: BitmapFormat::Rgba(rgba),
                },
                "RAW",
            )
            .handle)
    }

    fn update_texture(
        &mut self,
        handle: BitmapHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<BitmapHandle, Error> {
        let texture = if let Some(texture) = self.textures.get(handle.0) {
            &texture.texture
        } else {
            return Err("update_texture: Bitmap not registered".into());
        };

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        self.descriptors.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * extent.width),
                rows_per_image: None,
            },
            extent,
        );

        Ok(handle)
    }
}

fn create_quad_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    let vertices = [
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
    ];
    let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

    let vbo = create_buffer_with_data(
        device,
        bytemuck::cast_slice(&vertices),
        wgpu::BufferUsages::VERTEX,
        create_debug_label!("Quad vbo"),
    );

    let ibo = create_buffer_with_data(
        device,
        bytemuck::cast_slice(&indices),
        wgpu::BufferUsages::INDEX,
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
        wgpu::BufferUsages::UNIFORM,
        create_debug_label!("Quad tex transforms"),
    );

    (vbo, ibo, tex_transforms)
}

#[derive(Debug)]
struct Texture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}
