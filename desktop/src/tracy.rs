use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::Metadata;
use tracing_subscriber::fmt::format::DefaultFields;
use tracing_tracy::client::{frame_image, frame_mark, register_demangler};
use tracing_tracy::Config;

#[derive(Default)]
pub struct RuffleTracyConfig(DefaultFields);

register_demangler!();

impl Config for RuffleTracyConfig {
    type Formatter = DefaultFields;

    fn formatter(&self) -> &Self::Formatter {
        &self.0
    }

    fn stack_depth(&self, _metadata: &Metadata<'_>) -> u16 {
        // How much, if any, of the stack trace to capture for each event
        // Obviously, this adds overhead
        0
    }
}

// Just to help avoid lots of #[cfg(feature = "tracy")] elsewhere, we'll wrap the arc/mutex here
#[derive(Clone, Debug)]
pub struct FrameCapturesHolder(Arc<Mutex<FrameCaptures>>);

impl FrameCapturesHolder {
    pub fn new(device: &wgpu::Device) -> Self {
        Self(Arc::new(Mutex::new(FrameCaptures::new(device))))
    }

    pub fn set_target(&self, device: &wgpu::Device, texture: Option<&wgpu::Texture>) {
        self.0
            .lock()
            .expect("FramesCaptures must not be poisoned")
            .set_target(device, texture);
    }

    pub fn capture_frame(&self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        self.0
            .lock()
            .expect("FramesCaptures must not be poisoned")
            .capture_frame(device, encoder);
    }

    pub fn finish_frame(&self) {
        self.0
            .lock()
            .expect("FramesCaptures must not be poisoned")
            .finish_frame();
    }
}

#[derive(Debug)]
pub struct FrameCaptures {
    available_buffers: Vec<PooledBuffer>,
    submitted_captures: Vec<PendingCapture>,
    mapped_captures: Vec<PendingCapture>,
    current_frame: u64,
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group_layout: wgpu::BindGroupLayout,
    target: Option<CaptureTarget>,
    sampler: wgpu::Sampler,
}

#[derive(Debug)]
struct PooledBuffer {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    buffer: wgpu::Buffer,
}

#[derive(Debug)]
struct CaptureTarget {
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
}

#[derive(Debug)]
struct PendingCapture {
    buffer: PooledBuffer,
    width: u32,
    height: u32,
    frame_num: u64,
    ready: Arc<AtomicBool>,
}

impl PendingCapture {
    pub fn upload(&self, current_frame: u64) {
        // If we're so far behind we couldn't even fit into a u8, just drop it.
        let Some(offset) = current_frame
            .checked_sub(self.frame_num)
            .and_then(|o| u8::try_from(o).ok())
        else {
            return;
        };

        // Tracy needs a raw rgba image with no padding, so we have to remove it
        let mut unpadded = Vec::with_capacity(self.width as usize * self.height as usize * 4);
        let padded_width = wgpu::util::align_to(self.width * 4, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = self.buffer.buffer.slice(..).get_mapped_range();
        for row in 0..self.height {
            let start = row as usize * padded_width as usize;
            let end = start + (self.width as usize * 4);
            unpadded.extend_from_slice(&buffer[start..end]);
        }
        frame_image(
            &unpadded,
            self.width as u16,
            self.height as u16,
            offset,
            false,
        );
    }
}

impl PooledBuffer {
    pub fn new(device: &wgpu::Device, target: &CaptureTarget) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: target.width,
                height: target.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (target.padded_bytes_per_row * target.height * 4) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        PooledBuffer {
            texture,
            view,
            buffer,
        }
    }
}

impl FrameCaptures {
    /// Maximum width of a texture to capture
    /// This MUST be a multiple of 4.
    const MAX_WIDTH: u32 = 320;

    /// Maximum height of a texture to capture.
    /// This MUST be a multiple of 4.
    const MAX_HEIGHT: u32 = 180;

    pub fn new(device: &wgpu::Device) -> Self {
        let blit_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });
        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&blit_bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader_source = r"
@group(0) @binding(0) var frame_sampler: sampler;
@group(0) @binding(1) var frame_texture: texture_2d<f32>;
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};
@vertex fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var result: VertexOutput;
    let x = i32(vertex_index) / 2;
    let y = i32(vertex_index) & 1;
    let tc = vec2<f32>(
        f32(x) * 2.0,
        f32(y) * 2.0
    );
    result.position = vec4<f32>(
        tc.x * 2.0 - 1.0,
        1.0 - tc.y * 2.0,
        0.0, 1.0
    );
    result.tex_coord = tc;
    return result;
}
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(frame_texture, frame_sampler, in.tex_coord);
}
";
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });
        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            }),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            current_frame: 0,
            blit_pipeline,
            available_buffers: Default::default(),
            submitted_captures: Default::default(),
            mapped_captures: Default::default(),
            blit_bind_group_layout,
            target: None,
            sampler,
        }
    }

    /// Sets the target texture that will be captured and uploaded to Tracy.
    pub fn set_target(&mut self, device: &wgpu::Device, texture: Option<&wgpu::Texture>) {
        if let Some(texture) = texture {
            // The ideal size of a captured texture should be 320x180
            // Preserve the texture aspect ratio (if possible) whilst maintaining that limit
            // However - the final size MUST be divisible by 4.
            let scale_x = texture.width() as f32 / Self::MAX_WIDTH as f32;
            let scale_y = texture.height() as f32 / Self::MAX_HEIGHT as f32;
            let scale = scale_x.max(scale_y).max(1.0);
            let width = (texture.width() as f32 / scale).floor() as u32 / 4 * 4;
            let height = (texture.height() as f32 / scale).floor() as u32 / 4 * 4;
            let view = texture.create_view(&Default::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.blit_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
            });

            // 4 bytes per pixel (Tracy wants RGBA)
            let bytes_per_row = width * 4;
            let padded_bytes_per_row =
                wgpu::util::align_to(bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

            if let Some(old_target) = &self.target {
                if old_target.width != width || old_target.height != height {
                    // We've changed the size of the ideal texture, let's recreate our buffers
                    self.available_buffers.clear();
                }
            }

            self.target = Some(CaptureTarget {
                bind_group,
                width,
                height,
                padded_bytes_per_row,
            });
            self.available_buffers.clear();
        } else {
            self.target = None;
        }
    }

    /// Performs a capture and records it for later upload.
    /// This should be called once you've finished rendering to the texture,
    /// but before you submit the frame.
    pub fn capture_frame(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        let Some(target) = &self.target else {
            return;
        };

        // Take an existing buffer, or make one if we've run out
        let buffer = self
            .available_buffers
            .pop()
            .unwrap_or_else(|| PooledBuffer::new(device, target));

        // Copy the frame to the texture
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &buffer.view,
                resolve_target: None,
                ops: Default::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.blit_pipeline);
        render_pass.set_bind_group(0, &target.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);

        // Copy the texture to the buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &buffer.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(target.padded_bytes_per_row),
                    rows_per_image: Some(target.height),
                },
            },
            wgpu::Extent3d {
                width: target.width,
                height: target.height,
                depth_or_array_layers: 1,
            },
        );
        let capture = PendingCapture {
            buffer,
            width: target.width,
            height: target.height,
            frame_num: self.current_frame,
            ready: Arc::new(AtomicBool::new(false)),
        };
        self.submitted_captures.push(capture);
    }

    /// Uploads any finished captures to Tracy, and marks a Tracy frame boundary.
    /// This should be called _after_ `wgpu::Queue::submit`, but before you present to the surface.
    pub fn finish_frame(&mut self) {
        frame_mark();
        self.current_frame += 1;
        let current_size = self.target.as_ref().map(|t| (t.width, t.height));
        for capture in std::mem::take(&mut self.submitted_captures) {
            let ready = capture.ready.clone();
            capture
                .buffer
                .buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |_| {
                    ready.store(true, Ordering::Relaxed);
                });
            self.mapped_captures.push(capture);
        }
        for capture in std::mem::take(&mut self.mapped_captures) {
            if capture.ready.load(Ordering::Relaxed) {
                capture.upload(self.current_frame);
                capture.buffer.buffer.unmap();

                if current_size == Some((capture.width, capture.height)) {
                    self.available_buffers.push(capture.buffer);
                }
            } else {
                self.mapped_captures.push(capture);
            }
        }
    }
}
