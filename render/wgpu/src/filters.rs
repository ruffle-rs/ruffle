mod bevel;
mod blur;
mod color_matrix;
mod displacement_map;
mod drop_shadow;
mod glow;
mod shader;

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::bevel::BevelFilter;
use crate::filters::blur::BlurFilter;
use crate::filters::color_matrix::ColorMatrixFilter;
use crate::filters::displacement_map::DisplacementMapFilter;
use crate::filters::drop_shadow::DropShadowFilter;
use crate::filters::glow::GlowFilter;
use crate::filters::shader::ShaderFilter;
use crate::surface::target::CommandTarget;
use bytemuck::{Pod, Zeroable};
use ruffle_render::filters::Filter;
use wgpu::util::StagingBelt;
use wgpu::vertex_attr_array;

#[derive(Debug)]
pub struct FilterSource<'a> {
    pub texture: &'a wgpu::Texture,
    pub point: (u32, u32),
    pub size: (u32, u32),
}

impl<'a> FilterSource<'a> {
    pub fn for_entire_texture(texture: &'a wgpu::Texture) -> Self {
        Self {
            texture,
            point: (0, 0),
            size: (texture.width(), texture.height()),
        }
    }

    pub fn vertices(&self) -> [FilterVertex; 4] {
        let source_width = self.texture.width() as f32;
        let source_height = self.texture.height() as f32;
        let left = self.point.0;
        let top = self.point.1;
        let right = left + self.size.0;
        let bottom = top + self.size.1;
        [
            FilterVertex {
                position: [0.0, 0.0],
                uv: [left as f32 / source_width, top as f32 / source_height],
            },
            FilterVertex {
                position: [1.0, 0.0],
                uv: [right as f32 / source_width, top as f32 / source_height],
            },
            FilterVertex {
                position: [1.0, 1.0],
                uv: [right as f32 / source_width, bottom as f32 / source_height],
            },
            FilterVertex {
                position: [0.0, 1.0],
                uv: [left as f32 / source_width, bottom as f32 / source_height],
            },
        ]
    }

    pub fn vertices_with_blur_offset(&self, blur_offset: (f32, f32)) -> [FilterVertexWithBlur; 4] {
        let source_width = self.texture.width() as f32;
        let source_height = self.texture.height() as f32;
        let source_left = self.point.0;
        let source_top = self.point.1;
        let source_right = source_left + self.size.0;
        let source_bottom = source_top + self.size.1;
        [
            FilterVertexWithBlur {
                position: [0.0, 0.0],
                source_uv: [
                    source_left as f32 / source_width,
                    source_top as f32 / source_height,
                ],
                blur_uv: [
                    (source_left as f32 + blur_offset.0) / source_width,
                    (source_top as f32 + blur_offset.1) / source_height,
                ],
            },
            FilterVertexWithBlur {
                position: [1.0, 0.0],
                source_uv: [
                    source_right as f32 / source_width,
                    source_top as f32 / source_height,
                ],
                blur_uv: [
                    (source_right as f32 + blur_offset.0) / source_width,
                    (source_top as f32 + blur_offset.1) / source_height,
                ],
            },
            FilterVertexWithBlur {
                position: [1.0, 1.0],
                source_uv: [
                    source_right as f32 / source_width,
                    source_bottom as f32 / source_height,
                ],
                blur_uv: [
                    (source_right as f32 + blur_offset.0) / source_width,
                    (source_bottom as f32 + blur_offset.1) / source_height,
                ],
            },
            FilterVertexWithBlur {
                position: [0.0, 1.0],
                source_uv: [
                    source_left as f32 / source_width,
                    source_bottom as f32 / source_height,
                ],
                blur_uv: [
                    (source_left as f32 + blur_offset.0) / source_width,
                    (source_bottom as f32 + blur_offset.1) / source_height,
                ],
            },
        ]
    }

    pub fn vertices_with_highlight_and_shadow(
        &self,
        blur_offset: (f32, f32),
    ) -> [FilterVertexWithDoubleBlur; 4] {
        let source_width = self.texture.width() as f32;
        let source_height = self.texture.height() as f32;
        let source_left = self.point.0 as f32;
        let source_top = self.point.1 as f32;
        let source_right = (self.point.0 + self.size.0) as f32;
        let source_bottom = (self.point.1 + self.size.1) as f32;
        [
            FilterVertexWithDoubleBlur {
                position: [0.0, 0.0],
                source_uv: [source_left / source_width, source_top / source_height],
                blur_uv_left: [
                    (source_left + blur_offset.0) / source_width,
                    (source_top + blur_offset.1) / source_height,
                ],
                blur_uv_right: [
                    (source_left - blur_offset.0) / source_width,
                    (source_top - blur_offset.1) / source_height,
                ],
            },
            FilterVertexWithDoubleBlur {
                position: [1.0, 0.0],
                source_uv: [source_right / source_width, source_top / source_height],
                blur_uv_left: [
                    (source_right + blur_offset.0) / source_width,
                    (source_top + blur_offset.1) / source_height,
                ],
                blur_uv_right: [
                    (source_right - blur_offset.0) / source_width,
                    (source_top - blur_offset.1) / source_height,
                ],
            },
            FilterVertexWithDoubleBlur {
                position: [1.0, 1.0],
                source_uv: [source_right / source_width, source_bottom / source_height],
                blur_uv_left: [
                    (source_right + blur_offset.0) / source_width,
                    (source_bottom + blur_offset.1) / source_height,
                ],
                blur_uv_right: [
                    (source_right - blur_offset.0) / source_width,
                    (source_bottom - blur_offset.1) / source_height,
                ],
            },
            FilterVertexWithDoubleBlur {
                position: [0.0, 1.0],
                source_uv: [source_left / source_width, source_bottom / source_height],
                blur_uv_left: [
                    (source_left + blur_offset.0) / source_width,
                    (source_bottom + blur_offset.1) / source_height,
                ],
                blur_uv_right: [
                    (source_left - blur_offset.0) / source_width,
                    (source_bottom - blur_offset.1) / source_height,
                ],
            },
        ]
    }
}

pub struct Filters {
    pub blur: BlurFilter,
    pub color_matrix: ColorMatrixFilter,
    pub shader: ShaderFilter,
    pub glow: GlowFilter,
    pub bevel: BevelFilter,
    pub displacement_map: DisplacementMapFilter,
}

impl Filters {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            blur: BlurFilter::new(device),
            color_matrix: ColorMatrixFilter::new(device),
            shader: ShaderFilter::new(),
            glow: GlowFilter::new(device),
            bevel: BevelFilter::new(device),
            displacement_map: DisplacementMapFilter::new(device),
        }
    }

    pub fn apply(
        &self,
        descriptors: &Descriptors,
        draw_encoder: &mut wgpu::CommandEncoder,
        texture_pool: &mut TexturePool,
        staging_belt: &mut StagingBelt,
        source: FilterSource,
        filter: Filter,
    ) -> CommandTarget {
        let target = match filter {
            Filter::ColorMatrixFilter(filter) => Some(descriptors.filters.color_matrix.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                staging_belt,
                &source,
                &filter,
            )),
            Filter::BlurFilter(filter) => descriptors.filters.blur.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                staging_belt,
                &source,
                &filter,
            ),
            Filter::ShaderFilter(shader) => Some(descriptors.filters.shader.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                &source,
                shader,
            )),
            Filter::GlowFilter(filter) => Some(descriptors.filters.glow.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                staging_belt,
                &source,
                &filter,
                &self.blur,
                (0.0, 0.0),
            )),
            Filter::DropShadowFilter(filter) => Some(DropShadowFilter::apply(
                descriptors,
                texture_pool,
                draw_encoder,
                staging_belt,
                &source,
                &filter,
                &self.blur,
                &self.glow,
            )),
            Filter::BevelFilter(filter) => Some(descriptors.filters.bevel.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                staging_belt,
                &source,
                &filter,
                &self.blur,
            )),
            Filter::DisplacementMapFilter(filter) => descriptors.filters.displacement_map.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                staging_belt,
                &source,
                &filter,
            ),
            filter => {
                static WARNED_FILTERS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
                let name = match filter {
                    Filter::GradientGlowFilter(_) => "GradientGlowFilter",
                    Filter::GradientBevelFilter(_) => "GradientBevelFilter",
                    Filter::ConvolutionFilter(_) => "ConvolutionFilter",
                    Filter::ColorMatrixFilter(_)
                    | Filter::BlurFilter(_)
                    | Filter::GlowFilter(_)
                    | Filter::DropShadowFilter(_)
                    | Filter::BevelFilter(_)
                    | Filter::DisplacementMapFilter(_)
                    | Filter::ShaderFilter(_) => unreachable!(),
                };
                // Only warn once per filter type
                if WARNED_FILTERS
                    .get_or_init(Default::default)
                    .lock()
                    .unwrap()
                    .insert(name)
                {
                    tracing::warn!("Unsupported filter {filter:?}");
                }
                None
            }
        };

        let target = target.unwrap_or_else(|| {
            // Apply a default color matrix - it's essentially a blit
            // TODO: Not need to do this.
            descriptors.filters.color_matrix.apply(
                descriptors,
                texture_pool,
                draw_encoder,
                staging_belt,
                &source,
                &Default::default(),
            )
        });

        // We're about to perform a copy, so make sure that we've applied
        // a clear (in case no other draw commands were issued, we still need
        // the background clear color applied)
        target.ensure_cleared(draw_encoder);
        target
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FilterVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

pub const VERTEX_BUFFERS_DESCRIPTION_FILTERS: [wgpu::VertexBufferLayout; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<FilterVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ],
    }];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FilterVertexWithBlur {
    pub position: [f32; 2],
    pub source_uv: [f32; 2],
    pub blur_uv: [f32; 2],
}

pub const VERTEX_BUFFERS_DESCRIPTION_FILTERS_WITH_BLUR: [wgpu::VertexBufferLayout; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<FilterVertexWithBlur>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x2,
        ],
    }];

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FilterVertexWithDoubleBlur {
    pub position: [f32; 2],
    pub source_uv: [f32; 2],
    pub blur_uv_left: [f32; 2],
    pub blur_uv_right: [f32; 2],
}

pub const VERTEX_BUFFERS_DESCRIPTION_FILTERS_WITH_DOUBLE_BLUR: [wgpu::VertexBufferLayout; 1] =
    [wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<FilterVertexWithDoubleBlur>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x2,
            3 => Float32x2,
        ],
    }];
