use crate::backend::RenderTargetMode;
use crate::blend::TrivialBlend;
use crate::blend::{BlendType, ComplexBlend};
use crate::buffer_pool::TexturePool;
use crate::mesh::{as_mesh, DrawType, Mesh};
use crate::surface::target::CommandTarget;
use crate::surface::Surface;
use crate::{
    as_texture, ColorAdjustments, Descriptors, MaskState, Pipelines, PushConstants, Transforms,
    UniformBuffer,
};
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::{BitmapHandle, PixelSnapping};
use ruffle_render::commands::{Command, RenderBlendMode};
use ruffle_render::matrix::Matrix;
use ruffle_render::pixel_bender::PixelBenderShaderHandle;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use swf::{BlendMode, Color, ColorTransform, Fixed8};

use super::target::PoolOrArcTexture;

pub struct CommandRenderer<'pass, 'frame: 'pass, 'global: 'frame> {
    pipelines: &'frame Pipelines,
    descriptors: &'global Descriptors,
    num_masks: u32,
    mask_state: MaskState,
    render_pass: wgpu::RenderPass<'pass>,
    uniform_buffers: &'frame mut UniformBuffer<'global, Transforms>,
    color_buffers: &'frame mut UniformBuffer<'global, ColorAdjustments>,
    uniform_encoder: &'frame mut wgpu::CommandEncoder,
    needs_stencil: bool,
}

impl<'pass, 'frame: 'pass, 'global: 'frame> CommandRenderer<'pass, 'frame, 'global> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pipelines: &'frame Pipelines,
        descriptors: &'global Descriptors,
        uniform_buffers: &'frame mut UniformBuffer<'global, Transforms>,
        color_buffers: &'frame mut UniformBuffer<'global, ColorAdjustments>,
        uniform_encoder: &'frame mut wgpu::CommandEncoder,
        render_pass: wgpu::RenderPass<'pass>,
        num_masks: u32,
        mask_state: MaskState,
        needs_stencil: bool,
    ) -> Self {
        Self {
            pipelines,
            num_masks,
            mask_state,
            render_pass,
            descriptors,
            uniform_buffers,
            color_buffers,
            uniform_encoder,
            needs_stencil,
        }
    }

    pub fn execute(&mut self, command: &'frame DrawCommand) {
        if self.needs_stencil {
            match self.mask_state {
                MaskState::NoMask => {}
                MaskState::DrawMaskStencil => {
                    self.render_pass.set_stencil_reference(self.num_masks - 1);
                }
                MaskState::DrawMaskedContent => {
                    self.render_pass.set_stencil_reference(self.num_masks);
                }
                MaskState::ClearMaskStencil => {
                    self.render_pass.set_stencil_reference(self.num_masks);
                }
            }
        }

        match command {
            DrawCommand::RenderBitmap {
                bitmap,
                transform,
                smoothing,
                blend_mode,
                render_stage3d,
                pixel_snapping,
            } => self.render_bitmap(
                bitmap,
                transform,
                *smoothing,
                *blend_mode,
                *render_stage3d,
                *pixel_snapping,
            ),
            DrawCommand::RenderTexture {
                _texture,
                binds,
                transform,
                blend_mode,
            } => self.render_texture(transform, binds, *blend_mode),
            DrawCommand::RenderShape { shape, transform } => self.render_shape(shape, transform),
            DrawCommand::DrawRect { color, matrix } => self.draw_rect(color, matrix),
            DrawCommand::PushMask => self.push_mask(),
            DrawCommand::ActivateMask => self.activate_mask(),
            DrawCommand::DeactivateMask => self.deactivate_mask(),
            DrawCommand::PopMask => self.pop_mask(),
        }
    }

    pub fn prep_color(&mut self) {
        if self.needs_stencil {
            self.render_pass
                .set_pipeline(self.pipelines.color.pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.color.stencilless_pipeline());
        }
    }

    pub fn prep_gradient(&mut self, bind_group: &'pass wgpu::BindGroup) {
        if self.needs_stencil {
            self.render_pass
                .set_pipeline(self.pipelines.gradients.pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.gradients.stencilless_pipeline());
        }

        self.render_pass.set_bind_group(
            if self.descriptors.limits.max_push_constant_size > 0 {
                1
            } else {
                3
            },
            bind_group,
            &[],
        );
    }

    pub fn prep_bitmap(
        &mut self,
        bind_group: &'pass wgpu::BindGroup,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
    ) {
        match (self.needs_stencil, render_stage3d) {
            (true, true) => {
                self.render_pass
                    .set_pipeline(&self.pipelines.bitmap_opaque_dummy_stencil);
            }
            (true, false) => {
                self.render_pass
                    .set_pipeline(self.pipelines.bitmap[blend_mode].pipeline_for(self.mask_state));
            }
            (false, true) => {
                self.render_pass.set_pipeline(&self.pipelines.bitmap_opaque);
            }
            (false, false) => {
                self.render_pass
                    .set_pipeline(self.pipelines.bitmap[blend_mode].stencilless_pipeline());
            }
        }

        self.render_pass.set_bind_group(
            if self.descriptors.limits.max_push_constant_size > 0 {
                1
            } else {
                3
            },
            bind_group,
            &[],
        );
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

    pub fn apply_transform(&mut self, matrix: &Matrix, color_adjustments: &ColorTransform) {
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

        if self.descriptors.limits.max_push_constant_size > 0 {
            self.render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::cast_slice(&[PushConstants {
                    transforms: Transforms { world_matrix },
                    colors: color_adjustments.into(),
                }]),
            );
        } else {
            self.uniform_buffers.write_uniforms(
                &self.descriptors.device,
                &self.descriptors.bind_layouts.transforms,
                self.uniform_encoder,
                &mut self.render_pass,
                1,
                &Transforms { world_matrix },
            );

            if color_adjustments == &ColorTransform::IDENTITY {
                self.render_pass.set_bind_group(
                    2,
                    &self.descriptors.default_color_bind_group,
                    &[0],
                );
            } else {
                self.color_buffers.write_uniforms(
                    &self.descriptors.device,
                    &self.descriptors.bind_layouts.color_transforms,
                    self.uniform_encoder,
                    &mut self.render_pass,
                    2,
                    &color_adjustments.into(),
                );
            }
        }
    }

    pub fn render_bitmap(
        &mut self,
        bitmap: &'frame BitmapHandle,
        transform: &Transform,
        smoothing: bool,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
        pixel_snapping: PixelSnapping,
    ) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass
                .push_debug_group(&format!("render_bitmap {:?}", bitmap.0));
        }
        let texture = as_texture(bitmap);

        let descriptors = self.descriptors;
        let bind = texture.bind_group(
            smoothing,
            &descriptors.device,
            &descriptors.bind_layouts.bitmap,
            &descriptors.quad,
            bitmap.clone(),
            &descriptors.bitmap_samplers,
        );
        self.prep_bitmap(&bind.bind_group, blend_mode, render_stage3d);
        let mut matrix = transform.matrix;
        pixel_snapping.apply(&mut matrix);
        matrix *= Matrix::scale(
            texture.texture.width() as f32,
            texture.texture.height() as f32,
        );
        self.apply_transform(&matrix, &transform.color_transform);

        self.draw(
            self.descriptors.quad.vertices_pos.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.pop_debug_group();
        }
    }

    pub fn render_texture(
        &mut self,
        transform: &Transform,
        bind_group: &'frame wgpu::BindGroup,
        blend_mode: TrivialBlend,
    ) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.push_debug_group("render_texture");
        }
        self.prep_bitmap(bind_group, blend_mode, false);
        self.apply_transform(&transform.matrix, &transform.color_transform);

        self.draw(
            self.descriptors.quad.vertices_pos.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.pop_debug_group();
        }
    }

    pub fn render_shape(&mut self, shape: &'frame ShapeHandle, transform: &Transform) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.push_debug_group("render_shape");
        }

        let mesh = as_mesh(shape);
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
                    self.prep_bitmap(&binds.bind_group, TrivialBlend::Normal, false);
                }
            }
            self.apply_transform(&transform.matrix, &transform.color_transform);

            self.draw(
                mesh.vertex_buffer.slice(draw.vertices.clone()),
                mesh.index_buffer.slice(draw.indices.clone()),
                num_indices,
            );
        }
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.pop_debug_group();
        }
    }

    pub fn draw_rect(&mut self, color: &Color, matrix: &Matrix) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.push_debug_group("draw_rect");
        }
        self.prep_color();

        if color == &Color::WHITE {
            self.apply_transform(matrix, &ColorTransform::IDENTITY);
        } else {
            self.apply_transform(
                matrix,
                &ColorTransform {
                    r_multiply: Fixed8::from_f32(f32::from(color.r) / 255.0),
                    g_multiply: Fixed8::from_f32(f32::from(color.g) / 255.0),
                    b_multiply: Fixed8::from_f32(f32::from(color.b) / 255.0),
                    a_multiply: Fixed8::from_f32(f32::from(color.a) / 255.0),
                    ..Default::default()
                },
            );
        }

        self.draw(
            self.descriptors.quad.vertices_pos_color.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.pop_debug_group();
        }
    }

    pub fn push_mask(&mut self) {
        debug_assert!(
            self.mask_state == MaskState::NoMask || self.mask_state == MaskState::DrawMaskedContent
        );
        self.num_masks += 1;
        self.mask_state = MaskState::DrawMaskStencil;
        self.render_pass.set_stencil_reference(self.num_masks - 1);
    }

    pub fn activate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskStencil);
        self.mask_state = MaskState::DrawMaskedContent;
        self.render_pass.set_stencil_reference(self.num_masks);
    }

    pub fn deactivate_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskedContent);
        self.mask_state = MaskState::ClearMaskStencil;
        self.render_pass.set_stencil_reference(self.num_masks);
    }

    pub fn pop_mask(&mut self) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::ClearMaskStencil);
        self.num_masks -= 1;
        self.render_pass.set_stencil_reference(self.num_masks);
        if self.num_masks == 0 {
            self.mask_state = MaskState::NoMask;
        } else {
            self.mask_state = MaskState::DrawMaskedContent;
        };
    }

    pub fn num_masks(&self) -> u32 {
        self.num_masks
    }

    pub fn mask_state(&self) -> MaskState {
        self.mask_state
    }
}

pub enum Chunk {
    Draw(Vec<DrawCommand>, bool),
    Blend(PoolOrArcTexture, ChunkBlendMode, bool),
}

#[derive(Debug)]
pub enum ChunkBlendMode {
    Complex(ComplexBlend),
    Shader(PixelBenderShaderHandle),
}

#[derive(Debug)]
pub enum DrawCommand {
    RenderBitmap {
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
        pixel_snapping: PixelSnapping,
    },
    RenderTexture {
        _texture: PoolOrArcTexture,
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

#[derive(Copy, Clone)]
pub enum LayerRef<'a> {
    None,
    Current,
    Parent(&'a CommandTarget),
}

/// Replaces every blend with a RenderBitmap, with the subcommands rendered out to a temporary texture
/// Every complex blend will be its own item, but every other draw will be chunked together
#[allow(clippy::too_many_arguments)]
pub fn chunk_blends<'a>(
    commands: Vec<Command>,
    descriptors: &'a Descriptors,
    uniform_buffers: &mut UniformBuffer<'a, Transforms>,
    color_buffers: &mut UniformBuffer<'a, ColorAdjustments>,
    uniform_encoder: &mut wgpu::CommandEncoder,
    draw_encoder: &mut wgpu::CommandEncoder,
    meshes: &'a Vec<Mesh>,
    quality: StageQuality,
    width: u32,
    height: u32,
    nearest_layer: LayerRef,
    texture_pool: &mut TexturePool,
) -> Vec<Chunk> {
    let mut result = vec![];
    let mut current = vec![];
    let mut needs_stencil = false;
    let mut num_masks = 0;

    for command in commands {
        match command {
            Command::Blend(commands, blend_mode) => {
                let mut surface = Surface::new(
                    descriptors,
                    quality,
                    width,
                    height,
                    wgpu::TextureFormat::Rgba8Unorm,
                );
                let target_layer = if let RenderBlendMode::Builtin(BlendMode::Layer) = &blend_mode {
                    LayerRef::Current
                } else {
                    nearest_layer
                };
                let blend_type = BlendType::from(blend_mode);
                let clear_color = blend_type.default_color();
                let target = surface.draw_commands(
                    RenderTargetMode::FreshWithColor(clear_color),
                    descriptors,
                    meshes,
                    commands,
                    uniform_buffers,
                    color_buffers,
                    uniform_encoder,
                    draw_encoder,
                    target_layer,
                    texture_pool,
                );
                target.ensure_cleared(draw_encoder);

                match blend_type {
                    BlendType::Trivial(blend_mode) => {
                        let transform = Transform {
                            matrix: Matrix::scale(target.width() as f32, target.height() as f32),
                            color_transform: Default::default(),
                        };
                        let texture = target.take_color_texture();
                        let bind_group =
                            descriptors
                                .device
                                .create_bind_group(&wgpu::BindGroupDescriptor {
                                    layout: &descriptors.bind_layouts.bitmap,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: descriptors
                                                .quad
                                                .texture_transforms
                                                .as_entire_binding(),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 1,
                                            resource: wgpu::BindingResource::TextureView(
                                                texture.view(),
                                            ),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 2,
                                            resource: wgpu::BindingResource::Sampler(
                                                descriptors
                                                    .bitmap_samplers
                                                    .get_sampler(false, false),
                                            ),
                                        },
                                    ],
                                    label: None,
                                });
                        current.push(DrawCommand::RenderTexture {
                            _texture: texture,
                            binds: bind_group,
                            transform,
                            blend_mode,
                        })
                    }
                    blend_type => {
                        if !current.is_empty() {
                            result.push(Chunk::Draw(std::mem::take(&mut current), needs_stencil));
                        }
                        let chunk_blend_mode = match blend_type {
                            BlendType::Complex(complex) => ChunkBlendMode::Complex(complex),
                            BlendType::Shader(shader) => ChunkBlendMode::Shader(shader),
                            _ => unreachable!(),
                        };
                        result.push(Chunk::Blend(
                            target.take_color_texture(),
                            chunk_blend_mode,
                            num_masks > 0,
                        ));
                        needs_stencil = num_masks > 0;
                    }
                }
            }
            Command::RenderBitmap {
                bitmap,
                transform,
                smoothing,
                pixel_snapping,
            } => current.push(DrawCommand::RenderBitmap {
                bitmap,
                transform,
                smoothing,
                blend_mode: TrivialBlend::Normal,
                render_stage3d: false,
                pixel_snapping,
            }),
            Command::RenderStage3D { bitmap, transform } => {
                current.push(DrawCommand::RenderBitmap {
                    bitmap,
                    transform,
                    smoothing: false,
                    blend_mode: TrivialBlend::Normal,
                    render_stage3d: true,
                    pixel_snapping: PixelSnapping::Never,
                })
            }
            Command::RenderShape { shape, transform } => {
                current.push(DrawCommand::RenderShape { shape, transform })
            }
            Command::DrawRect { color, matrix } => {
                current.push(DrawCommand::DrawRect { color, matrix })
            }
            Command::PushMask => {
                needs_stencil = true;
                num_masks += 1;
                current.push(DrawCommand::PushMask);
            }
            Command::ActivateMask => {
                needs_stencil = true;
                current.push(DrawCommand::ActivateMask);
            }
            Command::DeactivateMask => {
                needs_stencil = true;
                current.push(DrawCommand::DeactivateMask);
            }
            Command::PopMask => {
                needs_stencil = true;
                num_masks -= 1;
                current.push(DrawCommand::PopMask);
            }
        }
    }

    if !current.is_empty() {
        result.push(Chunk::Draw(current, needs_stencil));
    }

    result
}
