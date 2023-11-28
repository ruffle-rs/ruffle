use crate::backends::DesktopUiBackend;
use crate::cli::Opt;
use crate::custom_event::RuffleEvent;
use crate::gui::movie::{MovieView, MovieViewRenderer};
use crate::gui::{RuffleGui, MENU_HEIGHT};
use crate::player::{PlayerController, PlayerOptions};
use anyhow::anyhow;
use egui::{Context, ViewportId};
use fontdb::{Database, Family, Query, Source};
use ruffle_core::Player;
use ruffle_render_wgpu::backend::{request_adapter_and_device, WgpuRenderBackend};
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::utils::{format_list, get_backend_names};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard};
use std::time::{Duration, Instant};
use unic_langid::LanguageIdentifier;
use url::Url;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Theme, Window};

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
    /// If this is set, we should not render the main menu.
    no_gui: bool,
}

impl GuiController {
    pub fn new(
        window: Rc<Window>,
        event_loop: &EventLoop<RuffleEvent>,
        opt: &Opt,
    ) -> anyhow::Result<Self> {
        let backend: wgpu::Backends = opt.graphics.into();
        if wgpu::Backends::SECONDARY.contains(backend) {
            tracing::warn!(
                "{} graphics backend support may not be fully supported.",
                format_list(&get_backend_names(backend), "and")
            );
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(window.as_ref()) }?;
        let (adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
            backend,
            &instance,
            Some(&surface),
            opt.power.into(),
            opt.trace_path(),
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
        let descriptors = Descriptors::new(instance, adapter, device, queue);
        let egui_ctx = Context::default();
        if let Some(Theme::Light) = window.theme() {
            egui_ctx.set_visuals(egui::Visuals::light());
        }
        egui_ctx.set_pixels_per_point(window.scale_factor() as f32);

        let mut egui_winit = egui_winit::State::new(ViewportId::ROOT, window.as_ref(), None, None);
        egui_winit.set_max_texture_side(descriptors.limits.max_texture_dimension_2d as usize);

        let movie_view_renderer = Arc::new(MovieViewRenderer::new(
            &descriptors.device,
            surface_format,
            window.fullscreen().is_none() && !opt.no_gui,
            size.height,
            window.scale_factor(),
        ));
        let egui_renderer = egui_wgpu::Renderer::new(&descriptors.device, surface_format, None, 1);
        let event_loop = event_loop.create_proxy();
        let gui = RuffleGui::new(event_loop, opt.movie_url.clone(), PlayerOptions::from(opt));
        let system_fonts = load_system_fonts(gui.locale.to_owned()).unwrap_or_default();
        egui_ctx.set_fonts(system_fonts);

        egui_extras::install_image_loaders(&egui_ctx);

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
            no_gui: opt.no_gui,
        })
    }

    pub fn descriptors(&self) -> &Arc<Descriptors> {
        &self.descriptors
    }

    #[must_use]
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        if let winit::event::WindowEvent::Resized(size) = &event {
            if size.width > 0 && size.height > 0 {
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
                    self.window.fullscreen().is_none() && !self.no_gui,
                    size.height,
                    self.window.scale_factor(),
                );
                self.size = *size;
            }
        }

        if let winit::event::WindowEvent::ThemeChanged(theme) = &event {
            let visuals = match theme {
                Theme::Dark => egui::Visuals::dark(),
                Theme::Light => egui::Visuals::light(),
            };
            self.egui_ctx.set_visuals(visuals);
        }

        let response = self.egui_winit.on_window_event(&self.egui_ctx, event);
        if response.repaint {
            self.window.request_redraw();
        }
        response.consumed
    }

    pub fn create_movie(
        &mut self,
        player: &mut PlayerController,
        opt: PlayerOptions,
        movie_url: Url,
    ) {
        let movie_view = MovieView::new(
            self.movie_view_renderer.clone(),
            &self.descriptors.device,
            self.size.width,
            self.size.height,
        );
        player.create(&opt, &movie_url, movie_view);
        self.gui.on_player_created(
            opt,
            movie_url,
            player
                .get()
                .expect("Player must exist after being created."),
        );
    }

    pub fn render(&mut self, mut player: Option<MutexGuard<Player>>) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Surface became unavailable");

        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let show_menu = self.window.fullscreen().is_none() && !self.no_gui;
        let mut full_output = self.egui_ctx.run(raw_input, |context| {
            self.gui.update(
                context,
                show_menu,
                player.as_deref_mut(),
                if show_menu {
                    MENU_HEIGHT as f64 * self.window.scale_factor()
                } else {
                    0.0
                },
            );
        });
        self.repaint_after = full_output
            .viewport_output
            .get(&ViewportId::ROOT)
            .expect("Root viewport must exist")
            .repaint_delay;

        // If we're not in a UI, tell egui which cursor we prefer to use instead
        if !self.egui_ctx.wants_pointer_input() {
            if let Some(player) = player.as_deref() {
                full_output.platform_output.cursor_icon = player
                    .ui()
                    .downcast_ref::<DesktopUiBackend>()
                    .unwrap_or_else(|| panic!("UI Backend should be DesktopUiBackend"))
                    .cursor();
            }
        }
        self.egui_winit.handle_platform_output(
            &self.window,
            &self.egui_ctx,
            full_output.platform_output,
        );

        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

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

        let movie_view = if let Some(player) = player.as_deref_mut() {
            let renderer = player
                .renderer_mut()
                .downcast_mut::<WgpuRenderBackend<MovieView>>()
                .expect("Renderer must be correct type");
            Some(renderer.target())
        } else {
            None
        };

        {
            let surface_view = surface_texture.texture.create_view(&Default::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                label: Some("egui_render"),
                ..Default::default()
            });

            if let Some(movie_view) = movie_view {
                movie_view.render(&self.movie_view_renderer, &mut render_pass);
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

    pub fn show_open_dialog(&mut self) {
        self.gui.open_file_advanced()
    }
}

// try to load known unicode supporting fonts to draw cjk characters in egui
fn load_system_fonts(locale: LanguageIdentifier) -> anyhow::Result<egui::FontDefinitions> {
    let mut font_database = Database::default();
    font_database.load_system_fonts();

    let mut families = Vec::new();
    if let Some(windows_font) = match locale.language.as_str() {
        "ja" => Some(Family::Name("MS UI Gothic")),
        "zh" => Some(match locale.to_string().as_str() {
            "zh-CN" => Family::Name("Microsoft YaHei"),
            _ => Family::Name("Microsoft JhengHei"),
        }),
        "ko" => Some(Family::Name("Malgun Gothic")),
        _ => None,
    } {
        families.push(windows_font);
    }
    if let Some(linux_font) = match locale.language.as_str() {
        "ja" => Some(Family::Name("Noto Sans CJK JP")),
        "zh" => Some(match locale.to_string().as_str() {
            "zh-CN" => Family::Name("Noto Sans CJK SC"),
            _ => Family::Name("Noto Sans CJK TC"),
        }),
        "ko" => Some(Family::Name("Noto Sans CJK KR")),
        _ => Some(Family::Name("Noto Sans")),
    } {
        families.push(linux_font);
    }
    families.extend(
        [
            Family::Name("Arial Unicode MS"), // macos
            Family::SansSerif,
        ]
        .iter(),
    );

    let system_unicode_fonts = Query {
        families: &families,
        ..Query::default()
    };

    let id = font_database
        .query(&system_unicode_fonts)
        .ok_or(anyhow!("no unicode fonts found!"))?;
    let (name, src, index) = font_database
        .face(id)
        .map(|f| (f.post_script_name.clone(), f.source.clone(), f.index))
        .expect("id not found in font database");

    let mut fontdata = match src {
        Source::File(path) => {
            let data = std::fs::read(path)?;
            egui::FontData::from_owned(data)
        }
        Source::Binary(bin) | Source::SharedFile(_, bin) => {
            let data = bin.as_ref().as_ref().to_vec();
            egui::FontData::from_owned(data)
        }
    };
    fontdata.index = index;
    tracing::info!("loaded cjk fallback font \"{}\"", name);

    let mut fd = egui::FontDefinitions::default();
    fd.font_data.insert(name.clone(), fontdata);
    fd.families
        .get_mut(&egui::FontFamily::Proportional)
        .expect("font family not found")
        .push(name);

    Ok(fd)
}
