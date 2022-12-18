use crate::commands::{CommandRenderer, CommandTarget};
use crate::mesh::Mesh;
use crate::uniform_buffer::BufferStorage;
use crate::utils::remove_srgb;
use crate::{Descriptors, Globals, Pipelines, TextureTransforms, Transforms, UniformBuffer};
use ruffle_render::commands::CommandList;
use std::sync::Arc;

#[derive(Debug)]
pub struct ResolveBuffer {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl ResolveBuffer {
    pub fn new(
        descriptors: &Descriptors,
        label: Option<String>,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let texture = descriptors.device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        });

        let view = texture.create_view(&Default::default());

        Self { texture, view }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

#[derive(Debug)]
pub struct FrameBuffer {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: wgpu::Extent3d,
}

impl FrameBuffer {
    pub fn new(
        descriptors: &Descriptors,
        label: Option<String>,
        sample_count: u32,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let texture = descriptors.device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        });

        let view = texture.create_view(&Default::default());

        Self {
            texture,
            view,
            size,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }
}

#[derive(Debug)]
pub struct BlendBuffer {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl BlendBuffer {
    pub fn new(
        descriptors: &Descriptors,
        label: Option<String>,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let texture = descriptors.device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        });
        let view = texture.create_view(&Default::default());

        Self { texture, view }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}

#[derive(Debug)]
pub struct DepthBuffer {
    view: wgpu::TextureView,
}

impl DepthBuffer {
    pub fn new(
        device: &wgpu::Device,
        label: Option<String>,
        msaa_sample_count: u32,
        size: wgpu::Extent3d,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: label.as_deref(),
            size,
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&Default::default());
        Self { view }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

#[derive(Debug)]
pub struct Surface {
    size: wgpu::Extent3d,
    sample_count: u32,
    pipelines: Arc<Pipelines>,
    globals: Globals,
    format: wgpu::TextureFormat,
    actual_surface_format: wgpu::TextureFormat,
}

impl Surface {
    pub fn new(
        descriptors: &Descriptors,
        sample_count: u32,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let frame_buffer_format = remove_srgb(surface_format);

        let globals = Globals::new(
            &descriptors.device,
            &descriptors.bind_layouts.globals,
            width,
            height,
        );

        let pipelines = descriptors.pipelines(sample_count, frame_buffer_format);
        Self {
            size,
            sample_count,
            pipelines,
            globals,
            format: frame_buffer_format,
            actual_surface_format: surface_format,
        }
    }

    pub fn draw_commands(
        &mut self,
        frame_view: &wgpu::TextureView,
        mut clear_color: Option<wgpu::Color>,
        descriptors: &Descriptors,
        uniform_buffers_storage: &mut BufferStorage<Transforms>,
        meshes: &Vec<Mesh>,
        commands: CommandList,
    ) -> Vec<wgpu::CommandBuffer> {
        uniform_buffers_storage.recall();
        let uniform_encoder_label = create_debug_label!("Uniform upload command encoder");
        let mut uniform_buffer = UniformBuffer::new(uniform_buffers_storage);
        let mut uniform_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: uniform_encoder_label.as_deref(),
                });

        let target = CommandTarget::new(
            &self.globals,
            &descriptors,
            self.size,
            self.format,
            self.sample_count,
        );

        let mut buffers = vec![];

        CommandRenderer::execute(
            &self.pipelines,
            &target,
            &meshes,
            &descriptors,
            &mut uniform_buffer,
            &mut uniform_encoder,
            commands,
            &mut buffers,
            &target,
            true,
            &mut clear_color,
        );

        let copy_bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &descriptors.bind_layouts.bitmap,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &descriptors.quad.texture_transforms,
                            offset: 0,
                            size: wgpu::BufferSize::new(
                                std::mem::size_of::<TextureTransforms>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&target.color_view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(
                            &descriptors.bitmap_samplers.get_sampler(false, false),
                        ),
                    },
                ],
                label: create_debug_label!("Copy sRGB bind group").as_deref(),
            });

        let pipeline = if self.actual_surface_format == self.format {
            descriptors.copy_pipeline(self.format)
        } else {
            descriptors.copy_srgb_pipeline(self.actual_surface_format)
        };

        let mut copy_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Frame copy command encoder").as_deref(),
                });

        let load = match clear_color {
            Some(color) => wgpu::LoadOp::Clear(color),
            None => wgpu::LoadOp::Load,
        };

        let mut render_pass = copy_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                ops: wgpu::Operations { load, store: true },
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            label: None,
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, self.globals.bind_group(), &[]);
        render_pass.set_bind_group(1, &target.whole_frame_bind_group(), &[0]);
        render_pass.set_bind_group(2, &copy_bind_group, &[]);

        render_pass.set_vertex_buffer(0, descriptors.quad.vertices.slice(..));
        render_pass.set_index_buffer(
            descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.draw_indexed(0..6, 0, 0..1);
        drop(render_pass);

        buffers.push(copy_encoder.finish());
        buffers.insert(0, uniform_encoder.finish());
        uniform_buffer.finish();
        buffers
    }
}
