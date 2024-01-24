use crate::backend::WgpuRenderBackend;
use crate::target::RenderTarget;
use crate::{
    as_texture, Descriptors, GradientUniforms, PosColorVertex, PosVertex, TextureTransforms,
};
use std::ops::Range;
use wgpu::util::DeviceExt;

use crate::buffer_builder::BufferBuilder;
use ruffle_render::backend::{RenderBackend, ShapeHandle, ShapeHandleImpl};
use ruffle_render::bitmap::BitmapSource;
use ruffle_render::tessellator::{Bitmap, Draw as LyonDraw, DrawType as TessDrawType, Gradient};
use swf::{CharacterId, GradientInterpolation};

/// How big to make gradient textures. Larger will keep more detail, but be slower and use more memory.
const GRADIENT_SIZE: usize = 256;

#[derive(Debug)]
pub struct Mesh {
    pub draws: Vec<Draw>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl ShapeHandleImpl for Mesh {}

pub fn as_mesh(handle: &ShapeHandle) -> &Mesh {
    <dyn ShapeHandleImpl>::downcast_ref(&*handle.0).expect("Shape handle must be a WGPU ShapeData")
}

#[derive(Debug)]
pub struct PendingDraw {
    pub draw_type: PendingDrawType,
    pub vertices: Range<wgpu::BufferAddress>,
    pub indices: Range<wgpu::BufferAddress>,
    pub num_indices: u32,
    pub num_mask_indices: u32,
}

impl PendingDraw {
    pub fn finish(
        self,
        descriptors: &Descriptors,
        uniform_buffer: &wgpu::Buffer,
        gradients: &[CommonGradient],
    ) -> Draw {
        Draw {
            draw_type: self
                .draw_type
                .finish(descriptors, uniform_buffer, gradients),
            vertices: self.vertices,
            indices: self.indices,
            num_indices: self.num_indices,
            num_mask_indices: self.num_mask_indices,
        }
    }
}

#[derive(Debug)]
pub struct Draw {
    pub draw_type: DrawType,
    pub vertices: Range<wgpu::BufferAddress>,
    pub indices: Range<wgpu::BufferAddress>,
    pub num_indices: u32,
    pub num_mask_indices: u32,
}

impl PendingDraw {
    #[allow(clippy::too_many_arguments)]
    pub fn new<T: RenderTarget>(
        backend: &mut WgpuRenderBackend<T>,
        source: &dyn BitmapSource,
        draw: LyonDraw,
        shape_id: CharacterId,
        draw_id: usize,
        uniform_buffer: &mut BufferBuilder,
        vertex_buffer: &mut BufferBuilder,
        index_buffer: &mut BufferBuilder,
    ) -> Option<Self> {
        let vertices = if matches!(draw.draw_type, TessDrawType::Color) {
            let vertices: Vec<_> = draw
                .vertices
                .into_iter()
                .map(PosColorVertex::from)
                .collect();
            vertex_buffer
                .add(&vertices)
                .expect("Mesh vertex buffer was too large!")
        } else {
            let vertices: Vec<_> = draw.vertices.into_iter().map(PosVertex::from).collect();
            vertex_buffer
                .add(&vertices)
                .expect("Mesh vertex buffer was too large!")
        };

        let indices = index_buffer
            .add(&draw.indices)
            .expect("Mesh index buffer was too large!");

        let index_count = draw.indices.len() as u32;
        let draw_type = match draw.draw_type {
            TessDrawType::Color => PendingDrawType::color(),
            TessDrawType::Gradient { matrix, gradient } => {
                PendingDrawType::gradient(gradient, matrix, shape_id, draw_id, uniform_buffer)
            }
            TessDrawType::Bitmap(bitmap) => {
                PendingDrawType::bitmap(bitmap, shape_id, draw_id, source, backend, uniform_buffer)?
            }
        };
        Some(PendingDraw {
            draw_type,
            vertices,
            indices,
            num_indices: index_count,
            num_mask_indices: draw.mask_index_count,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum PendingDrawType {
    Color,
    Gradient {
        texture_transforms_index: wgpu::BufferAddress,
        gradient_index: usize,
        bind_group_label: Option<String>,
    },
    Bitmap {
        texture_transforms_index: wgpu::BufferAddress,
        texture_view: wgpu::TextureView,
        is_repeating: bool,
        is_smoothed: bool,
        bind_group_label: Option<String>,
    },
}

/// Converts an RGBA color from sRGB space to linear color space.
fn srgb_to_linear(color: f32) -> f32 {
    if color <= 0.04045 {
        color / 12.92
    } else {
        f32::powf((color + 0.055) / 1.055, 2.4)
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl PendingDrawType {
    pub fn color() -> Self {
        PendingDrawType::Color
    }

    pub fn gradient(
        gradient_index: usize,
        matrix: [[f32; 3]; 3],
        shape_id: CharacterId,
        draw_id: usize,
        uniform_buffers: &mut BufferBuilder,
    ) -> Self {
        let tex_transforms_index = create_texture_transforms(&matrix, uniform_buffers);

        let bind_group_label =
            create_debug_label!("Shape {} (gradient) draw {} bindgroup", shape_id, draw_id);
        PendingDrawType::Gradient {
            texture_transforms_index: tex_transforms_index,
            gradient_index,
            bind_group_label,
        }
    }

    pub fn bitmap(
        bitmap: Bitmap,
        shape_id: CharacterId,
        draw_id: usize,
        source: &dyn BitmapSource,
        backend: &mut dyn RenderBackend,
        uniform_buffers: &mut BufferBuilder,
    ) -> Option<Self> {
        let handle = source.bitmap_handle(bitmap.bitmap_id, backend)?;
        let texture = as_texture(&handle);
        let texture_view = texture.texture.create_view(&Default::default());
        let texture_transforms_index = create_texture_transforms(&bitmap.matrix, uniform_buffers);
        let bind_group_label =
            create_debug_label!("Shape {} (bitmap) draw {} bindgroup", shape_id, draw_id);

        Some(PendingDrawType::Bitmap {
            texture_transforms_index,
            texture_view,
            is_repeating: bitmap.is_repeating,
            is_smoothed: bitmap.is_smoothed,
            bind_group_label,
        })
    }

    pub fn finish(
        self,
        descriptors: &Descriptors,
        uniform_buffer: &wgpu::Buffer,
        gradients: &[CommonGradient],
    ) -> DrawType {
        match self {
            PendingDrawType::Color => DrawType::Color,
            PendingDrawType::Gradient {
                texture_transforms_index,
                gradient_index,
                bind_group_label,
            } => {
                let common = &gradients[gradient_index];
                let bind_group = descriptors
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &descriptors.bind_layouts.gradient,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: uniform_buffer,
                                    offset: texture_transforms_index,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<TextureTransforms>() as u64,
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: uniform_buffer,
                                    offset: common.buffer_offset,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<GradientUniforms>() as u64,
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(&common.texture_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::Sampler(
                                    descriptors.bitmap_samplers.get_sampler(false, true),
                                ),
                            },
                        ],
                        label: bind_group_label.as_deref(),
                    });
                DrawType::Gradient { bind_group }
            }
            PendingDrawType::Bitmap {
                texture_transforms_index,
                texture_view,
                is_repeating,
                is_smoothed,
                bind_group_label,
            } => {
                let binds = BitmapBinds::new(
                    &descriptors.device,
                    &descriptors.bind_layouts.bitmap,
                    descriptors
                        .bitmap_samplers
                        .get_sampler(is_repeating, is_smoothed),
                    uniform_buffer,
                    texture_transforms_index,
                    texture_view,
                    bind_group_label,
                );

                DrawType::Bitmap { binds }
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum DrawType {
    Color,
    Gradient { bind_group: wgpu::BindGroup },
    Bitmap { binds: BitmapBinds },
}

#[derive(Debug)]
pub struct CommonGradient {
    texture_view: wgpu::TextureView,
    buffer_offset: wgpu::BufferAddress,
}

impl CommonGradient {
    pub fn new(
        descriptors: &Descriptors,
        gradient: Gradient,
        uniform_buffers: &mut BufferBuilder,
    ) -> Self {
        let colors = if gradient.records.is_empty() {
            [0; GRADIENT_SIZE * 4]
        } else {
            let mut colors = [0; GRADIENT_SIZE * 4];

            let convert = if gradient.interpolation == GradientInterpolation::LinearRgb {
                |c| srgb_to_linear(c / 255.0) * 255.0
            } else {
                |c| c
            };

            for t in 0..GRADIENT_SIZE {
                let mut last = 0;
                let mut next = 0;

                for (i, record) in gradient.records.iter().enumerate().rev() {
                    if (record.ratio as usize) < t {
                        last = i;
                        next = (i + 1).min(gradient.records.len() - 1);
                        break;
                    }
                }
                assert!(last == next || last + 1 == next);

                let last_record = &gradient.records[last];
                let next_record = &gradient.records[next];

                let a = if next == last {
                    // this can happen if we are before the first gradient record, or after the last one
                    0.0
                } else {
                    (t as f32 - last_record.ratio as f32)
                        / (next_record.ratio as f32 - last_record.ratio as f32)
                };
                colors[t * 4] = lerp(
                    convert(last_record.color.r as f32),
                    convert(next_record.color.r as f32),
                    a,
                ) as u8;
                colors[(t * 4) + 1] = lerp(
                    convert(last_record.color.g as f32),
                    convert(next_record.color.g as f32),
                    a,
                ) as u8;
                colors[(t * 4) + 2] = lerp(
                    convert(last_record.color.b as f32),
                    convert(next_record.color.b as f32),
                    a,
                ) as u8;
                colors[(t * 4) + 3] =
                    lerp(last_record.color.a as f32, next_record.color.a as f32, a) as u8;
            }

            colors
        };
        let texture = descriptors.device.create_texture_with_data(
            &descriptors.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: GRADIENT_SIZE as u32,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &colors[..],
        );
        let view = texture.create_view(&Default::default());

        let buffer_offset = uniform_buffers
            .add(&[GradientUniforms::from(gradient)])
            .expect("Mesh uniform buffer was too large!")
            .start;

        Self {
            texture_view: view,
            buffer_offset,
        }
    }
}

#[derive(Debug)]
pub struct BitmapBinds {
    pub bind_group: wgpu::BindGroup,
}

impl BitmapBinds {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        uniform_buffer: &wgpu::Buffer,
        texture_transforms: wgpu::BufferAddress,
        texture_view: wgpu::TextureView,
        label: Option<String>,
    ) -> Self {
        let bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: uniform_buffer,
                            offset: texture_transforms,
                            size: wgpu::BufferSize::new(
                                std::mem::size_of::<TextureTransforms>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
                label: label.as_deref(),
            });
        Self { bind_group }
    }
}

fn create_texture_transforms(
    matrix: &[[f32; 3]; 3],
    buffer: &mut BufferBuilder,
) -> wgpu::BufferAddress {
    let mut texture_transform = [[0.0; 4]; 4];
    texture_transform[0][..3].copy_from_slice(&matrix[0]);
    texture_transform[1][..3].copy_from_slice(&matrix[1]);
    texture_transform[2][..3].copy_from_slice(&matrix[2]);
    buffer
        .add(&[texture_transform])
        .expect("Mesh uniform buffer was too large!")
        .start
}
