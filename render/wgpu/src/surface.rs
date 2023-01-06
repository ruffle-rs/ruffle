mod commands;
pub mod target;

use crate::blend::ComplexBlend;
use crate::buffer_pool::TexturePool;
use crate::mesh::Mesh;
use crate::surface::commands::{chunk_blends, Chunk, CommandRenderer};
use crate::uniform_buffer::BufferStorage;
use crate::utils::remove_srgb;
use crate::{ColorAdjustments, Descriptors, MaskState, Pipelines, Transforms, UniformBuffer};
use ruffle_render::commands::CommandList;
use std::sync::Arc;
use target::CommandTarget;
use tracing::instrument;

#[derive(Debug)]
pub struct Surface {
    size: wgpu::Extent3d,
    sample_count: u32,
    pipelines: Arc<Pipelines>,
    format: wgpu::TextureFormat,
    actual_surface_format: wgpu::TextureFormat,
}

impl Surface {
    pub fn new(
        descriptors: &Descriptors,
        sample_count: u32,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let frame_buffer_format = remove_srgb(surface_format);

        let pipelines = descriptors.pipelines(sample_count, frame_buffer_format);
        Self {
            size,
            sample_count,
            pipelines,
            format: frame_buffer_format,
            actual_surface_format: surface_format,
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip_all)]
    pub fn draw_commands_to(
        &mut self,
        frame_view: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
        descriptors: &Descriptors,
        uniform_buffers_storage: &mut BufferStorage<Transforms>,
        color_buffers_storage: &mut BufferStorage<ColorAdjustments>,
        meshes: &Vec<Mesh>,
        commands: CommandList,
        texture_pool: &mut TexturePool,
    ) -> Vec<wgpu::CommandBuffer> {
        uniform_buffers_storage.recall();
        color_buffers_storage.recall();
        let uniform_encoder_label = create_debug_label!("Uniform upload command encoder");
        let mut uniform_buffer = UniformBuffer::new(uniform_buffers_storage);
        let mut color_buffer = UniformBuffer::new(color_buffers_storage);
        let mut uniform_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: uniform_encoder_label.as_deref(),
                });
        let label = create_debug_label!("Draw encoder");
        let mut draw_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: label.as_deref(),
                });

        let target = self.draw_commands(
            clear_color.unwrap_or(wgpu::Color::TRANSPARENT),
            descriptors,
            meshes,
            commands,
            &mut uniform_buffer,
            &mut color_buffer,
            &mut uniform_encoder,
            &mut draw_encoder,
            None,
            texture_pool,
        );
        let mut buffers = vec![draw_encoder.finish()];

        let copy_bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &descriptors.bind_layouts.bitmap,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: descriptors.quad.texture_transforms.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&target.color_view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(
                            &descriptors.bitmap_samplers.get_sampler(false, false),
                        ),
                    },
                ],
                label: create_debug_label!("Copy sRGB bind group").as_deref(),
            });

        let pipeline = if self.actual_surface_format == self.format {
            descriptors.copy_pipeline(self.format)
        } else {
            descriptors.copy_srgb_pipeline(self.actual_surface_format)
        };

        let mut copy_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: create_debug_label!("Frame copy command encoder").as_deref(),
                });

        let load = match clear_color {
            Some(color) => wgpu::LoadOp::Clear(color),
            None => wgpu::LoadOp::Load,
        };

        let mut render_pass = copy_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame_view,
                ops: wgpu::Operations { load, store: true },
                resolve_target: None,
            })],
            depth_stencil_attachment: None,
            label: create_debug_label!("Copy back to render target").as_deref(),
        });

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, target.globals().bind_group(), &[]);
        render_pass.set_bind_group(1, &target.whole_frame_bind_group(descriptors), &[0]);
        render_pass.set_bind_group(2, &copy_bind_group, &[]);

        render_pass.set_vertex_buffer(0, descriptors.quad.vertices.slice(..));
        render_pass.set_index_buffer(
            descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.draw_indexed(0..6, 0, 0..1);
        drop(render_pass);

        buffers.push(copy_encoder.finish());
        buffers.insert(0, uniform_encoder.finish());
        uniform_buffer.finish();
        color_buffer.finish();

        buffers
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip_all)]
    pub fn draw_commands<'frame, 'global: 'frame>(
        &mut self,
        clear_color: wgpu::Color,
        descriptors: &'global Descriptors,
        meshes: &'global Vec<Mesh>,
        commands: CommandList,
        uniform_buffers: &'frame mut UniformBuffer<'global, Transforms>,
        color_buffers: &'frame mut UniformBuffer<'global, ColorAdjustments>,
        uniform_encoder: &'frame mut wgpu::CommandEncoder,
        draw_encoder: &'frame mut wgpu::CommandEncoder,
        nearest_layer: Option<&'frame CommandTarget>,
        texture_pool: &mut TexturePool,
    ) -> CommandTarget {
        let target = CommandTarget::new(
            &descriptors,
            texture_pool,
            self.size,
            self.format,
            self.sample_count,
        );
        let mut num_masks = 0;
        let mut mask_state = MaskState::NoMask;
        let chunks = chunk_blends(
            commands.commands,
            descriptors,
            uniform_buffers,
            color_buffers,
            uniform_encoder,
            draw_encoder,
            meshes,
            target.sample_count(),
            target.width(),
            target.height(),
            nearest_layer.unwrap_or(&target),
            texture_pool,
        );

        for chunk in chunks {
            match chunk {
                Chunk::Draw(chunk, needs_depth) => {
                    let mut render_pass =
                        draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: create_debug_label!(
                                "Chunked draw calls {}",
                                if needs_depth {
                                    "(with depth)"
                                } else {
                                    "(Depthless)"
                                }
                            )
                            .as_deref(),
                            color_attachments: &[target.color_attachments(clear_color)],
                            depth_stencil_attachment: if needs_depth {
                                target.depth_attachment(&descriptors, texture_pool)
                            } else {
                                None
                            },
                        });
                    render_pass.set_bind_group(0, target.globals().bind_group(), &[]);
                    let mut renderer = CommandRenderer::new(
                        &self.pipelines,
                        &meshes,
                        &descriptors,
                        uniform_buffers,
                        color_buffers,
                        uniform_encoder,
                        render_pass,
                        num_masks,
                        mask_state,
                        needs_depth,
                    );

                    for command in &chunk {
                        renderer.execute(command);
                    }

                    num_masks = renderer.num_masks();
                    mask_state = renderer.mask_state();
                }
                Chunk::Blend(texture, blend_mode, needs_depth) => {
                    let parent = match blend_mode {
                        ComplexBlend::Alpha | ComplexBlend::Erase => {
                            nearest_layer.unwrap_or(&target)
                        }
                        _ => &target,
                    };

                    let parent_blend_buffer =
                        parent.update_blend_buffer(&descriptors, texture_pool, draw_encoder);

                    let texture_view = texture.create_view(&Default::default());
                    let blend_bind_group =
                        descriptors
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                label: create_debug_label!(
                                    "Complex blend binds {:?} {}",
                                    blend_mode,
                                    if needs_depth {
                                        "(with depth)"
                                    } else {
                                        "(Depthless)"
                                    }
                                )
                                .as_deref(),
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
                                ],
                            });

                    let mut render_pass =
                        draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: create_debug_label!(
                                "Complex blend {:?} {}",
                                blend_mode,
                                if needs_depth {
                                    "(with depth)"
                                } else {
                                    "(Depthless)"
                                }
                            )
                            .as_deref(),
                            color_attachments: &[target.color_attachments(clear_color)],
                            depth_stencil_attachment: if needs_depth {
                                target.depth_attachment(&descriptors, texture_pool)
                            } else {
                                None
                            },
                        });
                    render_pass.set_bind_group(0, target.globals().bind_group(), &[]);

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
                        render_pass.set_pipeline(
                            self.pipelines.complex_blends[blend_mode].pipeline_for(mask_state),
                        );
                    } else {
                        render_pass.set_pipeline(
                            self.pipelines.complex_blends[blend_mode].depthless_pipeline(),
                        );
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
        }

        target
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }
}
