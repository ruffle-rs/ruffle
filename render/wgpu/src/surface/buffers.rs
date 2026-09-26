use crate::buffer_builder::{BufferBuilder, BufferFull};
use crate::descriptors::Descriptors;
use crate::{PosUvVertex, PosVertex, Transforms};
use std::mem;
use std::ops::Range;
use wgpu::util::StagingBelt;

const ESTIMATED_OBJECTS_PER_CHUNK: u64 = 200;

pub struct ChunkBuffers {
    pub transforms: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub fn transforms_per_draw(limits: &wgpu::Limits) -> u64 {
    let maximum = limits.max_uniform_buffer_binding_size / size_of::<Transforms>() as u64;
    maximum.min(ESTIMATED_OBJECTS_PER_CHUNK)
}

impl ChunkBuffers {
    pub fn new(descriptors: &Descriptors) -> Self {
        let transforms = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: transforms_per_draw(&descriptors.limits) * size_of::<Transforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_buffer = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (mem::size_of::<PosUvVertex>() as u64 * 4 * ESTIMATED_OBJECTS_PER_CHUNK)
                .min(descriptors.limits.max_buffer_size),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (wgpu::IndexFormat::Uint32.byte_size() as u64 * 6 * ESTIMATED_OBJECTS_PER_CHUNK)
                .min(descriptors.limits.max_buffer_size),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &descriptors.bind_layouts.transforms,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transforms.as_entire_binding(),
                }],
            });
        Self {
            transforms,
            bind_group,
            vertex_buffer,
            index_buffer,
        }
    }
}

#[derive(Copy, Clone)]
pub enum DrawVertices<'a> {
    CustomPosUvQuad(&'a [u8]),
    PosUvQuad,
    PosQuad,
    None,
}

#[derive(Clone, Debug)]
pub struct DrawOffsets {
    pub vertices: Option<Range<wgpu::BufferAddress>>,
    pub indices: Range<wgpu::BufferAddress>,
    pub instances: Range<u32>,
}

pub struct ChunkBufferBuilder {
    transforms: BufferBuilder,
    vertices: BufferBuilder,
    indices: BufferBuilder,
    quad_pos_vertices: Range<wgpu::BufferAddress>,
    quad_pos_uv_vertices: Range<wgpu::BufferAddress>,
    quad_indices: Range<wgpu::BufferAddress>,
}

impl ChunkBufferBuilder {
    pub fn new(chunk_buffers: &ChunkBuffers, limits: &wgpu::Limits) -> Self {
        let mut transforms = BufferBuilder::new(limits, 0);
        transforms.set_buffer_limit(chunk_buffers.transforms.size());
        let mut vertices = BufferBuilder::new(limits, 0);
        vertices.set_buffer_limit(chunk_buffers.vertex_buffer.size());
        let mut indices = BufferBuilder::new(limits, 0);
        indices.set_buffer_limit(chunk_buffers.index_buffer.size());

        // It's extremely likely that every chunk will have a quad, so let's prime those eagerly and save some complexity later
        let quad_pos_uv_vertices = vertices
            .add(&[
                PosUvVertex {
                    position: [0.0, 0.0],
                    uv: [0.0, 0.0, 1.0],
                },
                PosUvVertex {
                    position: [1.0, 0.0],
                    uv: [1.0, 0.0, 1.0],
                },
                PosUvVertex {
                    position: [1.0, 1.0],
                    uv: [1.0, 1.0, 1.0],
                },
                PosUvVertex {
                    position: [0.0, 1.0],
                    uv: [0.0, 1.0, 1.0],
                },
            ])
            .expect("Buffer was just created, quad vertices should fit");
        let quad_pos_vertices = vertices
            .add(&[
                PosVertex {
                    position: [0.0, 0.0],
                },
                PosVertex {
                    position: [1.0, 0.0],
                },
                PosVertex {
                    position: [1.0, 1.0],
                },
                PosVertex {
                    position: [0.0, 1.0],
                },
            ])
            .expect("Buffer was just created, quad vertices should fit");
        let quad_indices = indices
            .add(&[0u32, 1, 2, 0, 2, 3])
            .expect("Buffer was just created, quad indices should fit");

        Self {
            transforms,
            vertices,
            indices,
            quad_pos_vertices,
            quad_pos_uv_vertices,
            quad_indices,
        }
    }

    pub fn add(
        &mut self,
        transform: Transforms,
        vertices: DrawVertices,
    ) -> Result<DrawOffsets, BufferFull> {
        let transform_address = self.transforms.add(&[transform])?;
        let (vertices, indices) = match vertices {
            DrawVertices::CustomPosUvQuad(vertices) => (
                Some(self.vertices.add(vertices)?),
                self.quad_indices.clone(),
            ),
            DrawVertices::PosQuad => (
                Some(self.quad_pos_vertices.clone()),
                self.quad_indices.clone(),
            ),
            DrawVertices::PosUvQuad => (
                Some(self.quad_pos_uv_vertices.clone()),
                self.quad_indices.clone(),
            ),
            DrawVertices::None => (None, self.quad_indices.clone()),
        };
        let instance = (transform_address.start as usize / size_of::<Transforms>()) as u32;
        Ok(DrawOffsets {
            vertices,
            indices,
            instances: instance..instance + 1,
        })
    }

    pub fn copy_to(
        self,
        staging_belt: &mut StagingBelt,
        draw_encoder: &mut wgpu::CommandEncoder,
        chunk_buffers: &ChunkBuffers,
    ) {
        self.transforms
            .copy_to(staging_belt, draw_encoder, &chunk_buffers.transforms);
        self.vertices
            .copy_to(staging_belt, draw_encoder, &chunk_buffers.vertex_buffer);
        self.indices
            .copy_to(staging_belt, draw_encoder, &chunk_buffers.index_buffer);
    }
}
