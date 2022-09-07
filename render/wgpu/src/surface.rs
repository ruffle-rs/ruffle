use crate::descriptors::Quad;
use crate::frame::Frame;
use crate::layouts::BindLayouts;
use crate::surface::Surface::{Direct, DirectSrgb, Resolve, ResolveSrgb};
use crate::target::RenderTargetFrame;
use crate::uniform_buffer::BufferStorage;
use crate::{
    create_buffer_with_data, ColorAdjustments, Descriptors, Globals, TextureTransforms, Transforms,
    UniformBuffer,
};

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
    _transforms_buffer: wgpu::Buffer,
    transforms_bind_group: wgpu::BindGroup,
}

impl Srgb {
    pub fn new(
        device: &wgpu::Device,
        layouts: &BindLayouts,
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
            &descriptors.onscreen.pipelines,
            &descriptors,
            UniformBuffer::new(uniform_buffers_storage),
            srgb_render_pass,
            uniform_encoder,
        );

        srgb_frame.prep_srgb_copy(&self.bind_group);
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
    },
    DirectSrgb {
        srgb: Srgb,
        depth: DepthTexture,
    },
    Resolve {
        frame_buffer: FrameBuffer,
        depth: DepthTexture,
    },
    ResolveSrgb {
        frame_buffer: FrameBuffer,
        srgb: Srgb,
        depth: DepthTexture,
    },
}

impl Surface {
    pub fn new(
        descriptors: &Descriptors,
        msaa_sample_count: u32,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
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

        let srgb = if surface_format != frame_buffer_format {
            Some(Srgb::new(
                &descriptors.device,
                &descriptors.bind_layouts,
                &descriptors.quad,
                frame_buffer_format,
                width,
                height,
            ))
        } else {
            None
        };

        let depth = DepthTexture::new(&descriptors.device, msaa_sample_count, width, height);

        match (frame_buffer, srgb) {
            (Some(frame_buffer), None) => Resolve {
                frame_buffer,
                depth,
            },
            (None, None) => Direct { depth },
            (Some(frame_buffer), Some(srgb)) => ResolveSrgb {
                frame_buffer,
                depth,
                srgb,
            },
            (None, Some(srgb)) => DirectSrgb { depth, srgb },
        }
    }

    pub fn view<'a, T: RenderTargetFrame>(&'a self, frame: &'a T) -> &wgpu::TextureView {
        match self {
            Direct { .. } => frame.view(),
            DirectSrgb { srgb, .. } => &srgb.view,
            Resolve { frame_buffer, .. } => &frame_buffer.view,
            ResolveSrgb { frame_buffer, .. } => &frame_buffer.view,
        }
    }

    pub fn resolve_target<'a, T: RenderTargetFrame>(
        &'a self,
        frame: &'a T,
    ) -> Option<&wgpu::TextureView> {
        match self {
            Direct { .. } => None,
            DirectSrgb { .. } => None,
            Resolve { .. } => Some(&frame.view()),
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

    pub fn copy_srgb<T: RenderTargetFrame>(
        &self,
        frame: &T,
        descriptors: &Descriptors,
        globals: &Globals,
        uniform_buffers_storage: &mut BufferStorage<Transforms>,
        uniform_encoder: &mut wgpu::CommandEncoder,
    ) -> Option<wgpu::CommandBuffer> {
        match self {
            Direct { .. } => None,
            DirectSrgb { srgb, .. } => Some(srgb.copy_srgb(
                frame.view(),
                descriptors,
                globals,
                uniform_buffers_storage,
                uniform_encoder,
            )),
            Resolve { .. } => None,
            ResolveSrgb { srgb, .. } => Some(srgb.copy_srgb(
                frame.view(),
                descriptors,
                globals,
                uniform_buffers_storage,
                uniform_encoder,
            )),
        }
    }
}
