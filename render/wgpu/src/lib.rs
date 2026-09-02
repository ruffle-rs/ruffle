// Remove this when we decide on how to handle multithreaded rendering (especially on wasm)
#![allow(clippy::arc_with_non_send_sync)]

use crate::backend::ActiveFrame;
use crate::bitmaps::BitmapSamplers;
use crate::buffer_pool::{BufferPool, PoolEntry};
use crate::mesh::BitmapBinds;
use crate::pipelines::Pipelines;
use crate::target::{RenderTarget, SwapChainTarget};
use crate::utils::{
    BufferDimensions, capture_image, create_buffer_with_data, format_list, get_backend_names,
};
use bytemuck::{Pod, Zeroable};
use descriptors::Descriptors;
use enum_map::Enum;
use ruffle_render::backend::RawTexture;
use ruffle_render::bitmap::{BitmapHandle, BitmapHandleImpl, PixelRegion, SyncHandle};
use ruffle_render::shape_utils::GradientType;
use ruffle_render::tessellator::{Gradient as TessGradient, Vertex as TessVertex};
use std::any::Any;
use std::cell::{Cell, OnceCell};
use std::sync::Arc;
use swf::GradientSpread;
pub use wgpu;
pub use wgpu_profiler;

type Error = Box<dyn std::error::Error>;

#[macro_use]
pub mod utils;

mod bitmaps;
mod context3d;
mod globals;
mod pipelines;
mod pixel_bender;
pub mod target;

pub mod backend;
mod blend;
mod buffer_builder;
mod buffer_pool;
#[cfg(feature = "clap")]
pub mod clap;
pub mod descriptors;
mod dynamic_transforms;
mod filters;
mod layouts;
mod mesh;
mod shaders;
mod surface;

impl BitmapHandleImpl for Texture {}

pub fn as_texture(handle: &BitmapHandle) -> &Texture {
    <dyn Any>::downcast_ref(&*handle.0).unwrap()
}

pub fn raw_texture_as_texture(handle: &dyn RawTexture) -> &wgpu::Texture {
    <dyn Any>::downcast_ref(handle).unwrap()
}

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
    mult_color: [f32; 4],
    add_color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct PosVertex {
    position: [f32; 2],
}

impl From<TessVertex> for PosVertex {
    fn from(vertex: TessVertex) -> Self {
        Self {
            position: [vertex.x, vertex.y],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct PosUvVertex {
    position: [f32; 2],
    uv: [f32; 3],
}

impl PosUvVertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32, t: f32) -> Self {
        let position = [x, y];
        let uv = [u, v, t];
        Self { position, uv }
    }

    pub fn from_tessellator(vertex: TessVertex, texture_matrix: &[[f32; 3]; 3]) -> Self {
        let position = [vertex.x, vertex.y];
        let uv = Self::transform_uv(texture_matrix, vertex.x, vertex.y);
        Self { position, uv }
    }

    fn transform_uv(matrix: &[[f32; 3]; 3], x: f32, y: f32) -> [f32; 3] {
        [
            matrix[0][0] * x + matrix[1][0] * y + matrix[2][0],
            matrix[0][1] * x + matrix[1][1] * y + matrix[2][1],
            1.0,
        ]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct PosColorVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl From<TessVertex> for PosColorVertex {
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
    focal_point: f32,
    interpolation: i32,
    shape: i32,
    repeat: i32,
}

impl From<TessGradient> for GradientUniforms {
    fn from(gradient: TessGradient) -> Self {
        Self {
            focal_point: gradient.focal_point.to_f32().clamp(-0.98, 0.98),
            interpolation: (gradient.interpolation == swf::GradientInterpolation::LinearRgb) as i32,
            shape: match gradient.gradient_type {
                GradientType::Linear => 1,
                GradientType::Radial => 2,
                GradientType::Focal => 3,
            },
            repeat: match gradient.repeat_mode {
                GradientSpread::Pad => 1,
                GradientSpread::Reflect => 2,
                GradientSpread::Repeat => 3,
            },
        }
    }
}

#[derive(Debug)]
pub enum QueueSyncHandle {
    AlreadyCopied {
        index: Option<wgpu::SubmissionIndex>,
        buffer: PoolEntry<wgpu::Buffer, BufferDimensions>,
        copy_dimensions: BufferDimensions,
        descriptors: Arc<Descriptors>,
    },
    NotCopied {
        handle: BitmapHandle,
        copy_area: PixelRegion,
        descriptors: Arc<Descriptors>,
        pool: Arc<BufferPool<wgpu::Buffer, BufferDimensions>>,
    },
}

impl SyncHandle for QueueSyncHandle {}

impl QueueSyncHandle {
    pub fn capture<R, F: FnOnce(&[u8], u32) -> R>(
        self,
        with_rgba: F,
        frame: &mut ActiveFrame,
    ) -> R {
        match self {
            QueueSyncHandle::AlreadyCopied {
                index,
                buffer,
                copy_dimensions,
                descriptors,
            } => capture_image(
                &descriptors.device,
                &buffer,
                &copy_dimensions,
                index,
                with_rgba,
            ),
            QueueSyncHandle::NotCopied {
                handle,
                copy_area,
                descriptors,
                pool,
            } => {
                let texture = as_texture(&handle);

                let buffer_dimensions = BufferDimensions::new(
                    copy_area.width() as usize,
                    copy_area.height() as usize,
                    texture.texture.format(),
                );

                let buffer = pool.take(&descriptors, buffer_dimensions.clone());
                frame.command_encoder.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: copy_area.x_min,
                            y: copy_area.y_min,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyBufferInfo {
                        buffer: &buffer,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(buffer_dimensions.padded_bytes_per_row),
                            rows_per_image: None,
                        },
                    },
                    wgpu::Extent3d {
                        width: copy_area.width(),
                        height: copy_area.height(),
                        depth_or_array_layers: 1,
                    },
                );
                let index = frame.submit_direct(&descriptors);

                let image = capture_image(
                    &descriptors.device,
                    &buffer,
                    &buffer_dimensions,
                    Some(index),
                    with_rgba,
                );

                // After we've read pixels from a texture enough times, we'll store this buffer so that
                // future reads will be faster (it'll copy as part of the draw process instead)
                texture
                    .copy_count
                    .set(texture.copy_count.get().saturating_add(1));

                image
            }
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    repeating_linear: OnceCell<BitmapBinds>,
    repeating_nearest: OnceCell<BitmapBinds>,
    clamped_linear: OnceCell<BitmapBinds>,
    clamped_nearest: OnceCell<BitmapBinds>,
    copy_count: Cell<u8>,
}

/// Bytes of texture memory Ruffle itself has asked for and not yet released
/// - bitmap textures, cached display objects, pooled render targets - and
/// how many such textures there are. Not every backend can report texture
/// memory, so this is counted here for the memory report.
static TEXTURE_BYTES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static TEXTURE_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Approximate memory of a texture: its pixels at the format's block size.
pub(crate) fn texture_bytes(texture: &wgpu::Texture) -> usize {
    let size = texture.size();
    let bytes_per_block = texture.format().block_copy_size(None).unwrap_or(4) as usize;
    size.width as usize
        * size.height as usize
        * size.depth_or_array_layers as usize
        * bytes_per_block
}

pub(crate) fn track_texture_created(texture: &wgpu::Texture) {
    use std::sync::atomic::Ordering;
    TEXTURE_BYTES.fetch_add(texture_bytes(texture), Ordering::Relaxed);
    TEXTURE_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn track_texture_dropped(texture: &wgpu::Texture) {
    use std::sync::atomic::Ordering;
    TEXTURE_BYTES.fetch_sub(texture_bytes(texture), Ordering::Relaxed);
    TEXTURE_COUNT.fetch_sub(1, Ordering::Relaxed);
}

/// `(textures alive, their bytes)` as tracked by Ruffle.
pub fn tracked_texture_totals() -> (usize, usize) {
    use std::sync::atomic::Ordering;
    (
        TEXTURE_COUNT.load(Ordering::Relaxed),
        TEXTURE_BYTES.load(Ordering::Relaxed),
    )
}

impl Drop for Texture {
    fn drop(&mut self) {
        track_texture_dropped(&self.texture);
    }
}

impl Texture {
    pub(crate) fn new(texture: wgpu::Texture) -> Self {
        track_texture_created(&texture);
        Self {
            texture,
            repeating_linear: Default::default(),
            repeating_nearest: Default::default(),
            clamped_linear: Default::default(),
            clamped_nearest: Default::default(),
            copy_count: Cell::new(0),
        }
    }

    pub fn bind_group(
        &self,
        repeating: bool,
        smoothed: bool,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        handle: BitmapHandle,
        samplers: &BitmapSamplers,
    ) -> &BitmapBinds {
        let bind = match (repeating, smoothed) {
            (true, true) => &self.repeating_linear,
            (true, false) => &self.repeating_nearest,
            (false, true) => &self.clamped_linear,
            (false, false) => &self.clamped_nearest,
        };
        bind.get_or_init(|| {
            BitmapBinds::new(
                device,
                layout,
                samplers.get_sampler(repeating, smoothed),
                self.texture.create_view(&Default::default()),
                create_debug_label!("Bitmap {:?} bind group (smoothed: {})", handle.0, smoothed),
            )
        })
    }
}
