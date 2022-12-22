use crate::buffer_pool::{PoolEntry, TexturePool};
use crate::mesh::{BitmapBinds, DrawType, Mesh};
use crate::pipelines::{BlendType, ComplexBlend, TrivialBlend};
use crate::surface::{BlendBuffer, DepthBuffer, FrameBuffer, ResolveBuffer, Surface};
use crate::utils::create_buffer_with_data;
use crate::{
    as_texture, ColorAdjustments, Descriptors, Globals, MaskState, Pipelines, Transforms,
    UniformBuffer,
};
use once_cell::sync::OnceCell;
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::commands::{Command, CommandList};
use ruffle_render::matrix::Matrix;
use ruffle_render::transform::Transform;
use swf::{BlendMode, Color};

pub struct CommandTarget {
    frame_buffer: FrameBuffer,
    blend_buffer: OnceCell<BlendBuffer>,
    resolve_buffer: Option<ResolveBuffer>,
    depth: OnceCell<DepthBuffer>,
    globals: Globals,
    size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    sample_count: u32,
    whole_frame_bind_group: OnceCell<(wgpu::Buffer, wgpu::BindGroup)>,
}

impl CommandTarget {
    pub fn new(
        descriptors: &Descriptors,
        pool: &mut TexturePool,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        sample_count: u32,
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

        let globals = Globals::new(
            &descriptors.device,
            &descriptors.bind_layouts.globals,
            size.width,
            size.height,
        );

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
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn take_color_texture(self) -> PoolEntry<wgpu::Texture> {
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
                    color_adjustments: ColorAdjustments {
                        mult_color: [1.0, 1.0, 1.0, 1.0],
                        add_color: [0.0, 0.0, 0.0, 0.0],
                    },
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
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &transforms_buffer,
                                    offset: 0,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<Transforms>() as u64
                                    ),
                                }),
                            }],
                            label: create_debug_label!("Whole-frame transforms bind group")
                                .as_deref(),
                        });
                (transforms_buffer, whole_frame_bind_group)
            })
            .1
    }

    pub fn color_attachments(
        &self,
        clear: Option<wgpu::Color>,
    ) -> Option<wgpu::RenderPassColorAttachment> {
        Some(wgpu::RenderPassColorAttachment {
            view: &self.frame_buffer.view(),
            resolve_target: self.resolve_buffer.as_ref().map(|b| b.view()),
            ops: wgpu::Operations {
                load: if let Some(color) = clear {
                    wgpu::LoadOp::Clear(color)
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
        clear: bool,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        let depth = self
            .depth
            .get_or_init(|| DepthBuffer::new(descriptors, self.sample_count, self.size, pool));
        Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth.view(),
            depth_ops: Some(wgpu::Operations {
                load: if clear {
                    wgpu::LoadOp::Clear(0.0)
                } else {
                    wgpu::LoadOp::Load
                },
                store: true,
            }),
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
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
}

pub struct CommandRenderer<'pass, 'frame: 'pass, 'global: 'frame> {
    pipelines: &'frame Pipelines,
    meshes: &'global Vec<Mesh>,
    descriptors: &'global Descriptors,
    num_masks: u32,
    mask_state: MaskState,
    render_pass: wgpu::RenderPass<'pass>,
    uniform_buffers: &'frame mut UniformBuffer<'global, Transforms>,
    uniform_encoder: &'frame mut wgpu::CommandEncoder,
    needs_depth: bool,
}

impl<'pass, 'frame: 'pass, 'global: 'frame> CommandRenderer<'pass, 'frame, 'global> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pipelines: &'frame Pipelines,
        meshes: &'global Vec<Mesh>,
        descriptors: &'global Descriptors,
        uniform_buffers: &'frame mut UniformBuffer<'global, Transforms>,
        uniform_encoder: &'frame mut wgpu::CommandEncoder,
        render_pass: wgpu::RenderPass<'pass>,
        num_masks: u32,
        mask_state: MaskState,
        needs_depth: bool,
    ) -> Self {
        Self {
            pipelines,
            meshes,
            num_masks,
            mask_state,
            render_pass,
            descriptors,
            uniform_buffers,
            uniform_encoder,
            needs_depth,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute(
        pipelines: &'frame Pipelines,
        target: &'pass CommandTarget,
        meshes: &'global Vec<Mesh>,
        descriptors: &'global Descriptors,
        uniform_buffers: &'frame mut UniformBuffer<'global, Transforms>,
        uniform_encoder: &'frame mut wgpu::CommandEncoder,
        commands: &CommandList,
        nearest_layer: &'pass CommandTarget,
        clear_color: &mut Option<wgpu::Color>,
        draw_encoder: &'pass mut wgpu::CommandEncoder,
        texture_pool: &mut TexturePool,
    ) {
        let mut first = true;
        let mut num_masks = 0;
        let mut mask_state = MaskState::NoMask;
        let (chunks, needs_depth) = chunk_blends(
            &commands.0,
            descriptors,
            uniform_buffers,
            uniform_encoder,
            draw_encoder,
            meshes,
            target.sample_count,
            target.size.width,
            target.size.height,
            nearest_layer,
            texture_pool,
        );

        for chunk in chunks {
            match chunk {
                Chunk::Draw(chunk) => {
                    let mut render_pass =
                        draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[target.color_attachments(clear_color.take())],
                            depth_stencil_attachment: if needs_depth {
                                target.depth_attachment(&descriptors, texture_pool, first)
                            } else {
                                None
                            },
                        });
                    render_pass.set_bind_group(0, target.globals.bind_group(), &[]);
                    let mut renderer = CommandRenderer::new(
                        &pipelines,
                        &meshes,
                        &descriptors,
                        uniform_buffers,
                        uniform_encoder,
                        render_pass,
                        num_masks,
                        mask_state,
                        needs_depth,
                    );

                    for command in &chunk {
                        if needs_depth {
                            match renderer.mask_state {
                                MaskState::NoMask => {}
                                MaskState::DrawMaskStencil => {
                                    renderer
                                        .render_pass
                                        .set_stencil_reference(renderer.num_masks - 1);
                                }
                                MaskState::DrawMaskedContent => {
                                    renderer
                                        .render_pass
                                        .set_stencil_reference(renderer.num_masks);
                                }
                                MaskState::ClearMaskStencil => {
                                    renderer
                                        .render_pass
                                        .set_stencil_reference(renderer.num_masks);
                                }
                            }
                        }

                        match command {
                            DrawCommand::RenderBitmap {
                                bitmap,
                                transform,
                                smoothing,
                                blend_mode,
                            } => {
                                renderer.render_bitmap(bitmap, &transform, *smoothing, *blend_mode)
                            }
                            DrawCommand::RenderTexture {
                                _texture,
                                binds,
                                transform,
                                blend_mode,
                            } => renderer.render_texture(&transform, binds, *blend_mode),
                            DrawCommand::RenderShape { shape, transform } => {
                                renderer.render_shape(*shape, &transform)
                            }
                            DrawCommand::DrawRect { color, matrix } => {
                                renderer.draw_rect(color, &matrix)
                            }
                            DrawCommand::PushMask => renderer.push_mask(),
                            DrawCommand::ActivateMask => renderer.activate_mask(),
                            DrawCommand::DeactivateMask => renderer.deactivate_mask(),
                            DrawCommand::PopMask => renderer.pop_mask(),
                        }
                    }

                    num_masks = renderer.num_masks;
                    mask_state = renderer.mask_state;
                }
                Chunk::Blend(texture, blend_mode) => {
                    let parent = match blend_mode {
                        ComplexBlend::Alpha | ComplexBlend::Erase => nearest_layer,
                        _ => target,
                    };

                    let parent_blend_buffer =
                        parent.update_blend_buffer(&descriptors, texture_pool, draw_encoder);

                    let texture_view = texture.create_view(&Default::default());
                    let blend_bind_group =
                        descriptors
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                label: None,
                                layout: &descriptors.bind_layouts.blend,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::TextureView(
                                            parent_blend_buffer.view(),
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::TextureView(&texture_view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 2,
                                        resource: wgpu::BindingResource::Sampler(
                                            descriptors.bitmap_samplers.get_sampler(false, false),
                                        ),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 3,
                                        resource: descriptors
                                            .blend_buffer(blend_mode)
                                            .as_entire_binding(),
                                    },
                                ],
                            });

                    let mut render_pass =
                        draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[target.color_attachments(clear_color.take())],
                            depth_stencil_attachment: if needs_depth {
                                target.depth_attachment(descriptors, texture_pool, first)
                            } else {
                                None
                            },
                        });
                    render_pass.set_bind_group(0, target.globals.bind_group(), &[]);

                    if needs_depth {
                        match mask_state {
                            MaskState::NoMask => {}
                            MaskState::DrawMaskStencil => {
                                render_pass.set_stencil_reference(num_masks - 1);
                            }
                            MaskState::DrawMaskedContent => {
                                render_pass.set_stencil_reference(num_masks);
                            }
                            MaskState::ClearMaskStencil => {
                                render_pass.set_stencil_reference(num_masks);
                            }
                        }
                        render_pass.set_pipeline(pipelines.complex_blend.pipeline_for(mask_state));
                    } else {
                        render_pass.set_pipeline(pipelines.complex_blend.depthless_pipeline());
                    }

                    render_pass.set_bind_group(1, target.whole_frame_bind_group(descriptors), &[0]);
                    render_pass.set_bind_group(2, &blend_bind_group, &[]);

                    render_pass.set_vertex_buffer(0, descriptors.quad.vertices.slice(..));
                    render_pass.set_index_buffer(
                        descriptors.quad.indices.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    render_pass.draw_indexed(0..6, 0, 0..1);
                    drop(render_pass);
                }
            }
            first = false;
        }
    }

    pub fn prep_color(&mut self) {
        if self.needs_depth {
            self.render_pass
                .set_pipeline(self.pipelines.color.pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.color.depthless_pipeline());
        }
    }

    pub fn prep_gradient(&mut self, bind_group: &'pass wgpu::BindGroup) {
        if self.needs_depth {
            self.render_pass
                .set_pipeline(self.pipelines.gradient.pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.gradient.depthless_pipeline());
        }

        self.render_pass.set_bind_group(2, bind_group, &[]);
    }

    pub fn prep_bitmap(&mut self, bind_group: &'pass wgpu::BindGroup, blend_mode: TrivialBlend) {
        if self.needs_depth {
            self.render_pass
                .set_pipeline(self.pipelines.bitmap[blend_mode].pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.bitmap[blend_mode].depthless_pipeline());
        }

        self.render_pass.set_bind_group(2, bind_group, &[]);
    }

    pub fn draw(
        &mut self,
        vertices: wgpu::BufferSlice<'pass>,
        indices: wgpu::BufferSlice<'pass>,
        num_indices: u32,
    ) {
        self.render_pass.set_vertex_buffer(0, vertices);
        self.render_pass
            .set_index_buffer(indices, wgpu::IndexFormat::Uint32);

        self.render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    pub fn apply_transform(
        &mut self,
        matrix: &ruffle_render::matrix::Matrix,
        color_adjustments: ColorAdjustments,
    ) {
        let world_matrix = [
            [matrix.a, matrix.b, 0.0, 0.0],
            [matrix.c, matrix.d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                matrix.tx.to_pixels() as f32,
                matrix.ty.to_pixels() as f32,
                0.0,
                1.0,
            ],
        ];

        self.uniform_buffers.write_uniforms(
            &self.descriptors.device,
            &self.descriptors.bind_layouts.transforms,
            &mut self.uniform_encoder,
            &mut self.render_pass,
            1,
            &Transforms {
                world_matrix,
                color_adjustments,
            },
        );
    }

    fn render_bitmap(
        &mut self,
        bitmap: &'frame BitmapHandle,
        transform: &Transform,
        smoothing: bool,
        blend_mode: TrivialBlend,
    ) {
        let texture = as_texture(bitmap);

        self.apply_transform(
            &(transform.matrix
                * ruffle_render::matrix::Matrix {
                    a: texture.width as f32,
                    d: texture.height as f32,
                    ..Default::default()
                }),
            ColorAdjustments::from(transform.color_transform),
        );
        let descriptors = self.descriptors;
        let bind = texture.bind_group(
            smoothing,
            &descriptors.device,
            &descriptors.bind_layouts.bitmap,
            &descriptors.quad,
            bitmap.clone(),
            &descriptors.bitmap_samplers,
        );

        self.prep_bitmap(&bind.bind_group, blend_mode);

        self.draw(
            self.descriptors.quad.vertices.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
    }

    fn render_texture(
        &mut self,
        transform: &Transform,
        bind_group: &'frame wgpu::BindGroup,
        blend_mode: TrivialBlend,
    ) {
        self.apply_transform(
            &transform.matrix,
            ColorAdjustments::from(transform.color_transform),
        );
        self.prep_bitmap(bind_group, blend_mode);

        self.draw(
            self.descriptors.quad.vertices.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        self.apply_transform(
            &transform.matrix,
            ColorAdjustments::from(transform.color_transform),
        );

        let mesh = &self.meshes[shape.0];
        for draw in &mesh.draws {
            let num_indices = if self.mask_state != MaskState::DrawMaskStencil
                && self.mask_state != MaskState::ClearMaskStencil
            {
                draw.num_indices
            } else {
                // Omit strokes when drawing a mask stencil.
                draw.num_mask_indices
            };
            if num_indices == 0 {
                continue;
            }

            match &draw.draw_type {
                DrawType::Color => {
                    self.prep_color();
                }
                DrawType::Gradient { bind_group, .. } => {
                    self.prep_gradient(bind_group);
                }
                DrawType::Bitmap { binds, .. } => {
                    self.prep_bitmap(&binds.bind_group, TrivialBlend::Normal);
                }
            }

            self.draw(
                draw.vertex_buffer.slice(..),
                draw.index_buffer.slice(..),
                num_indices,
            );
        }
    }

    fn draw_rect(&mut self, color: &Color, matrix: &ruffle_render::matrix::Matrix) {
        self.apply_transform(
            &matrix,
            ColorAdjustments {
                mult_color: [
                    f32::from(color.r) / 255.0,
                    f32::from(color.g) / 255.0,
                    f32::from(color.b) / 255.0,
                    f32::from(color.a) / 255.0,
                ],
                add_color: [0.0, 0.0, 0.0, 0.0],
            },
        );

        self.prep_color();
        self.draw(
            self.descriptors.quad.vertices.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
    }

    fn push_mask(&mut self) {
        debug_assert!(
            self.mask_state == MaskState::NoMask || self.mask_state == MaskState::DrawMaskedContent
        );
        self.num_masks += 1;
        self.mask_state = MaskState::DrawMaskStencil;
        self.render_pass.set_stencil_reference(self.num_masks - 1);
    }

    fn activate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskStencil);
        self.mask_state = MaskState::DrawMaskedContent;
        self.render_pass.set_stencil_reference(self.num_masks);
    }

    fn deactivate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskedContent);
        self.mask_state = MaskState::ClearMaskStencil;
        self.render_pass.set_stencil_reference(self.num_masks);
    }

    fn pop_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::ClearMaskStencil);
        self.num_masks -= 1;
        self.render_pass.set_stencil_reference(self.num_masks);
        if self.num_masks == 0 {
            self.mask_state = MaskState::NoMask;
        } else {
            self.mask_state = MaskState::DrawMaskedContent;
        };
    }
}

pub enum Chunk {
    Draw(Vec<DrawCommand>),
    Blend(PoolEntry<wgpu::Texture>, ComplexBlend),
}

#[derive(Debug)]
pub enum DrawCommand {
    RenderBitmap {
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        blend_mode: TrivialBlend,
    },
    RenderTexture {
        _texture: PoolEntry<wgpu::Texture>,
        binds: wgpu::BindGroup,
        transform: Transform,
        blend_mode: TrivialBlend,
    },
    RenderShape {
        shape: ShapeHandle,
        transform: Transform,
    },
    DrawRect {
        color: Color,
        matrix: Matrix,
    },
    PushMask,
    ActivateMask,
    DeactivateMask,
    PopMask,
}

/// Replaces every blend with a RenderBitmap, with the subcommands rendered out to a temporary texture
/// Every complex blend will be its own item, but every other draw will be chunked together
#[allow(clippy::too_many_arguments)]
fn chunk_blends<'a>(
    commands: &[Command],
    descriptors: &'a Descriptors,
    uniform_buffers: &mut UniformBuffer<'a, Transforms>,
    uniform_encoder: &mut wgpu::CommandEncoder,
    draw_encoder: &mut wgpu::CommandEncoder,
    meshes: &'a Vec<Mesh>,
    sample_count: u32,
    width: u32,
    height: u32,
    nearest_layer: &CommandTarget,
    texture_pool: &mut TexturePool,
) -> (Vec<Chunk>, bool) {
    let mut result = vec![];
    let mut current = vec![];
    let mut needs_depth = false;

    for command in commands {
        match command {
            Command::Blend(commands, blend_mode) => {
                let mut surface = Surface::new(
                    &descriptors,
                    sample_count,
                    width,
                    height,
                    wgpu::TextureFormat::Rgba8Unorm,
                );
                let target = surface.draw_commands(
                    Some(wgpu::Color::TRANSPARENT),
                    &descriptors,
                    &meshes,
                    commands,
                    uniform_buffers,
                    uniform_encoder,
                    draw_encoder,
                    if blend_mode == &BlendMode::Layer {
                        None
                    } else {
                        Some(nearest_layer)
                    },
                    texture_pool,
                );

                match BlendType::from(*blend_mode) {
                    BlendType::Trivial(blend_mode) => {
                        let transform = Transform {
                            matrix: Matrix::scale(target.width() as f32, target.height() as f32),
                            color_transform: Default::default(),
                        };
                        let texture = target.take_color_texture();
                        let binds = BitmapBinds::new(
                            &descriptors.device,
                            &descriptors.bind_layouts.bitmap,
                            descriptors.bitmap_samplers.get_sampler(false, false),
                            &descriptors.quad.texture_transforms,
                            texture.create_view(&Default::default()),
                            None,
                        );
                        current.push(DrawCommand::RenderTexture {
                            _texture: texture,
                            binds: binds.bind_group,
                            transform,
                            blend_mode,
                        })
                    }
                    BlendType::Complex(blend_mode) => {
                        if !current.is_empty() {
                            result.push(Chunk::Draw(std::mem::take(&mut current)));
                        }
                        result.push(Chunk::Blend(target.take_color_texture(), blend_mode));
                    }
                }
            }
            Command::RenderBitmap {
                bitmap,
                transform,
                smoothing,
            } => current.push(DrawCommand::RenderBitmap {
                bitmap: bitmap.to_owned(),
                transform: transform.to_owned(),
                smoothing: *smoothing,
                blend_mode: TrivialBlend::Normal,
            }),
            Command::RenderShape { shape, transform } => current.push(DrawCommand::RenderShape {
                shape: shape.to_owned(),
                transform: transform.to_owned(),
            }),
            Command::DrawRect { color, matrix } => current.push(DrawCommand::DrawRect {
                color: color.to_owned(),
                matrix: matrix.to_owned(),
            }),
            Command::PushMask => {
                needs_depth = true;
                current.push(DrawCommand::PushMask);
            }
            Command::ActivateMask => {
                needs_depth = true;
                current.push(DrawCommand::ActivateMask);
            }
            Command::DeactivateMask => {
                needs_depth = true;
                current.push(DrawCommand::DeactivateMask);
            }
            Command::PopMask => {
                needs_depth = true;
                current.push(DrawCommand::PopMask);
            }
        }
    }

    if !current.is_empty() {
        result.push(Chunk::Draw(current));
    }

    (result, needs_depth)
}
