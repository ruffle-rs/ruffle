use crate::backend::WgpuRenderBackend;
use crate::target::RenderTarget;
use crate::{Descriptors, GradientUniforms, PosColorVertex, PosUvVertex, as_texture};
use std::any::Any;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use wgpu::util::DeviceExt;

use crate::buffer_builder::BufferBuilder;
use ruffle_render::backend::{ShapeHandle, ShapeHandleImpl};
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

/// Bytes of vertex and index buffers held by live meshes, and how many
/// meshes there are. Kept for memory diagnostics: not every backend reports
/// buffer memory, and tessellated shapes are the largest thing a resident
/// movie owns.
static MESH_BYTES: AtomicUsize = AtomicUsize::new(0);
static MESH_COUNT: AtomicUsize = AtomicUsize::new(0);

impl Mesh {
    pub fn new(draws: Vec<Draw>, vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer) -> Self {
        MESH_BYTES.fetch_add(
            (vertex_buffer.size() + index_buffer.size()) as usize,
            Ordering::Relaxed,
        );
        MESH_COUNT.fetch_add(1, Ordering::Relaxed);
        Self {
            draws,
            vertex_buffer,
            index_buffer,
        }
    }

    /// `(meshes alive, bytes of their vertex and index buffers)`.
    pub fn live_totals() -> (usize, usize) {
        (
            MESH_COUNT.load(Ordering::Relaxed),
            MESH_BYTES.load(Ordering::Relaxed),
        )
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        MESH_BYTES.fetch_sub(
            (self.vertex_buffer.size() + self.index_buffer.size()) as usize,
            Ordering::Relaxed,
        );
        MESH_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

impl ShapeHandleImpl for Mesh {}

pub fn as_mesh(handle: &ShapeHandle) -> &Mesh {
    <dyn Any>::downcast_ref(&*handle.0).expect("Shape handle must be a WGPU ShapeData")
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
    pub fn new<T: RenderTarget>(
        backend: &mut WgpuRenderBackend<T>,
        source: &dyn BitmapSource,
        draw: LyonDraw,
        shape_id: CharacterId,
        draw_id: usize,
        vertex_buffer: &mut BufferBuilder,
        index_buffer: &mut BufferBuilder,
    ) -> Option<Self> {
        let vertices = match &draw.draw_type {
            TessDrawType::Color => {
                let vertices: Vec<_> = draw
                    .vertices
                    .into_iter()
                    .map(PosColorVertex::from)
                    .collect();
                vertex_buffer
                    .add(&vertices)
                    .expect("Mesh vertex buffer was too large!")
            }
            TessDrawType::Gradient { matrix, .. } => {
                let vertices: Vec<_> = draw
                    .vertices
                    .into_iter()
                    .map(|v| PosUvVertex::from_tessellator(v, matrix))
                    .collect();
                vertex_buffer
                    .add(&vertices)
                    .expect("Mesh vertex buffer was too large!")
            }
            TessDrawType::Bitmap(bitmap) => {
                let vertices: Vec<_> = draw
                    .vertices
                    .into_iter()
                    .map(|v| PosUvVertex::from_tessellator(v, &bitmap.matrix))
                    .collect();
                vertex_buffer
                    .add(&vertices)
                    .expect("Mesh vertex buffer was too large!")
            }
        };

        let indices = index_buffer
            .add(&draw.indices)
            .expect("Mesh index buffer was too large!");

        let index_count = draw.indices.len() as u32;
        let draw_type = match draw.draw_type {
            TessDrawType::Color => PendingDrawType::color(),
            TessDrawType::Gradient {
                matrix: _,
                gradient,
            } => PendingDrawType::gradient(gradient, shape_id, draw_id),
            TessDrawType::Bitmap(bitmap) => PendingDrawType::bitmap(bitmap, source, backend)?,
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

#[derive(Debug)]
pub enum PendingDrawType {
    Color,
    Gradient {
        gradient_index: usize,
        bind_group_label: Option<String>,
    },
    Bitmap {
        binds: BitmapBinds,
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

    pub fn gradient(gradient_index: usize, shape_id: CharacterId, draw_id: usize) -> Self {
        let bind_group_label =
            create_debug_label!("Shape {} (gradient) draw {} bindgroup", shape_id, draw_id);
        PendingDrawType::Gradient {
            gradient_index,
            bind_group_label,
        }
    }

    pub fn bitmap<T: RenderTarget>(
        bitmap: Bitmap,
        source: &dyn BitmapSource,
        backend: &mut WgpuRenderBackend<T>,
    ) -> Option<Self> {
        let handle = source.bitmap_handle(bitmap.bitmap_id, backend)?;
        let texture = as_texture(&handle);
        let binds = texture.bind_group(
            bitmap.is_repeating,
            bitmap.is_smoothed,
            &backend.descriptors.device,
            &backend.descriptors.bind_layouts.bitmap,
            handle.clone(),
            &backend.descriptors.bitmap_samplers,
        );

        Some(PendingDrawType::Bitmap {
            binds: binds.clone(),
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
                                    offset: common.buffer_offset,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<GradientUniforms>() as u64,
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&common.texture_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Sampler(
                                    descriptors.bitmap_samplers.get_sampler(false, true),
                                ),
                            },
                        ],
                        label: bind_group_label.as_deref(),
                    });
                DrawType::Gradient { bind_group }
            }
            PendingDrawType::Bitmap { binds } => DrawType::Bitmap { binds },
        }
    }
}

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
            let mut last = 0;
            let mut next;

            let convert = if gradient.interpolation == GradientInterpolation::LinearRgb {
                |c| srgb_to_linear(c / 255.0) * 255.0
            } else {
                |c| c
            };

            for t in 0..GRADIENT_SIZE {
                if last + 1 < gradient.records.len()
                    && t > gradient.records[last + 1].ratio as usize
                {
                    last += 1;
                }
                next = (last + 1).min(gradient.records.len() - 1);

                assert!(last == next || last + 1 == next);

                let last_record = &gradient.records[last];
                let next_record = &gradient.records[next];

                let a = if t <= last_record.ratio as usize || last_record.ratio == next_record.ratio
                {
                    // We are before the first gradient record,
                    // or this record's ratio is equal to the next one,
                    // meaning we need to do a full stop of this color for 1 pixel.
                    0.0
                } else if t > next_record.ratio as usize {
                    // We are after the last record
                    1.0
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

#[derive(Debug, Clone)]
pub struct BitmapBinds {
    pub bind_group: wgpu::BindGroup,
}

impl BitmapBinds {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        texture_view: wgpu::TextureView,
        label: Option<String>,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: label.as_deref(),
        });
        Self { bind_group }
    }
}
