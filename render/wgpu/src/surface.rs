use crate::commands::CommandRenderer;
use crate::frame::Frame;
use crate::mesh::Mesh;
use crate::uniform_buffer::BufferStorage;
use crate::{Descriptors, Globals, Pipelines, Transforms, UniformBuffer};
use ruffle_render::commands::CommandList;
use std::sync::Arc;

#[derive(Debug)]
pub struct FrameBuffer {
    view: wgpu::TextureView,
}

impl FrameBuffer {
    pub fn new(
        device: &wgpu::Device,
        msaa_sample_count: u32,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: create_debug_label!("Framebuffer texture").as_deref(),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&Default::default());
        Self { view }
    }
}

#[derive(Debug)]
pub struct DepthTexture {
    view: wgpu::TextureView,
}

impl DepthTexture {
    pub fn new(device: &wgpu::Device, msaa_sample_count: u32, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: create_debug_label!("Depth texture").as_deref(),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&Default::default());
        Self { view }
    }
}

#[derive(Debug)]
pub struct Surface {
    frame_buffer: Option<FrameBuffer>,
    depth: DepthTexture,
    pipelines: Arc<Pipelines>,
}

impl Surface {
    pub fn new(
        descriptors: &Descriptors,
        msaa_sample_count: u32,
        width: u32,
        height: u32,
        frame_buffer_format: wgpu::TextureFormat,
    ) -> Self {
        let frame_buffer = if msaa_sample_count > 1 {
            Some(FrameBuffer::new(
                &descriptors.device,
                msaa_sample_count,
                width,
                height,
                frame_buffer_format,
            ))
        } else {
            None
        };

        let depth = DepthTexture::new(&descriptors.device, msaa_sample_count, width, height);
        let pipelines = descriptors.pipelines(msaa_sample_count, frame_buffer_format);
        Self {
            frame_buffer,
            depth,
            pipelines,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_commands(
        &self,
        frame_view: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
        descriptors: &Descriptors,
        globals: &mut Globals,
        uniform_buffers_storage: &mut BufferStorage<Transforms>,
        meshes: &Vec<Mesh>,
        commands: CommandList,
    ) -> Vec<wgpu::CommandBuffer> {
        let label = create_debug_label!("Draw encoder");
        let mut draw_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: label.as_deref(),
                });

        let uniform_encoder_label = create_debug_label!("Uniform upload command encoder");
        let mut uniform_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: uniform_encoder_label.as_deref(),
                });

        globals.update_uniform(&descriptors.device, &mut draw_encoder);

        let load = match clear_color {
            Some(color) => wgpu::LoadOp::Clear(color),
            None => wgpu::LoadOp::Load,
        };

        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.frame_buffer.as_ref().map_or(&frame_view, |f| &f.view),
                ops: wgpu::Operations { load, store: true },
                resolve_target: if self.frame_buffer.is_some() {
                    Some(&frame_view)
                } else {
                    None
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: false,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
            label: None,
        });
        render_pass.set_bind_group(0, globals.bind_group(), &[]);

        uniform_buffers_storage.recall();
        let mut uniform_buffer = UniformBuffer::new(uniform_buffers_storage);
        commands.execute(&mut CommandRenderer::new(
            Frame::new(
                &self.pipelines,
                &descriptors,
                &mut uniform_buffer,
                render_pass,
                &mut uniform_encoder,
            ),
            meshes,
            descriptors.quad.vertices.slice(..),
            descriptors.quad.indices.slice(..),
        ));
        uniform_buffer.finish();

        vec![uniform_encoder.finish(), draw_encoder.finish()]
    }
}
