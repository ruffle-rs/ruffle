use naga::valid::{Capabilities, ValidationFlags, Validator};
use ruffle_render::backend::{Context3DTriangleFace, Context3DVertexBufferFormat};

use wgpu::{
    BindGroupEntry, BindingResource, BufferDescriptor, BufferUsages, FrontFace, SamplerBindingType,
    TextureView,
};
use wgpu::{Buffer, DepthStencilState, StencilFaceState};
use wgpu::{ColorTargetState, ColorWrites, RenderPipelineDescriptor, TextureFormat, VertexState};

use std::borrow::Cow;
use std::cell::Cell;
use std::num::NonZeroU64;
use std::rc::Rc;

use crate::context3d::VertexBufferWrapper;
use crate::descriptors::Descriptors;

use super::{ShaderModuleAgal, VertexAttributeInfo, MAX_VERTEX_ATTRIBUTES};

const AGAL_NUM_VERTEX_CONSTANTS: u64 = 128;
const AGAL_NUM_FRAGMENT_CONSTANTS: u64 = 28;
pub(super) const AGAL_FLOATS_PER_REGISTER: u64 = 4;

const VERTEX_SHADER_UNIFORMS_BUFFER_SIZE: u64 =
    AGAL_NUM_VERTEX_CONSTANTS * AGAL_FLOATS_PER_REGISTER * std::mem::size_of::<f32>() as u64;
const FRAGMENT_SHADER_UNIFORMS_BUFFER_SIZE: u64 =
    AGAL_NUM_FRAGMENT_CONSTANTS * AGAL_FLOATS_PER_REGISTER * std::mem::size_of::<f32>() as u64;

const SAMPLER_REPEAT_LINEAR: u32 = 2;
const SAMPLER_REPEAT_NEAREST: u32 = 3;
const SAMPLER_CLAMP_LINEAR: u32 = 4;
const SAMPLER_CLAMP_NEAREST: u32 = 5;

const TEXTURE_START_BIND_INDEX: u32 = 6;

// The flash Context3D API is similar to OpenGL - it has many methods
// which modify the current state (`setVertexBufferAt`, `setCulling`, etc.)
// These methods can be called at any time.
//
// In WGPU, this state is associated by a `RenderPipeline` object,
// which needs to be rebuilt whenever the state changes.
//
// To match up these APIs, we store the current state in `CurentPipeline`.
// Whenever a state-changing `Context3DCommand` is executed, we mark the `CurrentPipeline`
// as dirty. When a `wgpu::RenderPipeline` is actually needed by `drawTriangles`,
// we build a new `wgpu::RenderPipeline` from the `CurrentPipeline` state (if it's dirty).
//
// The `CurrentPipeline` state (including the compiled `wgpu::RenderPipeline`) is stored
// in `WgpuContext3D`, and is re-used across calls to `present`. Due to lifetime issues,
// we don't actually store the `wgpu::RenderPipeline` in `CurrentPipeline` - it's
// instead stored in `WgpuContext3D`.
pub struct CurrentPipeline {
    vertex_shader: Option<Rc<ShaderModuleAgal>>,
    fragment_shader: Option<Rc<ShaderModuleAgal>>,

    culling: Context3DTriangleFace,

    bound_textures: [Option<BoundTextureData>; 8],

    pub vertex_shader_uniforms: Buffer,
    pub fragment_shader_uniforms: Buffer,

    has_depth_texture: bool,

    depth_mask: bool,
    pass_compare_mode: wgpu::CompareFunction,

    color_component: wgpu::BlendComponent,
    alpha_component: wgpu::BlendComponent,

    dirty: Cell<bool>,
}

pub struct BoundTextureData {
    pub view: TextureView,
    pub cube: bool,
}

impl CurrentPipeline {
    pub fn new(descriptors: &Descriptors) -> Self {
        let vertex_shader_uniforms = descriptors.device.create_buffer(&BufferDescriptor {
            label: Some("Vertex shader uniforms"),
            size: VERTEX_SHADER_UNIFORMS_BUFFER_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let fragment_shader_uniforms = descriptors.device.create_buffer(&BufferDescriptor {
            label: Some("Fragment shader uniforms"),
            size: FRAGMENT_SHADER_UNIFORMS_BUFFER_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        CurrentPipeline {
            vertex_shader: None,
            fragment_shader: None,
            bound_textures: std::array::from_fn(|_| None),
            vertex_shader_uniforms,
            fragment_shader_uniforms,
            dirty: Cell::new(true),
            culling: Context3DTriangleFace::None,

            has_depth_texture: false,

            depth_mask: true,
            pass_compare_mode: wgpu::CompareFunction::LessEqual,
            color_component: wgpu::BlendComponent::REPLACE,
            alpha_component: wgpu::BlendComponent::REPLACE,
        }
    }
    pub fn set_vertex_shader(&mut self, shader: Rc<ShaderModuleAgal>) {
        if let Some(old_shader) = &self.vertex_shader {
            // If we change the shader, we need to recompile the pipeline.
            if !Rc::ptr_eq(old_shader, &shader) {
                self.dirty.set(true);
            }
        }
        self.vertex_shader = Some(shader);
    }

    pub fn set_fragment_shader(&mut self, shader: Rc<ShaderModuleAgal>) {
        if let Some(old_shader) = &self.fragment_shader {
            // If we change the shader, we need to recompile the pipeline.
            if !Rc::ptr_eq(old_shader, &shader) {
                self.dirty.set(true);
            }
        }
        self.fragment_shader = Some(shader);
    }

    pub fn update_texture_at(&mut self, index: usize, texture: Option<BoundTextureData>) {
        // FIXME - determine if the texture actually changed
        self.dirty.set(true);
        self.bound_textures[index] = texture;
    }

    pub fn update_vertex_buffer_at(&mut self, _index: usize) {
        // FIXME - check if it's the same, so we can skip rebuilding the pipeline
        self.dirty.set(true);
    }

    pub fn update_depth(&mut self, depth_mask: bool, pass_compare_mode: wgpu::CompareFunction) {
        if self.depth_mask != depth_mask || self.pass_compare_mode != pass_compare_mode {
            self.dirty.set(true);
        }
        self.depth_mask = depth_mask;
        self.pass_compare_mode = pass_compare_mode;
    }

    pub fn update_has_depth_texture(&mut self, has_depth_texture: bool) {
        if self.has_depth_texture != has_depth_texture {
            self.dirty.set(true);
            self.has_depth_texture = has_depth_texture;
        }
    }

    /// If the pipeline is dirty, recompiles it and returns `Some(freshly_compiled_pipeline`)
    /// Otherwise, returns `None`.
    pub fn rebuild_pipeline(
        &self,
        descriptors: &Descriptors,
        vertex_attributes: &[Option<VertexAttributeInfo>; MAX_VERTEX_ATTRIBUTES],
    ) -> Option<(wgpu::RenderPipeline, wgpu::BindGroup)> {
        if !self.dirty.get() {
            return None;
        }

        self.dirty.set(false);

        let mut layout_entries = vec![
            // Vertex shader program constants
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Fragment shader program constants
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // One sampler per filter/wrapping combination - see BitmapFilters
            // An AGAL shader can use any of these samplers, so
            // we need to bind them all.
            wgpu::BindGroupLayoutEntry {
                binding: SAMPLER_REPEAT_LINEAR,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: SAMPLER_REPEAT_NEAREST,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: SAMPLER_CLAMP_LINEAR,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: SAMPLER_CLAMP_NEAREST,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ];

        for (i, bound_texture) in self.bound_textures.iter().enumerate() {
            if let Some(bound_texture) = bound_texture {
                let dimension = if bound_texture.cube {
                    wgpu::TextureViewDimension::Cube
                } else {
                    wgpu::TextureViewDimension::D2
                };
                layout_entries.push(wgpu::BindGroupLayoutEntry {
                    binding: TEXTURE_START_BIND_INDEX + i as u32,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: dimension,
                        multisampled: false,
                    },
                    count: None,
                });
            }
        }

        let globals_layout_label = create_debug_label!("Globals bind group layout");
        let bind_group_layout =
            descriptors
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: globals_layout_label.as_deref(),
                    entries: &layout_entries,
                });

        let bind_group_label = create_debug_label!("Bind group");

        let mut bind_group_entries = vec![
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.vertex_shader_uniforms,
                    offset: 0,
                    size: Some(NonZeroU64::new(VERTEX_SHADER_UNIFORMS_BUFFER_SIZE).unwrap()),
                }),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.fragment_shader_uniforms,
                    offset: 0,
                    size: Some(NonZeroU64::new(FRAGMENT_SHADER_UNIFORMS_BUFFER_SIZE).unwrap()),
                }),
            },
            BindGroupEntry {
                binding: SAMPLER_REPEAT_LINEAR,
                resource: BindingResource::Sampler(&descriptors.bitmap_samplers.repeat_linear),
            },
            BindGroupEntry {
                binding: SAMPLER_REPEAT_NEAREST,
                resource: BindingResource::Sampler(&descriptors.bitmap_samplers.repeat_nearest),
            },
            BindGroupEntry {
                binding: SAMPLER_CLAMP_LINEAR,
                resource: BindingResource::Sampler(&descriptors.bitmap_samplers.clamp_linear),
            },
            BindGroupEntry {
                binding: SAMPLER_CLAMP_NEAREST,
                resource: BindingResource::Sampler(&descriptors.bitmap_samplers.clamp_nearest),
            },
        ];

        for (i, bound_texture) in self.bound_textures.iter().enumerate() {
            if let Some(bound_texture) = bound_texture {
                bind_group_entries.push(BindGroupEntry {
                    binding: TEXTURE_START_BIND_INDEX + i as u32,
                    resource: BindingResource::TextureView(&bound_texture.view),
                });
            }
        }

        let bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: bind_group_label.as_deref(),
                layout: &bind_group_layout,
                entries: &bind_group_entries,
            });

        let pipeline_layout_label = create_debug_label!("Pipeline layout");
        let pipeline_layout =
            descriptors
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: pipeline_layout_label.as_deref(),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let agal_attributes = vertex_attributes.clone().map(|attr| {
            attr.map(|attr| match attr.format {
                Context3DVertexBufferFormat::Float4 => naga_agal::VertexAttributeFormat::Float4,
                Context3DVertexBufferFormat::Float3 => naga_agal::VertexAttributeFormat::Float3,
                Context3DVertexBufferFormat::Float2 => naga_agal::VertexAttributeFormat::Float2,
                Context3DVertexBufferFormat::Float1 => naga_agal::VertexAttributeFormat::Float1,
                Context3DVertexBufferFormat::Bytes4 => naga_agal::VertexAttributeFormat::Bytes4,
            })
        });

        let vertex_naga = naga_agal::agal_to_naga(
            &self
                .vertex_shader
                .as_ref()
                .expect("Missing vertex shader!")
                .0,
            &agal_attributes,
        )
        .expect("Vertex shader failed to compile");

        let fragment_naga = naga_agal::agal_to_naga(
            &self
                .fragment_shader
                .as_ref()
                .expect("Missing fragment shader")
                .0,
            &[None; 8],
        )
        .expect("Fragment shader failed to compile");

        let vertex_module = descriptors
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Vertex shader"),
                source: wgpu::ShaderSource::Naga(Cow::Owned(vertex_naga)),
            });

        let fragment_module =
            descriptors
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Fragment shader"),
                    source: wgpu::ShaderSource::Naga(Cow::Owned(fragment_naga)),
                });

        struct BufferData {
            buffer: Rc<VertexBufferWrapper>,
            attrs: Vec<wgpu::VertexAttribute>,
            total_size: usize,
        }

        // The user can call Context3D.setVertexBufferAt with a a mixture of vertex buffers.
        // We need to create one 'BufferData' struct for each distinct vertex buffer
        // across all of the calls to 'setVertexBufferAt'. The 'BufferData' keeps track
        // of all of the bound indices associated with that buffer.
        let mut index_per_buffer: Vec<BufferData> = Vec::new();

        for (i, attr) in vertex_attributes.iter().enumerate() {
            if let Some(attr) = attr {
                let (format, entry_size_bytes) = match attr.format {
                    Context3DVertexBufferFormat::Float4 => (
                        wgpu::VertexFormat::Float32x4,
                        4 * std::mem::size_of::<f32>(),
                    ),
                    Context3DVertexBufferFormat::Float3 => (
                        wgpu::VertexFormat::Float32x3,
                        3 * std::mem::size_of::<f32>(),
                    ),
                    Context3DVertexBufferFormat::Float2 => (
                        wgpu::VertexFormat::Float32x2,
                        2 * std::mem::size_of::<f32>(),
                    ),
                    Context3DVertexBufferFormat::Float1 => {
                        (wgpu::VertexFormat::Float32, std::mem::size_of::<f32>())
                    }
                    // AGAL shaders always work with floating-point values, so
                    // we use Unorm8x4 to convert the bytes to floats in the range
                    // [0, 1].
                    Context3DVertexBufferFormat::Bytes4 => (wgpu::VertexFormat::Unorm8x4, 4),
                };

                let buffer_data = index_per_buffer
                    .iter_mut()
                    .find(|data| Rc::ptr_eq(&data.buffer, &attr.buffer));

                let buffer_data = if let Some(buffer_data) = buffer_data {
                    buffer_data
                } else {
                    index_per_buffer.push(BufferData {
                        buffer: attr.buffer.clone(),
                        attrs: Vec::new(),
                        total_size: 0,
                    });
                    index_per_buffer.last_mut().unwrap()
                };

                // FIXME - assert that this matches up with the AS3-supplied offset
                buffer_data.total_size += entry_size_bytes;
                buffer_data.attrs.push(wgpu::VertexAttribute {
                    format,
                    offset: attr.offset_in_32bit_units * 4,
                    shader_location: i as u32,
                })
            }
        }

        let cull_mode = match self.culling {
            Context3DTriangleFace::Back => Some(wgpu::Face::Back),
            Context3DTriangleFace::Front => Some(wgpu::Face::Front),
            Context3DTriangleFace::FrontAndBack => {
                tracing::error!("FrontAndBack culling not supported!");
                None
            }
            Context3DTriangleFace::None => None,
        };

        let depth_stencil = if self.has_depth_texture {
            Some(DepthStencilState {
                format: TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: self.depth_mask,
                depth_compare: self.pass_compare_mode,
                // FIXME - implement this
                stencil: wgpu::StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: Default::default(),
            })
        } else {
            None
        };

        let wgpu_vertex_buffers = index_per_buffer
            .iter()
            .map(|data| {
                // This value is set when Context3D.createVertexBuffer is called.
                // We may not all of the data associated with a single vertex
                // (e.g. we might have 8 floats per vertex, but only
                // call setVertexBufferAt once to bind the first 4 floats per vertex.
                // However, the total size of the bindings can be at most the total
                // amount of data per vertex. Verify that here
                let data_bytes_per_vertex = (data.buffer.data_32_per_vertex * 4) as u64;
                if data.total_size > data_bytes_per_vertex as usize {
                    panic!("Total size of bound vertex attributes {:?} exceeds data_bytes_per_vertex {:?}", data.total_size,
                    data_bytes_per_vertex);
                }

                let attrs = &data.attrs;
                wgpu::VertexBufferLayout {
                    array_stride: data_bytes_per_vertex,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: attrs,
                }
            })
            .collect::<Vec<_>>();

        let compiled = descriptors
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: create_debug_label!("RenderPipeline").as_deref(),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &vertex_module,
                    entry_point: naga_agal::SHADER_ENTRY_POINT,
                    buffers: &wgpu_vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_module,
                    entry_point: naga_agal::SHADER_ENTRY_POINT,
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: self.color_component,
                            alpha: self.alpha_component,
                        }),
                        write_mask: ColorWrites::all(),
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    // Stage3d appears to use clockwise winding:
                    // https://stackoverflow.com/questions/8677498/stage3d-culling-confusion
                    front_face: FrontFace::Cw,
                    cull_mode,
                    ..Default::default()
                },
                depth_stencil,
                multisample: Default::default(),
                multiview: Default::default(),
            });
        Some((compiled, bind_group))
    }

    pub fn set_culling(&mut self, face: Context3DTriangleFace) {
        self.culling = face;
        self.dirty.set(true);
    }

    pub fn update_blend_factors(
        &mut self,
        color_component: wgpu::BlendComponent,
        alpha_component: wgpu::BlendComponent,
    ) {
        if color_component != self.color_component || alpha_component != self.alpha_component {
            self.color_component = color_component;
            self.alpha_component = alpha_component;
            self.dirty.set(true);
        }
    }
}

// This is useful for debugging shader issues
#[allow(dead_code)]
fn to_wgsl(module: &naga::Module) -> String {
    let mut out = String::new();

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let module_info = validator
        .validate(module)
        .unwrap_or_else(|e| panic!("Validation failed: {:#?}", e));

    let mut writer =
        naga::back::wgsl::Writer::new(&mut out, naga::back::wgsl::WriterFlags::EXPLICIT_TYPES);

    writer.write(module, &module_info).expect("Writing failed");
    out
}
