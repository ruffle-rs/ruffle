// Remove this when we decide on how to handle multithreaded rendering (especially on wasm)
#![allow(clippy::arc_with_non_send_sync)]

use crate::backend::ActiveFrame;
use crate::bitmaps::BitmapSamplers;
use crate::buffer_pool::{BufferPool, PoolEntry};
use crate::descriptors::Quad;
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
use std::sync::atomic::{AtomicU64, Ordering};
use swf::GradientSpread;
pub use wgpu;

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
struct TextureTransforms {
    u_matrix: [[f32; 4]; 4],
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
    bind_linear: OnceCell<BitmapBinds>,
    bind_nearest: OnceCell<BitmapBinds>,
    copy_count: Cell<u8>,
    _diagnostic_registration: Option<TextureDiagnosticRegistration>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum TextureDiagnosticKind {
    RegisteredBitmap,
    EmptyTexture,
    PixelBenderTemporary,
}

#[derive(Debug, Default)]
pub(crate) struct TextureDiagnosticCounters {
    live_registered_bitmaps: AtomicU64,
    live_registered_bitmap_bytes: AtomicU64,
    live_empty_textures: AtomicU64,
    live_empty_texture_bytes: AtomicU64,
    live_pixelbender_temporaries: AtomicU64,
    live_pixelbender_temporary_bytes: AtomicU64,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TextureDiagnosticSnapshot {
    pub live_registered_bitmaps: u64,
    pub live_registered_bitmap_bytes: u64,
    pub live_empty_textures: u64,
    pub live_empty_texture_bytes: u64,
    pub live_pixelbender_temporaries: u64,
    pub live_pixelbender_temporary_bytes: u64,
}

impl TextureDiagnosticCounters {
    pub fn register(
        self: &Arc<Self>,
        kind: TextureDiagnosticKind,
        bytes: u64,
    ) -> TextureDiagnosticRegistration {
        self.add(kind, bytes);
        TextureDiagnosticRegistration {
            counters: self.clone(),
            kind,
            bytes,
        }
    }

    pub fn snapshot(&self) -> TextureDiagnosticSnapshot {
        TextureDiagnosticSnapshot {
            live_registered_bitmaps: self.live_registered_bitmaps.load(Ordering::Relaxed),
            live_registered_bitmap_bytes: self.live_registered_bitmap_bytes.load(Ordering::Relaxed),
            live_empty_textures: self.live_empty_textures.load(Ordering::Relaxed),
            live_empty_texture_bytes: self.live_empty_texture_bytes.load(Ordering::Relaxed),
            live_pixelbender_temporaries: self.live_pixelbender_temporaries.load(Ordering::Relaxed),
            live_pixelbender_temporary_bytes: self
                .live_pixelbender_temporary_bytes
                .load(Ordering::Relaxed),
        }
    }

    fn add(&self, kind: TextureDiagnosticKind, bytes: u64) {
        let (count, byte_count) = self.counters(kind);
        count.fetch_add(1, Ordering::Relaxed);
        byte_count.fetch_add(bytes, Ordering::Relaxed);
    }

    fn remove(&self, kind: TextureDiagnosticKind, bytes: u64) {
        let (count, byte_count) = self.counters(kind);
        count.fetch_sub(1, Ordering::Relaxed);
        byte_count.fetch_sub(bytes, Ordering::Relaxed);
    }

    fn counters(&self, kind: TextureDiagnosticKind) -> (&AtomicU64, &AtomicU64) {
        match kind {
            TextureDiagnosticKind::RegisteredBitmap => (
                &self.live_registered_bitmaps,
                &self.live_registered_bitmap_bytes,
            ),
            TextureDiagnosticKind::EmptyTexture => {
                (&self.live_empty_textures, &self.live_empty_texture_bytes)
            }
            TextureDiagnosticKind::PixelBenderTemporary => (
                &self.live_pixelbender_temporaries,
                &self.live_pixelbender_temporary_bytes,
            ),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TextureDiagnosticRegistration {
    counters: Arc<TextureDiagnosticCounters>,
    kind: TextureDiagnosticKind,
    bytes: u64,
}

impl Drop for TextureDiagnosticRegistration {
    fn drop(&mut self) {
        self.counters.remove(self.kind, self.bytes);
    }
}

impl Texture {
    pub(crate) fn new(texture: wgpu::Texture) -> Self {
        Self {
            texture,
            bind_linear: Default::default(),
            bind_nearest: Default::default(),
            copy_count: Cell::new(0),
            _diagnostic_registration: None,
        }
    }

    pub(crate) fn new_with_diagnostic(
        texture: wgpu::Texture,
        counters: &Arc<TextureDiagnosticCounters>,
        kind: TextureDiagnosticKind,
        bytes: u64,
    ) -> Self {
        Self {
            texture,
            bind_linear: Default::default(),
            bind_nearest: Default::default(),
            copy_count: Cell::new(0),
            _diagnostic_registration: Some(counters.register(kind, bytes)),
        }
    }

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
                device,
                layout,
                samplers.get_sampler(false, smoothed),
                &quad.texture_transforms,
                0 as wgpu::BufferAddress,
                self.texture.create_view(&Default::default()),
                create_debug_label!("Bitmap {:?} bind group (smoothed: {})", handle.0, smoothed),
            )
        })
    }
}
