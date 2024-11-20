mod commands;
pub mod target;

use crate::backend::RenderTargetMode;
use crate::blend::{BlendType, ComplexBlend};
use crate::buffer_pool::TexturePool;
use crate::dynamic_transforms::DynamicTransforms;
use crate::filters::FilterSource;
use crate::mesh::Mesh;
use crate::pixel_bender::{run_pixelbender_shader_impl, ShaderMode};
pub use crate::surface::commands::LayerRef;
use crate::surface::commands::{chunk_blends, Chunk, CommandRenderer};
use crate::utils::run_copy_pipeline;
use crate::utils::{remove_srgb, supported_sample_count};
use crate::{Descriptors, MaskState, Pipelines, Transforms};
use ruffle_render::commands::{CommandList, RenderBlendMode};
use ruffle_render::matrix::Matrix;
use ruffle_render::pixel_bender::{ImageInputTexture, PixelBenderShaderArgument};
use ruffle_render::quality::StageQuality;
use std::sync::Arc;
use swf::BlendMode;
use target::CommandTarget;
use tracing::instrument;
use wgpu::BufferAddress;

#[derive(Debug)]
pub struct Surface {
    size: wgpu::Extent3d,
    quality: StageQuality,
    sample_count: u32,
    pipelines: Arc<Pipelines>,
    format: wgpu::TextureFormat,
    actual_surface_format: wgpu::TextureFormat,
}

impl Surface {
    pub fn new(
        descriptors: &Descriptors,
        quality: StageQuality,
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

        let sample_count = supported_sample_count(
            &descriptors.adapter,
            quality.sample_count(),
            frame_buffer_format,
        );
        let pipelines = descriptors.pipelines(sample_count, frame_buffer_format);
        Self {
            size,
            quality,
            sample_count,
            pipelines,
            format: frame_buffer_format,
            actual_surface_format: surface_format,
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip_all)]
    pub fn draw_commands_and_copy_to<'frame, 'global: 'frame>(
        &mut self,
        frame_view: &wgpu::TextureView,
        render_target_mode: RenderTargetMode,
        descriptors: &'global Descriptors,
        staging_belt: &'frame mut wgpu::util::StagingBelt,
        dynamic_transforms: &'global DynamicTransforms,
        draw_encoder: &'frame mut wgpu::CommandEncoder,
        meshes: &'global Vec<Mesh>,
        commands: CommandList,
        layer: LayerRef,
        texture_pool: &mut TexturePool,
    ) {
        let target = self.draw_commands(
            render_target_mode,
            descriptors,
            meshes,
            commands,
            staging_belt,
            dynamic_transforms,
            draw_encoder,
            layer,
            texture_pool,
        );

        run_copy_pipeline(
            descriptors,
            self.format,
            self.actual_surface_format,
            frame_view,
            target.color_view(),
            target.whole_frame_bind_group(descriptors),
            target.globals(),
            1,
            draw_encoder,
        );
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip_all)]
    pub fn draw_commands<'frame, 'global: 'frame>(
        &mut self,
        render_target_mode: RenderTargetMode,
        descriptors: &'global Descriptors,
        meshes: &'global Vec<Mesh>,
        commands: CommandList,
        staging_belt: &'global mut wgpu::util::StagingBelt,
        dynamic_transforms: &'global DynamicTransforms,
        draw_encoder: &'frame mut wgpu::CommandEncoder,
        nearest_layer: LayerRef<'frame>,
        texture_pool: &mut TexturePool,
    ) -> CommandTarget {
        let target = CommandTarget::new(
            descriptors,
            texture_pool,
            self.size,
            self.format,
            self.sample_count,
            render_target_mode,
            draw_encoder,
        );

        let mut num_masks = 0;
        let mut mask_state = MaskState::NoMask;
        let chunks = chunk_blends(commands, descriptors, dynamic_transforms);

        for chunk in chunks {
            match chunk {
                Chunk::Draw(chunk, needs_stencil, transform_buffers) => {
                    transform_buffers.copy_to(
                        staging_belt,
                        &descriptors.device,
                        draw_encoder,
                        &dynamic_transforms.buffer,
                    );
                    let mut render_pass =
                        draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: create_debug_label!(
                                "Chunked draw calls {}",
                                if needs_stencil {
                                    "(with stencil)"
                                } else {
                                    "(Stencilless)"
                                }
                            )
                            .as_deref(),
                            color_attachments: &[target.color_attachments()],
                            depth_stencil_attachment: if needs_stencil {
                                target.stencil_attachment(descriptors, texture_pool)
                            } else {
                                None
                            },
                            ..Default::default()
                        });
                    render_pass.set_bind_group(0, target.globals().bind_group(), &[]);
                    let mut renderer = CommandRenderer::new(
                        &self.pipelines,
                        descriptors,
                        dynamic_transforms,
                        render_pass,
                        num_masks,
                        mask_state,
                        needs_stencil,
                    );

                    for command in &chunk {
                        renderer.execute(command);
                    }

                    num_masks = renderer.num_masks();
                    mask_state = renderer.mask_state();
                }
                Chunk::Blend(chunk, needs_stencil, blend_mode) => {
                    let is_layer = matches!(blend_mode, RenderBlendMode::Builtin(BlendMode::Layer));
                    let blend_type = BlendType::from(blend_mode);
                    let mut child = Surface::new(
                        descriptors,
                        self.quality,
                        target.width(),
                        target.height(),
                        wgpu::TextureFormat::Rgba8Unorm,
                    );
                    let parent = match blend_type {
                        BlendType::Complex(ComplexBlend::Alpha | ComplexBlend::Erase) => {
                            match nearest_layer {
                                LayerRef::None => {
                                    // An Alpha or Erase with no Layer above it should be ignored
                                    continue;
                                }
                                LayerRef::Current => &target,
                                LayerRef::Parent(layer) => layer,
                            }
                        }
                        _ => &target,
                    };
                    let parent_blend_buffer =
                        parent.update_blend_buffer(descriptors, texture_pool, draw_encoder);

                    let child_target = child.draw_commands(
                        RenderTargetMode::FreshWithColor(blend_type.default_color()),
                        descriptors,
                        meshes,
                        chunk,
                        staging_belt,
                        dynamic_transforms,
                        draw_encoder,
                        match (is_layer, nearest_layer) {
                            (true, _) => LayerRef::Current,
                            (false, LayerRef::Current) => LayerRef::Parent(&target),
                            (false, layer) => layer,
                        },
                        texture_pool,
                    );
                    child_target.ensure_cleared(draw_encoder);
                    let child_texture = child_target.take_color_texture();

                    match blend_type {
                        BlendType::Trivial(blend_mode) => {
                            let matrix =
                                Matrix::scale(parent.width() as f32, parent.height() as f32);
                            let transform = [Transforms {
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
                                mult_color: [1.0, 1.0, 1.0, 1.0],
                                add_color: [0.0, 0.0, 0.0, 0.0],
                            }];
                            let transform_bytes: &[u8] = bytemuck::cast_slice(&transform);
                            staging_belt
                                .write_buffer(
                                    draw_encoder,
                                    &dynamic_transforms.buffer,
                                    BufferAddress::default(),
                                    wgpu::BufferSize::new(transform_bytes.len() as u64).unwrap(),
                                    &descriptors.device,
                                )
                                .copy_from_slice(transform_bytes);
                            let mut render_pass =
                                draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: create_debug_label!(
                                        "Apply trivial blend {blend_mode:?} {}",
                                        if needs_stencil {
                                            "(with stencil)"
                                        } else {
                                            "(Stencilless)"
                                        }
                                    )
                                    .as_deref(),
                                    color_attachments: &[target.color_attachments()],
                                    depth_stencil_attachment: if needs_stencil {
                                        target.stencil_attachment(descriptors, texture_pool)
                                    } else {
                                        None
                                    },
                                    ..Default::default()
                                });
                            render_pass.set_bind_group(0, target.globals().bind_group(), &[]);
                            let mut renderer = CommandRenderer::new(
                                &self.pipelines,
                                descriptors,
                                dynamic_transforms,
                                render_pass,
                                num_masks,
                                mask_state,
                                needs_stencil,
                            );
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
                                                    child_texture.view(),
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
                            renderer.render_texture(0, &bind_group, blend_mode);
                        }
                        BlendType::Complex(blend_mode) => {
                            let blend_bind_group =
                                descriptors
                                    .device
                                    .create_bind_group(&wgpu::BindGroupDescriptor {
                                        label: create_debug_label!(
                                            "Complex blend binds {:?} {}",
                                            blend_mode,
                                            if needs_stencil {
                                                "(with stencil)"
                                            } else {
                                                "(Stencilless)"
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
                                                resource: wgpu::BindingResource::TextureView(
                                                    child_texture.view(),
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
                                    });

                            let mut render_pass =
                                draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: create_debug_label!(
                                        "Apply complex blend {:?} {}",
                                        blend_mode,
                                        if needs_stencil {
                                            "(with stencil)"
                                        } else {
                                            "(Stencilless)"
                                        }
                                    )
                                    .as_deref(),
                                    color_attachments: &[target.color_attachments()],
                                    depth_stencil_attachment: if needs_stencil {
                                        target.stencil_attachment(descriptors, texture_pool)
                                    } else {
                                        None
                                    },
                                    ..Default::default()
                                });
                            render_pass.set_bind_group(0, target.globals().bind_group(), &[]);

                            if needs_stencil {
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
                                    self.pipelines.complex_blends[blend_mode]
                                        .pipeline_for(mask_state),
                                );
                            } else {
                                render_pass.set_pipeline(
                                    self.pipelines.complex_blends[blend_mode]
                                        .stencilless_pipeline(),
                                );
                            }

                            render_pass.set_bind_group(
                                1,
                                target.whole_frame_bind_group(descriptors),
                                &[0],
                            );
                            render_pass.set_bind_group(2, &blend_bind_group, &[]);

                            render_pass
                                .set_vertex_buffer(0, descriptors.quad.vertices_pos.slice(..));
                            render_pass.set_index_buffer(
                                descriptors.quad.indices.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );

                            render_pass.draw_indexed(0..6, 0, 0..1);
                        }
                        BlendType::Shader(shader) => {
                            run_pixelbender_shader_impl(
                                descriptors,
                                shader,
                                ShaderMode::Filter,
                                &[
                                    PixelBenderShaderArgument::ImageInput {
                                        index: 0,
                                        channels: 0xFF,
                                        name: "background".to_string(),
                                        texture: Some(ImageInputTexture::TextureRef(
                                            parent_blend_buffer.texture(),
                                        )),
                                    },
                                    PixelBenderShaderArgument::ImageInput {
                                        index: 1,
                                        channels: 0xff,
                                        name: "foreground".to_string(),
                                        texture: Some(ImageInputTexture::TextureRef(
                                            child_texture.texture(),
                                        )),
                                    },
                                ],
                                parent_blend_buffer.texture(),
                                draw_encoder,
                                target.color_attachments(),
                                target.sample_count(),
                                &FilterSource::for_entire_texture(child_texture.texture()),
                            )
                            .expect("Failed to run PixelBender blend mode");
                        }
                    }
                    drop(child_texture);
                }
            }
        }

        // If nothing happened, ensure it's cleared so we don't operate on garbage data
        target.ensure_cleared(draw_encoder);

        target
    }

    pub fn quality(&self) -> StageQuality {
        self.quality
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }
}
