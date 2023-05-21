mod movie;

use crate::custom_event::RuffleEvent;
use crate::gui::movie::MovieView;
use anyhow::{anyhow, Result};
use egui::*;
use ruffle_render_wgpu::backend::request_adapter_and_device;
use ruffle_render_wgpu::descriptors::Descriptors;
use ruffle_render_wgpu::utils::{format_list, get_backend_names};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::window::Window;

/// Integration layer conneting wgpu+winit to egui.
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
    movie_view: MovieView,
}

impl GuiController {
    pub fn new(
        window: Rc<Window>,
        event_loop: &EventLoop<RuffleEvent>,
        trace_path: Option<&Path>,
        backend: wgpu::Backends,
        power_preference: wgpu::PowerPreference,
    ) -> Result<Self> {
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
        let descriptors = Descriptors::new(adapter, device, queue);
        let egui_ctx = Context::default();
        let mut egui_winit = egui_winit::State::new(event_loop);
        egui_winit.set_pixels_per_point(window.scale_factor() as f32);
        egui_winit.set_max_texture_side(descriptors.limits.max_texture_dimension_2d as usize);
        let surface_format = surface
            .get_capabilities(&descriptors.adapter)
            .formats
            .first()
            .cloned()
            .expect("At least one format should be supported");

        let game_view = MovieView::new(&descriptors.device, surface_format);
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
            movie_view: game_view,
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
        }
        let response = self.egui_winit.on_event(&self.egui_ctx, event);
        if response.repaint {
            self.window.request_redraw();
        }
        response.consumed
    }

    pub fn render(&mut self, movie: &wgpu::Texture) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Surface became unavailable");

        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |context| {
            self.gui.update(context);
        });
        self.repaint_after = full_output.repaint_after;

        self.egui_winit.handle_platform_output(
            &self.window,
            &self.egui_ctx,
            full_output.platform_output,
        );
        let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes);

        let size = self.window.inner_size();
        let scale_factor = self.window.scale_factor() as f32;
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
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

            // First draw the movie - this also clears the surface
            self.movie_view
                .render(&self.descriptors.device, &mut encoder, movie, &surface_view);

            // Then any UI
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui_render"),
            });

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

    pub fn set_ui_visible(&mut self, value: bool) {
        self.gui.set_ui_visible(value);
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

/// The main controller for the Ruffle GUI.
pub struct RuffleGui {
    event_loop: EventLoopProxy<RuffleEvent>,
    esc_start_time: Option<Instant>,
    open_url_text: String,
    is_esc_down: bool,
    is_ui_visible: bool,
    is_about_visible: bool,
    is_open_url_prompt_visible: bool,
    context_menu: Vec<ruffle_core::ContextMenuItem>,
}

impl RuffleGui {
    fn new(event_loop: EventLoopProxy<RuffleEvent>) -> Self {
        Self {
            event_loop,
            esc_start_time: None,
            open_url_text: String::new(),
            is_esc_down: false,
            is_ui_visible: false,
            is_about_visible: false,
            is_open_url_prompt_visible: false,
            context_menu: vec![],
        }
    }

    /// Renders all of the main Ruffle UI, including the main menu and context menus.
    fn update(&mut self, egui_ctx: &egui::Context) {
        egui_ctx.input_mut(|input| {
            // Listen for Esc press to toggle GUI.
            let mut esc_this_frame = false;
            if input.key_down(Key::Escape) {
                if !self.is_esc_down {
                    esc_this_frame = true;
                }
                self.is_esc_down = true;
            } else {
                self.is_esc_down = false;
            }

            if self.is_ui_visible {
                if esc_this_frame {
                    self.set_ui_visible(false);
                }
            } else if self.is_esc_down {
                // Require holding Esc to show the UI to avoid interfering with games that use Esc.
                if esc_this_frame {
                    self.esc_start_time = Some(Instant::now());
                }

                if let Some(esc_start_time) = self.esc_start_time {
                    let esc_duration = Instant::now().duration_since(esc_start_time);
                    const HOLD_ESCAPE_TIME: f32 = 0.5;
                    if esc_duration >= Duration::from_secs_f32(HOLD_ESCAPE_TIME) {
                        self.set_ui_visible(true);
                    }
                }
            }
        });

        if self.is_ui_visible {
            self.main_menu_bar(egui_ctx);
            self.about_window(egui_ctx);
            self.open_url_prompt(egui_ctx);
        }

        if !self.context_menu.is_empty() {
            self.context_menu(egui_ctx);
        }
    }

    pub fn set_ui_visible(&mut self, value: bool) {
        self.is_ui_visible = value;
        self.esc_start_time = None;
    }

    pub fn show_context_menu(&mut self, menu: Vec<ruffle_core::ContextMenuItem>) {
        self.context_menu = menu;
    }

    pub fn is_context_menu_visible(&self) -> bool {
        !self.context_menu.is_empty()
    }

    /// Renders the main menu bar at the top of the window.
    fn main_menu_bar(&mut self, egui_ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(egui_ctx, |ui| {
            // TODO(mike): Make some MenuItem struct with shortcut info to handle this more cleanly.
            if ui.ctx().input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::O))
            }) {
                self.open_file(ui);
            }
            if ui.ctx().input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Q))
            }) {
                self.request_exit(ui);
            }

            menu::bar(ui, |ui| {
                menu::menu_button(ui, "File", |ui| {
                    let mut shortcut;
                    shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);

                    if Button::new("Open File...")
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui)
                        .clicked()
                    {
                        self.open_file(ui);
                    }

                    if Button::new("Open URL...").ui(ui).clicked() {
                        self.show_open_url_prompt(ui);
                    }

                    ui.separator();

                    shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);
                    if Button::new("Exit")
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui)
                        .clicked()
                    {
                        self.request_exit(ui);
                    }
                });
                menu::menu_button(ui, "Help", |ui| {
                    if ui.button("Discord").clicked() {
                        self.launch_discord(ui);
                    }
                    ui.separator();
                    if ui.button("About Ruffle...").clicked() {
                        self.show_about_screen(ui);
                    }
                })
            });
        });
    }

    fn about_window(&mut self, egui_ctx: &egui::Context) {
        egui::Window::new("About Ruffle")
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.is_about_visible)
            .show(egui_ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Ruffle")
                            .color(Color32::from_rgb(0xFF, 0xAD, 0x33))
                            .size(24.0),
                    );
                    ui.label(crate::RUFFLE_VERSION);
                })
            });
    }

    /// Renders the right-click context menu.
    fn context_menu(&mut self, egui_ctx: &egui::Context) {
        let mut item_clicked = false;
        let mut menu_visible = false;
        // TODO: What is the proper way in egui to spawn a random context menu?
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(egui_ctx, |_| {})
            .response
            .context_menu(|ui| {
                menu_visible = true;
                for (i, item) in self.context_menu.iter().enumerate() {
                    if i != 0 && item.separator_before {
                        ui.separator();
                    }
                    let clicked = if item.checked {
                        Checkbox::new(&mut true, &item.caption).ui(ui).clicked()
                    } else {
                        Button::new(&item.caption).ui(ui).clicked()
                    };
                    if clicked {
                        let _ = self
                            .event_loop
                            .send_event(RuffleEvent::ContextMenuItemClicked(i));
                        item_clicked = true;
                    }
                }
            });

        if item_clicked
            || !menu_visible
            || egui_ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
        {
            // Hide menu.
            self.context_menu.clear();
        }
    }

    fn open_file(&mut self, ui: &mut egui::Ui) {
        let _ = self.event_loop.send_event(RuffleEvent::OpenFile);
        ui.close_menu();
    }

    fn open_url_prompt(&mut self, egui_ctx: &egui::Context) {
        let mut close_prompt = false;
        egui::Window::new("Open URL")
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .open(&mut self.is_open_url_prompt_visible)
            .show(egui_ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let (enter_pressed, esc_pressed) = ui.ctx().input_mut(|input| {
                        (
                            input.consume_key(Modifiers::NONE, Key::Enter),
                            input.consume_key(Modifiers::NONE, Key::Escape),
                        )
                    });
                    ui.text_edit_singleline(&mut self.open_url_text);
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() || enter_pressed {
                            if let Ok(url) = url::Url::parse(&self.open_url_text) {
                                let _ = self.event_loop.send_event(RuffleEvent::OpenURL(url));
                            } else {
                                // TODO: Show error prompt.
                                tracing::error!("Invalid URL: {}", self.open_url_text);
                            }
                            close_prompt = true;
                        }
                        if ui.button("Cancel").clicked() || esc_pressed {
                            close_prompt = true;
                        }
                    });
                });
            });
        if close_prompt {
            self.is_open_url_prompt_visible = false;
        }
    }

    fn request_exit(&mut self, ui: &mut egui::Ui) {
        let _ = self.event_loop.send_event(RuffleEvent::ExitRequested);
        ui.close_menu();
    }

    fn launch_discord(&mut self, ui: &mut egui::Ui) {
        const RUFFLE_DISCORD_URL: &str = "https://discord.gg/ruffle";
        let _ = webbrowser::open(RUFFLE_DISCORD_URL);
        ui.close_menu();
    }

    fn show_about_screen(&mut self, ui: &mut egui::Ui) {
        self.is_about_visible = true;
        ui.close_menu();
    }

    fn show_open_url_prompt(&mut self, ui: &mut egui::Ui) {
        self.is_open_url_prompt_visible = true;
        ui.close_menu();
    }
}
