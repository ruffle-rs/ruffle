use crate::gui::MENU_HEIGHT;
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::target::{RenderTarget, RenderTargetFrame};
use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct MovieViewRenderer {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    vertices: wgpu::Buffer,
}

fn get_vertices(has_menu: bool, height: u32, scale_factor: f64) -> [[f32; 4]; 6] {
    let top = if has_menu {
        let menu_height = MENU_HEIGHT as f64 * scale_factor;
        1.0 - ((menu_height / height as f64) * 2.0) as f32
    } else {
        1.0
    };
    // x y u v
    [
        [-1.0, top, 0.0, 0.0],  // tl
        [1.0, top, 1.0, 0.0],   // tr
        [1.0, -1.0, 1.0, 1.0],  // br
        [1.0, -1.0, 1.0, 1.0],  // br
        [-1.0, -1.0, 0.0, 1.0], // bl
        [-1.0, top, 0.0, 0.0],  // tl
    ]
}

impl MovieViewRenderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        has_menu: bool,
        height: u32,
        scale_factor: f64,
    ) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("blit.wgsl"))),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: "vs_main",
                module: &module,
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 4 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    // 0: vec2 position
                    // 1: vec2 texture coordinates
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                unclipped_depth: false,
                conservative: false,
                cull_mode: None,
                front_face: wgpu::FrontFace::default(),
                polygon_mode: wgpu::PolygonMode::default(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: !0,
            },

            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: if surface_format.is_srgb() {
                    "fs_main_srgb_framebuffer"
                } else {
                    "fs_main_linear_framebuffer"
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&get_vertices(has_menu, height, scale_factor)),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            bind_group_layout,
            pipeline,
            sampler,
            vertices,
        }
    }

    pub fn update_resolution(
        &self,
        descriptors: &Descriptors,
        has_menu: bool,
        height: u32,
        scale_factor: f64,
    ) {
        descriptors.queue.write_buffer(
            &self.vertices,
            0,
            bytemuck::cast_slice(&get_vertices(has_menu, height, scale_factor)),
        );
    }
}

#[derive(Debug)]
pub struct MovieView {
    renderer: Arc<MovieViewRenderer>,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

impl MovieView {
    pub fn new(
        renderer: Arc<MovieViewRenderer>,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &renderer.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&renderer.sampler),
                },
            ],
        });
        Self {
            renderer,
            texture,
            bind_group,
        }
    }

    pub fn render(
        &self,
        renderer: &MovieViewRenderer,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        render_pass.set_pipeline(&renderer.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, renderer.vertices.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}

impl RenderTarget for MovieView {
    type Frame = MovieViewFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        *self = MovieView::new(self.renderer.clone(), device, width, height);
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.texture.format()
    }

    fn width(&self) -> u32 {
        self.texture.width()
    }

    fn height(&self) -> u32 {
        self.texture.height()
    }

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SurfaceError> {
        Ok(MovieViewFrame(
            self.texture.create_view(&Default::default()),
        ))
    }

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        _frame: Self::Frame,
    ) -> wgpu::SubmissionIndex {
        queue.submit(command_buffers)
    }
}

#[derive(Debug)]
pub struct MovieViewFrame(wgpu::TextureView);

impl RenderTargetFrame for MovieViewFrame {
    fn into_view(self) -> wgpu::TextureView {
        self.0
    }

    fn view(&self) -> &wgpu::TextureView {
        &self.0
    }
}
