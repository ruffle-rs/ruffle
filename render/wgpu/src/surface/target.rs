use crate::Transforms;
use crate::backend::RenderTargetMode;
use crate::buffer_pool::{AlwaysCompatible, PoolEntry, TexturePool};
use crate::descriptors::Descriptors;
use crate::globals::Globals;
use crate::utils::create_buffer_with_data;
use crate::utils::run_copy_pipeline;
use std::cell::OnceCell;
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

    pub fn new_manual(texture: wgpu::Texture) -> Self {
        Self {
            texture: PoolOrArcTexture::Manual((
                texture.clone(),
                texture.create_view(&Default::default()),
            )),
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        self.texture.view()
    }

    pub fn texture(&self) -> &wgpu::Texture {
        self.texture.texture()
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
    Manual((wgpu::Texture, wgpu::TextureView)),
}

impl PoolOrArcTexture {
    pub fn texture(&self) -> &wgpu::Texture {
        match self {
            PoolOrArcTexture::Pool(t) => &t.0,
            PoolOrArcTexture::Manual(t) => &t.0,
        }
    }
    pub fn view(&self) -> &wgpu::TextureView {
        match self {
            PoolOrArcTexture::Pool(t) => &t.1,
            PoolOrArcTexture::Manual(t) => &t.1,
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

    pub fn new_manual(texture: wgpu::Texture, size: wgpu::Extent3d) -> Self {
        Self {
            texture: PoolOrArcTexture::Manual((
                texture.clone(),
                texture.create_view(&Default::default()),
            )),
            size,
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        self.texture.view()
    }

    pub fn texture(&self) -> &wgpu::Texture {
        self.texture.texture()
    }

    pub fn take_texture(self) -> PoolOrArcTexture {
        self.texture
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }
}

/// A readable texture+view pair for use as blend shader input.
pub struct BlendSource<'a> {
    pub texture: &'a wgpu::Texture,
    pub view: &'a wgpu::TextureView,
}

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

    pub fn as_source(&self) -> BlendSource<'_> {
        BlendSource {
            texture: self.texture(),
            view: self.view(),
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.1
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture.0
    }
}

#[derive(Debug)]
pub struct StencilBuffer {
    texture: PoolEntry<(wgpu::Texture, wgpu::TextureView), AlwaysCompatible>,
}

impl StencilBuffer {
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
    depth: OnceCell<StencilBuffer>,
    globals: Arc<Globals>,
    size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    sample_count: u32,
    whole_frame_bind_group: OnceCell<(wgpu::Buffer, wgpu::BindGroup)>,
    color_needs_clear: OnceCell<bool>,
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
        Self::new_with_offset(
            descriptors,
            pool,
            size,
            format,
            sample_count,
            render_target_mode,
            encoder,
            0,
            0,
        )
    }

    #[expect(clippy::too_many_arguments)]
    pub fn new_with_offset(
        descriptors: &Descriptors,
        pool: &mut TexturePool,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        sample_count: u32,
        render_target_mode: RenderTargetMode,
        encoder: &mut wgpu::CommandEncoder,
        offset_x: u32,
        offset_y: u32,
    ) -> Self {
        let globals = if offset_x == 0 && offset_y == 0 {
            pool.get_globals(descriptors, size.width, size.height)
        } else {
            pool.get_globals_with_offset(descriptors, offset_x, offset_y, size.width, size.height)
        };

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

        let (frame_buffer, resolve_buffer) =
            if let RenderTargetMode::ExistingWithColor(texture, _) = &render_target_mode {
                if sample_count > 1 {
                    (
                        make_pooled_frame_buffer(),
                        Some(ResolveBuffer::new_manual(texture.clone())),
                    )
                } else {
                    (
                        FrameBuffer::new_manual(texture.clone(), texture.size()),
                        None,
                    )
                }
            } else if sample_count > 1 {
                (
                    make_pooled_frame_buffer(),
                    Some(ResolveBuffer::new(
                        descriptors,
                        size,
                        format,
                        wgpu::TextureUsages::COPY_SRC
                            | wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::RENDER_ATTACHMENT,
                        pool,
                    )),
                )
            } else {
                (make_pooled_frame_buffer(), None)
            };

        if let RenderTargetMode::FreshWithTexture(texture) = &render_target_mode {
            if let Some(resolve_buffer) = &resolve_buffer {
                encoder.copy_texture_to_texture(
                    texture.as_image_copy(),
                    resolve_buffer.texture().as_image_copy(),
                    size,
                );
            }

            if sample_count > 1 {
                run_copy_pipeline(
                    descriptors,
                    format,
                    frame_buffer.texture.view(),
                    &texture.create_view(&Default::default()),
                    get_whole_frame_bind_group(&whole_frame_bind_group, descriptors, size),
                    &globals,
                    sample_count,
                    encoder,
                );
            } else {
                encoder.copy_texture_to_texture(
                    texture.as_image_copy(),
                    frame_buffer.texture().as_image_copy(),
                    size,
                );
            }
        }

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
            color_needs_clear: OnceCell::new(),
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
        if self.render_target_mode.color().is_some() {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: create_debug_label!("Clearing command target").as_deref(),
                color_attachments: &[self.color_attachments()],
                ..Default::default()
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

    pub fn color_attachments(&self) -> Option<wgpu::RenderPassColorAttachment<'_>> {
        let mut load = wgpu::LoadOp::Load;
        if self.color_needs_clear.set(false).is_ok()
            && let Some(clear_color) = self.render_target_mode.color()
        {
            load = wgpu::LoadOp::Clear(clear_color);
        }
        Some(wgpu::RenderPassColorAttachment {
            view: self.frame_buffer.view(),
            resolve_target: self.resolve_buffer.as_ref().map(|b| b.view()),
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn stencil_attachment(
        &self,
        descriptors: &Descriptors,
        pool: &mut TexturePool,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment<'_>> {
        let new_buffer = self.depth.get().is_none();
        let stencil = self
            .depth
            .get_or_init(|| StencilBuffer::new(descriptors, self.sample_count, self.size, pool));
        Some(wgpu::RenderPassDepthStencilAttachment {
            view: stencil.view(),
            depth_ops: None,
            stencil_ops: Some(wgpu::Operations {
                load: if new_buffer {
                    wgpu::LoadOp::Clear(0)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            }),
        })
    }

    /// Get the full-viewport blend source.
    pub fn update_blend_buffer(
        &self,
        descriptors: &Descriptors,
        pool: &mut TexturePool,
        encoder: &mut wgpu::CommandEncoder,
    ) -> BlendSource<'_> {
        let blend_buffer = self.blend_buffer.get_or_init(|| {
            BlendBuffer::new(
                descriptors,
                self.size,
                self.format,
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                pool,
            )
        });
        self.ensure_cleared(encoder);
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: self
                    .resolve_buffer
                    .as_ref()
                    .map(|b| b.texture())
                    .unwrap_or_else(|| self.frame_buffer.texture()),
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            wgpu::TexelCopyTextureInfo {
                texture: blend_buffer.texture(),
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            self.frame_buffer.size(),
        );
        blend_buffer.as_source()
    }

    /// Copy only a sub-region into a bounds-sized blend buffer.
    /// When the full blend buffer is up-to-date, copies from it instead.
    #[expect(clippy::too_many_arguments)]
    pub fn update_blend_buffer_region(
        &self,
        descriptors: &Descriptors,
        pool: &mut TexturePool,
        encoder: &mut wgpu::CommandEncoder,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> BlendBuffer {
        let alloc_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let blend_buffer = BlendBuffer::new(
            descriptors,
            alloc_size,
            self.format,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            pool,
        );
        self.ensure_cleared(encoder);
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: self
                    .resolve_buffer
                    .as_ref()
                    .map(|b| b.texture())
                    .unwrap_or_else(|| self.frame_buffer.texture()),
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: Default::default(),
            },
            wgpu::TexelCopyTextureInfo {
                texture: blend_buffer.texture(),
                mip_level: 0,
                origin: Default::default(),
                aspect: Default::default(),
            },
            alloc_size,
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
            // Shared by the copy pipeline (which only reads `world_matrix`) and the
            // full-viewport blend path. The copy pipeline ignores the color vectors,
            // and the blend shaders reinterpret these bytes as `BlendTransforms` — an
            // all-zero UV transform, i.e. identity (no region remapping).
            let transform = Transforms {
                world_matrix: [
                    [size.width as f32, 0.0, 0.0, 0.0],
                    [0.0, size.height as f32, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                mult_color: [0.0, 0.0, 0.0, 0.0],
                add_color: [0.0, 0.0, 0.0, 0.0],
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
