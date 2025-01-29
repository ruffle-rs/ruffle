mod context_menu;
mod controller;
pub mod dialogs;
mod menu_bar;
mod movie;
mod picker;
mod theme;
mod widgets;

pub use controller::GuiController;
pub use dialogs::DialogDescriptor;
pub use movie::MovieView;
pub use picker::FilePicker;
use std::borrow::Cow;
pub use theme::ThemePreference;
use url::Url;

use crate::custom_event::RuffleEvent;
use crate::gui::context_menu::ContextMenu;
use crate::player::LaunchOptions;
use crate::preferences::GlobalPreferences;
use dialogs::Dialogs;
use egui::*;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{static_loader, Loader};
use menu_bar::MenuBar;
use rfd::AsyncFileDialog;
use ruffle_core::debug_ui::Message as DebugMessage;
use ruffle_core::{Player, PlayerEvent};
use std::collections::HashMap;
use std::sync::{MutexGuard, Weak};
use std::{fs, mem};
use unic_langid::LanguageIdentifier;
use winit::event_loop::EventLoopProxy;

static_loader! {
    static TEXTS = {
        locales: "./assets/texts",
        fallback_language: "en-US"
    };
}

pub fn text<'a>(locale: &LanguageIdentifier, id: &'a str) -> Cow<'a, str> {
    TEXTS
        .try_lookup(locale, id)
        .map(Cow::Owned)
        .unwrap_or_else(|| {
            tracing::error!("Unknown desktop text id '{id}'");
            Cow::Borrowed(id)
        })
}

pub fn optional_text(locale: &LanguageIdentifier, id: &str) -> Option<String> {
    TEXTS
        .lookup_single_language::<&str>(locale, id, None)
        .inspect_err(|e| tracing::trace!("Error looking up text: {e}"))
        .ok()
}

pub fn available_languages() -> Vec<&'static LanguageIdentifier> {
    let mut result: Vec<_> = TEXTS.locales().collect();
    result.sort();
    result
}

#[allow(dead_code)]
pub fn text_with_args(
    locale: &LanguageIdentifier,
    id: &'static str,
    args: &HashMap<Cow<'static, str>, FluentValue>,
) -> Cow<'static, str> {
    TEXTS
        .try_lookup_with_args(locale, id, args)
        .map(Cow::Owned)
        .unwrap_or_else(|| {
            tracing::error!("Unknown desktop text id '{id}'");
            Cow::Borrowed(id)
        })
}

pub enum LocalizableText {
    NonLocalizedText(Cow<'static, str>),
    LocalizedText(&'static str),
}

impl LocalizableText {
    pub fn localize(&self, locale: &LanguageIdentifier) -> Cow<'_, str> {
        match self {
            LocalizableText::NonLocalizedText(Cow::Borrowed(text)) => Cow::Borrowed(text),
            LocalizableText::NonLocalizedText(Cow::Owned(text)) => Cow::Borrowed(text),
            LocalizableText::LocalizedText(id) => text(locale, id),
        }
    }
}

/// Size of the top menu bar in pixels.
/// This is the offset at which the movie will be shown,
/// and added to the window size if trying to match a movie.
pub const MENU_HEIGHT: u32 = 24;

/// The main controller for the Ruffle GUI.
pub struct RuffleGui {
    event_loop: EventLoopProxy<RuffleEvent>,
    context_menu: Option<ContextMenu>,
    dialogs: Dialogs,
    menu_bar: MenuBar,
    was_suspended_before_debug: bool,
    taking_screenshot: bool,
    preferences: GlobalPreferences,
}

impl RuffleGui {
    fn new(
        window: Weak<winit::window::Window>,
        event_loop: EventLoopProxy<RuffleEvent>,
        default_path: Option<Url>,
        default_launch_options: LaunchOptions,
        preferences: GlobalPreferences,
    ) -> Self {
        Self {
            was_suspended_before_debug: false,
            taking_screenshot: false,

            context_menu: None,
            dialogs: Dialogs::new(
                preferences.clone(),
                default_launch_options.clone(),
                default_path,
                window.clone(),
                event_loop.clone(),
            ),
            menu_bar: MenuBar::new(
                event_loop.clone(),
                default_launch_options,
                preferences.clone(),
            ),

            event_loop,
            preferences,
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
        let locale = self.preferences.language();

        self.menu_bar
            .consume_shortcuts(egui_ctx, &mut self.dialogs, player.as_deref_mut());
        if show_menu {
            self.menu_bar
                .show(&locale, egui_ctx, &mut self.dialogs, player.as_deref_mut());
        }

        self.dialogs.show(&locale, egui_ctx, player.as_deref_mut());

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
                let dialog = AsyncFileDialog::new().set_file_name(&item.suggested_name);
                let picker = self.dialogs.file_picker();
                let result = picker.show_dialog(dialog, |d| d.save_file());
                if let Some(result) = result {
                    tokio::spawn(async move {
                        let Some(handle) = result.await else {
                            return;
                        };
                        let path = handle.path();
                        if let Err(e) = fs::write(path, item.data) {
                            tracing::error!(
                                "Couldn't save {} to {path:?}: {e}",
                                item.suggested_name,
                            );
                        }
                    });
                }
            }

            if let Some(context_menu) = &mut self.context_menu {
                if !context_menu.show(&locale, egui_ctx, &self.event_loop, player.is_fullscreen()) {
                    self.close_context_menu(player);
                }
            }
        };
    }

    pub fn show_context_menu(
        &mut self,
        menu: Vec<ruffle_core::ContextMenuItem>,
        close_event: PlayerEvent,
    ) {
        if !menu.is_empty() {
            self.context_menu = Some(ContextMenu::new(menu, close_event));
        }
    }

    pub fn close_context_menu(&mut self, player: &mut Player) {
        if let Some(context_menu) = mem::take(&mut self.context_menu) {
            player.handle_event(context_menu.close_event());
        }
    }

    pub fn is_context_menu_visible(&self) -> bool {
        self.context_menu.is_some()
    }

    /// Notifies the GUI that the player has been destroyed.
    fn on_player_destroyed(&mut self) {
        self.dialogs.close_dialogs_with_notifiers();
    }

    /// Notifies the GUI that a new player was created.
    fn on_player_created(
        &mut self,
        opt: LaunchOptions,
        movie_url: Url,
        mut player: MutexGuard<Player>,
    ) {
        self.menu_bar.currently_opened = Some((movie_url.clone(), opt.clone()));

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

    pub fn set_taking_screenshot(&mut self, state: bool) {
        self.taking_screenshot = state;
    }

    pub fn get_taking_screenshot(&mut self) -> bool {
        self.taking_screenshot
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
                        if Button::new(text(&self.locale, "debug-menu-take-screenshot")).ui(ui).clicked() {
                            ui.close_menu();
                            self.set_taking_screenshot(true)
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
                    ui.label(
                        RichText::new("Ruffle")
                            .color(Color32::from_rgb(0xFF, 0xAD, 0x33))
                            .size(32.0),
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
      
        self.dialogs
            .recreate_open_dialog(opt, Some(movie_url), self.event_loop.clone());

        player.set_volume(self.dialogs.volume_controls.get_volume());
    }
}
