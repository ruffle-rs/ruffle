use crate::backend::WgpuRenderBackend;
use crate::target::RenderTarget;
use crate::{
    create_buffer_with_data, Descriptors, GradientStorage, GradientUniforms, Texture,
    TextureTransforms, Vertex,
};

use ruffle_render::bitmap::BitmapSource;
use ruffle_render::tessellator::{Bitmap, Draw as LyonDraw, DrawType as TessDrawType, Gradient};
use swf::CharacterId;

#[derive(Debug)]
pub struct Mesh {
    pub draws: Vec<Draw>,
}

#[derive(Debug)]
pub struct Draw {
    pub draw_type: DrawType,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub num_mask_indices: u32,
}

impl Draw {
    pub fn new<T: RenderTarget>(
        backend: &mut WgpuRenderBackend<T>,
        source: &dyn BitmapSource,
        draw: LyonDraw,
        shape_id: CharacterId,
        draw_id: usize,
        bitmap_registry: &FnvHashMap<BitmapHandle, Texture>,
    ) -> Self {
        let vertices: Vec<_> = draw.vertices.into_iter().map(Vertex::from).collect();
        let descriptors = backend.descriptors();
        let vertex_buffer = create_buffer_with_data(
            &descriptors.device,
            bytemuck::cast_slice(&vertices),
            wgpu::BufferUsages::VERTEX,
            create_debug_label!("Shape {} ({}) vbo", shape_id, draw.draw_type.name()),
        );

        let index_buffer = create_buffer_with_data(
            &descriptors.device,
            bytemuck::cast_slice(&draw.indices),
            wgpu::BufferUsages::INDEX,
            create_debug_label!("Shape {} ({}) ibo", shape_id, draw.draw_type.name()),
        );

        let index_count = draw.indices.len() as u32;
        match draw.draw_type {
            TessDrawType::Color => Draw {
                draw_type: DrawType::color(),
                vertex_buffer,
                index_buffer,
                num_indices: index_count,
                num_mask_indices: draw.mask_index_count,
            },
            TessDrawType::Gradient(gradient) => Draw {
                draw_type: DrawType::gradient(&descriptors, gradient, shape_id, draw_id),
                vertex_buffer,
                index_buffer,
                num_indices: index_count,
                num_mask_indices: draw.mask_index_count,
            },
            TessDrawType::Bitmap(bitmap) => Draw {
                draw_type: DrawType::bitmap(backend, source, bitmap, shape_id, draw_id),
                vertex_buffer,
                index_buffer,
                num_indices: index_count,
                num_mask_indices: draw.mask_index_count,
            },
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum DrawType {
    Color,
    Gradient {
        texture_transforms: wgpu::Buffer,
        gradient: wgpu::Buffer,
        bind_group: wgpu::BindGroup,
    },
    Bitmap {
        texture_transforms: wgpu::Buffer,
        binds: BitmapBinds,
    },
}

impl DrawType {
    pub fn color() -> Self {
        DrawType::Color
    }

    pub fn gradient(
        descriptors: &Descriptors,
        gradient: Gradient,
        shape_id: CharacterId,
        draw_id: usize,
    ) -> Self {
        let tex_transforms_ubo = create_texture_transforms(
            &descriptors.device,
            &gradient.matrix,
            create_debug_label!(
                "Shape {} draw {} textransforms ubo transfer buffer",
                shape_id,
                draw_id
            ),
        );

        let (gradient_ubo, buffer_size) =
            if descriptors.limits.max_storage_buffers_per_shader_stage > 0 {
                (
                    create_buffer_with_data(
                        &descriptors.device,
                        bytemuck::cast_slice(&[GradientStorage::from(gradient)]),
                        wgpu::BufferUsages::STORAGE,
                        create_debug_label!(
                            "Shape {} draw {} gradient ubo transfer buffer",
                            shape_id,
                            draw_id
                        ),
                    ),
                    wgpu::BufferSize::new(std::mem::size_of::<GradientStorage>() as u64),
                )
            } else {
                (
                    create_buffer_with_data(
                        &descriptors.device,
                        bytemuck::cast_slice(&[GradientUniforms::from(gradient)]),
                        wgpu::BufferUsages::UNIFORM,
                        create_debug_label!(
                            "Shape {} draw {} gradient ubo transfer buffer",
                            shape_id,
                            draw_id
                        ),
                    ),
                    wgpu::BufferSize::new(std::mem::size_of::<GradientUniforms>() as u64),
                )
            };

        let bind_group_label =
            create_debug_label!("Shape {} (gradient) draw {} bindgroup", shape_id, draw_id);
        let bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &descriptors.bind_layouts.gradient,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &tex_transforms_ubo,
                            offset: 0,
                            size: wgpu::BufferSize::new(
                                std::mem::size_of::<TextureTransforms>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &gradient_ubo,
                            offset: 0,
                            size: buffer_size,
                        }),
                    },
                ],
                label: bind_group_label.as_deref(),
            });
        DrawType::Gradient {
            texture_transforms: tex_transforms_ubo,
            gradient: gradient_ubo,
            bind_group,
        }
    }

    pub fn bitmap(
        descriptors: &Descriptors,
        bitmap_registry: &FnvHashMap<BitmapHandle, Texture>,
        bitmap: Bitmap,
        shape_id: CharacterId,
        draw_id: usize,
    ) -> Self {
        let entry = bitmap_registry.get(&bitmap.bitmap).unwrap();
        let texture_view = entry.texture.create_view(&Default::default());

        let texture_transforms = create_texture_transforms(
            &backend.descriptors().device,
            &bitmap.matrix,
            create_debug_label!(
                "Shape {} draw {} textransforms ubo transfer buffer",
                shape_id,
                draw_id
            ),
        );

        let bind_group_label =
            create_debug_label!("Shape {} (bitmap) draw {} bindgroup", shape_id, draw_id);
        let binds = BitmapBinds::new(
            &descriptors.device,
            &descriptors.bind_layouts.bitmap,
            descriptors
                .bitmap_samplers
                .get_sampler(bitmap.is_repeating, bitmap.is_smoothed),
            &texture_transforms,
            texture_view,
            bind_group_label,
        );

        DrawType::Bitmap {
            texture_transforms,
            binds,
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
        texture_transforms: &wgpu::Buffer,
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
                            buffer: &texture_transforms,
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
    device: &wgpu::Device,
    matrix: &[[f32; 3]; 3],
    label: Option<String>,
) -> wgpu::Buffer {
    let mut texture_transform = [[0.0; 4]; 4];
    texture_transform[0][..3].copy_from_slice(&matrix[0]);
    texture_transform[1][..3].copy_from_slice(&matrix[1]);
    texture_transform[2][..3].copy_from_slice(&matrix[2]);

    create_buffer_with_data(
        &device,
        bytemuck::cast_slice(&[texture_transform]),
        wgpu::BufferUsages::UNIFORM,
        label,
    )
}
