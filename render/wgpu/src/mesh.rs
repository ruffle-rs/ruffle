use crate::backend::WgpuRenderBackend;
use crate::buffer_builder::BufferBuilder;
use crate::target::RenderTarget;
use crate::{
    as_texture, Descriptors, GradientUniforms, PosColorVertex, PosVertex, TextureTransforms,
};
use ruffle_render::backend::{RenderBackend, ShapeHandle, ShapeHandleImpl};
use ruffle_render::bitmap::BitmapSource;
use ruffle_render::tessellator::{Bitmap, Draw as LyonDraw, DrawType as TessDrawType, Gradient};
use std::convert::identity;
use std::ops::Range;
use swf::{CharacterId, GradientInterpolation};
use wgpu::util::DeviceExt;

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
    pub fn finish(self, descriptors: &Descriptors, uniform_buffer: &wgpu::Buffer) -> Draw {
        Draw {
            draw_type: self.draw_type.finish(descriptors, uniform_buffer),
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
            vertex_buffer.add(&vertices)
        } else {
            let vertices: Vec<_> = draw.vertices.into_iter().map(PosVertex::from).collect();
            vertex_buffer.add(&vertices)
        };

        let indices = index_buffer.add(&draw.indices);

        let index_count = draw.indices.len() as u32;
        let draw_type = match draw.draw_type {
            TessDrawType::Color => PendingDrawType::color(),
            TessDrawType::Gradient(gradient) => PendingDrawType::gradient(
                backend.descriptors(),
                gradient,
                shape_id,
                draw_id,
                uniform_buffer,
            ),
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
        gradient: wgpu::BufferAddress,
        bind_group_label: Option<String>,
        colors: wgpu::TextureView,
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

impl PendingDrawType {
    pub fn color() -> Self {
        PendingDrawType::Color
    }

    pub fn gradient(
        descriptors: &Descriptors,
        gradient: Gradient,
        shape_id: CharacterId,
        draw_id: usize,
        uniform_buffers: &mut BufferBuilder,
    ) -> Self {
        let tex_transforms_index = create_texture_transforms(&gradient.matrix, uniform_buffers);
        let colors = if gradient.records.is_empty() {
            [0; GRADIENT_SIZE * 4 * 2]
        } else {
            let mut colors = [0; GRADIENT_SIZE * 4 * 2];

            let convert = match gradient.interpolation {
                GradientInterpolation::Rgb => identity,
                GradientInterpolation::LinearRgb => |c| srgb_to_linear(c / 255.0) * 255.0,
            };

            // Store gradient colors in the upper half of the texture (y = 1).
            for (i, record) in gradient.records.iter().enumerate() {
                colors[GRADIENT_SIZE * 4 + i * 4] = convert(record.color.r as f32) as u8;
                colors[GRADIENT_SIZE * 4 + i * 4 + 1] = convert(record.color.g as f32) as u8;
                colors[GRADIENT_SIZE * 4 + i * 4 + 2] = convert(record.color.b as f32) as u8;
                colors[GRADIENT_SIZE * 4 + i * 4 + 3] = record.color.a;
            }

            // Duplicate last record.
            let record = gradient.records.last().unwrap();
            colors[GRADIENT_SIZE * 4 + gradient.records.len() * 4] =
                convert(record.color.r as f32) as u8;
            colors[GRADIENT_SIZE * 4 + gradient.records.len() * 4 + 1] =
                convert(record.color.g as f32) as u8;
            colors[GRADIENT_SIZE * 4 + gradient.records.len() * 4 + 2] =
                convert(record.color.b as f32) as u8;
            colors[GRADIENT_SIZE * 4 + gradient.records.len() * 4 + 3] = record.color.a;

            // Store `t` values in the lower half of the texture (y = 0).
            let mut records = gradient.records.iter().enumerate().peekable();
            let mut last = None;
            for t in 0..GRADIENT_SIZE {
                // Find the two gradient records bordering our position.
                while let Some(next) = records.next_if(|(_, r)| t as u8 > r.ratio) {
                    last = Some(next);
                }
                let next = records.peek();

                colors[t * 4] = match (last, next) {
                    (None, _) => {
                        // We are before the first gradient record.
                        0
                    }
                    (Some((i, _)), None) => {
                        // We are after the last gradient record.
                        (u16::from(u8::MAX) * i as u16 / (gradient.records.len() - 1) as u16) as u8
                    }
                    (Some((i, last)), Some((_, next))) => {
                        let a = u16::from(u8::MAX) * u16::from(t as u8 - last.ratio)
                            / u16::from(next.ratio - last.ratio);
                        ((u16::from(u8::MAX) * i as u16 + a) / (gradient.records.len() - 1) as u16)
                            as u8
                    }
                };
            }

            colors
        };
        let texture = descriptors.device.create_texture_with_data(
            &descriptors.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: GRADIENT_SIZE as u32,
                    height: 2,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            &colors[..],
        );
        let view = texture.create_view(&Default::default());

        let gradient = uniform_buffers
            .add(&[GradientUniforms::from(gradient)])
            .start;

        let bind_group_label =
            create_debug_label!("Shape {} (gradient) draw {} bindgroup", shape_id, draw_id);
        PendingDrawType::Gradient {
            texture_transforms_index: tex_transforms_index,
            gradient,
            bind_group_label,
            colors: view,
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

    pub fn finish(self, descriptors: &Descriptors, uniform_buffer: &wgpu::Buffer) -> DrawType {
        match self {
            PendingDrawType::Color => DrawType::Color,
            PendingDrawType::Gradient {
                texture_transforms_index,
                gradient,
                bind_group_label,
                colors,
            } => {
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
                                    offset: gradient,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<GradientUniforms>() as u64,
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(&colors),
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
    buffer.add(&[texture_transform]).start
}
