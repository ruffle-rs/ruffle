use super::target::PoolOrArcTexture;
use crate::backend::RenderTargetMode;
use crate::blend::TrivialBlend;
use crate::buffer_builder::BufferBuilder;
use crate::buffer_pool::TexturePool;
use crate::dynamic_transforms::DynamicTransforms;
use crate::mesh::{DrawType, Mesh, as_mesh};
use crate::surface::Surface;
use crate::surface::target::CommandTarget;
use crate::{Descriptors, MaskState, Pipelines, Transforms, as_texture};
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::{BitmapHandle, PixelSnapping};
use ruffle_render::commands::{CommandHandler, CommandList, RenderBlendMode};
use ruffle_render::lines::{emulate_line, emulate_line_rect};
use ruffle_render::matrix::Matrix;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use std::mem;
use swf::{BlendMode, Color, ColorTransform, Twips};
use wgpu::Backend;
use wgpu_profiler::Scope;

pub struct CommandRenderer<'encoder> {
    pipelines: &'encoder Pipelines,
    descriptors: &'encoder Descriptors,
    num_masks: u32,
    mask_state: MaskState,
    needs_stencil: bool,
    dynamic_transforms: &'encoder DynamicTransforms,
}

impl<'encoder> CommandRenderer<'encoder> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pipelines: &'encoder Pipelines,
        descriptors: &'encoder Descriptors,
        dynamic_transforms: &'encoder DynamicTransforms,
        num_masks: u32,
        mask_state: MaskState,
        needs_stencil: bool,
    ) -> Self {
        Self {
            pipelines,
            num_masks,
            mask_state,
            descriptors,
            needs_stencil,
            dynamic_transforms,
        }
    }

    pub fn execute(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        command: &'encoder DrawCommand,
    ) {
        if self.needs_stencil {
            match self.mask_state {
                MaskState::NoMask => {}
                MaskState::DrawMaskStencil => {
                    render_pass.set_stencil_reference(self.num_masks - 1);
                }
                MaskState::DrawMaskedContent => {
                    render_pass.set_stencil_reference(self.num_masks);
                }
                MaskState::ClearMaskStencil => {
                    render_pass.set_stencil_reference(self.num_masks);
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
                render_pass,
                bitmap,
                *transform_buffer,
                *smoothing,
                *blend_mode,
                *render_stage3d,
            ),
            DrawCommand::RenderShape {
                shape,
                transform_buffer,
            } => self.render_shape(render_pass, shape, *transform_buffer),
            DrawCommand::DrawRect { transform_buffer } => {
                self.draw_rect(render_pass, *transform_buffer)
            }
            DrawCommand::DrawLine { transform_buffer } => {
                self.draw_lines::<false>(render_pass, *transform_buffer)
            }
            DrawCommand::DrawLineRect { transform_buffer } => {
                self.draw_lines::<true>(render_pass, *transform_buffer)
            }
            DrawCommand::PushMask => self.push_mask(render_pass),
            DrawCommand::ActivateMask => self.activate_mask(render_pass),
            DrawCommand::DeactivateMask => self.deactivate_mask(render_pass),
            DrawCommand::PopMask => self.pop_mask(render_pass),
            DrawCommand::RenderAlphaMask {
                maskee,
                mask,
                binds,
                transform_buffer,
            } => self.render_alpha_mask(render_pass, maskee, mask, binds, *transform_buffer),
        }
    }

    pub fn prep_color(&self, render_pass: &mut wgpu::RenderPass<'encoder>) {
        if self.needs_stencil {
            render_pass.set_pipeline(self.pipelines.color.pipeline_for(self.mask_state));
        } else {
            render_pass.set_pipeline(self.pipelines.color.stencilless_pipeline());
        }
    }

    pub fn prep_lines(&self, render_pass: &mut wgpu::RenderPass<'encoder>) {
        if self.needs_stencil {
            render_pass.set_pipeline(self.pipelines.lines.pipeline_for(self.mask_state));
        } else {
            render_pass.set_pipeline(self.pipelines.lines.stencilless_pipeline());
        }
    }

    pub fn prep_gradient(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        bind_group: &'encoder wgpu::BindGroup,
    ) {
        if self.needs_stencil {
            render_pass.set_pipeline(self.pipelines.gradients.pipeline_for(self.mask_state));
        } else {
            render_pass.set_pipeline(self.pipelines.gradients.stencilless_pipeline());
        }

        render_pass.set_bind_group(2, bind_group, &[]);
    }

    pub fn prep_bitmap(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        bind_group: &'encoder wgpu::BindGroup,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
    ) {
        match (self.needs_stencil, render_stage3d) {
            (true, true) => {
                render_pass.set_pipeline(&self.pipelines.bitmap_opaque_dummy_stencil);
            }
            (true, false) => {
                render_pass
                    .set_pipeline(self.pipelines.bitmap[blend_mode].pipeline_for(self.mask_state));
            }
            (false, true) => {
                render_pass.set_pipeline(&self.pipelines.bitmap_opaque);
            }
            (false, false) => {
                render_pass.set_pipeline(self.pipelines.bitmap[blend_mode].stencilless_pipeline());
            }
        }

        render_pass.set_bind_group(2, bind_group, &[]);
    }

    pub fn prep_alpha_mask(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        bind_group: &'encoder wgpu::BindGroup,
    ) {
        if self.needs_stencil {
            render_pass.set_pipeline(self.pipelines.alpha_mask.pipeline_for(self.mask_state));
        } else {
            render_pass.set_pipeline(self.pipelines.alpha_mask.stencilless_pipeline());
        }

        render_pass.set_bind_group(2, bind_group, &[]);
    }

    pub fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        vertices: wgpu::BufferSlice<'encoder>,
        indices: wgpu::BufferSlice<'encoder>,
        num_indices: u32,
    ) {
        render_pass.set_vertex_buffer(0, vertices);
        render_pass.set_index_buffer(indices, wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    pub fn render_bitmap(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        bitmap: &'encoder BitmapHandle,
        transform_buffer: wgpu::DynamicOffset,
        smoothing: bool,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
    ) {
        if cfg!(feature = "render_debug_labels") {
            render_pass.push_debug_group(&format!("render_bitmap {:?}", bitmap.0));
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
        self.prep_bitmap(render_pass, &bind.bind_group, blend_mode, render_stage3d);
        render_pass.set_bind_group(1, &self.dynamic_transforms.bind_group, &[transform_buffer]);

        self.draw(
            render_pass,
            self.descriptors.quad.vertices_pos.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
        if cfg!(feature = "render_debug_labels") {
            render_pass.pop_debug_group();
        }
    }

    pub fn render_texture(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        transform_buffer: wgpu::DynamicOffset,
        bind_group: &'encoder wgpu::BindGroup,
        blend_mode: TrivialBlend,
    ) {
        if cfg!(feature = "render_debug_labels") {
            render_pass.push_debug_group("render_texture");
        }
        self.prep_bitmap(render_pass, bind_group, blend_mode, false);

        render_pass.set_bind_group(1, &self.dynamic_transforms.bind_group, &[transform_buffer]);

        self.draw(
            render_pass,
            self.descriptors.quad.vertices_pos.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
        if cfg!(feature = "render_debug_labels") {
            render_pass.pop_debug_group();
        }
    }

    pub fn render_shape(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        shape: &'encoder ShapeHandle,
        transform_buffer: wgpu::DynamicOffset,
    ) {
        if cfg!(feature = "render_debug_labels") {
            render_pass.push_debug_group("render_shape");
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
                    self.prep_color(render_pass);
                }
                DrawType::Gradient { bind_group, .. } => {
                    self.prep_gradient(render_pass, bind_group);
                }
                DrawType::Bitmap { binds, .. } => {
                    self.prep_bitmap(render_pass, &binds.bind_group, TrivialBlend::Normal, false);
                }
            }
            render_pass.set_bind_group(1, &self.dynamic_transforms.bind_group, &[transform_buffer]);

            self.draw(
                render_pass,
                mesh.vertex_buffer.slice(draw.vertices.clone()),
                mesh.index_buffer.slice(draw.indices.clone()),
                num_indices,
            );
        }
        if cfg!(feature = "render_debug_labels") {
            render_pass.pop_debug_group();
        }
    }

    pub fn render_alpha_mask(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        _maskee: &PoolOrArcTexture,
        _mask: &PoolOrArcTexture,
        bind_group: &'encoder wgpu::BindGroup,
        transform_buffer: wgpu::DynamicOffset,
    ) {
        if cfg!(feature = "render_debug_labels") {
            render_pass.push_debug_group("render_alpha_mask");
        }

        self.prep_alpha_mask(render_pass, bind_group);

        render_pass.set_bind_group(1, &self.dynamic_transforms.bind_group, &[transform_buffer]);

        self.draw(
            render_pass,
            self.descriptors.quad.vertices_pos.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );

        if cfg!(feature = "render_debug_labels") {
            render_pass.pop_debug_group();
        }
    }

    pub fn draw_rect(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        transform_buffer: wgpu::DynamicOffset,
    ) {
        if cfg!(feature = "render_debug_labels") {
            render_pass.push_debug_group("draw_rect");
        }
        self.prep_color(render_pass);

        render_pass.set_bind_group(1, &self.dynamic_transforms.bind_group, &[transform_buffer]);

        self.draw(
            render_pass,
            self.descriptors.quad.vertices_pos_color.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
        );
        if cfg!(feature = "render_debug_labels") {
            render_pass.pop_debug_group();
        }
    }

    pub fn draw_lines<const RECT: bool>(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        transform_buffer: wgpu::DynamicOffset,
    ) {
        if cfg!(feature = "render_debug_labels") {
            render_pass.push_debug_group("draw_lines");
        }
        self.prep_lines(render_pass);

        render_pass.set_bind_group(1, &self.dynamic_transforms.bind_group, &[transform_buffer]);

        self.draw(
            render_pass,
            self.descriptors.quad.vertices_pos_color.slice(..),
            if RECT {
                self.descriptors.quad.indices_line_rect.slice(..)
            } else {
                self.descriptors.quad.indices_line.slice(..)
            },
            if RECT { 5 } else { 2 },
        );
        if cfg!(feature = "render_debug_labels") {
            render_pass.pop_debug_group();
        }
    }

    pub fn push_mask(&mut self, render_pass: &mut wgpu::RenderPass<'encoder>) {
        debug_assert!(
            self.mask_state == MaskState::NoMask || self.mask_state == MaskState::DrawMaskedContent
        );
        self.num_masks += 1;
        self.mask_state = MaskState::DrawMaskStencil;
        render_pass.set_stencil_reference(self.num_masks - 1);
    }

    pub fn activate_mask(&mut self, render_pass: &mut wgpu::RenderPass<'encoder>) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskStencil);
        self.mask_state = MaskState::DrawMaskedContent;
        render_pass.set_stencil_reference(self.num_masks);
    }

    pub fn deactivate_mask(&mut self, render_pass: &mut wgpu::RenderPass<'encoder>) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::DrawMaskedContent);
        self.mask_state = MaskState::ClearMaskStencil;
        render_pass.set_stencil_reference(self.num_masks);
    }

    pub fn pop_mask(&mut self, render_pass: &mut wgpu::RenderPass<'encoder>) {
        debug_assert!(self.num_masks > 0 && self.mask_state == MaskState::ClearMaskStencil);
        self.num_masks -= 1;
        render_pass.set_stencil_reference(self.num_masks);
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
    Draw {
        chunk: Vec<DrawCommand>,
        needs_stencil: bool,
        transforms: BufferBuilder,
    },
    Blend {
        commands: CommandList,
        blend_mode: RenderBlendMode,
        needs_stencil: bool,
    },
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
    RenderAlphaMask {
        maskee: Box<PoolOrArcTexture>,
        mask: Box<PoolOrArcTexture>,
        binds: wgpu::BindGroup,
        transform_buffer: wgpu::DynamicOffset,
    },
    RenderShape {
        shape: ShapeHandle,
        transform_buffer: wgpu::DynamicOffset,
    },
    DrawRect {
        transform_buffer: wgpu::DynamicOffset,
    },
    DrawLine {
        transform_buffer: wgpu::DynamicOffset,
    },
    DrawLineRect {
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

/// Splits the command list at blend boundaries while leaving blend subcommands lazy.
/// Every blend will be its own item, but adjacent ordinary draws are chunked together.
#[expect(clippy::too_many_arguments)]
pub fn chunk_blends<'encoder, 'global: 'encoder>(
    commands: CommandList,
    descriptors: &'encoder Descriptors,
    staging_belt: &'encoder mut wgpu::util::StagingBelt,
    dynamic_transforms: &'encoder DynamicTransforms,
    draw_encoder: &'encoder mut Scope<'global, wgpu::CommandEncoder>,
    meshes: &'encoder Vec<Mesh>,
    quality: StageQuality,
    width: u32,
    height: u32,
    texture_pool: &'encoder mut TexturePool,
) -> Vec<Chunk> {
    WgpuCommandHandler::new(
        descriptors,
        staging_belt,
        dynamic_transforms,
        draw_encoder,
        meshes,
        quality,
        width,
        height,
        texture_pool,
    )
    .chunk_blends(commands)
}

struct WgpuCommandHandler<'encoder, 'global: 'encoder> {
    descriptors: &'encoder Descriptors,
    quality: StageQuality,
    width: u32,
    height: u32,
    meshes: &'encoder Vec<Mesh>,
    staging_belt: &'encoder mut wgpu::util::StagingBelt,
    dynamic_transforms: &'encoder DynamicTransforms,
    draw_encoder: &'encoder mut Scope<'global, wgpu::CommandEncoder>,
    texture_pool: &'encoder mut TexturePool,
    emulate_lines: bool,

    result: Vec<Chunk>,
    current: Vec<DrawCommand>,
    transforms: BufferBuilder,
    needs_stencil: bool,
    num_masks: i32,
}

impl<'encoder, 'global: 'encoder> WgpuCommandHandler<'encoder, 'global> {
    #[expect(clippy::too_many_arguments)]
    fn new(
        descriptors: &'encoder Descriptors,
        staging_belt: &'encoder mut wgpu::util::StagingBelt,
        dynamic_transforms: &'encoder DynamicTransforms,
        draw_encoder: &'encoder mut Scope<'global, wgpu::CommandEncoder>,
        meshes: &'encoder Vec<Mesh>,
        quality: StageQuality,
        width: u32,
        height: u32,
        texture_pool: &'encoder mut TexturePool,
    ) -> Self {
        let transforms = Self::new_transforms(descriptors, dynamic_transforms);

        // DirectX does support drawing lines, but it's very inconsistent.
        // With MSAA, lines have 1.4px thickness, which makes them too thick.
        // Without MSAA, lines have 1px thickness, but their placement is sometimes off.
        let emulate_lines = descriptors.backend == Backend::Dx12;

        Self {
            descriptors,
            quality,
            width,
            height,
            meshes,
            staging_belt,
            dynamic_transforms,
            draw_encoder,
            texture_pool,
            emulate_lines,

            result: vec![],
            current: vec![],
            transforms,
            needs_stencil: false,
            num_masks: 0,
        }
    }

    fn new_transforms(
        descriptors: &'encoder Descriptors,
        dynamic_transforms: &'encoder DynamicTransforms,
    ) -> BufferBuilder {
        let mut transforms = BufferBuilder::new_for_uniform(&descriptors.limits);
        transforms.set_buffer_limit(dynamic_transforms.buffer.size());
        transforms
    }

    /// Splits the command list at blend boundaries while leaving blend subcommands lazy.
    /// Every blend will be its own item, but adjacent ordinary draws are chunked together.
    fn chunk_blends(&mut self, commands: CommandList) -> Vec<Chunk> {
        commands.execute(self);

        let current = mem::take(&mut self.current);
        let mut result = mem::take(&mut self.result);
        let needs_stencil = mem::take(&mut self.needs_stencil);
        let transforms = mem::replace(
            &mut self.transforms,
            Self::new_transforms(self.descriptors, self.dynamic_transforms),
        );

        if !current.is_empty() {
            result.push(Chunk::Draw {
                chunk: current,
                needs_stencil,
                transforms,
            });
        }

        result
    }

    fn add_to_current(
        &mut self,
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
        if let Ok(transform_range) = self.transforms.add(&[transform]) {
            self.current.push(command_builder(
                transform_range.start as wgpu::DynamicOffset,
            ));
        } else {
            self.result.push(Chunk::Draw {
                chunk: mem::take(&mut self.current),
                needs_stencil: self.needs_stencil,
                transforms: mem::replace(
                    &mut self.transforms,
                    BufferBuilder::new_for_uniform(&self.descriptors.limits),
                ),
            });
            self.transforms
                .set_buffer_limit(self.dynamic_transforms.buffer.size());
            let transform_range = self
                .transforms
                .add(&[transform])
                .expect("Buffer must be able to fit a new thing, it was just emptied");
            self.current.push(command_builder(
                transform_range.start as wgpu::DynamicOffset,
            ));
        }
    }
}

impl CommandHandler for WgpuCommandHandler<'_, '_> {
    fn blend(&mut self, commands: CommandList, blend_mode: RenderBlendMode) {
        // A Layer only changes rendering when one of its children blends with
        // the layer's contents. Avoid allocating a full-frame intermediate
        // texture for the common case where it contains only normal draws.
        if matches!(blend_mode, RenderBlendMode::Builtin(BlendMode::Layer))
            && !commands.requires_layer_isolation()
        {
            commands.execute(self);
            return;
        }

        if !self.current.is_empty() {
            self.result.push(Chunk::Draw {
                chunk: mem::take(&mut self.current),
                needs_stencil: self.needs_stencil,
                transforms: mem::replace(
                    &mut self.transforms,
                    BufferBuilder::new_for_uniform(&self.descriptors.limits),
                ),
            });
        }
        self.transforms
            .set_buffer_limit(self.dynamic_transforms.buffer.size());
        self.result.push(Chunk::Blend {
            commands,
            blend_mode,
            needs_stencil: self.num_masks > 0,
        });
        self.needs_stencil = self.num_masks > 0;
    }

    fn render_bitmap(
        &mut self,
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        pixel_snapping: PixelSnapping,
    ) {
        let mut matrix = transform.matrix;
        {
            let texture = as_texture(&bitmap);
            pixel_snapping.apply(&mut matrix);
            matrix *= Matrix::scale(
                texture.texture.width() as f32,
                texture.texture.height() as f32,
            );
        }
        self.add_to_current(matrix, transform.color_transform, |transform_buffer| {
            DrawCommand::RenderBitmap {
                bitmap,
                transform_buffer,
                smoothing,
                blend_mode: TrivialBlend::Normal,
                render_stage3d: false,
            }
        });
    }
    fn render_stage3d(&mut self, bitmap: BitmapHandle, transform: Transform) {
        let mut matrix = transform.matrix;
        {
            let texture = as_texture(&bitmap);
            matrix *= Matrix::scale(
                texture.texture.width() as f32,
                texture.texture.height() as f32,
            );
        }
        self.add_to_current(matrix, transform.color_transform, |transform_buffer| {
            DrawCommand::RenderBitmap {
                bitmap,
                transform_buffer,
                smoothing: false,
                blend_mode: TrivialBlend::Normal,
                render_stage3d: true,
            }
        });
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform) {
        self.add_to_current(
            transform.matrix,
            transform.color_transform,
            |transform_buffer| DrawCommand::RenderShape {
                shape,
                transform_buffer,
            },
        );
    }

    fn draw_rect(&mut self, color: Color, matrix: Matrix) {
        self.add_to_current(
            matrix,
            ColorTransform::multiply_from(color),
            |transform_buffer| DrawCommand::DrawRect { transform_buffer },
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
                matrix,
                ColorTransform::multiply_from(color),
                |transform_buffer| DrawCommand::DrawLine { transform_buffer },
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
                matrix,
                ColorTransform::multiply_from(color),
                |transform_buffer| DrawCommand::DrawLineRect { transform_buffer },
            );
        }
    }

    fn push_mask(&mut self) {
        self.needs_stencil = true;
        self.num_masks += 1;
        self.current.push(DrawCommand::PushMask);
    }

    fn activate_mask(&mut self) {
        self.needs_stencil = true;
        self.current.push(DrawCommand::ActivateMask);
    }

    fn deactivate_mask(&mut self) {
        self.needs_stencil = true;
        self.current.push(DrawCommand::DeactivateMask);
    }

    fn pop_mask(&mut self) {
        self.needs_stencil = true;
        self.num_masks -= 1;
        self.current.push(DrawCommand::PopMask);
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
            self.dynamic_transforms,
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
            self.dynamic_transforms,
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

        self.add_to_current(matrix, Default::default(), |transform_buffer| {
            DrawCommand::RenderAlphaMask {
                maskee: Box::new(maskee),
                mask: Box::new(mask),
                binds,
                transform_buffer,
            }
        });
    }
}
