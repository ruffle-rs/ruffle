use crate::pipelines::BlendMode as ActualBlendMode;
use crate::target::RenderTargetFrame;
use crate::Pipelines;
use crate::{
    ColorAdjustments, Descriptors, DrawType, Globals, MaskState, Mesh, RegistryData, Transforms,
    UniformBuffer,
};
use fnv::FnvHashMap;
use ruffle_render::backend::ShapeHandle;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::commands::CommandHandler;
use ruffle_render::transform::Transform;
use swf::{BlendMode, Color};

pub struct Frame<'a, T: RenderTargetFrame> {
    pipelines: &'a Pipelines,
    descriptors: &'a Descriptors,
    globals: &'a Globals,
    uniform_buffers: UniformBuffer<'a, Transforms>,
    mask_state: MaskState,
    num_masks: u32,
    target: &'a T,
    uniform_encoder: &'a mut wgpu::CommandEncoder,
    render_pass: wgpu::RenderPass<'a>,
    blend_modes: Vec<BlendMode>,
    blend_mode: ActualBlendMode,
    bitmap_registry: &'a FnvHashMap<BitmapHandle, RegistryData>,
    quad_vbo: &'a wgpu::Buffer,
    quad_ibo: &'a wgpu::Buffer,
    meshes: &'a Vec<Mesh>,
}

impl<'a, T: RenderTargetFrame> Frame<'a, T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pipelines: &'a Pipelines,
        descriptors: &'a Descriptors,
        globals: &'a Globals,
        uniform_buffers: UniformBuffer<'a, Transforms>,
        target: &'a T,
        quad_vbo: &'a wgpu::Buffer,
        quad_ibo: &'a wgpu::Buffer,
        meshes: &'a Vec<Mesh>,
        render_pass: wgpu::RenderPass<'a>,
        uniform_encoder: &'a mut wgpu::CommandEncoder,
        bitmap_registry: &'a FnvHashMap<BitmapHandle, RegistryData>,
    ) -> Self {
        Self {
            pipelines,
            descriptors,
            globals,
            uniform_buffers,
            mask_state: MaskState::NoMask,
            num_masks: 0,
            target,
            uniform_encoder,
            render_pass,
            blend_modes: vec![BlendMode::Normal],
            blend_mode: ActualBlendMode::Normal,
            bitmap_registry,
            quad_vbo,
            quad_ibo,
            meshes,
        }
    }

    pub fn swap_srgb(
        &mut self,
        copy_srgb_bind_group: &wgpu::BindGroup,
        width: f32,
        height: f32,
    ) -> wgpu::CommandBuffer {
        let mut copy_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Frame copy command encoder").as_deref(),
                });

        let mut render_pass = copy_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.target.view(),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            label: None,
        });

        render_pass.set_pipeline(&self.pipelines.copy_srgb_pipeline);
        render_pass.set_bind_group(0, self.globals.bind_group(), &[]);
        self.uniform_buffers.write_uniforms(
            &self.descriptors.device,
            &self.descriptors.bind_layouts.transforms,
            &mut self.uniform_encoder,
            &mut render_pass,
            1,
            &Transforms {
                world_matrix: [
                    [width, 0.0, 0.0, 0.0],
                    [0.0, height, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                color_adjustments: ColorAdjustments {
                    mult_color: [1.0, 1.0, 1.0, 1.0],
                    add_color: [0.0, 0.0, 0.0, 0.0],
                },
            },
        );
        render_pass.set_bind_group(2, copy_srgb_bind_group, &[]);
        render_pass.set_bind_group(
            3,
            self.descriptors
                .bitmap_samplers
                .get_bind_group(false, false),
            &[],
        );
        render_pass.set_vertex_buffer(0, self.quad_vbo.slice(..));
        render_pass.set_index_buffer(self.quad_ibo.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..6, 0, 0..1);
        drop(render_pass);

        copy_encoder.finish()
    }

    pub fn finish(self) {
        self.uniform_buffers.finish()
    }

    fn draw_color(
        &mut self,
        vertices: wgpu::BufferSlice<'a>,
        indices: wgpu::BufferSlice<'a>,
        num_indices: u32,
    ) {
        self.render_pass.set_pipeline(
            self.pipelines
                .color_pipelines
                .pipeline_for(self.blend_mode, self.mask_state),
        );

        self.render_pass.set_vertex_buffer(0, vertices);
        self.render_pass
            .set_index_buffer(indices, wgpu::IndexFormat::Uint32);

        self.render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    fn draw_gradient(
        &mut self,
        vertices: wgpu::BufferSlice<'a>,
        indices: wgpu::BufferSlice<'a>,
        num_indices: u32,
        bind_group: &'a wgpu::BindGroup,
    ) {
        self.render_pass.set_pipeline(
            self.pipelines
                .gradient_pipelines
                .pipeline_for(self.blend_mode, self.mask_state),
        );
        self.render_pass.set_bind_group(2, bind_group, &[]);

        self.render_pass.set_vertex_buffer(0, vertices);
        self.render_pass
            .set_index_buffer(indices, wgpu::IndexFormat::Uint32);

        self.render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    fn draw_bitmap(
        &mut self,
        vertices: wgpu::BufferSlice<'a>,
        indices: wgpu::BufferSlice<'a>,
        num_indices: u32,
        bind_group: &'a wgpu::BindGroup,
        is_repeating: bool,
        is_smoothed: bool,
    ) {
        self.render_pass.set_pipeline(
            self.pipelines
                .bitmap_pipelines
                .pipeline_for(self.blend_mode, self.mask_state),
        );
        self.render_pass.set_bind_group(2, bind_group, &[]);
        self.render_pass.set_bind_group(
            3,
            self.descriptors
                .bitmap_samplers
                .get_bind_group(is_repeating, is_smoothed),
            &[],
        );

        self.render_pass.set_vertex_buffer(0, vertices);
        self.render_pass
            .set_index_buffer(indices, wgpu::IndexFormat::Uint32);

        self.render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    fn apply_transform(
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
}

impl<'a, T: RenderTargetFrame> CommandHandler for Frame<'a, T> {
    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform, smoothing: bool) {
        if let Some(entry) = self.bitmap_registry.get(&bitmap) {
            let texture = &entry.texture_wrapper;

            self.apply_transform(
                &(transform.matrix
                    * ruffle_render::matrix::Matrix {
                        a: texture.width as f32,
                        d: texture.height as f32,
                        ..Default::default()
                    }),
                ColorAdjustments::from(transform.color_transform),
            );

            self.draw_bitmap(
                self.quad_vbo.slice(..),
                self.quad_ibo.slice(..),
                6,
                &texture.bind_group,
                false,
                smoothing,
            );
        }
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
                    self.draw_color(
                        draw.vertex_buffer.slice(..),
                        draw.index_buffer.slice(..),
                        num_indices,
                    );
                }
                DrawType::Gradient { bind_group, .. } => {
                    self.draw_gradient(
                        draw.vertex_buffer.slice(..),
                        draw.index_buffer.slice(..),
                        num_indices,
                        bind_group,
                    );
                }
                DrawType::Bitmap {
                    is_repeating,
                    is_smoothed,
                    bind_group,
                    ..
                } => {
                    self.draw_bitmap(
                        draw.vertex_buffer.slice(..),
                        draw.index_buffer.slice(..),
                        num_indices,
                        bind_group,
                        *is_repeating,
                        *is_smoothed,
                    );
                }
            }
        }
    }

    fn draw_rect(&mut self, color: Color, matrix: &ruffle_render::matrix::Matrix) {
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

        self.draw_color(self.quad_vbo.slice(..), self.quad_ibo.slice(..), 6);
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
        self.mask_state = if self.num_masks == 0 {
            MaskState::NoMask
        } else {
            self.render_pass.set_stencil_reference(self.num_masks);
            MaskState::DrawMaskedContent
        };
    }

    fn push_blend_mode(&mut self, blend: BlendMode) {
        self.blend_modes.push(blend);
        self.blend_mode = blend.into();
    }

    fn pop_blend_mode(&mut self) {
        self.blend_modes.pop();
        self.blend_mode = self
            .blend_modes
            .last()
            .map(|b| ActualBlendMode::from(*b))
            .unwrap_or(ActualBlendMode::Normal);
    }
}
