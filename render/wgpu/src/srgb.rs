use crate::layouts::BindLayouts;
use crate::{
    create_buffer_with_data, ColorAdjustments, Descriptors, Globals, Quad, TextureTransforms,
    Transforms,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Srgb {
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    copy_pipeline: Arc<wgpu::RenderPipeline>,
    _transforms_buffer: wgpu::Buffer,
    transforms_bind_group: wgpu::BindGroup,
}

impl Srgb {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        layouts: &BindLayouts,
        sampler: &wgpu::Sampler,
        copy_pipeline: Arc<wgpu::RenderPipeline>,
        quad: &Quad,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: create_debug_label!("Copy sRGB framebuffer texture").as_deref(),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let view = texture.create_view(&Default::default());
        let bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layouts.bitmap,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &quad.texture_transforms,
                            offset: 0,
                            size: wgpu::BufferSize::new(
                                std::mem::size_of::<TextureTransforms>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: create_debug_label!("Copy sRGB bind group").as_deref(),
            });
        let transform = Transforms {
            world_matrix: [
                [width as f32, 0.0, 0.0, 0.0],
                [0.0, height as f32, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            color_adjustments: ColorAdjustments {
                mult_color: [1.0, 1.0, 1.0, 1.0],
                add_color: [0.0, 0.0, 0.0, 0.0],
            },
        };
        let transforms_buffer = create_buffer_with_data(
            &device,
            bytemuck::cast_slice(&[transform]),
            wgpu::BufferUsages::UNIFORM,
            create_debug_label!("Copy sRGB transforms buffer"),
        );
        let transforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layouts.transforms,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &transforms_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<Transforms>() as u64),
                }),
            }],
            label: create_debug_label!("Copy sRGB transforms bind group").as_deref(),
        });

        Self {
            view,
            bind_group,
            copy_pipeline,
            _transforms_buffer: transforms_buffer,
            transforms_bind_group,
        }
    }

    pub fn copy_srgb(
        &self,
        view: &wgpu::TextureView,
        descriptors: &Descriptors,
        globals: &Globals,
    ) -> wgpu::CommandBuffer {
        let mut copy_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Frame copy command encoder").as_deref(),
                });

        let mut render_pass = copy_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            label: None,
        });
        render_pass.set_pipeline(&&self.copy_pipeline);

        render_pass.set_bind_group(0, globals.bind_group(), &[]);
        render_pass.set_bind_group(1, &self.transforms_bind_group, &[0]);
        render_pass.set_bind_group(2, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, descriptors.quad.vertices.slice(..));
        render_pass.set_index_buffer(
            descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.draw_indexed(0..6, 0, 0..1);
        drop(render_pass);

        copy_encoder.finish()
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
