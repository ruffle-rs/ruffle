use crate::backend::RenderTargetMode;
use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::{FilterSource, FilterVertex, VERTEX_BUFFERS_DESCRIPTION_FILTERS};
use crate::surface::target::CommandTarget;
use crate::utils::SampleCountMap;
use bytemuck::{Pod, Zeroable};
use std::sync::OnceLock;
use swf::ConvolutionFilter as ConvolutionFilterArgs;
use wgpu::util::StagingBelt;

/// Four packed taps share one WGSL `vec4` of offsets and one `vec4` of weights.
const TAP_GROUP_LANES: usize = 4;

/// Flash Player never applies more than a 15x15 kernel (the AVM2 property
/// setters clamp matrixX/matrixY to 15), so a kernel has at most 225 taps.
/// 64 groups leave a little slack over the 225-entry dense maximum.
const MAX_TAP_GROUPS: usize = 64;
const MAX_TAPS: usize = MAX_TAP_GROUPS * TAP_GROUP_LANES; // 256

/// The effective kernel dimension Flash Player renders with, regardless of
/// what the SWF tag or script supplied.
const MAX_KERNEL_DIM: usize = 15;

/// A packed convolution tap: the WGSL side stores four taps per pair of
/// `vec4`s. `offsets[lane]` packs `(dx, dy)` into the low two bytes
/// (`dx + 127` in the low byte, `dy + 127` in the next), and `weights[lane]`
/// is the kernel weight. Matrix dimensions in SWF are `u8`, so tap offsets
/// stay within `-127..=127` and the biased values fit a byte.
// WGSL `vec4` members are 16-byte aligned, so the group needs an explicit
// 16-byte alignment here too: `[f32; 4]` alone would only be 4-byte aligned.
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
struct TapGroup {
    offsets: [i32; TAP_GROUP_LANES],
    weights: [f32; TAP_GROUP_LANES],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
struct ConvolutionFilterUniform {
    source_point: [f32; 2],
    source_size: [f32; 2],
    texture_size: [f32; 2],
    divisor: f32,
    bias: f32,
    use_default_color: f32,
    preserve_alpha: f32,
    tap_count: u32,
    _pad_scalar: f32,
    // Uniform-address-space arrays must be 16-byte aligned, so the shader
    // side cannot pad with an `array<f32>`: one scalar plus one vec4.
    _pad_vec: [f32; 4],
    default_color: [f32; 4],
    taps: [TapGroup; MAX_TAP_GROUPS],
}

pub struct ConvolutionFilter {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    vertices_size: wgpu::BufferSize,
    uniform_size: wgpu::BufferSize,
    pipelines: SampleCountMap<OnceLock<wgpu::RenderPipeline>>,
}

/// The WGSL struct in `shaders/filter/convolution.wgsl` must have exactly the
/// same size and field offsets as this one, because both sides read the same
/// bytes.
///
/// The whole block is 2_128 bytes: an 80-byte header (with `default_color`
/// 16-byte aligned at 64 and the tap array starting at 80) plus 64 tap groups
/// of 32 bytes, room for the 225 taps of the 15x15 kernel Flash caps at.
const _: () = {
    assert!(std::mem::size_of::<TapGroup>() == 32);
    assert!(std::mem::size_of::<ConvolutionFilterUniform>() == 2_128);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, source_point) == 0);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, source_size) == 8);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, texture_size) == 16);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, divisor) == 24);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, bias) == 28);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, use_default_color) == 32);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, preserve_alpha) == 36);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, tap_count) == 40);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, _pad_scalar) == 44);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, _pad_vec) == 48);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, default_color) == 64);
    assert!(std::mem::offset_of!(ConvolutionFilterUniform, taps) == 80);
};

impl ConvolutionFilter {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform_size = std::mem::size_of::<ConvolutionFilterUniform>() as u64;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(uniform_size),
                    },
                    count: None,
                },
            ],
            label: create_debug_label!("Convolution filter binds").as_deref(),
        });

        let vertices_size = std::mem::size_of::<[FilterVertex; 4]>() as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: vertices_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipelines: Default::default(),
            pipeline_layout,
            vertex_buffer,
            uniform_buffer,
            bind_group_layout,
            vertices_size: wgpu::BufferSize::new(vertices_size).expect("Definitely not zero."),
            uniform_size: wgpu::BufferSize::new(uniform_size).expect("Definitely not zero."),
        }
    }

    /// Packs the kernel into non-zero taps. Returns `None` if the kernel does
    /// not fit; the caller then falls back to the (incorrect, but non-fatal)
    /// copy behaviour.
    fn pack_taps(filter: &ConvolutionFilterArgs) -> Option<([TapGroup; MAX_TAP_GROUPS], u32)> {
        let cols = usize::from(filter.num_matrix_cols);
        let rows = usize::from(filter.num_matrix_rows);
        // Flash clamps the rendered kernel to the top-left 15x15
        // sub-matrix, even when the SWF tag carries larger dimensions. The
        // AVM2 setters already clamp, but tags (and any other caller) may
        // still supply up to 255.
        let eff_cols = cols.min(MAX_KERNEL_DIM);
        let eff_rows = rows.min(MAX_KERNEL_DIM);
        let center_col = eff_cols / 2;
        let center_row = eff_rows / 2;

        // `Pod` derives imply `Copy` for the array; initialise explicitly so
        // the unused lanes read as weight 0 / offset (127, 127).
        let mut taps: [TapGroup; MAX_TAP_GROUPS] = [TapGroup {
            offsets: [0; TAP_GROUP_LANES],
            weights: [0.0; TAP_GROUP_LANES],
        }; MAX_TAP_GROUPS];
        let mut tap_count = 0usize;

        for row in 0..eff_rows {
            for col in 0..eff_cols {
                // Flash clamps the dimensions first and then interprets the
                // matrix array as the flat `eff_cols * eff_rows` sub-matrix
                // (the first 225 entries), rather than taking the top-left
                // corner of the original row layout, so the stride here is
                // the clamped width. AVM2 filters are already resized.
                let weight = filter.matrix[row * eff_cols + col];
                if weight == 0.0 {
                    // A zero-weighted tap contributes nothing, whatever its
                    // edge/clamp/default-colour treatment would be: skip it.
                    continue;
                }
                if tap_count >= MAX_TAPS {
                    return None;
                }

                let dx = col as i32 - center_col as i32;
                let dy = row as i32 - center_row as i32;
                let group = tap_count / TAP_GROUP_LANES;
                let lane = tap_count % TAP_GROUP_LANES;
                taps[group].offsets[lane] = (dx + 127) | ((dy + 127) << 8);
                taps[group].weights[lane] = weight;
                tap_count += 1;
            }
        }

        Some((taps, tap_count as u32))
    }

    /// Returns whether this filter can be applied by the shader. The shader
    /// only stores non-zero taps of the clamped 15x15 kernel, which has at
    /// most 225 entries, so a well-formed filter always fits. An empty or
    /// malformed one (zero dimensions, short matrix, non-finite divisor/bias)
    /// is rejected and silently copied by the caller.
    pub fn is_supported(filter: &ConvolutionFilterArgs) -> bool {
        let cols = usize::from(filter.num_matrix_cols);
        let rows = usize::from(filter.num_matrix_rows);
        let required = cols * rows;
        cols > 0
            && rows > 0
            && required <= filter.matrix.len()
            && filter.divisor.is_finite()
            && filter.bias.is_finite()
            && Self::pack_taps(filter).is_some()
    }

    fn pipeline(&self, descriptors: &Descriptors, msaa_sample_count: u32) -> &wgpu::RenderPipeline {
        self.pipelines.get_or_init(msaa_sample_count, || {
            let label = create_debug_label!("Convolution Filter ({} msaa)", msaa_sample_count);
            descriptors
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: label.as_deref(),
                    layout: Some(&self.pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &descriptors.shaders.convolution_filter,
                        entry_point: Some("main_vertex"),
                        buffers: &VERTEX_BUFFERS_DESCRIPTION_FILTERS,
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::default(),
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: msaa_sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &descriptors.shaders.convolution_filter,
                        entry_point: Some("main_fragment"),
                        targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                        compilation_options: Default::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                })
        })
    }

    pub fn apply(
        &self,
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut StagingBelt,
        source: &FilterSource,
        filter: &ConvolutionFilterArgs,
    ) -> Option<CommandTarget> {
        let (taps, tap_count) = Self::pack_taps(filter)?;

        let sample_count = source.texture.sample_count();
        let format = source.texture.format();
        let pipeline = self.pipeline(descriptors, sample_count);

        let target = CommandTarget::new(
            descriptors,
            texture_pool,
            wgpu::Extent3d {
                width: source.size.0,
                height: source.size.1,
                depth_or_array_layers: 1,
            },
            format,
            sample_count,
            RenderTargetMode::FreshWithColor(wgpu::Color::TRANSPARENT),
            draw_encoder,
        );

        let source_view = source.texture.create_view(&Default::default());
        staging_belt
            .write_buffer(draw_encoder, &self.uniform_buffer, 0, self.uniform_size)
            .copy_from_slice(bytemuck::cast_slice(&[ConvolutionFilterUniform {
                source_point: [source.point.0 as f32, source.point.1 as f32],
                source_size: [source.size.0 as f32, source.size.1 as f32],
                texture_size: [
                    source.texture.width() as f32,
                    source.texture.height() as f32,
                ],
                divisor: filter.divisor,
                bias: filter.bias,
                use_default_color: if filter.is_clamped() { 0.0 } else { 1.0 },
                preserve_alpha: if filter.is_preserve_alpha() { 1.0 } else { 0.0 },
                tap_count,
                _pad_scalar: 0.0,
                _pad_vec: [0.0; 4],
                default_color: [
                    f32::from(filter.default_color.r) / 255.0,
                    f32::from(filter.default_color.g) / 255.0,
                    f32::from(filter.default_color.b) / 255.0,
                    f32::from(filter.default_color.a) / 255.0,
                ],
                taps,
            }]));
        staging_belt
            .write_buffer(draw_encoder, &self.vertex_buffer, 0, self.vertices_size)
            .copy_from_slice(bytemuck::cast_slice(&[source.vertices()]));
        let filter_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: create_debug_label!("Filter group").as_deref(),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&source_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                ],
            });
        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: create_debug_label!("Convolution filter").as_deref(),
            color_attachments: &[target.color_attachments()],
            ..Default::default()
        });
        render_pass.set_pipeline(pipeline);

        render_pass.set_bind_group(0, &filter_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..6, 0, 0..1);
        Some(target)
    }
}
