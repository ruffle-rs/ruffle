use crate::buffer_builder::BufferBuilder;
use crate::buffer_pool::{BufferPool, TexturePool};
use crate::context3d::WgpuContext3D;
use crate::mesh::{Mesh, PendingDraw};
use crate::surface::Surface;
use crate::target::{MaybeOwnedBuffer, TextureTarget};
use crate::target::{RenderTargetFrame, TextureBufferInfo};
use crate::uniform_buffer::BufferStorage;
use crate::utils::BufferDimensions;
use crate::{
    as_texture, format_list, get_backend_names, ColorAdjustments, Descriptors, Error,
    QueueSyncHandle, RenderTarget, SwapChainTarget, Texture, Transforms,
};
use ruffle_render::backend::Context3D;
use ruffle_render::backend::{RenderBackend, ShapeHandle, ViewportDimensions};
use ruffle_render::bitmap::{Bitmap, BitmapHandle, BitmapSource, PixelRegion, SyncHandle};
use ruffle_render::commands::CommandList;
use ruffle_render::error::Error as BitmapError;
use ruffle_render::filters::Filter;
use ruffle_render::quality::StageQuality;
use ruffle_render::shape_utils::DistilledShape;
use ruffle_render::tessellator::ShapeTessellator;
use std::borrow::Cow;
use std::cell::Cell;
use std::mem;
use std::path::Path;
use std::sync::Arc;
use swf::Color;
use tracing::instrument;

/// How many times a texture must be written to & read back from,
/// before it's automatically allocated a buffer on each write.
const TEXTURE_READS_BEFORE_PROMOTION: u8 = 5;

pub struct WgpuRenderBackend<T: RenderTarget> {
    descriptors: Arc<Descriptors>,
    uniform_buffers_storage: BufferStorage<Transforms>,
    color_buffers_storage: BufferStorage<ColorAdjustments>,
    target: T,
    surface: Surface,
    meshes: Vec<Mesh>,
    shape_tessellator: ShapeTessellator,
    // This is currently unused - we just store it to report in
    // `get_viewport_dimensions`
    viewport_scale_factor: f64,
    texture_pool: TexturePool,
    offscreen_texture_pool: TexturePool,
    offscreen_buffer_pool: Arc<BufferPool<wgpu::Buffer, BufferDimensions>>,
}

impl WgpuRenderBackend<SwapChainTarget> {
    #[cfg(target_family = "wasm")]
    pub async fn for_canvas(canvas: web_sys::HtmlCanvasElement) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        });
        let surface = instance.create_surface_from_canvas(canvas)?;
        let (adapter, device, queue) = Self::request_device(
            wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            instance,
            Some(&surface),
            wgpu::PowerPreference::HighPerformance,
            None,
        )
        .await?;
        let descriptors = Descriptors::new(adapter, device, queue);
        let target =
            SwapChainTarget::new(surface, &descriptors.adapter, (1, 1), &descriptors.device);
        Self::new(Arc::new(descriptors), target)
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn for_window<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        window: &W,
        size: (u32, u32),
        backend: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<Self, Error> {
        if wgpu::Backends::SECONDARY.contains(backend) {
            tracing::warn!(
                "{} graphics backend support may not be fully supported.",
                format_list(&get_backend_names(backend), "and")
            );
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        });
        let surface = unsafe { instance.create_surface(window) }?;
        let (adapter, device, queue) = futures::executor::block_on(Self::request_device(
            backend,
            instance,
            Some(&surface),
            power_preference,
            trace_path,
        ))?;
        let descriptors = Descriptors::new(adapter, device, queue);
        let target = SwapChainTarget::new(surface, &descriptors.adapter, size, &descriptors.device);
        Self::new(Arc::new(descriptors), target)
    }
}

#[cfg(not(target_family = "wasm"))]
impl WgpuRenderBackend<crate::target::TextureTarget> {
    pub fn for_offscreen(
        size: (u32, u32),
        backend: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<Self, Error> {
        if wgpu::Backends::SECONDARY.contains(backend) {
            tracing::warn!(
                "{} graphics backend support may not be fully supported.",
                format_list(&get_backend_names(backend), "and")
            );
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        });
        let (adapter, device, queue) = futures::executor::block_on(Self::request_device(
            backend,
            instance,
            None,
            power_preference,
            trace_path,
        ))?;
        let descriptors = Descriptors::new(adapter, device, queue);
        let target = crate::target::TextureTarget::new(&descriptors.device, size)?;
        Self::new(Arc::new(descriptors), target)
    }

    pub fn capture_frame(&self) -> Option<image::RgbaImage> {
        use crate::utils::buffer_to_image;
        if let Some(buffer) = &self.target.buffer {
            let (buffer, dimensions) = buffer.buffer.inner();
            Some(buffer_to_image(
                &self.descriptors.device,
                buffer,
                dimensions,
                None,
                self.target.size,
            ))
        } else {
            None
        }
    }
}

impl<T: RenderTarget> WgpuRenderBackend<T> {
    pub fn new(descriptors: Arc<Descriptors>, target: T) -> Result<Self, Error> {
        if target.width() > descriptors.limits.max_texture_dimension_2d
            || target.height() > descriptors.limits.max_texture_dimension_2d
        {
            return Err(format!(
                "Render target texture cannot be larger than {}px on either dimension (requested {} x {})",
                descriptors.limits.max_texture_dimension_2d,
                target.width(),
                target.height()
            )
                .into());
        }

        let surface = Surface::new(
            &descriptors,
            StageQuality::Low,
            target.width(),
            target.height(),
            target.format(),
        );

        let uniform_buffers_storage =
            BufferStorage::from_alignment(descriptors.limits.min_uniform_buffer_offset_alignment);

        let color_buffers_storage =
            BufferStorage::from_alignment(descriptors.limits.min_uniform_buffer_offset_alignment);

        let offscreen_buffer_pool = BufferPool::new(Box::new(
            |descriptors: &Descriptors, dimensions: &BufferDimensions| {
                descriptors.device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: dimensions.size(),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                })
            },
        ));

        Ok(Self {
            descriptors,
            uniform_buffers_storage,
            color_buffers_storage,
            target,
            surface,
            meshes: Vec::new(),
            shape_tessellator: ShapeTessellator::new(),
            viewport_scale_factor: 1.0,
            texture_pool: TexturePool::new(),
            offscreen_texture_pool: TexturePool::new(),
            offscreen_buffer_pool: Arc::new(offscreen_buffer_pool),
        })
    }

    pub async fn request_device(
        backend: wgpu::Backends,
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface>,
        power_preference: wgpu::PowerPreference,
        trace_path: Option<&Path>,
    ) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), Error> {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            compatible_surface: surface,
            force_fallback_adapter: false,
        }).await
            .ok_or_else(|| {
                let names = get_backend_names(backend);
                if names.is_empty() {
                    "Ruffle requires hardware acceleration, but no compatible graphics device was found (no backend provided?)".to_string()
                } else if cfg!(any(windows, target_os = "macos")) {
                    format!("Ruffle does not support OpenGL on {}.", if cfg!(windows) { "Windows" } else { "macOS" })
                } else {
                    format!("Ruffle requires hardware acceleration, but no compatible graphics device was found supporting {}", format_list(&names, "or"))
                }
            })?;

        let (device, queue) = request_device(&adapter, trace_path).await?;
        Ok((adapter, device, queue))
    }

    fn register_shape_internal(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> Mesh {
        let shape_id = shape.id;
        let lyon_mesh = self
            .shape_tessellator
            .tessellate_shape(shape, bitmap_source);

        let mut draws = Vec::with_capacity(lyon_mesh.len());
        let mut uniform_buffer = BufferBuilder::new(
            self.descriptors.limits.min_uniform_buffer_offset_alignment as usize,
        );
        let mut vertex_buffer = BufferBuilder::new(0);
        let mut index_buffer = BufferBuilder::new(0);
        for draw in lyon_mesh {
            let draw_id = draws.len();
            if let Some(draw) = PendingDraw::new(
                self,
                bitmap_source,
                draw,
                shape_id,
                draw_id,
                &mut uniform_buffer,
                &mut vertex_buffer,
                &mut index_buffer,
            ) {
                draws.push(draw);
            }
        }

        let uniform_buffer = uniform_buffer.finish(
            &self.descriptors.device,
            create_debug_label!("Shape {} uniforms", shape_id),
            wgpu::BufferUsages::UNIFORM,
        );
        let vertex_buffer = vertex_buffer.finish(
            &self.descriptors.device,
            create_debug_label!("Shape {} vertices", shape_id),
            wgpu::BufferUsages::VERTEX,
        );
        let index_buffer = index_buffer.finish(
            &self.descriptors.device,
            create_debug_label!("Shape {} indices", shape_id),
            wgpu::BufferUsages::INDEX,
        );

        let draws = draws
            .into_iter()
            .map(|d| d.finish(&self.descriptors, &uniform_buffer))
            .collect();

        Mesh {
            draws,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn descriptors(&self) -> &Arc<Descriptors> {
        &self.descriptors
    }

    pub fn target(&self) -> &T {
        &self.target
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.descriptors.device
    }
}

impl<T: RenderTarget + 'static> RenderBackend for WgpuRenderBackend<T> {
    fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions) {
        // Avoid panics from creating 0-sized framebuffers.
        // TODO: find a way to bubble an error when the size is too large
        let width = std::cmp::max(
            std::cmp::min(
                dimensions.width,
                self.descriptors.limits.max_texture_dimension_2d,
            ),
            1,
        );
        let height = std::cmp::max(
            std::cmp::min(
                dimensions.height,
                self.descriptors.limits.max_texture_dimension_2d,
            ),
            1,
        );
        self.target.resize(&self.descriptors.device, width, height);

        self.surface = Surface::new(
            &self.descriptors,
            self.surface.quality(),
            width,
            height,
            self.target.format(),
        );

        self.viewport_scale_factor = dimensions.scale_factor;
        self.texture_pool = TexturePool::new();
    }

    fn create_context3d(
        &mut self,
    ) -> Result<Box<dyn ruffle_render::backend::Context3D>, BitmapError> {
        Ok(Box::new(WgpuContext3D::new(self.descriptors.clone())))
    }

    #[instrument(level = "debug", skip_all)]
    fn context3d_present(&mut self, context: &mut dyn Context3D) -> Result<(), BitmapError> {
        let context = context
            .as_any_mut()
            .downcast_mut::<WgpuContext3D>()
            .unwrap();
        context.present();
        Ok(())
    }

    fn debug_info(&self) -> Cow<'static, str> {
        let mut result = vec![];
        result.push("Renderer: wgpu".to_string());

        let info = self.descriptors.adapter.get_info();
        result.push(format!("Adapter Backend: {:?}", info.backend));
        result.push(format!("Adapter Name: {:?}", info.name));
        result.push(format!("Adapter Device Type: {:?}", info.device_type));
        result.push(format!("Adapter Driver Name: {:?}", info.driver));
        result.push(format!("Adapter Driver Info: {:?}", info.driver_info));

        let enabled_features = self.descriptors.device.features();
        let available_features = self.descriptors.adapter.features() - enabled_features;
        let current_limits = &self.descriptors.limits;

        result.push(format!("Enabled features: {enabled_features:?}"));
        result.push(format!("Available features: {available_features:?}"));
        result.push(format!("Current limits: {current_limits:?}"));
        result.push(format!("Surface quality: {}", self.surface.quality()));
        result.push(format!("Surface samples: {}", self.surface.sample_count()));
        result.push(format!("Surface size: {:?}", self.surface.size()));

        Cow::Owned(result.join("\n"))
    }

    fn name(&self) -> &'static str {
        "webgpu"
    }

    fn set_quality(&mut self, quality: StageQuality) {
        self.surface = Surface::new(
            &self.descriptors,
            quality,
            self.surface.size().width,
            self.surface.size().height,
            self.target.format(),
        );
    }

    fn viewport_dimensions(&self) -> ViewportDimensions {
        ViewportDimensions {
            width: self.target.width(),
            height: self.target.height(),
            scale_factor: self.viewport_scale_factor,
        }
    }

    #[instrument(level = "debug", skip_all)]
    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        let mesh = self.register_shape_internal(shape, bitmap_source);
        ShapeHandle(Arc::new(mesh))
    }

    #[instrument(level = "debug", skip_all)]
    fn submit_frame(&mut self, clear: Color, commands: CommandList) {
        let frame_output = match self.target.get_next_texture() {
            Ok(frame) => frame,
            Err(e) => {
                tracing::warn!("Couldn't begin new render frame: {}", e);
                // Attempt to recreate the swap chain in this case.
                self.target.resize(
                    &self.descriptors.device,
                    self.target.width(),
                    self.target.height(),
                );
                return;
            }
        };

        let command_buffers = self.surface.draw_commands_to(
            frame_output.view(),
            RenderTargetMode::FreshBuffer(wgpu::Color {
                r: f64::from(clear.r) / 255.0,
                g: f64::from(clear.g) / 255.0,
                b: f64::from(clear.b) / 255.0,
                a: f64::from(clear.a) / 255.0,
            }),
            &self.descriptors,
            &mut self.uniform_buffers_storage,
            &mut self.color_buffers_storage,
            &self.meshes,
            commands,
            &mut self.texture_pool,
        );

        self.target.submit(
            &self.descriptors.device,
            &self.descriptors.queue,
            command_buffers,
            frame_output,
        );
        self.uniform_buffers_storage.recall();
        self.color_buffers_storage.recall();
        self.offscreen_texture_pool = TexturePool::new();
    }

    #[instrument(level = "debug", skip_all)]
    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, BitmapError> {
        if bitmap.width() > self.descriptors.limits.max_texture_dimension_2d
            || bitmap.height() > self.descriptors.limits.max_texture_dimension_2d
        {
            return Err(BitmapError::TooLarge);
        }

        let bitmap = bitmap.to_rgba();
        let extent = wgpu::Extent3d {
            width: bitmap.width(),
            height: bitmap.height(),
            depth_or_array_layers: 1,
        };

        let texture_label = create_debug_label!("Bitmap");
        let texture = self
            .descriptors
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: texture_label.as_deref(),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC,
            });

        self.descriptors.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            bitmap.data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * extent.width),
                rows_per_image: None,
            },
            extent,
        );

        let handle = BitmapHandle(Arc::new(Texture {
            texture: Arc::new(texture),
            bind_linear: Default::default(),
            bind_nearest: Default::default(),
            width: bitmap.width(),
            height: bitmap.height(),
            copy_count: Cell::new(0),
        }));

        Ok(handle)
    }

    #[instrument(level = "debug", skip_all)]
    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap,
        region: PixelRegion,
    ) -> Result<(), BitmapError> {
        let texture = as_texture(handle);

        let extent = wgpu::Extent3d {
            width: region.width(),
            height: region.height(),
            depth_or_array_layers: 1,
        };
        let bitmap = bitmap.to_rgba();

        self.descriptors.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x_min,
                    y: region.y_min,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &bitmap.data()[(region.y_min * texture.width * 4) as usize
                ..(region.y_max * texture.width * 4) as usize],
            wgpu::ImageDataLayout {
                offset: (region.x_min * 4) as wgpu::BufferAddress,
                bytes_per_row: Some(4 * texture.width),
                rows_per_image: None,
            },
            extent,
        );

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    fn render_offscreen(
        &mut self,
        handle: BitmapHandle,
        commands: CommandList,
        quality: StageQuality,
        bounds: PixelRegion,
    ) -> Option<Box<dyn SyncHandle>> {
        let texture = as_texture(&handle);

        let extent = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };

        let buffer_info = if texture.copy_count.get() > TEXTURE_READS_BEFORE_PROMOTION {
            let copy_dimensions =
                BufferDimensions::new(bounds.width() as usize, bounds.height() as usize);
            let buffer = self
                .offscreen_buffer_pool
                .take(&self.descriptors, copy_dimensions.clone());
            Some(TextureBufferInfo {
                buffer: MaybeOwnedBuffer::Borrowed(buffer, copy_dimensions),
                copy_area: bounds,
            })
        } else {
            None
        };

        let mut target = TextureTarget {
            size: extent,
            texture: texture.texture.clone(),
            format: wgpu::TextureFormat::Rgba8Unorm,
            buffer: buffer_info,
        };

        let frame_output = target
            .get_next_texture()
            .expect("TextureTargetFrame.get_next_texture is infallible");

        let mut surface = Surface::new(
            &self.descriptors,
            quality,
            texture.width,
            texture.height,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let command_buffers = surface.draw_commands_to(
            frame_output.view(),
            RenderTargetMode::ExistingTexture(target.get_texture()),
            &self.descriptors,
            &mut self.uniform_buffers_storage,
            &mut self.color_buffers_storage,
            &self.meshes,
            commands,
            &mut self.offscreen_texture_pool,
        );
        let index = target.submit(
            &self.descriptors.device,
            &self.descriptors.queue,
            command_buffers,
            frame_output,
        );
        self.uniform_buffers_storage.recall();
        self.color_buffers_storage.recall();

        match target.take_buffer() {
            None => Some(Box::new(QueueSyncHandle::NotCopied {
                handle,
                copy_area: bounds,
                descriptors: self.descriptors.clone(),
                pool: self.offscreen_buffer_pool.clone(),
            })),
            Some(TextureBufferInfo {
                buffer: MaybeOwnedBuffer::Borrowed(buffer, copy_dimensions),
                ..
            }) => Some(Box::new(QueueSyncHandle::AlreadyCopied {
                index,
                buffer,
                copy_dimensions,
                descriptors: self.descriptors.clone(),
            })),
            Some(TextureBufferInfo {
                buffer: MaybeOwnedBuffer::Owned(..),
                ..
            }) => unreachable!("Buffer must be Borrowed as it was set to be Borrowed earlier"),
        }
    }

    fn apply_filter(
        &mut self,
        source: BitmapHandle,
        source_point: (u32, u32),
        source_size: (u32, u32),
        destination: BitmapHandle,
        dest_point: (u32, u32),
        filter: Filter,
    ) -> Option<Box<dyn SyncHandle>> {
        let source_texture = as_texture(&source);
        let dest_texture = as_texture(&destination);

        let copy_area = PixelRegion::for_whole_size(dest_texture.width, dest_texture.height);
        let buffer_info = if dest_texture.copy_count.get() >= TEXTURE_READS_BEFORE_PROMOTION {
            let copy_dimensions =
                BufferDimensions::new(dest_texture.width as usize, dest_texture.height as usize);
            let buffer = self
                .offscreen_buffer_pool
                .take(&self.descriptors, copy_dimensions.clone());
            Some(TextureBufferInfo {
                buffer: MaybeOwnedBuffer::Borrowed(buffer, copy_dimensions),
                copy_area,
            })
        } else {
            None
        };

        let mut target = TextureTarget {
            size: wgpu::Extent3d {
                width: dest_texture.width,
                height: dest_texture.height,
                depth_or_array_layers: 1,
            },
            texture: dest_texture.texture.clone(),
            format: wgpu::TextureFormat::Rgba8Unorm,
            buffer: buffer_info,
        };
        let frame_output = target
            .get_next_texture()
            .expect("TextureTargetFrame.get_next_texture is infallible");
        let surface = Surface::new(
            &self.descriptors,
            self.surface.quality(),
            dest_texture.width,
            dest_texture.height,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let label = create_debug_label!("Draw encoder");
        let mut draw_encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: label.as_deref(),
                });
        surface.apply_filter(
            &self.descriptors,
            &mut draw_encoder,
            &mut self.offscreen_texture_pool,
            source_texture,
            source_point,
            source_size,
            dest_texture,
            dest_point,
            filter,
        );
        let index = target.submit(
            &self.descriptors.device,
            &self.descriptors.queue,
            Some(draw_encoder.finish()),
            frame_output,
        );

        match target.take_buffer() {
            None => Some(Box::new(QueueSyncHandle::NotCopied {
                handle: destination,
                copy_area,
                descriptors: self.descriptors.clone(),
                pool: self.offscreen_buffer_pool.clone(),
            })),
            Some(TextureBufferInfo {
                buffer: MaybeOwnedBuffer::Borrowed(buffer, copy_dimensions),
                ..
            }) => Some(Box::new(QueueSyncHandle::AlreadyCopied {
                index,
                buffer,
                copy_dimensions,
                descriptors: self.descriptors.clone(),
            })),
            Some(TextureBufferInfo {
                buffer: MaybeOwnedBuffer::Owned(..),
                ..
            }) => unreachable!("Buffer must be Borrowed as it was set to be Borrowed earlier"),
        }
    }
}

// We try to request the highest limits we can get away with
async fn request_device(
    adapter: &wgpu::Adapter,
    trace_path: Option<&Path>,
) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
    // We start off with the lowest limits we actually need - basically GL-ES 3.0
    let mut limits = wgpu::Limits::downlevel_webgl2_defaults();
    // Then we increase parts of it to the maximum supported by the adapter, to take advantage of
    // more powerful hardware or capabilities
    limits = limits.using_resolution(adapter.limits());
    limits = limits.using_alignment(adapter.limits());

    let mut features = Default::default();

    let needed_size = (mem::size_of::<Transforms>() + mem::size_of::<ColorAdjustments>()) as u32;
    if adapter.features().contains(wgpu::Features::PUSH_CONSTANTS)
        && adapter.limits().max_push_constant_size >= needed_size
    {
        limits.max_push_constant_size = needed_size;
        features |= wgpu::Features::PUSH_CONSTANTS;
    }

    if adapter
        .features()
        .contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
    {
        features |= wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
    }

    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features,
                limits,
            },
            trace_path,
        )
        .await
}

/// Determines how we choose our frame buffer
#[derive(Clone)]
pub enum RenderTargetMode {
    // Construct a new frame buffer, clearng it with the provided color.
    // This is used when rendering to the actual display,
    // or when applying a filter. In both cases, we have a fixed background color,
    // and don't need to blend with anything else
    FreshBuffer(wgpu::Color),
    // Use the provided texture as our frame buffer. During rendering,
    // we will blend with the previous contents of the texture.
    // This is used in `render_offscreen`, as we need to blend with the previous
    // contents of our `BitmapData` texture
    ExistingTexture(Arc<wgpu::Texture>),
}
