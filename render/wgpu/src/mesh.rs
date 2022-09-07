use crate::{
    create_buffer_with_data, Descriptors, GradientStorage, GradientUniforms, RegistryData,
    TextureTransforms,
};
use fnv::FnvHashMap;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::tessellator::{Bitmap, Gradient};
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
        texture_view: wgpu::TextureView,
        is_smoothed: bool,
        is_repeating: bool,
        bind_group: wgpu::BindGroup,
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
        bitmap_registry: &FnvHashMap<BitmapHandle, RegistryData>,
        bitmap: Bitmap,
        shape_id: CharacterId,
        draw_id: usize,
    ) -> Self {
        let entry = bitmap_registry.get(&bitmap.bitmap).unwrap();
        let texture_view = entry
            .texture_wrapper
            .texture
            .create_view(&Default::default());
        let tex_transforms_ubo = create_texture_transforms(
            &descriptors.device,
            &bitmap.matrix,
            create_debug_label!(
                "Shape {} draw {} textransforms ubo transfer buffer",
                shape_id,
                draw_id
            ),
        );

        let bind_group_label =
            create_debug_label!("Shape {} (bitmap) draw {} bindgroup", shape_id, draw_id);
        let bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &descriptors.bind_layouts.bitmap,
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
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                ],
                label: bind_group_label.as_deref(),
            });
        DrawType::Bitmap {
            texture_transforms: tex_transforms_ubo,
            texture_view,
            is_smoothed: bitmap.is_smoothed,
            is_repeating: bitmap.is_repeating,
            bind_group,
        }
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
