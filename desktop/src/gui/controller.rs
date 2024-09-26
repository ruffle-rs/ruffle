use crate::backends::DesktopUiBackend;
use crate::custom_event::RuffleEvent;
use crate::gui::movie::{MovieView, MovieViewRenderer};
use crate::gui::theme::ThemeController;
use crate::gui::{RuffleGui, MENU_HEIGHT};
use crate::player::{LaunchOptions, PlayerController};
use crate::preferences::GlobalPreferences;
use anyhow::anyhow;
use egui::{Context, ViewportId};
use fontdb::{Database, Family, Query, Source};
use ruffle_core::{Player, PlayerEvent};
use ruffle_render_wgpu::backend::{request_adapter_and_device, WgpuRenderBackend};
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::utils::{format_list, get_backend_names};
use std::sync::{Arc, MutexGuard};
use std::time::{Duration, Instant};
use unic_langid::LanguageIdentifier;
use url::Url;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Theme, Window};

use super::{DialogDescriptor, FilePicker};

/// Integration layer connecting wgpu+winit to egui.
pub struct GuiController {
    descriptors: Arc<Descriptors>,
    egui_winit: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    gui: RuffleGui,
    window: Arc<Window>,
    last_update: Instant,
    repaint_after: Duration,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    movie_view_renderer: Arc<MovieViewRenderer>,
    // Note that `window.get_inner_size` can change at any point on x11, even between two lines of code.
    // Use this instead.
    size: PhysicalSize<u32>,
    /// If this is set, we should not render the main menu.
    no_gui: bool,
    theme_controller: ThemeController,
}

impl GuiController {
    pub async fn new(
        window: Arc<Window>,
        event_loop: &EventLoop<RuffleEvent>,
        preferences: GlobalPreferences,
        font_database: &Database,
        initial_movie_url: Option<Url>,
        no_gui: bool,
    ) -> anyhow::Result<Self> {
        let (instance, backend) = create_wgpu_instance(preferences.graphics_backends().into())?;
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window.as_ref())?)
        }?;
        let (adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
            backend,
            &instance,
            Some(&surface),
            preferences.graphics_power_preference().into(),
            preferences.cli.trace_path(),
        ))
        .map_err(|e| anyhow!(e.to_string()))?;
        let adapter_info = adapter.get_info();
        tracing::info!(
            "Using graphics API {} on {} (type: {:?})",
            adapter_info.backend.to_str(),
            adapter_info.name,
            adapter_info.device_type
        );
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
                desired_maximum_frame_latency: 2,
                alpha_mode: Default::default(),
                view_formats: Default::default(),
            },
        );
        let event_loop = event_loop.create_proxy();
        let descriptors = Descriptors::new(instance, adapter, device, queue);
        let egui_ctx = Context::default();

        let theme_controller =
            ThemeController::new(window.clone(), preferences.clone(), egui_ctx.clone()).await;
        let mut egui_winit = egui_winit::State::new(
            egui_ctx,
            ViewportId::ROOT,
            window.as_ref(),
            None,
            None,
            None,
        );
        egui_winit.set_max_texture_side(descriptors.limits.max_texture_dimension_2d as usize);

        let movie_view_renderer = Arc::new(MovieViewRenderer::new(
            &descriptors.device,
            surface_format,
            window.fullscreen().is_none() && !no_gui,
            size.height,
            window.scale_factor(),
        ));
        let egui_renderer =
            egui_wgpu::Renderer::new(&descriptors.device, surface_format, None, 1, true);
        let descriptors = Arc::new(descriptors);
        let gui = RuffleGui::new(
            Arc::downgrade(&window),
            event_loop,
            initial_movie_url.clone(),
            LaunchOptions::from(&preferences),
            preferences.clone(),
        );
        let system_fonts =
            load_system_fonts(font_database, preferences.language().to_owned()).unwrap_or_default();
        egui_winit.egui_ctx().set_fonts(system_fonts);

        egui_extras::install_image_loaders(egui_winit.egui_ctx());

        Ok(Self {
            descriptors,
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
            no_gui,
            theme_controller,
        })
    }

    pub fn set_theme(&self, theme: Theme) {
        self.theme_controller.set_theme(theme);
    }

    pub fn descriptors(&self) -> &Arc<Descriptors> {
        &self.descriptors
    }

    pub fn file_picker(&self) -> FilePicker {
        self.gui.dialogs.file_picker()
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.size = size;
            self.reconfigure_surface();
        }
    }

    pub fn reconfigure_surface(&mut self) {
        self.surface.configure(
            &self.descriptors.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width: self.size.width,
                height: self.size.height,
                present_mode: Default::default(),
                desired_maximum_frame_latency: 2,
                alpha_mode: Default::default(),
                view_formats: Default::default(),
            },
        );
        self.movie_view_renderer.update_resolution(
            &self.descriptors,
            self.window.fullscreen().is_none() && !self.no_gui,
            self.size.height,
            self.window.scale_factor(),
        );
    }

    #[must_use]
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::Resized(size) = &event {
            self.resize(*size);
        }

        if let WindowEvent::ThemeChanged(theme) = &event {
            self.set_theme(*theme);
        }

        if matches!(
            &event,
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    logical_key: Key::Named(NamedKey::Tab),
                    ..
                },
                ..
            }
        ) {
            // Prevent egui from consuming the Tab key.
            return false;
        }

        let response = self.egui_winit.on_window_event(&self.window, event);
        if response.repaint {
            self.window.request_redraw();
        }
        response.consumed
    }

    pub fn create_movie(
        &mut self,
        player: &mut PlayerController,
        opt: LaunchOptions,
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
        let surface_texture = match self.surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(e @ (SurfaceError::Lost | SurfaceError::Outdated)) => {
                // Reconfigure the surface if lost or outdated.
                // Some sources suggest ignoring `Outdated` and waiting for the next frame,
                // but I suspect this advice is related explicitly to resizing,
                // because the future resize event will reconfigure the surface.
                // However, resizing is not the only possible reason for the surface
                // to become outdated (resolution / refresh rate change, some internal
                // platform-specific reasons, wgpu bugs?).
                // Testing on Vulkan shows that reconfiguring the surface works in that case.
                tracing::warn!("Surface became unavailable: {:?}, reconfiguring", e);
                self.reconfigure_surface();
                return;
            }
            Err(e @ SurfaceError::Timeout) => {
                // An operation related to the surface took too long to complete.
                // This error may happen due to many reasons (GPU overload, GPU driver bugs, etc.),
                // the best thing we can do is skip a frame and wait.
                tracing::warn!("Surface became unavailable: {:?}, skipping a frame", e);
                return;
            }
            Err(SurfaceError::OutOfMemory) => {
                // Cannot help with that :(
                panic!("wgpu: Out of memory: no more memory left to allocate a new frame");
            }
        };

        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let show_menu = self.window.fullscreen().is_none() && !self.no_gui;
        let mut full_output = self.egui_winit.egui_ctx().run(raw_input, |context| {
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
        if !self.egui_winit.egui_ctx().wants_pointer_input() {
            if let Some(player) = player.as_deref() {
                full_output.platform_output.cursor_icon = player
                    .ui()
                    .downcast_ref::<DesktopUiBackend>()
                    .unwrap_or_else(|| panic!("UI Backend should be DesktopUiBackend"))
                    .cursor();
            }
        }
        self.egui_winit
            .handle_platform_output(&self.window, full_output.platform_output);

        let clipped_primitives = self
            .egui_winit
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let scale_factor = self.window.scale_factor() as f32;
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
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

            let mut render_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
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
                })
                .forget_lifetime();

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

    pub fn show_context_menu(
        &mut self,
        menu: Vec<ruffle_core::ContextMenuItem>,
        close_event: PlayerEvent,
    ) {
        self.gui.show_context_menu(menu, close_event);
    }

    pub fn is_context_menu_visible(&self) -> bool {
        self.gui.is_context_menu_visible()
    }

    pub fn needs_render(&self) -> bool {
        Instant::now().duration_since(self.last_update) >= self.repaint_after
    }

    pub fn show_open_dialog(&mut self) {
        self.gui.dialogs.open_file_advanced()
    }

    pub fn open_dialog(&mut self, dialog_event: DialogDescriptor) {
        self.gui.dialogs.open_dialog(dialog_event);
    }
}

fn create_wgpu_instance(
    preferred_backends: wgpu::Backends,
) -> anyhow::Result<(wgpu::Instance, wgpu::Backends)> {
    for backend in preferred_backends.iter() {
        if let Some(instance) = try_wgpu_backend(backend) {
            tracing::info!(
                "Using preferred backend {}",
                format_list(&get_backend_names(backend), "and")
            );
            return Ok((instance, backend));
        }
    }

    tracing::warn!(
        "Preferred backend(s) of {} not available; falling back to any",
        format_list(&get_backend_names(preferred_backends), "or")
    );

    for backend in wgpu::Backends::all() - preferred_backends {
        if let Some(instance) = try_wgpu_backend(backend) {
            tracing::info!(
                "Using fallback backend {}",
                format_list(&get_backend_names(backend), "and")
            );
            return Ok((instance, backend));
        }
    }

    Err(anyhow!(
        "No compatible graphics backends of any kind were available"
    ))
}

fn try_wgpu_backend(backend: wgpu::Backends) -> Option<wgpu::Instance> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: backend,
        flags: wgpu::InstanceFlags::default().with_env(),
        ..Default::default()
    });
    if instance.enumerate_adapters(backend).is_empty() {
        None
    } else {
        Some(instance)
    }
}

// try to load known unicode supporting fonts to draw cjk characters in egui
fn load_system_fonts(
    font_database: &Database,
    locale: LanguageIdentifier,
) -> anyhow::Result<egui::FontDefinitions> {
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
