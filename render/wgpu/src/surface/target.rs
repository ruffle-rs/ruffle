use crate::buffer_pool::{PoolEntry, TexturePool};
use crate::descriptors::Descriptors;
use crate::globals::Globals;
use crate::utils::create_buffer_with_data;
use crate::Transforms;
use once_cell::race::OnceBool;
use once_cell::sync::OnceCell;
use std::sync::Arc;

#[derive(Debug)]
pub struct ResolveBuffer {
    texture: PoolEntry<(wgpu::Texture, wgpu::TextureView)>,
}

impl ResolveBuffer {
    pub fn new(
        descriptors: &Descriptors,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        pool: &mut TexturePool,
    ) -> Self {
        let texture = pool.get_texture(descriptors, size, usage, format, 1);
        Self { texture }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.1
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture.0
    }

    pub fn take_texture(self) -> PoolEntry<(wgpu::Texture, wgpu::TextureView)> {
        self.texture
    }
}

#[derive(Debug)]
pub struct FrameBuffer {
    texture: PoolEntry<(wgpu::Texture, wgpu::TextureView)>,
    size: wgpu::Extent3d,
}

impl FrameBuffer {
    pub fn new(
        descriptors: &Descriptors,
        sample_count: u32,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        pool: &mut TexturePool,
    ) -> Self {
        let texture = pool.get_texture(descriptors, size, usage, format, sample_count);

        Self { texture, size }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.1
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture.0
    }

    pub fn take_texture(self) -> PoolEntry<(wgpu::Texture, wgpu::TextureView)> {
        self.texture
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }
}

#[derive(Debug)]
pub struct BlendBuffer {
    texture: PoolEntry<(wgpu::Texture, wgpu::TextureView)>,
}

impl BlendBuffer {
    pub fn new(
        descriptors: &Descriptors,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        pool: &mut TexturePool,
    ) -> Self {
        let texture = pool.get_texture(descriptors, size, usage, format, 1);

        Self { texture }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.1
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture.0
    }
}

#[derive(Debug)]
pub struct DepthBuffer {
    texture: PoolEntry<(wgpu::Texture, wgpu::TextureView)>,
}

impl DepthBuffer {
    pub fn new(
        descriptors: &Descriptors,
        msaa_sample_count: u32,
        size: wgpu::Extent3d,
        pool: &mut TexturePool,
    ) -> Self {
        let texture = pool.get_texture(
            descriptors,
            size,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::TextureFormat::Stencil8,
            msaa_sample_count,
        );

        Self { texture }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.1
    }
}

pub struct CommandTarget {
    frame_buffer: FrameBuffer,
    blend_buffer: OnceCell<BlendBuffer>,
    resolve_buffer: Option<ResolveBuffer>,
    depth: OnceCell<DepthBuffer>,
    globals: Arc<Globals>,
    size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    sample_count: u32,
    whole_frame_bind_group: OnceCell<(wgpu::Buffer, wgpu::BindGroup)>,
    color_needs_clear: OnceBool,
    clear_color: wgpu::Color,
}

impl CommandTarget {
    pub fn new(
        descriptors: &Descriptors,
        pool: &mut TexturePool,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        sample_count: u32,
        clear_color: wgpu::Color,
    ) -> Self {
        let frame_buffer = FrameBuffer::new(
            &descriptors,
            sample_count,
            size,
            format,
            if sample_count > 1 {
                wgpu::TextureUsages::RENDER_ATTACHMENT
            } else {
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING
            },
            pool,
        );

        let resolve_buffer = if sample_count > 1 {
            Some(ResolveBuffer::new(
                &descriptors,
                size,
                format,
                wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                pool,
            ))
        } else {
            None
        };

        let globals = pool.get_globals(descriptors, size.width, size.height);

        Self {
            frame_buffer,
            blend_buffer: OnceCell::new(),
            resolve_buffer,
            depth: OnceCell::new(),
            globals,
            size,
            format,
            sample_count,
            whole_frame_bind_group: OnceCell::new(),
            color_needs_clear: OnceBool::new(),
            clear_color,
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn ensure_cleared(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.color_needs_clear.get().is_some() {
            return;
        }
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: create_debug_label!("Clearing command target").as_deref(),
            color_attachments: &[self.color_attachments()],
            depth_stencil_attachment: None,
        });
    }

    pub fn take_color_texture(self) -> PoolEntry<(wgpu::Texture, wgpu::TextureView)> {
        self.resolve_buffer
            .map(|b| b.take_texture())
            .unwrap_or_else(|| self.frame_buffer.take_texture())
    }

    pub fn globals(&self) -> &Globals {
        &self.globals
    }

    pub fn whole_frame_bind_group(&self, descriptors: &Descriptors) -> &wgpu::BindGroup {
        &self
            .whole_frame_bind_group
            .get_or_init(|| {
                let transform = Transforms {
                    world_matrix: [
                        [self.size.width as f32, 0.0, 0.0, 0.0],
                        [0.0, self.size.height as f32, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                };
                let transforms_buffer = create_buffer_with_data(
                    &descriptors.device,
                    bytemuck::cast_slice(&[transform]),
                    wgpu::BufferUsages::UNIFORM,
                    create_debug_label!("Whole-frame transforms buffer"),
                );
                let whole_frame_bind_group =
                    descriptors
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &descriptors.bind_layouts.transforms,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: transforms_buffer.as_entire_binding(),
                            }],
                            label: create_debug_label!("Whole-frame transforms bind group")
                                .as_deref(),
                        });
                (transforms_buffer, whole_frame_bind_group)
            })
            .1
    }

    pub fn color_attachments(&self) -> Option<wgpu::RenderPassColorAttachment> {
        Some(wgpu::RenderPassColorAttachment {
            view: &self.frame_buffer.view(),
            resolve_target: self.resolve_buffer.as_ref().map(|b| b.view()),
            ops: wgpu::Operations {
                load: if self.color_needs_clear.set(false).is_ok() {
                    wgpu::LoadOp::Clear(self.clear_color)
                } else {
                    wgpu::LoadOp::Load
                },
                store: true,
            },
        })
    }

    pub fn depth_attachment(
        &self,
        descriptors: &Descriptors,
        pool: &mut TexturePool,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        let new_buffer = self.depth.get().is_none();
        let depth = self
            .depth
            .get_or_init(|| DepthBuffer::new(descriptors, self.sample_count, self.size, pool));
        Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth.view(),
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: false,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: if new_buffer {
                    wgpu::LoadOp::Clear(0)
                } else {
                    wgpu::LoadOp::Load
                },
                store: true,
            }),
        })
    }

    pub fn update_blend_buffer(
        &self,
        descriptors: &Descriptors,
        pool: &mut TexturePool,
        encoder: &mut wgpu::CommandEncoder,
    ) -> &BlendBuffer {
        let blend_buffer = self.blend_buffer.get_or_init(|| {
            BlendBuffer::new(
                &descriptors,
                self.size,
                self.format,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                pool,
            )
        });
        self.ensure_cleared(encoder);
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTextureBase {
                texture: self
                    .resolve_buffer
                    .as_ref()
                    .map(|b| b.texture())
                    .unwrap_or_else(|| self.frame_buffer.texture()),
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            wgpu::ImageCopyTextureBase {
                texture: blend_buffer.texture(),
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            self.frame_buffer.size(),
        );
        blend_buffer
    }

    pub fn color_view(&self) -> &wgpu::TextureView {
        self.resolve_buffer
            .as_ref()
            .map(|b| b.view())
            .unwrap_or_else(|| self.frame_buffer.view())
    }

    pub fn color_texture(&self) -> &wgpu::Texture {
        self.resolve_buffer
            .as_ref()
            .map(|b| b.texture())
            .unwrap_or_else(|| self.frame_buffer.texture())
    }
}
