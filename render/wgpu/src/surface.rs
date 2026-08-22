mod commands;
pub mod target;

use crate::backend::RenderTargetMode;
use crate::blend::{BlendType, ComplexBlend, TrivialBlend};
use crate::buffer_builder::BufferBuilder;
use crate::buffer_pool::TexturePool;
use crate::dynamic_transforms::DynamicTransforms;
use crate::filters::FilterSource;
use crate::mesh::Mesh;
use crate::pixel_bender::{ShaderMode, run_pixelbender_shader_impl};
use crate::surface::commands::{Chunk, CommandRenderer, chunk_blends};
use crate::utils::supported_sample_count;
use crate::{Descriptors, MaskState, Pipelines, Transforms};
use ruffle_render::commands::{CommandList, RenderBlendMode};
use ruffle_render::pixel_bender_support::{ImageInputTexture, PixelBenderShaderArgument};
use ruffle_render::quality::StageQuality;
use std::sync::Arc;
use swf::BlendMode;
use target::CommandTarget;
use tracing::instrument;

use crate::utils::run_copy_pipeline;

pub use crate::surface::commands::LayerRef;

#[derive(Debug)]
pub struct Surface {
    size: wgpu::Extent3d,
    quality: StageQuality,
    sample_count: u32,
    pipelines: Arc<Pipelines>,
    format: wgpu::TextureFormat,
}

impl Surface {
    pub fn new(
        descriptors: &Descriptors,
        quality: StageQuality,
        width: u32,
        height: u32,
        frame_buffer_format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

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
        }
    }

    #[expect(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip_all)]
    pub fn draw_commands_and_copy_to<'frame, 'global: 'frame>(
        &self,
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
            frame_view,
            target.color_view(),
            target.whole_frame_bind_group(descriptors),
            target.globals(),
            1,
            draw_encoder,
        );
    }

    #[expect(clippy::too_many_arguments)]
    #[instrument(level = "debug", skip_all)]
    pub fn draw_commands<'frame, 'global: 'frame>(
        &self,
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
        let chunks = chunk_blends(
            commands,
            descriptors,
            staging_belt,
            dynamic_transforms,
            draw_encoder,
            meshes,
            self.quality,
            target.width(),
            target.height(),
            texture_pool,
        );

        for chunk in chunks {
            match chunk {
                Chunk::Draw {
                    chunk,
                    needs_stencil,
                    transforms,
                } => {
                    transforms.copy_to(staging_belt, draw_encoder, &dynamic_transforms.buffer);
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
                Chunk::Blend {
                    commands,
                    blend_mode,
                    needs_stencil,
                } => {
                    // Render blends just before compositing them. This lets the child target
                    // return to the texture pool before the next sibling blend is rendered,
                    // so peak temporary texture use follows nesting depth instead of sibling count.
                    let is_layer = matches!(blend_mode, RenderBlendMode::Builtin(BlendMode::Layer));
                    let mut blend_type = BlendType::from(blend_mode);

                    // Shader blends are not supported inside stencil masks. Preserve the
                    // existing fallback without forcing eager texture allocation.
                    if needs_stencil && matches!(blend_type, BlendType::Shader(_)) {
                        blend_type = BlendType::Trivial(TrivialBlend::Normal);
                    }

                    // Rendering a blend subtree may itself update the nearest layer.
                    // Snapshot the backdrop first so those updates cannot leak into this
                    // blend's input.
                    let blend_parent = match &blend_type {
                        BlendType::Complex(ComplexBlend::Alpha | ComplexBlend::Erase) => {
                            Some(match nearest_layer {
                                LayerRef::None => continue,
                                LayerRef::Current => &target,
                                LayerRef::Parent(layer) => layer,
                            })
                        }
                        BlendType::Complex(_) | BlendType::Shader(_) => Some(&target),
                        BlendType::Trivial(_) => None,
                    };
                    let parent_blend_buffer = blend_parent.map(|parent| {
                        parent.update_blend_buffer(descriptors, texture_pool, draw_encoder)
                    });

                    let child_surface = Surface::new(
                        descriptors,
                        self.quality,
                        target.width(),
                        target.height(),
                        wgpu::TextureFormat::Rgba8Unorm,
                    );
                    let child_target = child_surface.draw_commands(
                        RenderTargetMode::FreshWithColor(blend_type.default_color()),
                        descriptors,
                        meshes,
                        commands,
                        staging_belt,
                        dynamic_transforms,
                        draw_encoder,
                        if is_layer {
                            LayerRef::Current
                        } else {
                            match nearest_layer {
                                LayerRef::Current => LayerRef::Parent(&target),
                                layer => layer,
                            }
                        },
                        texture_pool,
                    );
                    child_target.ensure_cleared(draw_encoder);
                    let child_texture = child_target.take_color_texture();

                    match blend_type {
                        BlendType::Trivial(blend_mode) => {
                            let transform = Transforms {
                                world_matrix: [
                                    [target.width() as f32, 0.0, 0.0, 0.0],
                                    [0.0, target.height() as f32, 0.0, 0.0],
                                    [0.0, 0.0, 1.0, 0.0],
                                    [0.0, 0.0, 0.0, 1.0],
                                ],
                                mult_color: [1.0, 1.0, 1.0, 1.0],
                                add_color: [0.0, 0.0, 0.0, 0.0],
                            };
                            let mut transforms =
                                BufferBuilder::new_for_uniform(&descriptors.limits);
                            transforms.set_buffer_limit(dynamic_transforms.buffer.size());
                            let transform_offset = transforms
                                .add(&[transform])
                                .expect("A single transform must fit in the transforms buffer")
                                .start
                                as wgpu::DynamicOffset;
                            transforms.copy_to(
                                staging_belt,
                                draw_encoder,
                                &dynamic_transforms.buffer,
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
                            renderer.render_texture(transform_offset, &bind_group, blend_mode);
                        }
                        BlendType::Complex(blend_mode) => {
                            let parent_blend_buffer = parent_blend_buffer
                                .expect("Complex blends must snapshot their backdrop");
                            let blend_bind_group =
                                descriptors
                                    .device
                                    .create_bind_group(&wgpu::BindGroupDescriptor {
                                        label: create_debug_label!(
                                            "Complex blend binds {blend_mode:?}"
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
                                        "Apply complex blend {blend_mode:?}"
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
                                    MaskState::DrawMaskedContent | MaskState::ClearMaskStencil => {
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
                            let parent_blend_buffer = parent_blend_buffer
                                .expect("Shader blends must snapshot their backdrop");
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
                                        channels: 0xFF,
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
