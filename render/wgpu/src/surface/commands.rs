use crate::blend::TrivialBlend;
use crate::blend::{BlendType, ComplexBlend};
use crate::buffer_pool::{PoolEntry, TexturePool};
use crate::mesh::{DrawType, Mesh};
use crate::surface::target::CommandTarget;
use crate::surface::Surface;
use crate::{
    as_texture, ColorAdjustments, Descriptors, MaskState, Pipelines, Transforms, UniformBuffer,
};
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::color_transform::ColorTransform;
use ruffle_render::commands::Command;
use ruffle_render::matrix::Matrix;
use ruffle_render::tessellator::GradientType;
use ruffle_render::transform::Transform;
use swf::{BlendMode, Color, Fixed8, GradientSpread};

pub struct CommandRenderer<'pass, 'frame: 'pass, 'global: 'frame> {
    pipelines: &'frame Pipelines,
    meshes: &'global Vec<Mesh>,
    descriptors: &'global Descriptors,
    num_masks: u32,
    mask_state: MaskState,
    render_pass: wgpu::RenderPass<'pass>,
    uniform_buffers: &'frame mut UniformBuffer<'global, Transforms>,
    color_buffers: &'frame mut UniformBuffer<'global, ColorAdjustments>,
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
        color_buffers: &'frame mut UniformBuffer<'global, ColorAdjustments>,
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
            color_buffers,
            uniform_encoder,
            needs_depth,
        }
    }

    pub fn execute(&mut self, command: &'frame DrawCommand) {
        if self.needs_depth {
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
            } => self.render_bitmap(bitmap, &transform, *smoothing, *blend_mode),
            DrawCommand::RenderTexture {
                _texture,
                binds,
                transform,
                blend_mode,
            } => self.render_texture(&transform, binds, *blend_mode),
            DrawCommand::RenderShape { shape, transform } => self.render_shape(*shape, &transform),
            DrawCommand::DrawRect { color, matrix } => self.draw_rect(color, &matrix),
            DrawCommand::PushMask => self.push_mask(),
            DrawCommand::ActivateMask => self.activate_mask(),
            DrawCommand::DeactivateMask => self.deactivate_mask(),
            DrawCommand::PopMask => self.pop_mask(),
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

    pub fn prep_gradient(
        &mut self,
        bind_group: &'pass wgpu::BindGroup,
        mode: GradientType,
        spread: GradientSpread,
    ) {
        if self.needs_depth {
            self.render_pass
                .set_pipeline(self.pipelines.gradients[mode][spread].pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.gradients[mode][spread].depthless_pipeline());
        }

        self.render_pass.set_bind_group(3, bind_group, &[]);
    }

    pub fn prep_bitmap(&mut self, bind_group: &'pass wgpu::BindGroup, blend_mode: TrivialBlend) {
        if self.needs_depth {
            self.render_pass
                .set_pipeline(self.pipelines.bitmap[blend_mode].pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.bitmap[blend_mode].depthless_pipeline());
        }

        self.render_pass.set_bind_group(3, bind_group, &[]);
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

        self.uniform_buffers.write_uniforms(
            &self.descriptors.device,
            &self.descriptors.bind_layouts.transforms,
            &mut self.uniform_encoder,
            &mut self.render_pass,
            1,
            &Transforms { world_matrix },
        );

        if color_adjustments == &ColorTransform::IDENTITY {
            self.render_pass
                .set_bind_group(2, &self.descriptors.default_color_bind_group, &[0]);
        } else {
            self.color_buffers.write_uniforms(
                &self.descriptors.device,
                &self.descriptors.bind_layouts.color_transforms,
                &mut self.uniform_encoder,
                &mut self.render_pass,
                2,
                &ColorAdjustments::from(*color_adjustments),
            );
        }
    }

    pub fn render_bitmap(
        &mut self,
        bitmap: &'frame BitmapHandle,
        transform: &Transform,
        smoothing: bool,
        blend_mode: TrivialBlend,
    ) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass
                .push_debug_group(&format!("render_bitmap {:?}", bitmap.0));
        }
        let texture = as_texture(bitmap);

        self.apply_transform(
            &(transform.matrix
                * Matrix {
                    a: texture.width as f32,
                    d: texture.height as f32,
                    ..Default::default()
                }),
            &transform.color_transform,
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
        self.apply_transform(&transform.matrix, &transform.color_transform);
        self.prep_bitmap(bind_group, blend_mode);

        self.draw(
            self.descriptors.quad.vertices.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.pop_debug_group();
        }
    }

    pub fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass
                .push_debug_group(&format!("render_shape {}", shape.0));
        }
        self.apply_transform(&transform.matrix, &transform.color_transform);

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
                DrawType::Gradient {
                    bind_group,
                    spread,
                    mode,
                    ..
                } => {
                    self.prep_gradient(bind_group, *mode, *spread);
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
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.pop_debug_group();
        }
    }

    pub fn draw_rect(&mut self, color: &Color, matrix: &Matrix) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.push_debug_group("draw_rect");
        }

        if color == &Color::WHITE {
            self.apply_transform(&matrix, &ColorTransform::IDENTITY);
        } else {
            self.apply_transform(
                &matrix,
                &ColorTransform {
                    r_mult: Fixed8::from_f32(f32::from(color.r) / 255.0),
                    g_mult: Fixed8::from_f32(f32::from(color.g) / 255.0),
                    b_mult: Fixed8::from_f32(f32::from(color.b) / 255.0),
                    a_mult: Fixed8::from_f32(f32::from(color.a) / 255.0),
                    ..Default::default()
                },
            );
        }

        self.prep_color();
        self.draw(
            self.descriptors.quad.vertices.slice(..),
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
    Blend(
        PoolEntry<(wgpu::Texture, wgpu::TextureView)>,
        ComplexBlend,
        bool,
    ),
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
        _texture: PoolEntry<(wgpu::Texture, wgpu::TextureView)>,
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
pub fn chunk_blends<'a>(
    commands: Vec<Command>,
    descriptors: &'a Descriptors,
    uniform_buffers: &mut UniformBuffer<'a, Transforms>,
    color_buffers: &mut UniformBuffer<'a, ColorAdjustments>,
    uniform_encoder: &mut wgpu::CommandEncoder,
    draw_encoder: &mut wgpu::CommandEncoder,
    meshes: &'a Vec<Mesh>,
    sample_count: u32,
    width: u32,
    height: u32,
    nearest_layer: &CommandTarget,
    texture_pool: &mut TexturePool,
) -> Vec<Chunk> {
    let mut result = vec![];
    let mut current = vec![];
    let mut needs_depth = false;
    let mut num_masks = 0;

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
                let clear_color = BlendType::from(blend_mode).default_color();
                let target = surface.draw_commands(
                    clear_color,
                    &descriptors,
                    &meshes,
                    commands,
                    uniform_buffers,
                    color_buffers,
                    uniform_encoder,
                    draw_encoder,
                    if blend_mode == BlendMode::Layer {
                        None
                    } else {
                        Some(nearest_layer)
                    },
                    texture_pool,
                );
                target.ensure_cleared(draw_encoder, clear_color);

                match BlendType::from(blend_mode) {
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
                                    layout: &&descriptors.bind_layouts.bitmap,
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
                                                &texture.1,
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
                    BlendType::Complex(blend_mode) => {
                        if !current.is_empty() {
                            result.push(Chunk::Draw(std::mem::take(&mut current), needs_depth));
                        }
                        result.push(Chunk::Blend(
                            target.take_color_texture(),
                            blend_mode,
                            num_masks > 0,
                        ));
                        needs_depth = num_masks > 0;
                    }
                }
            }
            Command::RenderBitmap {
                bitmap,
                transform,
                smoothing,
            } => current.push(DrawCommand::RenderBitmap {
                bitmap,
                transform,
                smoothing,
                blend_mode: TrivialBlend::Normal,
            }),
            Command::RenderShape { shape, transform } => {
                current.push(DrawCommand::RenderShape { shape, transform })
            }
            Command::DrawRect { color, matrix } => {
                current.push(DrawCommand::DrawRect { color, matrix })
            }
            Command::PushMask => {
                needs_depth = true;
                num_masks += 1;
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
                num_masks -= 1;
                current.push(DrawCommand::PopMask);
            }
        }
    }

    if !current.is_empty() {
        result.push(Chunk::Draw(current, needs_depth));
    }

    result
}
