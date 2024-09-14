use lru::LruCache;
use naga_agal::{SamplerConfig, VertexAttributeFormat};
use ruffle_render::backend::ShaderModule;
use std::{
    borrow::Cow,
    cell::{RefCell, RefMut},
    num::NonZeroUsize,
};
use wgpu::SamplerBindingType;

use super::MAX_VERTEX_ATTRIBUTES;

use crate::descriptors::Descriptors;

pub struct ShaderPairAgal {
    vertex_bytecode: Vec<u8>,

    fragment_bytecode: Vec<u8>,
    fragment_sampler_configs: [Option<SamplerConfig>; 8],
    // Caches compiled wgpu shader modules. The cache key represents all of the data
    // that we need to pass to `naga_agal::agal_to_naga` to compile a shader.
    compiled: RefCell<LruCache<ShaderCompileData, CompiledShaderProgram>>,
}

impl ShaderModule for ShaderPairAgal {}

pub struct CompiledShaderProgram {
    pub vertex_module: wgpu::ShaderModule,
    pub fragment_module: wgpu::ShaderModule,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl ShaderPairAgal {
    pub fn new(vertex_bytecode: Vec<u8>, fragment_bytecode: Vec<u8>) -> Self {
        let fragment_sampler_configs =
            naga_agal::extract_sampler_configs(&fragment_bytecode).unwrap();

        Self {
            vertex_bytecode,
            fragment_bytecode,
            fragment_sampler_configs,
            // TODO - figure out a good size for this cache.
            compiled: RefCell::new(LruCache::new(NonZeroUsize::new(2).unwrap())),
        }
    }

    pub fn fragment_sampler_configs(&self) -> &[Option<SamplerConfig>; 8] {
        &self.fragment_sampler_configs
    }

    pub fn compile(
        &self,
        descriptors: &Descriptors,
        data: ShaderCompileData,
    ) -> RefMut<'_, CompiledShaderProgram> {
        let compiled = self.compiled.borrow_mut();
        RefMut::map(compiled, |compiled| {
            // TODO: Figure out a way to avoid the clone when we have a cache hit
            compiled.get_or_insert_mut(data.clone(), || {
                let vertex_naga_module = naga_agal::agal_to_naga(
                    &self.vertex_bytecode,
                    &data.vertex_attributes,
                    &data.sampler_configs,
                )
                .unwrap();
                let vertex_module =
                    descriptors
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("AGAL vertex shader"),
                            source: wgpu::ShaderSource::Naga(Cow::Owned(vertex_naga_module)),
                        });

                let fragment_naga_module = naga_agal::agal_to_naga(
                    &self.fragment_bytecode,
                    &data.vertex_attributes,
                    &data.sampler_configs,
                )
                .unwrap();
                let fragment_module =
                    descriptors
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("AGAL fragment shader"),
                            source: wgpu::ShaderSource::Naga(Cow::Owned(fragment_naga_module)),
                        });

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
                ];

                for (i, texture_info) in data.texture_infos.iter().enumerate() {
                    if let Some(texture_info) = texture_info {
                        let dimension = match texture_info {
                            ShaderTextureInfo::D2 => wgpu::TextureViewDimension::D2,
                            ShaderTextureInfo::Cube => wgpu::TextureViewDimension::Cube,
                        };
                        layout_entries.push(wgpu::BindGroupLayoutEntry {
                            binding: naga_agal::TEXTURE_START_BIND_INDEX + i as u32,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: dimension,
                                multisampled: false,
                            },
                            count: None,
                        });
                        layout_entries.push(wgpu::BindGroupLayoutEntry {
                            binding: naga_agal::TEXTURE_SAMPLER_START_BIND_INDEX + i as u32,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
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

                CompiledShaderProgram {
                    vertex_module,
                    fragment_module,
                    bind_group_layout,
                }
            })
        })
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum ShaderTextureInfo {
    D2,
    Cube,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct ShaderCompileData {
    pub sampler_configs: [SamplerConfig; 8],
    pub vertex_attributes: [Option<VertexAttributeFormat>; MAX_VERTEX_ATTRIBUTES],
    pub texture_infos: [Option<ShaderTextureInfo>; 8],
}
