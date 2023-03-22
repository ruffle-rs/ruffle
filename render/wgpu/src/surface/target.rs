use crate::backend::RenderTargetMode;
use crate::buffer_pool::{AlwaysCompatible, PoolEntry, TexturePool};
use crate::descriptors::Descriptors;
use crate::globals::Globals;
use crate::surface::commands::run_copy_pipeline;
use crate::utils::create_buffer_with_data;
use crate::Transforms;
use once_cell::race::OnceBool;
use once_cell::sync::OnceCell;
use std::sync::Arc;

#[derive(Debug)]
pub struct ResolveBuffer {
    texture: PoolOrArcTexture,
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
        Self {
            texture: PoolOrArcTexture::Pool(texture),
        }
    }

    pub fn new_manual(texture: Arc<wgpu::Texture>) -> Self {
        Self {
            texture: PoolOrArcTexture::Manual((
                texture.clone(),
                texture.create_view(&Default::default()),
            )),
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        match self.texture {
            PoolOrArcTexture::Pool(ref texture) => &texture.1,
            PoolOrArcTexture::Manual(ref texture) => &texture.1,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        match self.texture {
            PoolOrArcTexture::Pool(ref texture) => &texture.0,
            PoolOrArcTexture::Manual(ref texture) => &texture.0,
        }
    }

    pub fn take_texture(self) -> PoolOrArcTexture {
        self.texture
    }
}

#[derive(Debug)]
pub struct FrameBuffer {
    texture: PoolOrArcTexture,
    size: wgpu::Extent3d,
}

#[derive(Debug)]
/// Holds either a `PoolEntry` texture, or an `Arc`-wrapped texture.
/// This is used to select between using a texture pool for our framebuffer/resolve-buffer
/// (when rendering to the main screen), or rendering to a non-pooled `Texture`
/// (when doing an offscreen render to a BitmapData texture)
pub enum PoolOrArcTexture {
    Pool(PoolEntry<(wgpu::Texture, wgpu::TextureView), AlwaysCompatible>),
    Manual((Arc<wgpu::Texture>, wgpu::TextureView)),
}

impl PoolOrArcTexture {
    pub fn view(&self) -> &wgpu::TextureView {
        match self {
            PoolOrArcTexture::Pool(ref texture) => &texture.1,
            PoolOrArcTexture::Manual(ref texture) => &texture.1,
        }
    }
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

        Self {
            texture: PoolOrArcTexture::Pool(texture),
            size,
        }
    }

    pub fn new_manual(texture: Arc<wgpu::Texture>, size: wgpu::Extent3d) -> Self {
        Self {
            texture: PoolOrArcTexture::Manual((
                texture.clone(),
                texture.create_view(&Default::default()),
            )),
            size,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        match self.texture {
            PoolOrArcTexture::Pool(ref texture) => &texture.1,
            PoolOrArcTexture::Manual(ref texture) => &texture.1,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        match self.texture {
            PoolOrArcTexture::Pool(ref texture) => &texture.0,
            PoolOrArcTexture::Manual(ref texture) => &texture.0,
        }
    }

    pub fn take_texture(self) -> PoolOrArcTexture {
        self.texture
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }
}

#[derive(Debug)]
pub struct BlendBuffer {
    texture: PoolEntry<(wgpu::Texture, wgpu::TextureView), AlwaysCompatible>,
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
    texture: PoolEntry<(wgpu::Texture, wgpu::TextureView), AlwaysCompatible>,
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
    render_target_mode: RenderTargetMode,
}

impl CommandTarget {
    pub fn new(
        descriptors: &Descriptors,
        pool: &mut TexturePool,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        sample_count: u32,
        render_target_mode: RenderTargetMode,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let globals = pool.get_globals(descriptors, size.width, size.height);

        let mut make_pooled_frame_buffer = || {
            FrameBuffer::new(
                descriptors,
                sample_count,
                size,
                format,
                if sample_count > 1 {
                    wgpu::TextureUsages::RENDER_ATTACHMENT
                } else {
                    wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::TEXTURE_BINDING
                },
                pool,
            )
        };

        let whole_frame_bind_group = OnceCell::new();

        let (frame_buffer, resolve_buffer) = match &render_target_mode {
            // In `FreshBuffer` mode, get a new frame buffer (and resolve buffer, if necessary)
            // from the pool. They will be cleared with the provided clear color
            // in `color_attachments`
            RenderTargetMode::FreshBuffer(_) => {
                let frame_buffer = make_pooled_frame_buffer();
                let resolve_buffer = if sample_count > 1 {
                    Some(ResolveBuffer::new(
                        descriptors,
                        size,
                        format,
                        wgpu::TextureUsages::COPY_SRC
                            | wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::RENDER_ATTACHMENT,
                        pool,
                    ))
                } else {
                    None
                };
                (frame_buffer, resolve_buffer)
            }
            // In `ExistingTexture` mode, we will use an existing texture
            // as either the frame buffer or resolve buffer.
            RenderTargetMode::ExistingTexture(texture) => {
                if sample_count > 1 {
                    // The exising texture always has a sample count of 1,
                    // so we need to create a new texture for the multisampled frame
                    // buffer. Our existing texture will be used as the resolve buffer,
                    // which is downsampled from the frame buffer.
                    let frame_buffer = make_pooled_frame_buffer();

                    // Both our frame buffer and resolve buffer need to start out
                    // in the same state, so copy our existing texture to the freshly
                    // allocated frame buffer. We cannot use `copy_texture_to_texture`,
                    // since the sample counts are different.
                    run_copy_pipeline(
                        descriptors,
                        format,
                        format,
                        size,
                        frame_buffer.texture.view(),
                        &texture.create_view(&Default::default()),
                        get_whole_frame_bind_group(&whole_frame_bind_group, descriptors, size),
                        &globals,
                        sample_count,
                        encoder,
                    );

                    (
                        frame_buffer,
                        Some(ResolveBuffer::new_manual(texture.clone())),
                    )
                } else {
                    // If multisampling is disabled, we don't need a resolve buffer.
                    // We can just use our existing texture as the frame buffer.
                    (FrameBuffer::new_manual(texture.clone(), size), None)
                }
            }
        };

        Self {
            frame_buffer,
            blend_buffer: OnceCell::new(),
            resolve_buffer,
            depth: OnceCell::new(),
            globals,
            size,
            format,
            sample_count,
            whole_frame_bind_group,
            color_needs_clear: OnceBool::new(),
            render_target_mode,
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn ensure_cleared(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.color_needs_clear.get().is_some() {
            return;
        }
        // If we don't have ClearType::Color (we have ClearType::Texture),
        // the there's no point in creating a new render pass that does nothing.
        if let RenderTargetMode::FreshBuffer(_) = self.render_target_mode {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: create_debug_label!("Clearing command target").as_deref(),
                color_attachments: &[self.color_attachments()],
                depth_stencil_attachment: None,
            });
        }
    }

    pub fn take_color_texture(self) -> PoolOrArcTexture {
        self.resolve_buffer
            .map(|b| b.take_texture())
            .unwrap_or_else(|| self.frame_buffer.take_texture())
    }

    pub fn globals(&self) -> &Globals {
        &self.globals
    }

    pub fn whole_frame_bind_group(&self, descriptors: &Descriptors) -> &wgpu::BindGroup {
        get_whole_frame_bind_group(&self.whole_frame_bind_group, descriptors, self.size)
    }

    pub fn color_attachments(&self) -> Option<wgpu::RenderPassColorAttachment> {
        let mut load = wgpu::LoadOp::Load;
        if self.color_needs_clear.set(false).is_ok() {
            if let RenderTargetMode::FreshBuffer(clear_color) = &self.render_target_mode {
                load = wgpu::LoadOp::Clear(*clear_color);
            }
        }
        Some(wgpu::RenderPassColorAttachment {
            view: self.frame_buffer.view(),
            resolve_target: self.resolve_buffer.as_ref().map(|b| b.view()),
            ops: wgpu::Operations { load, store: true },
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
                descriptors,
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

fn get_whole_frame_bind_group<'a>(
    once_cell: &'a OnceCell<(wgpu::Buffer, wgpu::BindGroup)>,
    descriptors: &Descriptors,
    size: wgpu::Extent3d,
) -> &'a wgpu::BindGroup {
    &once_cell
        .get_or_init(|| {
            let transform = Transforms {
                world_matrix: [
                    [size.width as f32, 0.0, 0.0, 0.0],
                    [0.0, size.height as f32, 0.0, 0.0],
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
                        label: create_debug_label!("Whole-frame transforms bind group").as_deref(),
                    });
            (transforms_buffer, whole_frame_bind_group)
        })
        .1
}
