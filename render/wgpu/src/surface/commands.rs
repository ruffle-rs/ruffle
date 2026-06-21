use crate::backend::RenderTargetMode;
use crate::blend::TrivialBlend;
use crate::blend::{BlendType, ComplexBlend};
use crate::buffer_builder::BufferBuilder;
use crate::buffer_pool::TexturePool;
use crate::dynamic_transforms::DynamicTransforms;
use crate::mesh::{DrawType, Mesh, as_mesh};
use crate::surface::Surface;
use crate::surface::target::CommandTarget;
use crate::{BlendTransforms, Descriptors, MaskState, Pipelines, Transforms, as_texture};
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::{BitmapHandle, PixelSnapping};
use ruffle_render::commands::{Command, CommandHandler, CommandList, RenderBlendMode};
use ruffle_render::lines::{emulate_line, emulate_line_rect};
use ruffle_render::matrix::Matrix;
use ruffle_render::pixel_bender::PixelBenderShaderHandle;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use std::mem;
use swf::{BlendMode, Color, ColorTransform, Twips};
use wgpu::Backend;

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
            DrawCommand::DrawLine { transform_buffer } => {
                self.draw_lines::<false>(*transform_buffer)
            }
            DrawCommand::DrawLineRect { transform_buffer } => {
                self.draw_lines::<true>(*transform_buffer)
            }
            DrawCommand::PushMask => self.push_mask(),
            DrawCommand::ActivateMask => self.activate_mask(),
            DrawCommand::DeactivateMask => self.deactivate_mask(),
            DrawCommand::PopMask => self.pop_mask(),
            DrawCommand::RenderAlphaMask {
                maskee,
                mask,
                binds,
                transform_buffer,
            } => self.render_alpha_mask(maskee, mask, binds, *transform_buffer),
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

    pub fn prep_lines(&mut self) {
        if self.needs_stencil {
            self.render_pass
                .set_pipeline(self.pipelines.lines.pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.lines.stencilless_pipeline());
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

    pub fn prep_alpha_mask(&mut self, bind_group: &'pass wgpu::BindGroup) {
        if self.needs_stencil {
            self.render_pass
                .set_pipeline(self.pipelines.alpha_mask.pipeline_for(self.mask_state));
        } else {
            self.render_pass
                .set_pipeline(self.pipelines.alpha_mask.stencilless_pipeline());
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

    pub fn render_alpha_mask(
        &mut self,
        _maskee: &PoolOrArcTexture,
        _mask: &PoolOrArcTexture,
        bind_group: &'frame wgpu::BindGroup,
        transform_buffer: wgpu::DynamicOffset,
    ) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.push_debug_group("render_alpha_mask");
        }

        self.prep_alpha_mask(bind_group);

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

    pub fn draw_lines<const RECT: bool>(&mut self, transform_buffer: wgpu::DynamicOffset) {
        if cfg!(feature = "render_debug_labels") {
            self.render_pass.push_debug_group("draw_lines");
        }
        self.prep_lines();

        self.render_pass.set_bind_group(
            1,
            &self.dynamic_transforms.bind_group,
            &[transform_buffer],
        );

        self.draw(
            self.descriptors.quad.vertices_pos_color.slice(..),
            if RECT {
                self.descriptors.quad.indices_line_rect.slice(..)
            } else {
                self.descriptors.quad.indices_line.slice(..)
            },
            if RECT { 5 } else { 2 },
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

/// Region within the viewport that a blend covers.
#[derive(Debug, Clone, Copy)]
pub struct BlendRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub enum Chunk {
    Draw {
        chunk: Vec<DrawCommand>,
        needs_stencil: bool,
        transforms: BufferBuilder,
    },
    Blend {
        texture: PoolOrArcTexture,
        blend_mode: ChunkBlendMode,
        needs_stencil: bool,
        /// Region within the viewport that this blend covers.
        region: Option<BlendRegion>,
        /// If set, a positioned transform carrying the UV offset/scale used to
        /// sample the region-sized parent/current textures. Boxed to keep the
        /// enum small.
        blend_transform: Option<Box<BlendTransforms>>,
    },
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
    RenderAlphaMask {
        maskee: PoolOrArcTexture,
        mask: PoolOrArcTexture,
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

/// Replaces every blend with a RenderBitmap, with the subcommands rendered out to a temporary texture
/// Every complex blend will be its own item, but every other draw will be chunked together
#[expect(clippy::too_many_arguments)]
pub fn chunk_blends<'a>(
    commands: CommandList,
    descriptors: &'a Descriptors,
    staging_belt: &'a mut wgpu::util::StagingBelt,
    dynamic_transforms: &'a DynamicTransforms,
    draw_encoder: &mut wgpu::CommandEncoder,
    meshes: &'a Vec<Mesh>,
    quality: StageQuality,
    width: u32,
    height: u32,
    offset_x: u32,
    offset_y: u32,
    nearest_layer: LayerRef,
    texture_pool: &mut TexturePool,
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
        offset_x,
        offset_y,
        nearest_layer,
        texture_pool,
    )
    .chunk_blends(commands)
}

struct WgpuCommandHandler<'a> {
    descriptors: &'a Descriptors,
    quality: StageQuality,
    width: u32,
    height: u32,
    offset_x: u32,
    offset_y: u32,
    nearest_layer: LayerRef<'a>,
    meshes: &'a Vec<Mesh>,
    staging_belt: &'a mut wgpu::util::StagingBelt,
    dynamic_transforms: &'a DynamicTransforms,
    draw_encoder: &'a mut wgpu::CommandEncoder,
    texture_pool: &'a mut TexturePool,
    emulate_lines: bool,

    result: Vec<Chunk>,
    current: Vec<DrawCommand>,
    transforms: BufferBuilder,
    needs_stencil: bool,
    num_masks: i32,
}

impl<'a> WgpuCommandHandler<'a> {
    #[expect(clippy::too_many_arguments)]
    fn new(
        descriptors: &'a Descriptors,
        staging_belt: &'a mut wgpu::util::StagingBelt,
        dynamic_transforms: &'a DynamicTransforms,
        draw_encoder: &'a mut wgpu::CommandEncoder,
        meshes: &'a Vec<Mesh>,
        quality: StageQuality,
        width: u32,
        height: u32,
        offset_x: u32,
        offset_y: u32,
        nearest_layer: LayerRef<'a>,
        texture_pool: &'a mut TexturePool,
    ) -> Self {
        let transforms = Self::new_transforms(descriptors, dynamic_transforms);

        let emulate_lines = descriptors.backend == Backend::Dx12;

        Self {
            descriptors,
            quality,
            width,
            height,
            offset_x,
            offset_y,
            nearest_layer,
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
        descriptors: &'a Descriptors,
        dynamic_transforms: &'a DynamicTransforms,
    ) -> BufferBuilder {
        let mut transforms = BufferBuilder::new_for_uniform(&descriptors.limits);
        transforms.set_buffer_limit(dynamic_transforms.buffer.size());
        transforms
    }

    /// Replaces every blend with a RenderBitmap, with the subcommands rendered out to a temporary texture
    /// Every complex blend will be its own item, but every other draw will be chunked together
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

/// Compute the AABB of a command in viewport space.
fn command_aabb(cmd: &Command) -> Option<(f32, f32, f32, f32)> {
    let corners: [(f32, f32); 4] = match cmd {
        Command::RenderBitmap {
            bitmap,
            transform,
            pixel_snapping,
            ..
        } => {
            let tex = as_texture(bitmap);
            let tw = tex.texture.width() as f32;
            let th = tex.texture.height() as f32;
            let mut m = transform.matrix;
            pixel_snapping.apply(&mut m);
            let tx = m.tx.to_pixels() as f32;
            let ty = m.ty.to_pixels() as f32;
            [
                (tx, ty),
                (tx + m.a * tw, ty + m.b * tw),
                (tx + m.c * th, ty + m.d * th),
                (tx + m.a * tw + m.c * th, ty + m.b * tw + m.d * th),
            ]
        }
        Command::RenderShape { shape, transform } => {
            let mesh = as_mesh(shape);
            let (bx0, by0, bx1, by1) = mesh.bounds;
            let m = &transform.matrix;
            let tx = m.tx.to_pixels() as f32;
            let ty = m.ty.to_pixels() as f32;
            [
                (tx + m.a * bx0 + m.c * by0, ty + m.b * bx0 + m.d * by0),
                (tx + m.a * bx1 + m.c * by0, ty + m.b * bx1 + m.d * by0),
                (tx + m.a * bx0 + m.c * by1, ty + m.b * bx0 + m.d * by1),
                (tx + m.a * bx1 + m.c * by1, ty + m.b * bx1 + m.d * by1),
            ]
        }
        Command::DrawRect { matrix, .. }
        | Command::DrawLine { matrix, .. }
        | Command::DrawLineRect { matrix, .. } => {
            let tx = matrix.tx.to_pixels() as f32;
            let ty = matrix.ty.to_pixels() as f32;
            [
                (tx, ty),
                (tx + matrix.a, ty + matrix.b),
                (tx + matrix.c, ty + matrix.d),
                (tx + matrix.a + matrix.c, ty + matrix.b + matrix.d),
            ]
        }
        Command::PushMask | Command::ActivateMask | Command::DeactivateMask | Command::PopMask => {
            return None;
        }
        _ => return None,
    };
    let min_x = corners.iter().map(|c| c.0).fold(f32::INFINITY, f32::min);
    let min_y = corners.iter().map(|c| c.1).fold(f32::INFINITY, f32::min);
    let max_x = corners
        .iter()
        .map(|c| c.0)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = corners
        .iter()
        .map(|c| c.1)
        .fold(f32::NEG_INFINITY, f32::max);
    Some((min_x, min_y, max_x, max_y))
}

/// Compute a tight BlendRegion for commands, clipped to viewport.
/// Returns None if bounds can't be computed, or the region isn't strictly
/// smaller than the viewport (in which case a full-viewport blend is cheaper).
fn compute_blend_region(
    commands: &[Command],
    offset_x: u32,
    offset_y: u32,
    width: u32,
    height: u32,
) -> Option<BlendRegion> {
    let mut r_min_x = f32::INFINITY;
    let mut r_min_y = f32::INFINITY;
    let mut r_max_x = f32::NEG_INFINITY;
    let mut r_max_y = f32::NEG_INFINITY;
    for cmd in commands {
        let (x0, y0, x1, y1) = command_aabb(cmd)?;
        r_min_x = r_min_x.min(x0);
        r_min_y = r_min_y.min(y0);
        r_max_x = r_max_x.max(x1);
        r_max_y = r_max_y.max(y1);
    }
    let off_x = offset_x as f32;
    let off_y = offset_y as f32;
    let lx0 = (r_min_x - off_x).max(0.0);
    let ly0 = (r_min_y - off_y).max(0.0);
    let lx1 = (r_max_x - off_x).min(width as f32);
    let ly1 = (r_max_y - off_y).min(height as f32);
    if lx0 >= lx1 || ly0 >= ly1 {
        return None;
    }
    let x = lx0.floor() as u32;
    let y = ly0.floor() as u32;
    let w = (lx1.ceil() as u32)
        .saturating_sub(x)
        .min(width.saturating_sub(x))
        .max(1);
    let h = (ly1.ceil() as u32)
        .saturating_sub(y)
        .min(height.saturating_sub(y))
        .max(1);
    // Only use a region when it's strictly smaller than the viewport — otherwise
    // the full-viewport blend is already the cheapest option.
    if w < width || h < height {
        Some(BlendRegion {
            x,
            y,
            width: w,
            height: h,
        })
    } else {
        None
    }
}

impl CommandHandler for WgpuCommandHandler<'_> {
    fn blend(&mut self, commands: CommandList, blend_mode: RenderBlendMode) {
        let target_layer = if let RenderBlendMode::Builtin(BlendMode::Layer) = &blend_mode {
            LayerRef::Current
        } else {
            self.nearest_layer
        };
        let blend_type = BlendType::from(blend_mode);

        // Compute tight bounds for the blend region.
        // Shader (PixelBender) blends use their own UV logic — no regional sizing.
        let blend_region = if matches!(&blend_type, BlendType::Shader(_)) {
            None
        } else {
            compute_blend_region(
                &commands.commands,
                self.offset_x,
                self.offset_y,
                self.width,
                self.height,
            )
        };
        let (surface_w, surface_h, surface_offset_x, surface_offset_y) =
            if let Some(ref r) = blend_region {
                (r.width, r.height, r.x + self.offset_x, r.y + self.offset_y)
            } else {
                (self.width, self.height, self.offset_x, self.offset_y)
            };

        let surface = Surface::new(
            self.descriptors,
            self.quality,
            surface_w,
            surface_h,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let clear_color = blend_type.default_color();
        let target = surface.draw_commands_with_offset(
            RenderTargetMode::FreshWithColor(clear_color),
            self.descriptors,
            self.meshes,
            commands,
            self.staging_belt,
            self.dynamic_transforms,
            self.draw_encoder,
            target_layer,
            self.texture_pool,
            surface_offset_x,
            surface_offset_y,
        );
        target.ensure_cleared(self.draw_encoder);

        // We currently do not support shader blends in masks. In order not to
        // break other parts of the scene, we just fall back to a normal blend.
        //
        // TODO Add support for shader blends in masks.
        let is_shader_blend_in_mask =
            self.num_masks > 0 && matches!(blend_type, BlendType::Shader(_));
        let blend_type = if is_shader_blend_in_mask {
            BlendType::Trivial(TrivialBlend::Normal)
        } else {
            blend_type
        };

        match blend_type {
            BlendType::Trivial(blend_mode) => {
                let transform = Transform {
                    matrix: Matrix {
                        a: target.width() as f32,
                        d: target.height() as f32,
                        tx: Twips::from_pixels(surface_offset_x as f64),
                        ty: Twips::from_pixels(surface_offset_y as f64),
                        ..Default::default()
                    },
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
                                    resource: self
                                        .descriptors
                                        .quad
                                        .texture_transforms
                                        .as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(texture.view()),
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
                let chunk_blend_mode = match blend_type {
                    BlendType::Complex(complex) => ChunkBlendMode::Complex(complex),
                    BlendType::Shader(shader) => ChunkBlendMode::Shader(shader),
                    _ => unreachable!(),
                };
                // UV transform: map viewport UV → region-local UV (0-1).
                // src_uv = uv * scale + offset, where:
                //   scale = viewport_size / region_size
                //   offset = -region_pos / region_size
                // Both textures are region-sized → same UV transform for both
                let blend_transform = blend_region.map(|r| {
                    let uv_scale_x = self.width as f32 / r.width as f32;
                    let uv_scale_y = self.height as f32 / r.height as f32;
                    let uv_off_x = -(r.x as f32) / r.width as f32;
                    let uv_off_y = -(r.y as f32) / r.height as f32;
                    let uv = [uv_off_x, uv_off_y, uv_scale_x, uv_scale_y];
                    // The quad is composited into the parent target using that
                    // target's globals, which expect absolute viewport coordinates,
                    // so position it at `surface_offset` (= region origin + this
                    // level's offset), matching the trivial-blend path. The UV
                    // transform stays in this level's local space (the globals
                    // subtract the level offset back out).
                    Box::new(BlendTransforms {
                        world_matrix: [
                            [r.width as f32, 0.0, 0.0, 0.0],
                            [0.0, r.height as f32, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [surface_offset_x as f32, surface_offset_y as f32, 0.0, 1.0],
                        ],
                        dst_uv_transform: uv,
                        src_uv_transform: uv,
                    })
                });
                self.result.push(Chunk::Blend {
                    texture: target.take_color_texture(),
                    blend_mode: chunk_blend_mode,
                    needs_stencil: self.num_masks > 0,
                    region: blend_region,
                    blend_transform,
                });
                self.needs_stencil = self.num_masks > 0;
            }
        }
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
                maskee,
                mask,
                binds,
                transform_buffer,
            }
        });
    }
}
