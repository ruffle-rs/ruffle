use crate::buffer_builder::BufferBuilder;
use crate::buffer_pool::TexturePool;
use crate::context3d::WgpuContext3D;
use crate::mesh::{Mesh, PendingDraw};
use crate::surface::Surface;
use crate::target::RenderTargetFrame;
use crate::target::TextureTarget;
use crate::uniform_buffer::BufferStorage;
use crate::{
    as_texture, format_list, get_backend_names, ColorAdjustments, Descriptors, Error,
    QueueSyncHandle, RenderTarget, SwapChainTarget, Texture, Transforms,
};
use gc_arena::MutationContext;
use ruffle_render::backend::{Context3D, Context3DCommand};
use ruffle_render::backend::{RenderBackend, ShapeHandle, ViewportDimensions};
use ruffle_render::bitmap::{Bitmap, BitmapHandle, BitmapSource, SyncHandle};
use ruffle_render::commands::CommandList;
use ruffle_render::error::Error as BitmapError;
use ruffle_render::filters::Filter;
use ruffle_render::quality::StageQuality;
use ruffle_render::shape_utils::DistilledShape;
use ruffle_render::tessellator::ShapeTessellator;
use std::borrow::Cow;
use std::cell::Cell;
use std::mem;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use swf::Color;
use tracing::instrument;
use wgpu::Extent3d;

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
}

impl WgpuRenderBackend<SwapChainTarget> {
    #[cfg(target_family = "wasm")]
    pub async fn for_canvas(
        canvas: &web_sys::HtmlCanvasElement,
        sample_count: u32,
    ) -> Result<Self, Error> {
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
        Self::new(Arc::new(descriptors), target, sample_count)
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
        Self::new(Arc::new(descriptors), target, 4)
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
        Self::new(Arc::new(descriptors), target, 4)
    }

    pub fn capture_frame(&self, premultiplied_alpha: bool) -> Option<image::RgbaImage> {
        use crate::utils::buffer_to_image;
        if let Some((buffer, dimensions)) = &self.target.buffer {
            Some(buffer_to_image(
                &self.descriptors.device,
                buffer,
                dimensions,
                None,
                self.target.size,
                premultiplied_alpha,
            ))
        } else {
            None
        }
    }
}

impl<T: RenderTarget> WgpuRenderBackend<T> {
    pub fn new(
        descriptors: Arc<Descriptors>,
        target: T,
        preferred_sample_count: u32,
    ) -> Result<Self, Error> {
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
            preferred_sample_count,
            target.width(),
            target.height(),
            target.format(),
        );

        let uniform_buffers_storage =
            BufferStorage::from_alignment(descriptors.limits.min_uniform_buffer_offset_alignment);

        let color_buffers_storage =
            BufferStorage::from_alignment(descriptors.limits.min_uniform_buffer_offset_alignment);

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
            self.surface.sample_count(),
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
        let texture_label = create_debug_label!("Render target texture");
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let dummy_texture = self
            .descriptors
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: texture_label.as_deref(),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                view_formats: &[format],
                usage: wgpu::TextureUsages::COPY_SRC,
            });

        let handle = BitmapHandle(Arc::new(Texture {
            bind_linear: Default::default(),
            bind_nearest: Default::default(),
            texture: Arc::new(dummy_texture),
            texture_offscreen: Default::default(),
            width: 0,
            height: 0,
            copy_count: Cell::new(0),
        }));
        Ok(Box::new(WgpuContext3D::new(
            self.descriptors.clone(),
            handle,
        )))
    }

    #[instrument(level = "debug", skip_all)]
    fn context3d_present<'gc>(
        &mut self,
        context: &mut dyn Context3D,
        commands: Vec<Context3DCommand<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), BitmapError> {
        let context = context
            .as_any_mut()
            .downcast_mut::<WgpuContext3D>()
            .unwrap();
        context.present(commands, mc);
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
        result.push(format!("Surface samples: {}", self.surface.sample_count()));
        result.push(format!("Surface size: {:?}", self.surface.size()));

        Cow::Owned(result.join("\n"))
    }

    fn set_quality(&mut self, quality: StageQuality) {
        self.surface = Surface::new(
            &self.descriptors,
            quality.sample_count(),
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
        let handle = ShapeHandle(self.meshes.len());
        let mesh = self.register_shape_internal(shape, bitmap_source);
        self.meshes.push(mesh);
        handle
    }

    #[instrument(level = "debug", skip_all)]
    fn replace_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
        handle: ShapeHandle,
    ) {
        let mesh = self.register_shape_internal(shape, bitmap_source);
        self.meshes[handle.0] = mesh;
    }

    #[instrument(level = "debug", skip_all)]
    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let shape = ruffle_render::shape_utils::swf_glyph_to_shape(glyph);
        let handle = ShapeHandle(self.meshes.len());
        let mesh = self.register_shape_internal(
            (&shape).into(),
            &ruffle_render::backend::null::NullBitmapSource,
        );
        self.meshes.push(mesh);
        handle
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
            Some(wgpu::Color {
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
                bytes_per_row: NonZeroU32::new(4 * extent.width),
                rows_per_image: None,
            },
            extent,
        );

        let handle = BitmapHandle(Arc::new(Texture {
            texture: Arc::new(texture),
            bind_linear: Default::default(),
            bind_nearest: Default::default(),
            texture_offscreen: Default::default(),
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
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<(), BitmapError> {
        let texture = as_texture(handle);

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        self.descriptors.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: Default::default(),
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * extent.width),
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
        width: u32,
        height: u32,
        commands: CommandList,
    ) -> Option<Box<dyn SyncHandle>> {
        let texture = as_texture(&handle);

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture_offscreen = texture.texture_offscreen.get();

        let mut target = TextureTarget {
            size: extent,
            texture: texture.texture.clone(),
            format: wgpu::TextureFormat::Rgba8Unorm,
            buffer: texture_offscreen.map(|t| (t.buffer.clone(), t.buffer_dimensions.clone())),
        };

        let frame_output = target
            .get_next_texture()
            .expect("TextureTargetFrame.get_next_texture is infallible");

        let mut surface = Surface::new(
            &self.descriptors,
            self.surface.sample_count(),
            width,
            height,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let command_buffers = surface.draw_commands_to(
            frame_output.view(),
            None,
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

        match texture_offscreen {
            Some(texture_offscreen) => Some(Box::new(QueueSyncHandle::AlreadyCopied {
                index,
                size: target.size,
                buffer: texture_offscreen.buffer.clone(),
                buffer_dimensions: texture_offscreen.buffer_dimensions.clone(),
                descriptors: self.descriptors.clone(),
            })),
            None => Some(Box::new(QueueSyncHandle::NotCopied {
                handle: handle.clone(),
                size: target.size,
                descriptors: self.descriptors.clone(),
            })),
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

        let mut target = TextureTarget {
            size: wgpu::Extent3d {
                width: dest_texture.width,
                height: dest_texture.height,
                depth_or_array_layers: 1,
            },
            texture: dest_texture.texture.clone(),
            format: wgpu::TextureFormat::Rgba8Unorm,
            buffer: dest_texture
                .texture_offscreen
                .get()
                .map(|t| (t.buffer.clone(), t.buffer_dimensions.clone())),
        };
        let texture_offscreen = dest_texture.texture_offscreen.get();
        let frame_output = target
            .get_next_texture()
            .expect("TextureTargetFrame.get_next_texture is infallible");
        let surface = Surface::new(
            &self.descriptors,
            self.preferred_sample_count,
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
        match texture_offscreen {
            Some(texture_offscreen) => Some(Box::new(QueueSyncHandle::AlreadyCopied {
                index,
                size: target.size,
                buffer: texture_offscreen.buffer.clone(),
                buffer_dimensions: texture_offscreen.buffer_dimensions.clone(),
                descriptors: self.descriptors.clone(),
            })),
            None => Some(Box::new(QueueSyncHandle::NotCopied {
                handle: destination.clone(),
                size: target.size,
                descriptors: self.descriptors.clone(),
            })),
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
