use crate::commands::CommandRenderer;
use crate::descriptors::Quad;
use crate::frame::Frame;
use crate::layouts::BindLayouts;
use crate::mesh::Mesh;
use crate::surface::Surface::{Direct, DirectSrgb, Resolve, ResolveSrgb};
use crate::uniform_buffer::BufferStorage;
use crate::utils::remove_srgb;
use crate::{
    create_buffer_with_data, ColorAdjustments, Descriptors, Globals, Pipelines, RegistryData,
    TextureTransforms, Transforms, UniformBuffer,
};
use fnv::FnvHashMap;
use ruffle_render::bitmap::BitmapHandle;
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
        uniform_buffers_storage: &mut BufferStorage<Transforms>,
        uniform_encoder: &mut wgpu::CommandEncoder,
        pipelines: &Pipelines,
    ) -> wgpu::CommandBuffer {
        let mut copy_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Frame copy command encoder").as_deref(),
                });

        let mut srgb_render_pass = copy_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        srgb_render_pass.set_bind_group(0, globals.bind_group(), &[]);
        srgb_render_pass.set_bind_group(1, &self.transforms_bind_group, &[0]);

        let mut srgb_frame = Frame::new(
            &pipelines,
            &descriptors,
            UniformBuffer::new(uniform_buffers_storage),
            srgb_render_pass,
            uniform_encoder,
        );

        srgb_frame.prep_srgb_copy(&self.bind_group, &self.copy_pipeline);
        srgb_frame.draw(
            descriptors.quad.vertices.slice(..),
            descriptors.quad.indices.slice(..),
            6,
        );

        drop(srgb_frame);
        copy_encoder.finish()
    }
}

#[derive(Debug)]
pub enum Surface {
    Direct {
        depth: DepthTexture,
        pipelines: Arc<Pipelines>,
    },
    DirectSrgb {
        srgb: Srgb,
        depth: DepthTexture,
        pipelines: Arc<Pipelines>,
    },
    Resolve {
        frame_buffer: FrameBuffer,
        depth: DepthTexture,
        pipelines: Arc<Pipelines>,
    },
    ResolveSrgb {
        frame_buffer: FrameBuffer,
        srgb: Srgb,
        depth: DepthTexture,
        pipelines: Arc<Pipelines>,
    },
}

impl Surface {
    pub fn new(
        descriptors: &Descriptors,
        msaa_sample_count: u32,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let frame_buffer_format = remove_srgb(surface_format);

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

        let srgb = if surface_format != frame_buffer_format {
            Some(Srgb::new(
                &descriptors.device,
                &descriptors.bind_layouts,
                &descriptors.bitmap_samplers.get_sampler(false, false),
                descriptors.copy_srgb_pipeline(surface_format),
                &descriptors.quad,
                frame_buffer_format,
                width,
                height,
            ))
        } else {
            None
        };

        let depth = DepthTexture::new(&descriptors.device, msaa_sample_count, width, height);
        let pipelines = descriptors.pipelines(msaa_sample_count, frame_buffer_format);

        match (frame_buffer, srgb) {
            (Some(frame_buffer), None) => Resolve {
                frame_buffer,
                depth,
                pipelines,
            },
            (None, None) => Direct { depth, pipelines },
            (Some(frame_buffer), Some(srgb)) => ResolveSrgb {
                frame_buffer,
                depth,
                srgb,
                pipelines,
            },
            (None, Some(srgb)) => DirectSrgb {
                depth,
                srgb,
                pipelines,
            },
        }
    }

    pub fn view<'a>(&'a self, frame: &'a wgpu::TextureView) -> &wgpu::TextureView {
        match self {
            Direct { .. } => frame,
            DirectSrgb { srgb, .. } => &srgb.view,
            Resolve { frame_buffer, .. } => &frame_buffer.view,
            ResolveSrgb { frame_buffer, .. } => &frame_buffer.view,
        }
    }

    pub fn resolve_target<'a>(
        &'a self,
        frame: &'a wgpu::TextureView,
    ) -> Option<&wgpu::TextureView> {
        match self {
            Direct { .. } => None,
            DirectSrgb { .. } => None,
            Resolve { .. } => Some(&frame),
            ResolveSrgb { srgb, .. } => Some(&srgb.view),
        }
    }

    pub fn depth(&self) -> &wgpu::TextureView {
        match self {
            Direct { depth, .. } => &depth.view,
            DirectSrgb { depth, .. } => &depth.view,
            Resolve { depth, .. } => &depth.view,
            ResolveSrgb { depth, .. } => &depth.view,
        }
    }

    pub fn pipelines(&self) -> &Pipelines {
        match self {
            Direct { pipelines, .. } => pipelines,
            DirectSrgb { pipelines, .. } => pipelines,
            Resolve { pipelines, .. } => pipelines,
            ResolveSrgb { pipelines, .. } => pipelines,
        }
    }

    pub fn copy_srgb(
        &self,
        frame: &wgpu::TextureView,
        descriptors: &Descriptors,
        globals: &Globals,
        uniform_buffers_storage: &mut BufferStorage<Transforms>,
        uniform_encoder: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::CommandBuffer> {
        match self {
            Direct { .. } => None,
            DirectSrgb { srgb, .. } => Some(srgb.copy_srgb(
                frame,
                descriptors,
                globals,
                uniform_buffers_storage,
                uniform_encoder,
                self.pipelines(),
            )),
            Resolve { .. } => None,
            ResolveSrgb { srgb, .. } => Some(srgb.copy_srgb(
                frame,
                descriptors,
                globals,
                uniform_buffers_storage,
                uniform_encoder,
                self.pipelines(),
            )),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_commands(
        &self,
        frame_view: &wgpu::TextureView,
        clear_color: wgpu::Color,
        descriptors: &Descriptors,
        globals: &mut Globals,
        uniform_buffers_storage: &mut BufferStorage<Transforms>,
        meshes: &Vec<Mesh>,
        bitmap_registry: &FnvHashMap<BitmapHandle, RegistryData>,
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

        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.view(frame_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: true,
                },
                resolve_target: self.resolve_target(frame_view),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.depth(),
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
        let mut frame = Frame::new(
            &self.pipelines(),
            &descriptors,
            UniformBuffer::new(uniform_buffers_storage),
            render_pass,
            &mut uniform_encoder,
        );
        commands.execute(&mut CommandRenderer::new(
            &mut frame,
            meshes,
            bitmap_registry,
            descriptors.quad.vertices.slice(..),
            descriptors.quad.indices.slice(..),
        ));
        frame.finish();

        let copy_encoder = self.copy_srgb(
            &frame_view,
            &descriptors,
            &globals,
            uniform_buffers_storage,
            &mut uniform_encoder,
        );

        let mut command_buffers = vec![uniform_encoder.finish(), draw_encoder.finish()];
        if let Some(copy_encoder) = copy_encoder {
            command_buffers.push(copy_encoder);
        }

        command_buffers
    }
}
