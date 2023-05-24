use crate::custom_event::RuffleEvent;
use crate::gui::movie::{MovieView, MovieViewRenderer};
use crate::gui::RuffleGui;
use anyhow::anyhow;
use egui::Context;
use ruffle_render_wgpu::backend::request_adapter_and_device;
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::utils::{format_list, get_backend_names};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;

/// Integration layer connecting wgpu+winit to egui.
pub struct GuiController {
    descriptors: Arc<Descriptors>,
    egui_ctx: egui::Context,
    egui_winit: egui_winit::State,
    egui_renderer: egui_wgpu::renderer::Renderer,
    gui: RuffleGui,
    window: Rc<Window>,
    last_update: Instant,
    repaint_after: Duration,
    surface: wgpu::Surface,
    surface_format: wgpu::TextureFormat,
    movie_view_renderer: Arc<MovieViewRenderer>,
    // Note that `window.get_inner_size` can change at any point on x11, even between two lines of code.
    // Use this instead.
    size: PhysicalSize<u32>,
}

impl GuiController {
    pub fn new(
        window: Rc<Window>,
        event_loop: &EventLoop<RuffleEvent>,
        trace_path: Option<&Path>,
        backend: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
    ) -> anyhow::Result<Self> {
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
        let surface = unsafe { instance.create_surface(window.as_ref()) }?;
        let (adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
            backend,
            instance,
            Some(&surface),
            power_preference,
            trace_path,
        ))
        .map_err(|e| anyhow!(e.to_string()))?;
        let surface_format = surface
            .get_capabilities(&adapter)
            .formats
            .first()
            .cloned()
            .expect("At least one format should be supported");
        let size = window.inner_size();
        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: Default::default(),
                alpha_mode: Default::default(),
                view_formats: Default::default(),
            },
        );
        let descriptors = Descriptors::new(adapter, device, queue);
        let egui_ctx = Context::default();
        let mut egui_winit = egui_winit::State::new(event_loop);
        egui_winit.set_pixels_per_point(window.scale_factor() as f32);
        egui_winit.set_max_texture_side(descriptors.limits.max_texture_dimension_2d as usize);

        let movie_view_renderer = Arc::new(MovieViewRenderer::new(
            &descriptors.device,
            surface_format,
            window.fullscreen().is_none(),
            size.height,
            window.scale_factor(),
        ));
        let egui_renderer = egui_wgpu::Renderer::new(&descriptors.device, surface_format, None, 1);
        let event_loop = event_loop.create_proxy();
        let gui = RuffleGui::new(event_loop);
        Ok(Self {
            descriptors: Arc::new(descriptors),
            egui_ctx,
            egui_winit,
            egui_renderer,
            gui,
            window,
            last_update: Instant::now(),
            repaint_after: Duration::ZERO,
            surface,
            surface_format,
            movie_view_renderer,
            size,
        })
    }

    pub fn descriptors(&self) -> &Arc<Descriptors> {
        &self.descriptors
    }

    #[must_use]
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        if let winit::event::WindowEvent::Resized(size) = &event {
            self.surface.configure(
                &self.descriptors.device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: self.surface_format,
                    width: size.width,
                    height: size.height,
                    present_mode: Default::default(),
                    alpha_mode: Default::default(),
                    view_formats: Default::default(),
                },
            );
            self.movie_view_renderer.update_resolution(
                &self.descriptors,
                self.window.fullscreen().is_none(),
                size.height,
                self.window.scale_factor(),
            );
            self.size = *size;
        }
        let response = self.egui_winit.on_event(&self.egui_ctx, event);
        if response.repaint {
            self.window.request_redraw();
        }
        response.consumed
    }

    pub fn create_movie_view(&self) -> MovieView {
        MovieView::new(
            self.movie_view_renderer.clone(),
            &self.descriptors.device,
            self.size.width,
            self.size.height,
        )
    }

    pub fn render(&mut self, movie: Option<&MovieView>) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Surface became unavailable");

        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |context| {
            self.gui
                .update(context, self.window.fullscreen().is_none(), movie.is_some());
        });
        self.repaint_after = full_output.repaint_after;

        self.egui_winit.handle_platform_output(
            &self.window,
            &self.egui_ctx,
            full_output.platform_output,
        );
        let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes);

        let scale_factor = self.window.scale_factor() as f32;
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: scale_factor,
        };

        let mut encoder =
            self.descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("egui encoder"),
                });

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(
                &self.descriptors.device,
                &self.descriptors.queue,
                *id,
                image_delta,
            );
        }

        let mut command_buffers = self.egui_renderer.update_buffers(
            &self.descriptors.device,
            &self.descriptors.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let surface_view = surface_texture.texture.create_view(&Default::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui_render"),
            });

            if let Some(movie) = movie {
                movie.render(&self.movie_view_renderer, &mut render_pass);
            }

            self.egui_renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        command_buffers.push(encoder.finish());
        self.descriptors.queue.submit(command_buffers);
        surface_texture.present();
    }

    pub fn show_context_menu(&mut self, menu: Vec<ruffle_core::ContextMenuItem>) {
        self.gui.show_context_menu(menu);
    }

    pub fn is_context_menu_visible(&self) -> bool {
        self.gui.is_context_menu_visible()
    }

    pub fn needs_render(&self) -> bool {
        Instant::now().duration_since(self.last_update) >= self.repaint_after
    }
}
