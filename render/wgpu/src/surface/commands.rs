use crate::backend::RenderTargetMode;
use crate::blend::BlendType;
use crate::blend::TrivialBlend;
use crate::buffer_pool::TexturePool;
use crate::mesh::{DrawType, Mesh, as_mesh};
use crate::surface::Surface;
use crate::surface::buffers::{ChunkBufferBuilder, ChunkBuffers, DrawOffsets, DrawVertices};
use crate::surface::chunk::{Chunk, ChunkBlendMode, DrawCommand};
use crate::surface::mask::{MaskMode, MaskState};
use crate::surface::target::{CommandTarget, PoolOrArcTexture};
use crate::{Descriptors, PosUvVertex, as_texture};
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::{BitmapHandle, PixelRegion, PixelSnapping};
use ruffle_render::commands::{CommandHandler, CommandList, RenderBlendMode};
use ruffle_render::lines::{emulate_line, emulate_line_rect};
use ruffle_render::matrix::Matrix;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use std::mem;
use swf::{BlendMode, Color, ColorTransform, Twips};
use wgpu::Backend;
use wgpu_profiler::Scope;

#[derive(Copy, Clone)]
pub enum LayerRef<'a> {
    None,
    Current,
    Parent(&'a CommandTarget),
}

/// Replaces every blend with a RenderBitmap, with the subcommands rendered out to a temporary texture
/// Every complex blend will be its own item, but every other draw will be chunked together
#[expect(clippy::too_many_arguments)]
pub fn chunk_blends<'encoder, 'global: 'encoder>(
    commands: CommandList,
    descriptors: &'encoder Descriptors,
    staging_belt: &'encoder mut wgpu::util::StagingBelt,
    chunk_buffers: &'encoder mut ChunkBuffers,
    draw_encoder: &'encoder mut Scope<'global, wgpu::CommandEncoder>,
    meshes: &'encoder Vec<Mesh>,
    quality: StageQuality,
    width: u32,
    height: u32,
    nearest_layer: LayerRef,
    texture_pool: &'encoder mut TexturePool,
) -> Vec<Chunk> {
    WgpuCommandHandler::new(
        descriptors,
        staging_belt,
        chunk_buffers,
        draw_encoder,
        meshes,
        quality,
        width,
        height,
        nearest_layer,
        texture_pool,
    )
    .chunk_blends(commands)
}

struct WgpuCommandHandler<'encoder, 'global: 'encoder> {
    descriptors: &'encoder Descriptors,
    quality: StageQuality,
    width: u32,
    height: u32,
    nearest_layer: LayerRef<'encoder>,
    meshes: &'encoder Vec<Mesh>,
    staging_belt: &'encoder mut wgpu::util::StagingBelt,
    chunk_buffers: &'encoder mut ChunkBuffers,
    draw_encoder: &'encoder mut Scope<'global, wgpu::CommandEncoder>,
    texture_pool: &'encoder mut TexturePool,
    emulate_lines: bool,

    result: Vec<Chunk>,
    current: Vec<DrawCommand>,
    chunk_buffer_builder: ChunkBufferBuilder,
    needs_stencil: bool,
    mask_state: MaskState,
}

impl<'encoder, 'global: 'encoder> WgpuCommandHandler<'encoder, 'global> {
    #[expect(clippy::too_many_arguments)]
    fn new(
        descriptors: &'encoder Descriptors,
        staging_belt: &'encoder mut wgpu::util::StagingBelt,
        chunk_buffers: &'encoder mut ChunkBuffers,
        draw_encoder: &'encoder mut Scope<'global, wgpu::CommandEncoder>,
        meshes: &'encoder Vec<Mesh>,
        quality: StageQuality,
        width: u32,
        height: u32,
        nearest_layer: LayerRef<'encoder>,
        texture_pool: &'encoder mut TexturePool,
    ) -> Self {
        let chunk_buffer_builder = ChunkBufferBuilder::new(chunk_buffers, &descriptors.limits);

        // DirectX does support drawing lines, but it's very inconsistent.
        // With MSAA, lines have 1.4px thickness, which makes them too thick.
        // Without MSAA, lines have 1px thickness, but their placement is sometimes off.
        let emulate_lines = descriptors.backend == Backend::Dx12;

        Self {
            descriptors,
            quality,
            width,
            height,
            nearest_layer,
            meshes,
            staging_belt,
            chunk_buffers,
            draw_encoder,
            texture_pool,
            emulate_lines,

            result: vec![],
            current: vec![],
            chunk_buffer_builder,
            needs_stencil: false,
            mask_state: MaskState::empty(),
        }
    }

    /// Replaces every blend with a RenderBitmap, with the subcommands rendered out to a temporary texture
    /// Every complex blend will be its own item, but every other draw will be chunked together
    fn chunk_blends(&mut self, commands: CommandList) -> Vec<Chunk> {
        commands.execute(self);

        let current = mem::take(&mut self.current);
        let mut result = mem::take(&mut self.result);
        let needs_stencil = mem::take(&mut self.needs_stencil);
        let chunk_buffer_builder = mem::replace(
            &mut self.chunk_buffer_builder,
            ChunkBufferBuilder::new(self.chunk_buffers, &self.descriptors.limits),
        );

        if !current.is_empty() {
            result.push(Chunk::Draw {
                chunk: current,
                needs_stencil,
                chunk_buffer_builder,
            });
        }

        result
    }

    fn push_command(&mut self, command: DrawCommand) {
        if let Some(existing) = self.current.last_mut()
            && existing.try_merge(&command)
        {
            return;
        }
        self.current.push(command);
    }

    fn add_to_current(
        &mut self,
        transform: &Transform,
        instance: &mut Option<u32>,
        vertices: DrawVertices,
        command_builder: impl FnOnce(DrawOffsets) -> DrawCommand,
    ) {
        let result = match *instance {
            Some(existing) => Ok(existing),
            None => self.chunk_buffer_builder.add_transform(transform),
        };
        let result = match result {
            Ok(new_instance) => {
                *instance = Some(new_instance);
                self.chunk_buffer_builder
                    .add_draw(&self.descriptors.quad, new_instance, vertices)
            }
            Err(e) => Err((e, vertices)),
        };

        match result {
            Ok(offsets) => {
                self.push_command(command_builder(offsets));
            }
            Err((_, vertices)) => {
                self.result.push(Chunk::Draw {
                    chunk: mem::take(&mut self.current),
                    needs_stencil: self.needs_stencil,
                    chunk_buffer_builder: mem::replace(
                        &mut self.chunk_buffer_builder,
                        ChunkBufferBuilder::new(self.chunk_buffers, &self.descriptors.limits),
                    ),
                });
                // The transform may already have been written to the old builder, but it's not valid
                // in the new one - so we must write it again.
                let new_instance = self
                    .chunk_buffer_builder
                    .add_transform(transform)
                    .expect("Buffer must be able to fit a new transform, it was just emptied");
                *instance = Some(new_instance);
                let offsets = self
                    .chunk_buffer_builder
                    .add_draw(&self.descriptors.quad, new_instance, vertices)
                    .expect("Buffer must be able to fit a new thing, it was just emptied");
                self.push_command(command_builder(offsets));
            }
        }
    }
}

impl CommandHandler for WgpuCommandHandler<'_, '_> {
    fn blend(&mut self, commands: CommandList, blend_mode: RenderBlendMode) {
        let surface = Surface::new(
            self.descriptors,
            self.quality,
            self.width,
            self.height,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let target_layer = if let RenderBlendMode::Builtin(BlendMode::Layer) = &blend_mode {
            LayerRef::Current
        } else {
            self.nearest_layer
        };
        let blend_type = BlendType::from(blend_mode);
        let clear_color = blend_type.default_color();
        let target = surface.draw_commands(
            RenderTargetMode::FreshWithColor(clear_color),
            self.descriptors,
            self.meshes,
            commands,
            self.staging_belt,
            self.chunk_buffers,
            self.draw_encoder,
            target_layer,
            self.texture_pool,
        );
        target.ensure_cleared(self.draw_encoder);

        // We currently do not support shader blends in masks. In order not to
        // break other parts of the scene, we just fall back to a normal blend.
        //
        // TODO Add support for shader blends in masks.
        let is_shader_blend_in_mask =
            self.mask_state.count > 0 && matches!(blend_type, BlendType::Shader(_));
        let blend_type = if is_shader_blend_in_mask {
            BlendType::Trivial(TrivialBlend::Normal)
        } else {
            blend_type
        };

        match blend_type {
            BlendType::Trivial(blend_mode) => {
                let transform = Transform {
                    matrix: Matrix::scale(target.width() as f32, target.height() as f32),
                    tz: 0.0,
                    color_transform: Default::default(),
                    perspective_projection: None,
                };
                let texture = target.take_color_texture();
                let bind_group =
                    self.descriptors
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.descriptors.bind_layouts.bitmap,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(texture.view()),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(
                                        self.descriptors.bitmap_samplers.get_sampler(false, false),
                                    ),
                                },
                            ],
                            label: None,
                        });
                self.add_to_current(
                    &transform,
                    &mut None,
                    DrawVertices::PosUvQuad,
                    |draw_offsets| DrawCommand::RenderTexture {
                        _texture: Some(texture),
                        binds: bind_group,
                        draw_offsets,
                        blend_mode,
                        render_stage3d: false,
                    },
                );
            }
            blend_type => {
                if !self.current.is_empty() {
                    self.result.push(Chunk::Draw {
                        chunk: mem::take(&mut self.current),
                        needs_stencil: self.needs_stencil,
                        chunk_buffer_builder: mem::replace(
                            &mut self.chunk_buffer_builder,
                            ChunkBufferBuilder::new(self.chunk_buffers, &self.descriptors.limits),
                        ),
                    });
                }
                let chunk_blend_mode = match blend_type {
                    BlendType::Complex(complex) => ChunkBlendMode::Complex(complex),
                    BlendType::Shader(shader) => ChunkBlendMode::Shader(shader),
                    _ => unreachable!(),
                };
                self.result.push(Chunk::Blend {
                    texture: target.take_color_texture(),
                    blend_mode: chunk_blend_mode,
                    needs_stencil: self.mask_state.count > 0,
                });
                self.needs_stencil = self.mask_state.count > 0;
            }
        }
    }

    fn render_bitmap(
        &mut self,
        bitmap: BitmapHandle,
        mut transform: Transform,
        smoothing: bool,
        pixel_snapping: PixelSnapping,
        region: PixelRegion,
    ) {
        let texture = as_texture(&bitmap);

        pixel_snapping.apply(&mut transform.matrix);
        transform.matrix *= Matrix::scale(region.width() as f32, region.height() as f32);

        let binds = texture.bind_group(
            false,
            smoothing,
            &self.descriptors.device,
            &self.descriptors.bind_layouts.bitmap,
            bitmap.clone(),
            &self.descriptors.bitmap_samplers,
        );

        let command_builder = |draw_offsets| DrawCommand::RenderTexture {
            _texture: Some(PoolOrArcTexture::Manual((
                texture.texture.clone(),
                texture.view.clone(),
            ))),
            binds: binds.bind_group.clone(),
            draw_offsets,
            blend_mode: TrivialBlend::Normal,
            render_stage3d: false,
        };

        let texture_width = texture.texture.width();
        let texture_height = texture.texture.height();

        if region.x_min == 0
            && region.y_min == 0
            && region.width() == texture_width
            && region.height() == texture_height
        {
            // Special case "full texture" so that we can merge subsequent draw calls
            self.add_to_current(
                &transform,
                &mut None,
                DrawVertices::PosUvQuad,
                command_builder,
            );
        } else {
            let (u0, u1, v0, v1) = (
                region.x_min as f32 / texture_width as f32,
                region.x_max as f32 / texture_width as f32,
                region.y_min as f32 / texture_height as f32,
                region.y_max as f32 / texture_height as f32,
            );
            let vertices = &[
                PosUvVertex::new(0.0, 0.0, u0, v0, 1.0),
                PosUvVertex::new(1.0, 0.0, u1, v0, 1.0),
                PosUvVertex::new(1.0, 1.0, u1, v1, 1.0),
                PosUvVertex::new(0.0, 1.0, u0, v1, 1.0),
            ];
            self.add_to_current(
                &transform,
                &mut None,
                DrawVertices::CustomPosUvQuad(bytemuck::cast_slice(vertices)),
                command_builder,
            );
        };
    }

    fn render_stage3d(&mut self, bitmap: BitmapHandle, mut transform: Transform) {
        {
            let texture = as_texture(&bitmap);
            transform.matrix *= Matrix::scale(
                texture.texture.width() as f32,
                texture.texture.height() as f32,
            );
        }
        let texture = as_texture(&bitmap);

        let binds = texture.bind_group(
            false,
            false,
            &self.descriptors.device,
            &self.descriptors.bind_layouts.bitmap,
            bitmap.clone(),
            &self.descriptors.bitmap_samplers,
        );
        self.add_to_current(
            &transform,
            &mut None,
            DrawVertices::PosUvQuad,
            |draw_offsets| DrawCommand::RenderTexture {
                _texture: Some(PoolOrArcTexture::Manual((
                    texture.texture.clone(),
                    texture.view.clone(),
                ))),
                draw_offsets,
                blend_mode: TrivialBlend::Normal,
                render_stage3d: true,
                binds: binds.bind_group.clone(),
            },
        );
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform) {
        let mesh = as_mesh(&shape);
        let mut instance = None;
        for draw in &mesh.draws {
            let indices = if self.mask_state.mode != MaskMode::DrawMaskStencil
                && self.mask_state.mode != MaskMode::ClearMaskStencil
            {
                draw.indices.clone()
            } else {
                // Omit strokes when drawing a mask stencil.
                draw.mask_indices.clone()
            };
            if indices.is_empty() {
                continue;
            }

            let vertices = DrawVertices::CustomBuffers {
                vertex_buffer: mesh.vertex_buffer.clone(),
                vertex_range: draw.vertices.clone(),
                index_buffer: mesh.index_buffer.clone(),
                indices,
            };
            match &draw.draw_type {
                DrawType::Color => {
                    self.add_to_current(&transform, &mut instance, vertices, |draw_offsets| {
                        DrawCommand::DrawColor { draw_offsets }
                    });
                }
                DrawType::Gradient { bind_group } => {
                    self.add_to_current(&transform, &mut instance, vertices, |draw_offsets| {
                        DrawCommand::RenderGradient {
                            bind_group: bind_group.clone(),
                            draw_offsets,
                        }
                    });
                }
                DrawType::Bitmap { binds } => {
                    self.add_to_current(&transform, &mut instance, vertices, |draw_offsets| {
                        DrawCommand::RenderTexture {
                            _texture: None,
                            binds: binds.bind_group.clone(),
                            blend_mode: TrivialBlend::Normal,
                            render_stage3d: false,
                            draw_offsets,
                        }
                    });
                }
            }
        }
    }

    fn draw_rect(&mut self, color: Color, matrix: Matrix) {
        self.add_to_current(
            &Transform {
                matrix,
                tz: 0.0,
                color_transform: ColorTransform::multiply_from(color),
                perspective_projection: None,
            },
            &mut None,
            DrawVertices::PosColorQuad,
            |draw_offsets| DrawCommand::DrawColor { draw_offsets },
        );
    }

    fn draw_line(&mut self, color: Color, mut matrix: Matrix) {
        if self.emulate_lines {
            let mut cl = CommandList::new();
            emulate_line(&mut cl, color, matrix);
            cl.execute(self);
        } else {
            matrix.tx += Twips::HALF_PX;
            matrix.ty += Twips::HALF_PX;
            self.add_to_current(
                &Transform {
                    matrix,
                    tz: 0.0,
                    color_transform: ColorTransform::multiply_from(color),
                    perspective_projection: None,
                },
                &mut None,
                DrawVertices::Line,
                |draw_offsets| DrawCommand::DrawLine { draw_offsets },
            );
        }
    }

    fn draw_line_rect(&mut self, color: Color, mut matrix: Matrix) {
        if self.emulate_lines {
            let mut cl = CommandList::new();
            emulate_line_rect(&mut cl, color, matrix);
            cl.execute(self);
        } else {
            matrix.tx += Twips::HALF_PX;
            matrix.ty += Twips::HALF_PX;
            self.add_to_current(
                &Transform {
                    matrix,
                    tz: 0.0,
                    color_transform: ColorTransform::multiply_from(color),
                    perspective_projection: None,
                },
                &mut None,
                DrawVertices::RectLine,
                |draw_offsets| DrawCommand::DrawLine { draw_offsets },
            );
        }
    }

    fn push_mask(&mut self) {
        self.needs_stencil = true;
        self.mask_state.push_mask();
        self.push_command(DrawCommand::SetMask(self.mask_state));
    }

    fn activate_mask(&mut self) {
        self.needs_stencil = true;
        self.mask_state.activate_mask();
        self.push_command(DrawCommand::SetMask(self.mask_state));
    }

    fn deactivate_mask(&mut self) {
        self.needs_stencil = true;
        self.mask_state.deactivate_mask();
        self.push_command(DrawCommand::SetMask(self.mask_state));
    }

    fn pop_mask(&mut self) {
        self.needs_stencil = true;
        self.mask_state.pop_mask();
        self.push_command(DrawCommand::SetMask(self.mask_state));
    }

    fn render_alpha_mask(&mut self, maskee_commands: CommandList, mask_commands: CommandList) {
        let surface = Surface::new(
            self.descriptors,
            self.quality,
            self.width,
            self.height,
            wgpu::TextureFormat::Rgba8Unorm,
        );

        let maskee = surface.draw_commands(
            RenderTargetMode::FreshWithColor(wgpu::Color::TRANSPARENT),
            self.descriptors,
            self.meshes,
            maskee_commands,
            self.staging_belt,
            self.chunk_buffers,
            self.draw_encoder,
            LayerRef::None,
            self.texture_pool,
        );
        maskee.ensure_cleared(self.draw_encoder);
        let matrix = Matrix::scale(maskee.width() as f32, maskee.height() as f32);
        let maskee = maskee.take_color_texture();

        let mask = surface.draw_commands(
            RenderTargetMode::FreshWithColor(wgpu::Color::TRANSPARENT),
            self.descriptors,
            self.meshes,
            mask_commands,
            self.staging_belt,
            self.chunk_buffers,
            self.draw_encoder,
            LayerRef::None,
            self.texture_pool,
        );
        mask.ensure_cleared(self.draw_encoder);
        let mask = mask.take_color_texture();

        let binds = self
            .descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.descriptors.bind_layouts.alpha_mask,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(maskee.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(mask.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(
                            self.descriptors.bitmap_samplers.get_sampler(false, false),
                        ),
                    },
                ],
                label: None,
            });

        self.add_to_current(
            &Transform {
                matrix,
                tz: 0.0,
                color_transform: Default::default(),
                perspective_projection: None,
            },
            &mut None,
            DrawVertices::PosQuad,
            |draw_offsets| DrawCommand::RenderAlphaMask {
                maskee,
                mask,
                binds,
                draw_offsets,
            },
        );
    }
}
