use crate::buffer_builder::{BufferBuilder, BufferFull};
use crate::descriptors::Descriptors;
use crate::{PosColorVertex, PosUvVertex, PosVertex, Transforms};
use std::ops::Range;
use wgpu::util::StagingBelt;

// [NA] The higher this value is, the more memory we'll reserve - ideally we can dynamically allocate this
// but for now this feels reasonable
const MAX_TRANSFORMS_PER_CHUNK: u64 = 512;

pub struct ChunkBuffers {
    pub transforms: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub fn transforms_per_draw(limits: &wgpu::Limits) -> u64 {
    let maximum = limits
        .max_uniform_buffer_binding_size
        .min(limits.max_buffer_size)
        / size_of::<Transforms>() as u64;
    maximum.min(MAX_TRANSFORMS_PER_CHUNK)
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
            size: descriptors.limits.max_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: descriptors.limits.max_buffer_size,
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

#[derive(Clone, Debug)]
pub enum DrawVertices<'a> {
    CustomPosUvQuad(&'a [u8]),
    PosUvQuad,
    PosColorQuad,
    PosQuad,
    Line,
    RectLine,
    CustomBuffers {
        vertex_buffer: wgpu::Buffer,
        vertex_range: Range<wgpu::BufferAddress>,
        index_buffer: wgpu::Buffer,
        index_range: Range<wgpu::BufferAddress>,
        index_count: u32,
    },
}

#[derive(Clone, Debug)]
pub struct DrawOffsets {
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub vertices: Range<wgpu::BufferAddress>,
    pub indices: Range<wgpu::BufferAddress>,
    pub instances: Range<u32>,
    pub index_count: u32,
}

impl DrawOffsets {
    pub fn try_merge(&mut self, other: &DrawOffsets) -> bool {
        if self.vertex_buffer == other.vertex_buffer
            && self.index_buffer == other.index_buffer
            && self.vertices == other.vertices
            && self.indices == other.indices
            && self.index_count == other.index_count
            && self.instances.end == other.instances.start
        {
            // It's the same exact draw and contiguous instances, let's just engulf the instances
            self.instances.end = other.instances.end;
            return true;
        }

        // [NA] We could, in theory, get smart and try to join contiguous meshes. Is it worth it? It would potentially harm the instancing.
        false
    }
}

pub struct ChunkBufferBuilder {
    transforms: BufferBuilder,
    vertices: BufferBuilder,
    indices: BufferBuilder,
    quad_pos_vertices: Range<wgpu::BufferAddress>,
    quad_pos_color_vertices: Range<wgpu::BufferAddress>,
    quad_pos_uv_vertices: Range<wgpu::BufferAddress>,
    quad_indices: Range<wgpu::BufferAddress>,
    line_indices: Range<wgpu::BufferAddress>,
    rect_line_indices: Range<wgpu::BufferAddress>,
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
        let quad_pos_color_vertices = vertices
            .add(&[
                PosColorVertex {
                    position: [0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                PosColorVertex {
                    position: [1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                PosColorVertex {
                    position: [1.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                PosColorVertex {
                    position: [0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ])
            .expect("Buffer was just created, quad vertices should fit");
        let quad_indices = indices
            .add(&[0u32, 1, 2, 0, 2, 3])
            .expect("Buffer was just created, quad indices should fit");
        let line_indices = indices
            .add(&[0u32, 1])
            .expect("Buffer was just created, line indices should fit");
        let rect_line_indices = indices
            .add(&[0u32, 1, 2, 3, 0])
            .expect("Buffer was just created, rect line indices should fit");

        Self {
            transforms,
            vertices,
            indices,
            quad_pos_vertices,
            quad_pos_color_vertices,
            quad_pos_uv_vertices,
            quad_indices,
            line_indices,
            rect_line_indices,
        }
    }

    pub fn add<'a>(
        &mut self,
        transform: Transforms,
        vertices: DrawVertices<'a>,
    ) -> Result<DrawOffsets, (BufferFull, DrawVertices<'a>)> {
        let transform_address = match self.transforms.add(&[transform]) {
            Ok(address) => address,
            Err(e) => return Err((e, vertices)),
        };
        let instance = (transform_address.start as usize / size_of::<Transforms>()) as u32;
        let instances = instance..instance + 1;
        Ok(match vertices {
            DrawVertices::CustomPosUvQuad(vertex_bytes) => DrawOffsets {
                vertex_buffer: None,
                index_buffer: None,
                vertices: self
                    .vertices
                    .add(vertex_bytes)
                    .map_err(|_| (BufferFull, vertices))?,
                indices: self.quad_indices.clone(),
                instances,
                index_count: 6,
            },
            DrawVertices::PosQuad => DrawOffsets {
                vertex_buffer: None,
                index_buffer: None,
                vertices: self.quad_pos_vertices.clone(),
                indices: self.quad_indices.clone(),
                instances,
                index_count: 6,
            },
            DrawVertices::PosColorQuad => DrawOffsets {
                vertex_buffer: None,
                index_buffer: None,
                vertices: self.quad_pos_color_vertices.clone(),
                indices: self.quad_indices.clone(),
                instances,
                index_count: 6,
            },
            DrawVertices::PosUvQuad => DrawOffsets {
                vertex_buffer: None,
                index_buffer: None,
                vertices: self.quad_pos_uv_vertices.clone(),
                indices: self.quad_indices.clone(),
                instances,
                index_count: 6,
            },
            DrawVertices::Line => DrawOffsets {
                vertex_buffer: None,
                index_buffer: None,
                vertices: self.quad_pos_color_vertices.clone(),
                indices: self.line_indices.clone(),
                instances,
                index_count: 2,
            },
            DrawVertices::RectLine => DrawOffsets {
                vertex_buffer: None,
                index_buffer: None,
                vertices: self.quad_pos_color_vertices.clone(),
                indices: self.rect_line_indices.clone(),
                instances,
                index_count: 5,
            },
            DrawVertices::CustomBuffers {
                vertex_buffer,
                vertex_range,
                index_buffer,
                index_range,
                index_count,
            } => DrawOffsets {
                vertex_buffer: Some(vertex_buffer),
                index_buffer: Some(index_buffer),
                vertices: vertex_range,
                indices: index_range,
                instances,
                index_count,
            },
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
