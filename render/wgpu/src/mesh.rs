use crate::backend::WgpuRenderBackend;
use crate::target::RenderTarget;
use crate::{
    as_texture, create_buffer_with_data, Descriptors, GradientStorage, GradientUniforms,
    PosColorVertex, PosVertex, TextureTransforms,
};

use crate::buffer_builder::BufferBuilder;
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::BitmapSource;
use ruffle_render::tessellator::{
    Bitmap, Draw as LyonDraw, DrawType as TessDrawType, Gradient, GradientType,
};
use swf::{CharacterId, GradientSpread};

#[derive(Debug)]
pub struct Mesh {
    pub draws: Vec<Draw>,
}

#[derive(Debug)]
pub struct PendingDraw {
    pub draw_type: PendingDrawType,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub num_mask_indices: u32,
}

impl PendingDraw {
    pub fn finish(self, descriptors: &Descriptors, uniform_buffer: &wgpu::Buffer) -> Draw {
        Draw {
            draw_type: self.draw_type.finish(descriptors, uniform_buffer),
            vertex_buffer: self.vertex_buffer,
            index_buffer: self.index_buffer,
            num_indices: self.num_indices,
            num_mask_indices: self.num_mask_indices,
        }
    }
}

#[derive(Debug)]
pub struct Draw {
    pub draw_type: DrawType,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
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
        uniform_buffer: &mut BufferBuilder,
    ) -> Option<Self> {
        let descriptors = backend.descriptors().clone();

        let vertex_buffer = if matches!(draw.draw_type, TessDrawType::Color) {
            let vertices: Vec<_> = draw
                .vertices
                .into_iter()
                .map(PosColorVertex::from)
                .collect();
            create_buffer_with_data(
                &descriptors.device,
                bytemuck::cast_slice(&vertices),
                wgpu::BufferUsages::VERTEX,
                create_debug_label!("Shape {} ({}) vbo", shape_id, draw.draw_type.name()),
            )
        } else {
            let vertices: Vec<_> = draw.vertices.into_iter().map(PosVertex::from).collect();
            create_buffer_with_data(
                &descriptors.device,
                bytemuck::cast_slice(&vertices),
                wgpu::BufferUsages::VERTEX,
                create_debug_label!("Shape {} ({}) vbo", shape_id, draw.draw_type.name()),
            )
        };

        let index_buffer = create_buffer_with_data(
            &descriptors.device,
            bytemuck::cast_slice(&draw.indices),
            wgpu::BufferUsages::INDEX,
            create_debug_label!("Shape {} ({}) ibo", shape_id, draw.draw_type.name()),
        );

        let index_count = draw.indices.len() as u32;
        let draw_type = match draw.draw_type {
            TessDrawType::Color => PendingDrawType::color(),
            TessDrawType::Gradient(gradient) => {
                PendingDrawType::gradient(&descriptors, gradient, shape_id, draw_id, uniform_buffer)
            }
            TessDrawType::Bitmap(bitmap) => {
                PendingDrawType::bitmap(bitmap, shape_id, draw_id, source, backend, uniform_buffer)?
            }
        };
        Some(PendingDraw {
            draw_type,
            vertex_buffer,
            index_buffer,
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
        gradient: wgpu::Buffer,
        spread: GradientSpread,
        mode: GradientType,
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

        let spread = gradient.repeat_mode;
        let mode = gradient.gradient_type;

        let gradient_ubo = if descriptors.limits.max_storage_buffers_per_shader_stage > 0 {
            create_buffer_with_data(
                &descriptors.device,
                bytemuck::cast_slice(&[GradientStorage::from(gradient)]),
                wgpu::BufferUsages::STORAGE,
                create_debug_label!(
                    "Shape {} draw {} gradient ubo transfer buffer",
                    shape_id,
                    draw_id
                ),
            )
        } else {
            create_buffer_with_data(
                &descriptors.device,
                bytemuck::cast_slice(&[GradientUniforms::from(gradient)]),
                wgpu::BufferUsages::UNIFORM,
                create_debug_label!(
                    "Shape {} draw {} gradient ubo transfer buffer",
                    shape_id,
                    draw_id
                ),
            )
        };

        let bind_group_label =
            create_debug_label!("Shape {} (gradient) draw {} bindgroup", shape_id, draw_id);
        PendingDrawType::Gradient {
            texture_transforms_index: tex_transforms_index,
            gradient: gradient_ubo,
            spread,
            mode,
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

    pub fn finish(self, descriptors: &Descriptors, uniform_buffer: &wgpu::Buffer) -> DrawType {
        match self {
            PendingDrawType::Color => DrawType::Color,
            PendingDrawType::Gradient {
                texture_transforms_index,
                gradient,
                spread,
                mode,
                bind_group_label,
            } => {
                let bind_group = descriptors
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &descriptors.bind_layouts.gradient,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &uniform_buffer,
                                    offset: texture_transforms_index,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<TextureTransforms>() as u64,
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: gradient.as_entire_binding(),
                            },
                        ],
                        label: bind_group_label.as_deref(),
                    });
                DrawType::Gradient {
                    gradient,
                    bind_group,
                    spread,
                    mode,
                }
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
                    &uniform_buffer,
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
    Gradient {
        gradient: wgpu::Buffer,
        bind_group: wgpu::BindGroup,
        spread: GradientSpread,
        mode: GradientType,
    },
    Bitmap {
        binds: BitmapBinds,
    },
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
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &uniform_buffer,
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
                        resource: wgpu::BindingResource::Sampler(&sampler),
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
    buffer.add(texture_transform)
}
