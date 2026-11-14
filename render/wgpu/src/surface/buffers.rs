use crate::buffer_builder::{BufferBuilder, BufferFull};
use crate::descriptors::{Descriptors, Quad};
use crate::{PosColorVertex, PosUvVertex, PosVertex, Transforms};
use ruffle_render::transform::Transform;
use std::ops::Range;
use wgpu::util::StagingBelt;

// [NA] The higher this value is, the more memory we'll reserve - ideally we can dynamically allocate this
// but for now this feels reasonable
const MAX_TRANSFORMS_PER_CHUNK: u64 = 512;

pub struct ChunkBuffers {
    pub transforms: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
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
            label: create_debug_label!("Chunk transform buffer").as_deref(),
            size: transforms_per_draw(&descriptors.limits) * size_of::<Transforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let vertex_buffer = descriptors.device.create_buffer(&wgpu::BufferDescriptor {
            label: create_debug_label!("Chunk vertex buffer").as_deref(),
            size: 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: create_debug_label!("Chunk transforms bind group").as_deref(),
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
        }
    }

    pub fn ensure_vertex_buffer_fit(&mut self, device: &wgpu::Device, required_size: u64) {
        if self.vertex_buffer.size() < required_size {
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: create_debug_label!("Chunk vertex buffer").as_deref(),
                size: required_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
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
        indices: Range<u32>,
    },
}

#[derive(Clone, Debug)]
pub struct DrawOffsets {
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: wgpu::Buffer,
    pub vertices: Range<wgpu::BufferAddress>,
    pub indices: Range<u32>,
    pub instances: Range<u32>,
}

impl DrawOffsets {
    pub fn try_merge(&mut self, other: &DrawOffsets) -> bool {
        if self.vertex_buffer == other.vertex_buffer
            && self.index_buffer == other.index_buffer
            && self.vertices == other.vertices
            && self.indices == other.indices
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
    quad_pos_vertices: Range<wgpu::BufferAddress>,
    quad_pos_color_vertices: Range<wgpu::BufferAddress>,
    quad_pos_uv_vertices: Range<wgpu::BufferAddress>,
}

impl ChunkBufferBuilder {
    pub fn new(chunk_buffers: &ChunkBuffers, limits: &wgpu::Limits) -> Self {
        let mut transforms = BufferBuilder::new(limits, 0);
        transforms.set_buffer_limit(chunk_buffers.transforms.size());
        let mut vertices = BufferBuilder::new(limits, 0);

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

        Self {
            transforms,
            vertices,
            quad_pos_vertices,
            quad_pos_color_vertices,
            quad_pos_uv_vertices,
        }
    }

    pub fn add_transform(&mut self, transform: &Transform) -> Result<u32, BufferFull> {
        let instance = Transforms {
            world_matrix: [
                [transform.matrix.a, transform.matrix.b, 0.0, 0.0],
                [transform.matrix.c, transform.matrix.d, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [
                    transform.matrix.tx.to_pixels() as f32,
                    transform.matrix.ty.to_pixels() as f32,
                    transform.tz as f32,
                    1.0,
                ],
            ],
            mult_color: transform.color_transform.mult_rgba_normalized(),
            add_color: transform.color_transform.add_rgba_normalized(),
        };
        let address = self.transforms.add(&[instance])?;
        Ok((address.start as usize / size_of::<Transforms>()) as u32)
    }

    pub fn add_draw<'a>(
        &mut self,
        quad: &Quad,
        instance: u32,
        vertices: DrawVertices<'a>,
    ) -> Result<DrawOffsets, (BufferFull, DrawVertices<'a>)> {
        let instances = instance..instance + 1;
        Ok(match vertices {
            DrawVertices::CustomPosUvQuad(vertex_bytes) => DrawOffsets {
                vertex_buffer: None,
                index_buffer: quad.index_buffer.clone(),
                vertices: self
                    .vertices
                    .add(vertex_bytes)
                    .map_err(|_| (BufferFull, vertices))?,
                indices: quad.indices_quad.clone(),
                instances,
            },
            DrawVertices::PosQuad => DrawOffsets {
                vertex_buffer: None,
                index_buffer: quad.index_buffer.clone(),
                vertices: self.quad_pos_vertices.clone(),
                indices: quad.indices_quad.clone(),
                instances,
            },
            DrawVertices::PosColorQuad => DrawOffsets {
                vertex_buffer: None,
                index_buffer: quad.index_buffer.clone(),
                vertices: self.quad_pos_color_vertices.clone(),
                indices: quad.indices_quad.clone(),
                instances,
            },
            DrawVertices::PosUvQuad => DrawOffsets {
                vertex_buffer: None,
                index_buffer: quad.index_buffer.clone(),
                vertices: self.quad_pos_uv_vertices.clone(),
                indices: quad.indices_quad.clone(),
                instances,
            },
            DrawVertices::Line => DrawOffsets {
                vertex_buffer: None,
                index_buffer: quad.index_buffer.clone(),
                vertices: self.quad_pos_color_vertices.clone(),
                indices: quad.indices_line.clone(),
                instances,
            },
            DrawVertices::RectLine => DrawOffsets {
                vertex_buffer: None,
                index_buffer: quad.index_buffer.clone(),
                vertices: self.quad_pos_color_vertices.clone(),
                indices: quad.indices_line_rect.clone(),
                instances,
            },
            DrawVertices::CustomBuffers {
                vertex_buffer,
                vertex_range,
                index_buffer,
                indices,
            } => DrawOffsets {
                vertex_buffer: Some(vertex_buffer),
                index_buffer,
                vertices: vertex_range,
                indices,
                instances,
            },
        })
    }

    pub fn copy_to(
        self,
        staging_belt: &mut StagingBelt,
        device: &wgpu::Device,
        draw_encoder: &mut wgpu::CommandEncoder,
        chunk_buffers: &mut ChunkBuffers,
    ) {
        self.transforms
            .copy_to(staging_belt, draw_encoder, &chunk_buffers.transforms);
        chunk_buffers.ensure_vertex_buffer_fit(device, self.vertices.len() as u64);
        self.vertices
            .copy_to(staging_belt, draw_encoder, &chunk_buffers.vertex_buffer);
    }
}
