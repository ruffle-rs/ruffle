use crate::blend::{ComplexBlend, TrivialBlend};
use crate::descriptors::Descriptors;
use crate::mesh::{DrawType, as_mesh};
use crate::pipelines::Pipelines;
use crate::surface::buffers::{ChunkBufferBuilder, ChunkBuffers, DrawOffsets};
use crate::surface::mask::{MaskMode, MaskState};
use crate::surface::target::PoolOrArcTexture;
use ruffle_render::backend::ShapeHandle;
use ruffle_render::pixel_bender::PixelBenderShaderHandle;
use std::ops::Range;

pub enum Chunk {
    Draw {
        chunk: Vec<DrawCommand>,
        needs_stencil: bool,
        chunk_buffer_builder: ChunkBufferBuilder,
    },
    Blend {
        texture: PoolOrArcTexture,
        blend_mode: ChunkBlendMode,
        needs_stencil: bool,
    },
}

#[derive(Debug)]
pub enum ChunkBlendMode {
    Complex(ComplexBlend),
    Shader(PixelBenderShaderHandle),
}

#[derive(Debug)]
pub enum DrawCommand {
    RenderTexture {
        // If this is from a texture pool, we need to keep a copy to make sure it isn't freed before we use it.
        // We don't actually _need_ that texture though, thus the _
        _texture: PoolOrArcTexture,
        binds: wgpu::BindGroup,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
        draw_offsets: DrawOffsets,
    },
    RenderAlphaMask {
        maskee: PoolOrArcTexture,
        mask: PoolOrArcTexture,
        binds: wgpu::BindGroup,
        instance_range: Range<u32>,
    },
    RenderShape {
        shape: ShapeHandle,
        instance_range: Range<u32>,
    },
    DrawRect {
        instance_range: Range<u32>,
    },
    DrawLine {
        instance_range: Range<u32>,
    },
    DrawLineRect {
        instance_range: Range<u32>,
    },
    SetMask(MaskState),
}

impl DrawCommand {
    pub fn name(&self) -> &'static str {
        match self {
            DrawCommand::RenderShape { .. } => "render shape",
            DrawCommand::RenderTexture { .. } => "render texture",
            DrawCommand::DrawRect { .. } => "draw rect",
            DrawCommand::DrawLine { .. } => "draw line",
            DrawCommand::DrawLineRect { .. } => "draw line rect",
            DrawCommand::SetMask(_) => "set mask state",
            DrawCommand::RenderAlphaMask { .. } => "render alpha mask",
        }
    }
}

pub struct CommandRenderer<'encoder> {
    pipelines: &'encoder Pipelines,
    descriptors: &'encoder Descriptors,
    mask_state: &'encoder mut MaskState,
    needs_stencil: bool,
    chunk_buffers: &'encoder ChunkBuffers,
}

impl<'encoder> CommandRenderer<'encoder> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pipelines: &'encoder Pipelines,
        descriptors: &'encoder Descriptors,
        chunk_buffers: &'encoder ChunkBuffers,
        mask_state: &'encoder mut MaskState,
        needs_stencil: bool,
    ) -> Self {
        Self {
            pipelines,
            mask_state,
            descriptors,
            needs_stencil,
            chunk_buffers,
        }
    }

    pub fn execute(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        command: &'encoder DrawCommand,
    ) {
        if self.needs_stencil {
            self.mask_state.apply_stencil_reference(render_pass);
        }

        match command {
            DrawCommand::RenderTexture {
                _texture,
                binds,
                blend_mode,
                draw_offsets,
                render_stage3d,
            } => self.render_texture(
                render_pass,
                binds,
                *blend_mode,
                *render_stage3d,
                draw_offsets.clone(),
            ),
            DrawCommand::RenderShape {
                shape,
                instance_range,
            } => self.render_shape(render_pass, shape, instance_range.clone()),
            DrawCommand::DrawRect { instance_range } => {
                self.draw_rect(render_pass, instance_range.clone())
            }
            DrawCommand::DrawLine { instance_range } => {
                self.draw_lines::<false>(render_pass, instance_range.clone())
            }
            DrawCommand::DrawLineRect { instance_range } => {
                self.draw_lines::<true>(render_pass, instance_range.clone())
            }
            DrawCommand::SetMask(mask_state) => self.set_mask_state(*mask_state),
            DrawCommand::RenderAlphaMask {
                maskee,
                mask,
                binds,
                instance_range,
            } => self.render_alpha_mask(render_pass, maskee, mask, binds, instance_range.clone()),
        }
    }

    pub fn prep_color(&self, render_pass: &mut wgpu::RenderPass<'encoder>) {
        if self.needs_stencil {
            render_pass.set_pipeline(self.pipelines.color.pipeline_for(self.mask_state.mode));
        } else {
            render_pass.set_pipeline(self.pipelines.color.stencilless_pipeline());
        }
    }

    pub fn prep_lines(&self, render_pass: &mut wgpu::RenderPass<'encoder>) {
        if self.needs_stencil {
            render_pass.set_pipeline(self.pipelines.lines.pipeline_for(self.mask_state.mode));
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
            render_pass.set_pipeline(self.pipelines.gradients.pipeline_for(self.mask_state.mode));
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
                render_pass.set_pipeline(
                    self.pipelines.bitmap[blend_mode].pipeline_for(self.mask_state.mode),
                );
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
            render_pass.set_pipeline(self.pipelines.alpha_mask.pipeline_for(self.mask_state.mode));
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
        instance_range: Range<u32>,
    ) {
        render_pass.set_vertex_buffer(0, vertices);
        render_pass.set_index_buffer(indices, wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..num_indices, 0, instance_range);
    }

    pub fn render_texture(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        bind_group: &'encoder wgpu::BindGroup,
        blend_mode: TrivialBlend,
        render_stage3d: bool,
        draw_offsets: DrawOffsets,
    ) {
        self.prep_bitmap(render_pass, bind_group, blend_mode, render_stage3d);

        let vertex_slice = if let Some(vertex_offset) = draw_offsets.vertices {
            self.chunk_buffers.vertex_buffer.slice(vertex_offset)
        } else {
            self.descriptors.quad.vertices_pos_uv.slice(..)
        };

        self.draw(
            render_pass,
            vertex_slice,
            self.chunk_buffers.index_buffer.slice(draw_offsets.indices),
            6,
            draw_offsets.instances,
        );
    }

    pub fn render_shape(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        shape: &'encoder ShapeHandle,
        instance_range: Range<u32>,
    ) {
        let mesh = as_mesh(shape);
        for draw in &mesh.draws {
            let num_indices = if self.mask_state.mode != MaskMode::DrawMaskStencil
                && self.mask_state.mode != MaskMode::ClearMaskStencil
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

            self.draw(
                render_pass,
                mesh.vertex_buffer.slice(draw.vertices.clone()),
                mesh.index_buffer.slice(draw.indices.clone()),
                num_indices,
                instance_range.clone(),
            );
        }
    }

    pub fn render_alpha_mask(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        _maskee: &PoolOrArcTexture,
        _mask: &PoolOrArcTexture,
        bind_group: &'encoder wgpu::BindGroup,
        instance_range: Range<u32>,
    ) {
        if cfg!(feature = "render_debug_labels") {
            render_pass.push_debug_group("render_alpha_mask");
        }

        self.prep_alpha_mask(render_pass, bind_group);

        self.draw(
            render_pass,
            self.descriptors.quad.vertices_pos.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
            instance_range,
        );

        if cfg!(feature = "render_debug_labels") {
            render_pass.pop_debug_group();
        }
    }

    pub fn draw_rect(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        instance_range: Range<u32>,
    ) {
        self.prep_color(render_pass);

        self.draw(
            render_pass,
            self.descriptors.quad.vertices_pos_color.slice(..),
            self.descriptors.quad.indices.slice(..),
            6,
            instance_range,
        );
    }

    pub fn draw_lines<const RECT: bool>(
        &self,
        render_pass: &mut wgpu::RenderPass<'encoder>,
        instance_range: Range<u32>,
    ) {
        self.prep_lines(render_pass);

        self.draw(
            render_pass,
            self.descriptors.quad.vertices_pos_color.slice(..),
            if RECT {
                self.descriptors.quad.indices_line_rect.slice(..)
            } else {
                self.descriptors.quad.indices_line.slice(..)
            },
            if RECT { 5 } else { 2 },
            instance_range,
        );
    }

    pub fn set_mask_state(&mut self, mask_state: MaskState) {
        *self.mask_state = mask_state;
    }
}
