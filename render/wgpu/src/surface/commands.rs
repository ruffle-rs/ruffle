use crate::backend::RenderTargetMode;
use crate::blend::TrivialBlend;
use crate::blend::{BlendType, ComplexBlend};
use crate::buffer_builder::BufferBuilder;
use crate::buffer_pool::TexturePool;
use crate::dynamic_transforms::DynamicTransforms;
use crate::mesh::{as_mesh, DrawType, Mesh};
use crate::surface::target::CommandTarget;
use crate::surface::Surface;
use crate::{as_texture, Descriptors, MaskState, Pipelines, Transforms};
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::commands::{Command, RenderBlendMode};
use ruffle_render::matrix::Matrix;
use ruffle_render::pixel_bender::PixelBenderShaderHandle;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use swf::{BlendMode, ColorTransform, Fixed8};

use super::target::PoolOrArcTexture;

pub struct CommandRenderer<'pass, 'frame: 'pass, 'global: 'frame> {
    pipelines: &'frame Pipelines,
    descriptors: &'global Descriptors,
    num_masks: u32,
    mask_state: MaskState,
    render_pass: wgpu::RenderPass<'pass>,
    needs_stencil: bool,
    dynamic_transforms: &'global DynamicTransforms,
}

impl<'pass, 'frame: 'pass, 'global: 'frame> CommandRenderer<'pass, 'frame, 'global> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pipelines: &'frame Pipelines,
        descriptors: &'global Descriptors,
        dynamic_transforms: &'global DynamicTransforms,
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
            needs_stencil,
            dynamic_transforms,
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
                transform_buffer,
                smoothing,
                blend_mode,
                render_stage3d,
            } => self.render_bitmap(
                bitmap,
                *transform_buffer,
                *smoothing,
                *blend_mode,
                *render_stage3d,
            ),
            DrawCommand::RenderTexture {
                _texture,
                binds,
                transform_buffer,
                blend_mode,
            } => self.render_texture(*transform_buffer, binds, *blend_mode),
            DrawCommand::RenderShape {
                shape,
                transform_buffer,
            } => self.render_shape(shape, *transform_buffer),
            DrawCommand::DrawRect { transform_buffer } => self.draw_rect(*transform_buffer),
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

        self.render_pass.set_bind_group(2, bind_group, &[]);
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

    pub fn render_bitmap(
        &mut self,
        bitmap: &'frame BitmapHandle,
        transform_buffer: wgpu::DynamicOffset,
        smoothing: bool,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
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
        self.render_pass.set_bind_group(
            1,
            &self.dynamic_transforms.bind_group,
            &[transform_buffer],
        );

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
        transform_buffer: wgpu::DynamicOffset,
        bind_group: &'frame wgpu::BindGroup,
        blend_mode: TrivialBlend,
    ) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.push_debug_group("render_texture");
        }
        self.prep_bitmap(bind_group, blend_mode, false);

        self.render_pass.set_bind_group(
            1,
            &self.dynamic_transforms.bind_group,
            &[transform_buffer],
        );

        self.draw(
            self.descriptors.quad.vertices_pos.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.pop_debug_group();
        }
    }

    pub fn render_shape(
        &mut self,
        shape: &'frame ShapeHandle,
        transform_buffer: wgpu::DynamicOffset,
    ) {
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
            self.render_pass.set_bind_group(
                1,
                &self.dynamic_transforms.bind_group,
                &[transform_buffer],
            );

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

    pub fn draw_rect(&mut self, transform_buffer: wgpu::DynamicOffset) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.push_debug_group("draw_rect");
        }
        self.prep_color();

        self.render_pass.set_bind_group(
            1,
            &self.dynamic_transforms.bind_group,
            &[transform_buffer],
        );

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
    Draw(Vec<DrawCommand>, bool, BufferBuilder),
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
        transform_buffer: wgpu::DynamicOffset,
        smoothing: bool,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
    },
    RenderTexture {
        _texture: PoolOrArcTexture,
        binds: wgpu::BindGroup,
        transform_buffer: wgpu::DynamicOffset,
        blend_mode: TrivialBlend,
    },
    RenderShape {
        shape: ShapeHandle,
        transform_buffer: wgpu::DynamicOffset,
    },
    DrawRect {
        transform_buffer: wgpu::DynamicOffset,
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
    staging_belt: &'a mut wgpu::util::StagingBelt,
    dynamic_transforms: &'a DynamicTransforms,
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
    let mut transforms = BufferBuilder::new_for_uniform(&descriptors.limits);

    transforms.set_buffer_limit(dynamic_transforms.buffer.size());

    fn add_to_current(
        result: &mut Vec<Chunk>,
        current: &mut Vec<DrawCommand>,
        transforms: &mut BufferBuilder,
        dynamic_transforms: &DynamicTransforms,
        needs_stencil: bool,
        descriptors: &Descriptors,
        matrix: Matrix,
        color_transform: ColorTransform,
        command_builder: impl FnOnce(wgpu::DynamicOffset) -> DrawCommand,
    ) {
        let transform = Transforms {
            world_matrix: [
                [matrix.a, matrix.b, 0.0, 0.0],
                [matrix.c, matrix.d, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [
                    matrix.tx.to_pixels() as f32,
                    matrix.ty.to_pixels() as f32,
                    0.0,
                    1.0,
                ],
            ],
            mult_color: color_transform.mult_rgba_normalized(),
            add_color: color_transform.add_rgba_normalized(),
        };
        if let Ok(transform_range) = transforms.add(&[transform]) {
            current.push(command_builder(
                transform_range.start as wgpu::DynamicOffset,
            ));
        } else {
            result.push(Chunk::Draw(
                std::mem::take(current),
                needs_stencil,
                std::mem::replace(
                    transforms,
                    BufferBuilder::new_for_uniform(&descriptors.limits),
                ),
            ));
            transforms.set_buffer_limit(dynamic_transforms.buffer.size());
            let transform_range = transforms
                .add(&[transform])
                .expect("Buffer must be able to fit a new thing, it was just emptied");
            current.push(command_builder(
                transform_range.start as wgpu::DynamicOffset,
            ));
        }
    }

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
                    staging_belt,
                    dynamic_transforms,
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
                        add_to_current(
                            &mut result,
                            &mut current,
                            &mut transforms,
                            dynamic_transforms,
                            needs_stencil,
                            descriptors,
                            transform.matrix,
                            transform.color_transform,
                            |transform_buffer| DrawCommand::RenderTexture {
                                _texture: texture,
                                binds: bind_group,
                                transform_buffer,
                                blend_mode,
                            },
                        );
                    }
                    blend_type => {
                        if !current.is_empty() {
                            result.push(Chunk::Draw(
                                std::mem::take(&mut current),
                                needs_stencil,
                                std::mem::replace(
                                    &mut transforms,
                                    BufferBuilder::new_for_uniform(&descriptors.limits),
                                ),
                            ));
                        }
                        transforms.set_buffer_limit(dynamic_transforms.buffer.size());
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
            } => {
                let mut matrix = transform.matrix;
                {
                    let texture = as_texture(&bitmap);
                    pixel_snapping.apply(&mut matrix);
                    matrix *= Matrix::scale(
                        texture.texture.width() as f32,
                        texture.texture.height() as f32,
                    );
                }
                add_to_current(
                    &mut result,
                    &mut current,
                    &mut transforms,
                    dynamic_transforms,
                    needs_stencil,
                    descriptors,
                    matrix,
                    transform.color_transform,
                    |transform_buffer| DrawCommand::RenderBitmap {
                        bitmap,
                        transform_buffer,
                        smoothing,
                        blend_mode: TrivialBlend::Normal,
                        render_stage3d: false,
                    },
                );
            }
            Command::RenderStage3D { bitmap, transform } => {
                let mut matrix = transform.matrix;
                {
                    let texture = as_texture(&bitmap);
                    matrix *= Matrix::scale(
                        texture.texture.width() as f32,
                        texture.texture.height() as f32,
                    );
                }
                add_to_current(
                    &mut result,
                    &mut current,
                    &mut transforms,
                    dynamic_transforms,
                    needs_stencil,
                    descriptors,
                    matrix,
                    transform.color_transform,
                    |transform_buffer| DrawCommand::RenderBitmap {
                        bitmap,
                        transform_buffer,
                        smoothing: false,
                        blend_mode: TrivialBlend::Normal,
                        render_stage3d: true,
                    },
                );
            }
            Command::RenderShape { shape, transform } => add_to_current(
                &mut result,
                &mut current,
                &mut transforms,
                dynamic_transforms,
                needs_stencil,
                descriptors,
                transform.matrix,
                transform.color_transform,
                |transform_buffer| DrawCommand::RenderShape {
                    shape,
                    transform_buffer,
                },
            ),
            Command::DrawRect { color, matrix } => add_to_current(
                &mut result,
                &mut current,
                &mut transforms,
                dynamic_transforms,
                needs_stencil,
                descriptors,
                matrix,
                ColorTransform {
                    r_multiply: Fixed8::from_f32(f32::from(color.r) / 255.0),
                    g_multiply: Fixed8::from_f32(f32::from(color.g) / 255.0),
                    b_multiply: Fixed8::from_f32(f32::from(color.b) / 255.0),
                    a_multiply: Fixed8::from_f32(f32::from(color.a) / 255.0),
                    ..Default::default()
                },
                |transform_buffer| DrawCommand::DrawRect { transform_buffer },
            ),
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
        result.push(Chunk::Draw(current, needs_stencil, transforms));
    }

    result
}
