use crate::bitmaps::BitmapSamplers;
use crate::descriptors::Quad;
use crate::globals::Globals;
use crate::mesh::BitmapBinds;
use crate::pipelines::Pipelines;
use crate::surface::Surface;
use crate::target::{RenderTarget, SwapChainTarget};
use crate::uniform_buffer::UniformBuffer;
use crate::utils::{create_buffer_with_data, format_list, get_backend_names, BufferDimensions};
use bytemuck::{Pod, Zeroable};
use descriptors::Descriptors;
use enum_map::Enum;
use once_cell::sync::OnceCell;
use ruffle_render::bitmap::BitmapHandle;
use ruffle_render::color_transform::ColorTransform;
use ruffle_render::tessellator::{Gradient as TessGradient, GradientType, Vertex as TessVertex};
pub use wgpu;

type Error = Box<dyn std::error::Error>;

#[macro_use]
mod utils;

mod bitmaps;
mod context3d;
mod globals;
mod pipelines;
pub mod target;
mod uniform_buffer;

pub mod backend;
#[cfg(feature = "clap")]
pub mod clap;
mod commands;
pub mod descriptors;
mod frame;
mod layouts;
mod mesh;
mod shaders;
mod surface;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum MaskState {
    NoMask,
    DrawMaskStencil,
    DrawMaskedContent,
    ClearMaskStencil,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Transforms {
    world_matrix: [[f32; 4]; 4],
    color_adjustments: ColorAdjustments,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TextureTransforms {
    u_matrix: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ColorAdjustments {
    mult_color: [f32; 4],
    add_color: [f32; 4],
}

impl From<ColorTransform> for ColorAdjustments {
    fn from(transform: ColorTransform) -> Self {
        Self {
            mult_color: transform.mult_rgba_normalized(),
            add_color: transform.add_rgba_normalized(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl From<TessVertex> for Vertex {
    fn from(vertex: TessVertex) -> Self {
        Self {
            position: [vertex.x, vertex.y],
            color: [
                f32::from(vertex.color.r) / 255.0,
                f32::from(vertex.color.g) / 255.0,
                f32::from(vertex.color.b) / 255.0,
                f32::from(vertex.color.a) / 255.0,
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GradientUniforms {
    colors: [[f32; 16]; 16],
    ratios: [[f32; 4]; 16],
    gradient_type: i32,
    num_colors: u32,
    repeat_mode: i32,
    interpolation: i32,
    focal_point: f32,
    _padding: [f32; 3],
}

impl From<TessGradient> for GradientUniforms {
    fn from(gradient: TessGradient) -> Self {
        let mut ratios = [[0.0; 4]; 16];
        let mut colors = [[0.0; 16]; 16];

        for i in 0..gradient.num_colors {
            ratios[i] = [gradient.ratios[i], 0.0, 0.0, 0.0];
            colors[i][0..4].copy_from_slice(&gradient.colors[i]);
        }

        Self {
            colors,
            ratios,
            gradient_type: match gradient.gradient_type {
                GradientType::Linear => 0,
                GradientType::Radial => 1,
                GradientType::Focal => 2,
            },
            num_colors: gradient.num_colors as u32,
            repeat_mode: match gradient.repeat_mode {
                swf::GradientSpread::Pad => 0,
                swf::GradientSpread::Repeat => 1,
                swf::GradientSpread::Reflect => 2,
            },
            interpolation: (gradient.interpolation == swf::GradientInterpolation::LinearRgb) as i32,
            focal_point: gradient.focal_point.to_f32(),
            _padding: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct GradientStorage {
    colors: [[f32; 4]; 16],
    ratios: [f32; 16],
    gradient_type: i32,
    num_colors: u32,
    repeat_mode: i32,
    interpolation: i32,
    focal_point: f32,
    _padding: [f32; 3],
}

impl From<TessGradient> for GradientStorage {
    fn from(gradient: TessGradient) -> Self {
        let mut ratios = [0.0; 16];
        let mut colors = [[0.0; 4]; 16];
        ratios[..gradient.num_colors].copy_from_slice(&gradient.ratios[..gradient.num_colors]);
        colors[..gradient.num_colors].copy_from_slice(&gradient.colors[..gradient.num_colors]);

        Self {
            colors,
            ratios,
            gradient_type: match gradient.gradient_type {
                GradientType::Linear => 0,
                GradientType::Radial => 1,
                GradientType::Focal => 2,
            },
            num_colors: gradient.num_colors as u32,
            repeat_mode: match gradient.repeat_mode {
                swf::GradientSpread::Pad => 0,
                swf::GradientSpread::Repeat => 1,
                swf::GradientSpread::Reflect => 2,
            },
            interpolation: (gradient.interpolation == swf::GradientInterpolation::LinearRgb) as i32,
            focal_point: gradient.focal_point.to_f32(),
            _padding: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    texture: wgpu::Texture,
    bind_linear: OnceCell<BitmapBinds>,
    bind_nearest: OnceCell<BitmapBinds>,
    texture_offscreen: Option<TextureOffscreen>,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn bind_group(
        &self,
        smoothed: bool,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        quad: &Quad,
        handle: BitmapHandle,
        samplers: &BitmapSamplers,
    ) -> &BitmapBinds {
        let bind = match smoothed {
            true => &self.bind_linear,
            false => &self.bind_nearest,
        };
        bind.get_or_init(|| {
            BitmapBinds::new(
                &device,
                &layout,
                samplers.get_sampler(false, smoothed),
                &quad.texture_transforms,
                self.texture.create_view(&Default::default()),
                create_debug_label!("Bitmap {} bind group (smoothed: {})", handle.0, smoothed),
            )
        })
    }
}

#[derive(Debug)]
struct TextureOffscreen {
    buffer: wgpu::Buffer,
    buffer_dimensions: BufferDimensions,
    surface: Surface,
}
