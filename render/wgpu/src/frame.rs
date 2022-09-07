use crate::pipelines::BlendMode;
use crate::target::RenderTargetFrame;
use crate::Pipelines;
use crate::{ColorAdjustments, Descriptors, Globals, MaskState, Transforms, UniformBuffer};

pub struct Frame<'a> {
    pipelines: &'a Pipelines,
    descriptors: &'a Descriptors,
    uniform_buffers: UniformBuffer<'a, Transforms>,
    mask_state: MaskState,
    uniform_encoder: &'a mut wgpu::CommandEncoder,
    render_pass: wgpu::RenderPass<'a>,
    blend_mode: BlendMode,
}

impl<'a> Frame<'a> {
    pub fn new(
        pipelines: &'a Pipelines,
        descriptors: &'a Descriptors,
        uniform_buffers: UniformBuffer<'a, Transforms>,
        render_pass: wgpu::RenderPass<'a>,
        uniform_encoder: &'a mut wgpu::CommandEncoder,
    ) -> Self {
        Self {
            pipelines,
            descriptors,
            uniform_buffers,
            mask_state: MaskState::NoMask,
            uniform_encoder,
            render_pass,
            blend_mode: BlendMode::Normal,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn swap_srgb<T: RenderTargetFrame>(
        &mut self,
        globals: &Globals,
        target: &'a T,
        copy_srgb_bind_group: &wgpu::BindGroup,
        width: f32,
        height: f32,
        quad_vertices: wgpu::BufferSlice<'a>,
        quad_indices: wgpu::BufferSlice<'a>,
    ) -> wgpu::CommandBuffer {
        let mut copy_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Frame copy command encoder").as_deref(),
                });

        let mut render_pass = copy_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.view(),
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
        render_pass.set_bind_group(0, globals.bind_group(), &[]);
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
        render_pass.set_vertex_buffer(0, quad_vertices);
        render_pass.set_index_buffer(quad_indices, wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..6, 0, 0..1);
        drop(render_pass);

        copy_encoder.finish()
    }

    pub fn finish(self) {
        self.uniform_buffers.finish()
    }

    pub fn prep_color(&mut self) {
        self.render_pass.set_pipeline(
            self.pipelines
                .color_pipelines
                .pipeline_for(self.blend_mode, self.mask_state),
        );
    }

    pub fn prep_gradient(&mut self, bind_group: &'a wgpu::BindGroup) {
        self.render_pass.set_pipeline(
            self.pipelines
                .gradient_pipelines
                .pipeline_for(self.blend_mode, self.mask_state),
        );
        self.render_pass.set_bind_group(2, bind_group, &[]);
    }

    pub fn prep_bitmap(
        &mut self,
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
    }

    pub fn draw(
        &mut self,
        vertices: wgpu::BufferSlice<'a>,
        indices: wgpu::BufferSlice<'a>,
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

    pub fn set_mask_state(&mut self, state: MaskState) {
        self.mask_state = state;
    }

    pub fn set_stencil(&mut self, num: u32) {
        self.render_pass.set_stencil_reference(num);
    }

    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.blend_mode = blend_mode;
    }

    pub fn mask_state(&self) -> MaskState {
        self.mask_state
    }
}
