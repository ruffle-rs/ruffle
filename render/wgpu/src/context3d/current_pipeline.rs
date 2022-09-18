use ruffle_render::backend::{Context3DTriangleFace, Context3DVertexBufferFormat};

use wgpu::FrontFace;
use wgpu::{
    BindGroupLayout, ColorTargetState, ColorWrites, RenderPipelineDescriptor, TextureFormat,
    VertexBufferLayout, VertexState,
};

use std::borrow::Cow;
use std::cell::Cell;
use std::rc::Rc;

use super::{ShaderModuleAgal, VertexAttributeInfo, MAX_VERTEX_ATTRIBUTES};

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

    dirty: Cell<bool>,
}

impl CurrentPipeline {
    pub fn new() -> Self {
        CurrentPipeline {
            vertex_shader: None,
            fragment_shader: None,
            dirty: Cell::new(true),
            culling: Context3DTriangleFace::None,
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

    pub fn update_vertex_buffer_at(&mut self, _index: usize) {
        // FIXME - check if it's the same, so we can skip rebuilding the pipeline
        self.dirty.set(true);
    }

    /// If the pipeline is dirty, recompiles it and returns `Some(freshly_compiled_pipeline`)
    /// Otherwise, returns `None`.
    pub fn rebuild_pipeline(
        &self,
        device: &wgpu::Device,
        bind_group_layout: &BindGroupLayout,
        vertex_attributes: &[Option<VertexAttributeInfo>; MAX_VERTEX_ATTRIBUTES],
    ) -> Option<wgpu::RenderPipeline> {
        if !self.dirty.get() {
            return None;
        }

        self.dirty.set(false);

        let pipeline_layout_label = create_debug_label!("Pipeline layout");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: pipeline_layout_label.as_deref(),
            bind_group_layouts: &[bind_group_layout],
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

        let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(vertex_naga)),
        });

        let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(fragment_naga)),
        });

        let mut stride = 0;

        let wgpu_attributes = vertex_attributes
            .iter()
            .enumerate()
            .flat_map(|(i, attr)| {
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
                        Context3DVertexBufferFormat::Bytes4 => (wgpu::VertexFormat::Uint8x4, 4),
                    };
                    // FIXME - assert that this matches up with the AS3-supplied offset
                    stride += entry_size_bytes;
                    Some(wgpu::VertexAttribute {
                        format,
                        offset: attr.offset_in_32bit_units * 4,
                        shader_location: i as u32,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let cull_mode = match self.culling {
            Context3DTriangleFace::Back => Some(wgpu::Face::Back),
            Context3DTriangleFace::Front => Some(wgpu::Face::Front),
            Context3DTriangleFace::FrontAndBack => {
                log::error!("FrontAndBack culling not supported!");
                None
            }
            Context3DTriangleFace::None => None,
        };

        let compiled = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: create_debug_label!("RenderPipeline").as_deref(),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex_module,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: stride as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu_attributes,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_module,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: None,
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
            // FIXME - get this from AS3
            depth_stencil: None,
            multisample: Default::default(),
            multiview: Default::default(),
        });
        Some(compiled)
    }

    pub fn set_culling(&mut self, face: Context3DTriangleFace) {
        self.culling = face;
        self.dirty.set(true);
    }
}
