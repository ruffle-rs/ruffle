use crate::custom_event::RuffleEvent;
use egui::*;
use std::rc::Rc;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::window::Window;

/// Integration layer conneting wgpu+winit to egui.
pub struct GuiController {
    egui_ctx: egui::Context,
    egui_winit: egui_winit::State,
    egui_renderer: egui_wgpu::renderer::Renderer,
    gui: RuffleGui,
    window: Rc<Window>,
}

impl GuiController {
    pub fn new<T: ruffle_render_wgpu::target::RenderTarget>(
        renderer: &ruffle_render_wgpu::backend::WgpuRenderBackend<T>,
        window: Rc<Window>,
        event_loop: &EventLoop<RuffleEvent>,
    ) -> Self {
        let egui_ctx = Context::default();
        let mut egui_winit = egui_winit::State::new(event_loop);
        egui_winit.set_pixels_per_point(window.scale_factor() as f32);
        egui_winit
            .set_max_texture_side(renderer.descriptors().limits.max_texture_dimension_2d as usize);

        let target_format = renderer.target().format();
        let egui_renderer = egui_wgpu::Renderer::new(renderer.device(), target_format, None, 1);
        let event_loop = event_loop.create_proxy();
        let gui = RuffleGui::new(event_loop);
        Self {
            egui_ctx,
            egui_winit,
            egui_renderer,
            gui,
            window,
        }
    }

    #[must_use]
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        let response = self.egui_winit.on_event(&self.egui_ctx, event);
        if response.repaint {
            self.window.request_redraw();
        }
        response.consumed
    }

    pub fn render(
        &mut self,
        render_ctx: ruffle_render_wgpu::backend::RenderCallbackParams,
    ) -> Vec<wgpu::CommandBuffer> {
        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |context| {
            self.gui.update(context);
        });

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
            render_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("egui encoder"),
                });

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(
                render_ctx.device,
                render_ctx.queue,
                *id,
                image_delta,
            );
        }

        let mut command_buffers = self.egui_renderer.update_buffers(
            render_ctx.device,
            render_ctx.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_ctx.texture_view,
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
        command_buffers
    }
}

/// The main controller for the Ruffle GUI.
pub struct RuffleGui {
    event_loop: EventLoopProxy<RuffleEvent>,
}

impl RuffleGui {
    fn new(event_loop: EventLoopProxy<RuffleEvent>) -> Self {
        Self { event_loop }
    }
}

impl RuffleGui {
    /// Renders all of the main Ruffle UI, including the main menu and context menus.
    fn update(&mut self, egui_ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(egui_ctx, |ui| {
            self.main_menu_bar(ui);
        });
    }

    /// Renders the main menu bar at the top of the window.
    fn main_menu_bar(&mut self, ui: &mut egui::Ui) {
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

                if Button::new("Open...")
                    .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                    .ui(ui)
                    .clicked()
                {
                    self.open_file(ui);
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
    }

    fn open_file(&mut self, ui: &mut egui::Ui) {
        let _ = self.event_loop.send_event(RuffleEvent::OpenFile);
        ui.close_menu();
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
        // TODO
        ui.close_menu();
    }
}
