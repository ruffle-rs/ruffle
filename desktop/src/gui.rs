mod controller;
mod movie;

use crate::custom_event::RuffleEvent;
use chrono::DateTime;
use egui::*;
use winit::event_loop::EventLoopProxy;

pub use controller::GuiController;
pub use movie::MovieView;

/// Size of the top menu bar in pixels.
/// This is the offset at which the movie will be shown,
/// and added to the window size if trying to match a movie.
pub const MENU_HEIGHT: u32 = 24;

/// The main controller for the Ruffle GUI.
pub struct RuffleGui {
    event_loop: EventLoopProxy<RuffleEvent>,
    open_url_text: String,
    is_about_visible: bool,
    is_open_url_prompt_visible: bool,
    context_menu: Vec<ruffle_core::ContextMenuItem>,
}

impl RuffleGui {
    fn new(event_loop: EventLoopProxy<RuffleEvent>) -> Self {
        Self {
            event_loop,
            open_url_text: String::new(),
            is_about_visible: false,
            is_open_url_prompt_visible: false,
            context_menu: vec![],
        }
    }

    /// Renders all of the main Ruffle UI, including the main menu and context menus.
    fn update(&mut self, egui_ctx: &egui::Context, show_menu: bool, has_movie: bool) {
        if show_menu {
            self.main_menu_bar(egui_ctx, has_movie);
        }

        self.about_window(egui_ctx);
        self.open_url_prompt(egui_ctx);

        if !self.context_menu.is_empty() {
            self.context_menu(egui_ctx);
        }
    }

    pub fn show_context_menu(&mut self, menu: Vec<ruffle_core::ContextMenuItem>) {
        self.context_menu = menu;
    }

    pub fn is_context_menu_visible(&self) -> bool {
        !self.context_menu.is_empty()
    }

    /// Renders the main menu bar at the top of the window.
    fn main_menu_bar(&mut self, egui_ctx: &egui::Context, has_movie: bool) {
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

                    if ui.add_enabled(has_movie, Button::new("Close")).clicked() {
                        self.close_movie(ui);
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
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.is_about_visible)
            .show(egui_ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Ruffle")
                            .color(Color32::from_rgb(0xFF, 0xAD, 0x33))
                            .size(32.0),
                    );
                    Grid::new("about_ruffle_version_info")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Version");
                            ui.label(env!("CARGO_PKG_VERSION"));
                            ui.end_row();

                            ui.label("Channel");
                            ui.label(env!("CFG_RELEASE_CHANNEL"));
                            ui.end_row();

                            ui.label("Build Time");
                            ui.label(
                                DateTime::parse_from_rfc3339(env!("VERGEN_BUILD_TIMESTAMP"))
                                    .map(|t| t.format("%c").to_string())
                                    .unwrap_or_else(|_| env!("VERGEN_BUILD_TIMESTAMP").to_string()),
                            );
                            ui.end_row();

                            ui.label("Commit ref");
                            ui.hyperlink_to(
                                env!("VERGEN_GIT_SHA"),
                                format!(
                                    "https://github.com/ruffle-rs/ruffle/commit/{}",
                                    env!("VERGEN_GIT_SHA")
                                ),
                            );
                            ui.end_row();

                            ui.label("Commit date");
                            ui.label(
                                DateTime::parse_from_rfc3339(env!("VERGEN_GIT_COMMIT_TIMESTAMP"))
                                    .map(|t| t.format("%c").to_string())
                                    .unwrap_or_else(|_| {
                                        env!("VERGEN_GIT_COMMIT_TIMESTAMP").to_string()
                                    }),
                            );
                            ui.end_row();

                            ui.label("Build Features");
                            ui.horizontal_wrapped(|ui| {
                                ui.label(env!("VERGEN_CARGO_FEATURES").replace(',', ", "));
                            });
                            ui.end_row();
                        });

                    ui.horizontal(|ui| {
                        ui.hyperlink_to("Website", "https://ruffle.rs");
                        ui.hyperlink_to("Github", "https://github.com/ruffle-rs/ruffle/");
                        ui.hyperlink_to("Discord", "https://discord.gg/ruffle");
                        ui.hyperlink_to("Sponsor", "https://opencollective.com/ruffle/");
                        ui.shrink_width_to_current();
                    });
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

    fn close_movie(&mut self, ui: &mut egui::Ui) {
        let _ = self.event_loop.send_event(RuffleEvent::CloseFile);
        ui.close_menu();
    }

    fn open_url_prompt(&mut self, egui_ctx: &egui::Context) {
        let mut close_prompt = false;
        egui::Window::new("Open URL")
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
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
