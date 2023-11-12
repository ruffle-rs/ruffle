mod controller;
mod movie;
mod open_dialog;

pub use controller::GuiController;
pub use movie::MovieView;
use std::borrow::Cow;
use url::Url;

use crate::custom_event::RuffleEvent;
use crate::gui::open_dialog::OpenDialog;
use crate::player::PlayerOptions;
use chrono::DateTime;
use egui::*;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{static_loader, Loader};
use rfd::FileDialog;
use ruffle_core::backend::ui::US_ENGLISH;
use ruffle_core::debug_ui::Message as DebugMessage;
use ruffle_core::Player;
use std::collections::HashMap;
use std::fs;
use std::sync::MutexGuard;
use sys_locale::get_locale;
use unic_langid::LanguageIdentifier;
use winit::event_loop::EventLoopProxy;

const VERGEN_UNKNOWN: &str = "VERGEN_IDEMPOTENT_OUTPUT";

static_loader! {
    static TEXTS = {
        locales: "./assets/texts",
        fallback_language: "en-US"
    };
}

pub fn text<'a>(locale: &LanguageIdentifier, id: &'a str) -> Cow<'a, str> {
    TEXTS.lookup(locale, id).map(Cow::Owned).unwrap_or_else(|| {
        tracing::error!("Unknown desktop text id '{id}'");
        Cow::Borrowed(id)
    })
}

#[allow(dead_code)]
pub fn text_with_args<'a, T: AsRef<str>>(
    locale: &LanguageIdentifier,
    id: &'a str,
    args: &HashMap<T, FluentValue>,
) -> Cow<'a, str> {
    TEXTS
        .lookup_with_args(locale, id, args)
        .map(Cow::Owned)
        .unwrap_or_else(|| {
            tracing::error!("Unknown desktop text id '{id}'");
            Cow::Borrowed(id)
        })
}

/// Size of the top menu bar in pixels.
/// This is the offset at which the movie will be shown,
/// and added to the window size if trying to match a movie.
pub const MENU_HEIGHT: u32 = 24;

/// The main controller for the Ruffle GUI.
pub struct RuffleGui {
    event_loop: EventLoopProxy<RuffleEvent>,
    is_about_visible: bool,
    is_volume_visible: bool,
    volume_controls: VolumeControls,
    is_open_dialog_visible: bool,
    context_menu: Vec<ruffle_core::ContextMenuItem>,
    open_dialog: OpenDialog,
    locale: LanguageIdentifier,
    default_player_options: PlayerOptions,
    currently_opened: Option<(Url, PlayerOptions)>,
    was_suspended_before_debug: bool,
}

impl RuffleGui {
    fn new(
        event_loop: EventLoopProxy<RuffleEvent>,
        default_path: Option<Url>,
        default_player_options: PlayerOptions,
    ) -> Self {
        // TODO: language negotiation + https://github.com/1Password/sys-locale/issues/14
        // This should also be somewhere else so it can be supplied through UiBackend too

        let preferred_locale = get_locale();
        let locale = preferred_locale
            .and_then(|l| l.parse().ok())
            .unwrap_or_else(|| US_ENGLISH.clone());

        Self {
            is_about_visible: false,
            is_volume_visible: false,
            volume_controls: VolumeControls::new(false, default_player_options.volume * 100.0),
            is_open_dialog_visible: false,
            was_suspended_before_debug: false,

            context_menu: vec![],
            open_dialog: OpenDialog::new(
                default_player_options.clone(),
                default_path,
                event_loop.clone(),
                locale.clone(),
            ),

            event_loop,
            locale,
            default_player_options,
            currently_opened: None,
        }
    }

    /// Renders all of the main Ruffle UI, including the main menu and context menus.
    fn update(
        &mut self,
        egui_ctx: &egui::Context,
        show_menu: bool,
        mut player: Option<&mut Player>,
        menu_height_offset: f64,
    ) {
        if show_menu {
            self.main_menu_bar(egui_ctx, player.as_deref_mut());
        }

        self.about_window(egui_ctx);
        self.open_dialog(egui_ctx);

        if let Some(player) = player {
            let was_suspended = player.debug_ui().should_suspend_player();
            player.show_debug_ui(egui_ctx, menu_height_offset);
            if was_suspended != player.debug_ui().should_suspend_player() {
                if player.debug_ui().should_suspend_player() {
                    self.was_suspended_before_debug = !player.is_playing();
                    player.set_is_playing(false);
                } else {
                    player.set_is_playing(!self.was_suspended_before_debug);
                }
            }
            for item in player.debug_ui().items_to_save() {
                std::thread::spawn(move || {
                    if let Some(path) = FileDialog::new()
                        .set_file_name(&item.suggested_name)
                        .save_file()
                    {
                        if let Err(e) = fs::write(&path, item.data) {
                            tracing::error!(
                                "Couldn't save {} to {path:?}: {e}",
                                item.suggested_name,
                            );
                        }
                    }
                });
            }

            self.volume_window(egui_ctx, Some(player));
        } else {
            self.volume_window(egui_ctx, None);
        }

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

    /// Notifies the GUI that a new player was created.
    fn on_player_created(
        &mut self,
        opt: PlayerOptions,
        movie_url: Url,
        mut player: MutexGuard<Player>,
    ) {
        self.currently_opened = Some((movie_url.clone(), opt.clone()));

        // Update dialog state to reflect the newly-opened movie's options.
        self.is_open_dialog_visible = false;
        self.open_dialog = OpenDialog::new(
            opt,
            Some(movie_url),
            self.event_loop.clone(),
            self.locale.clone(),
        );

        player.set_volume(self.volume_controls.get_volume());
    }

    /// Renders the main menu bar at the top of the window.
    fn main_menu_bar(&mut self, egui_ctx: &egui::Context, mut player: Option<&mut Player>) {
        egui::TopBottomPanel::top("menu_bar").show(egui_ctx, |ui| {
            // TODO(mike): Make some MenuItem struct with shortcut info to handle this more cleanly.
            if ui.ctx().input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::O))
            }) {
                self.open_file(ui);
            }
            if ui.ctx().input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::O))
            }) {
                self.open_file_advanced();
            }
            if ui.ctx().input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Q))
            }) {
                self.request_exit(ui);
            }
            if ui.ctx().input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::P))
            }) {
                if let Some(player) = &mut player {
                    player.set_is_playing(!player.is_playing());
                }
            }

            menu::bar(ui, |ui| {
                menu::menu_button(ui, text(&self.locale, "file-menu"), |ui| {
                    let mut shortcut;

                    shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
                    if Button::new(text(&self.locale, "file-menu-open-quick"))
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui)
                        .clicked()
                    {
                        self.open_file(ui);
                    }

                    shortcut = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::O);
                    if Button::new(text(&self.locale, "file-menu-open-advanced"))
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui).clicked() {
                        ui.close_menu();
                        self.open_file_advanced();
                    }

                    if ui.add_enabled(player.is_some(), Button::new(text(&self.locale, "file-menu-reload"))).clicked() {
                        self.reload_movie(ui);
                    }

                    if ui.add_enabled(player.is_some(), Button::new(text(&self.locale, "file-menu-close"))).clicked() {
                        self.close_movie(ui);
                    }

                    ui.separator();

                    shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);
                    if Button::new(text(&self.locale, "file-menu-exit"))
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui)
                        .clicked()
                    {
                        self.request_exit(ui);
                    }
                });
                menu::menu_button(ui, text(&self.locale, "controls-menu"), |ui| {
                    ui.add_enabled_ui(player.is_some(), |ui| {
                        let playing = player.as_ref().map(|p| p.is_playing()).unwrap_or_default();
                        let pause_shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::P);
                        if Button::new(text(&self.locale, if playing { "controls-menu-suspend" } else { "controls-menu-resume" })).shortcut_text(ui.ctx().format_shortcut(&pause_shortcut)).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.set_is_playing(!player.is_playing());
                            }
                        }
                    });
                    if Button::new(text(&self.locale, "controls-menu-volume")).ui(ui).clicked() {
                        self.show_volume_screen(ui);
                    }
                });
                menu::menu_button(ui, text(&self.locale, "debug-menu"), |ui| {
                    ui.add_enabled_ui(player.is_some(), |ui| {
                        if Button::new(text(&self.locale, "debug-menu-open-stage")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::TrackStage);
                            }
                        }
                        if Button::new(text(&self.locale, "debug-menu-open-movie")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::TrackTopLevelMovie);
                            }
                        }
                        if Button::new(text(&self.locale, "debug-menu-open-movie-list")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::ShowKnownMovies);
                            }
                        }
                        if Button::new(text(&self.locale, "debug-menu-search-display-objects")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::SearchForDisplayObject);
                            }
                        }
                    });
                });
                menu::menu_button(ui, text(&self.locale, "help-menu"), |ui| {
                    if ui.button(text(&self.locale, "help-menu-join-discord")).clicked() {
                        self.launch_website(ui, "https://discord.gg/ruffle");
                    }
                    if ui.button(text(&self.locale, "help-menu-report-a-bug")).clicked() {
                        self.launch_website(ui, "https://github.com/ruffle-rs/ruffle/issues/new?assignees=&labels=bug&projects=&template=bug_report.yml");
                    }
                    if ui.button(text(&self.locale, "help-menu-sponsor-development")).clicked() {
                        self.launch_website(ui, "https://opencollective.com/ruffle/");
                    }
                    if ui.button(text(&self.locale, "help-menu-translate-ruffle")).clicked() {
                        self.launch_website(ui, "https://crowdin.com/project/ruffle");
                    }
                    ui.separator();
                    if ui.button(text(&self.locale, "help-menu-about")).clicked() {
                        self.show_about_screen(ui);
                    }
                });
            });
        });
    }

    /// Renders the About Ruffle window.
    fn about_window(&mut self, egui_ctx: &egui::Context) {
        egui::Window::new(text(&self.locale, "about-ruffle"))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.is_about_visible)
            .show(egui_ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add(
                        Image::new(egui::include_image!("../assets/about_logo.png"))
                            .max_width(350.0),
                    );
                    Grid::new("about_ruffle_version_info")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(text(&self.locale, "about-ruffle-version"));
                            ui.label(env!("CARGO_PKG_VERSION"));
                            ui.end_row();

                            ui.label(text(&self.locale, "about-ruffle-channel"));
                            ui.label(env!("CFG_RELEASE_CHANNEL"));
                            ui.end_row();

                            let build_time = env!("VERGEN_BUILD_TIMESTAMP");
                            if build_time != VERGEN_UNKNOWN {
                                ui.label(text(&self.locale, "about-ruffle-build-time"));
                                ui.label(
                                    DateTime::parse_from_rfc3339(build_time)
                                        .map(|t| t.format("%c").to_string())
                                        .unwrap_or_else(|_| build_time.to_string()),
                                );
                                ui.end_row();
                            }

                            let sha = env!("VERGEN_GIT_SHA");
                            if sha != VERGEN_UNKNOWN {
                                ui.label(text(&self.locale, "about-ruffle-commit-ref"));
                                ui.hyperlink_to(
                                    sha,
                                    format!("https://github.com/ruffle-rs/ruffle/commit/{}", sha),
                                );
                                ui.end_row();
                            }

                            let commit_time = env!("VERGEN_GIT_COMMIT_TIMESTAMP");
                            if sha != VERGEN_UNKNOWN {
                                ui.label(text(&self.locale, "about-ruffle-commit-time"));
                                ui.label(
                                    DateTime::parse_from_rfc3339(commit_time)
                                        .map(|t| t.format("%c").to_string())
                                        .unwrap_or_else(|_| commit_time.to_string()),
                                );
                                ui.end_row();
                            }

                            ui.label(text(&self.locale, "about-ruffle-build-features"));
                            ui.horizontal_wrapped(|ui| {
                                ui.label(env!("VERGEN_CARGO_FEATURES").replace(',', ", "));
                            });
                            ui.end_row();
                        });

                    ui.horizontal(|ui| {
                        ui.hyperlink_to(
                            text(&self.locale, "about-ruffle-visit-website"),
                            "https://ruffle.rs",
                        );
                        ui.hyperlink_to(
                            text(&self.locale, "about-ruffle-visit-github"),
                            "https://github.com/ruffle-rs/ruffle/",
                        );
                        ui.hyperlink_to(
                            text(&self.locale, "about-ruffle-visit-discord"),
                            "https://discord.gg/ruffle",
                        );
                        ui.hyperlink_to(
                            text(&self.locale, "about-ruffle-visit-sponsor"),
                            "https://opencollective.com/ruffle/",
                        );
                        ui.shrink_width_to_current();
                    });
                })
            });
    }

    /// Renders the volume controls window.
    fn volume_window(&mut self, egui_ctx: &egui::Context, player: Option<&mut Player>) {
        egui::Window::new(text(&self.locale, "volume-controls"))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.is_volume_visible)
            .show(egui_ctx, |ui| {
                let mut changed_slider = false;

                let changed_checkbox = ui
                    .checkbox(
                        &mut self.volume_controls.is_muted,
                        text(&self.locale, "volume-controls-mute"),
                    )
                    .changed();

                ui.add_enabled_ui(!self.volume_controls.is_muted, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(text(&self.locale, "volume-controls-volume"));
                        changed_slider = ui
                            .add(Slider::new(&mut self.volume_controls.volume, 0.0..=100.0))
                            .changed();
                    });
                });

                if changed_checkbox || changed_slider {
                    if let Some(player) = player {
                        player.set_volume(self.volume_controls.get_volume());
                    }
                }
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
                        let button = Button::new(&item.caption).wrap(false);

                        ui.add_enabled(item.enabled, button).clicked()
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
        ui.close_menu();

        let _ = self
            .event_loop
            .send_event(RuffleEvent::BrowseAndOpen(Box::new(
                self.default_player_options.clone(),
            )));
    }

    fn open_file_advanced(&mut self) {
        self.is_open_dialog_visible = true;
    }

    fn close_movie(&mut self, ui: &mut egui::Ui) {
        let _ = self.event_loop.send_event(RuffleEvent::CloseFile);
        self.currently_opened = None;
        ui.close_menu();
    }

    fn reload_movie(&mut self, ui: &mut egui::Ui) {
        let _ = self.event_loop.send_event(RuffleEvent::CloseFile);
        if let Some((movie_url, opts)) = self.currently_opened.take() {
            let _ = self
                .event_loop
                .send_event(RuffleEvent::OpenURL(movie_url, opts.into()));
        }
        ui.close_menu();
    }

    fn open_dialog(&mut self, egui_ctx: &egui::Context) {
        if self.is_open_dialog_visible {
            let keep_open = self.open_dialog.show(egui_ctx);
            self.is_open_dialog_visible = keep_open;
        }
    }

    fn request_exit(&mut self, ui: &mut egui::Ui) {
        let _ = self.event_loop.send_event(RuffleEvent::ExitRequested);
        ui.close_menu();
    }

    fn launch_website(&mut self, ui: &mut egui::Ui, url: &str) {
        let _ = webbrowser::open(url);
        ui.close_menu();
    }

    fn show_about_screen(&mut self, ui: &mut egui::Ui) {
        self.is_about_visible = true;
        ui.close_menu();
    }

    fn show_volume_screen(&mut self, ui: &mut egui::Ui) {
        self.is_volume_visible = true;
        ui.close_menu();
    }
}

/// The volume controls of the Ruffle GUI.
pub struct VolumeControls {
    is_muted: bool,
    volume: f32,
}

impl VolumeControls {
    fn new(is_muted: bool, volume: f32) -> Self {
        Self { is_muted, volume }
    }

    /// Returns the volume between 0 and 1 (calculated out of the
    /// checkbox and the slider).
    fn get_volume(&self) -> f32 {
        if !self.is_muted {
            self.volume / 100.0
        } else {
            0.0
        }
    }
}
