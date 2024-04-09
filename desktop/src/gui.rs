mod context_menu;
mod controller;
mod dialogs;
mod movie;
mod widgets;

pub use controller::GuiController;
pub use movie::MovieView;
use ruffle_frontend_utils::recents::Recent;
use std::borrow::Cow;
use url::Url;

use crate::custom_event::RuffleEvent;
use crate::gui::context_menu::ContextMenu;
use crate::player::PlayerOptions;
use crate::preferences::GlobalPreferences;
use dialogs::Dialogs;
use egui::*;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{static_loader, Loader};
use rfd::FileDialog;
use ruffle_core::debug_ui::Message as DebugMessage;
use ruffle_core::Player;
use std::collections::HashMap;
use std::fs;
use std::sync::MutexGuard;
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
    TEXTS.lookup_single_language::<&str>(locale, id, None)
}

pub fn available_languages() -> Vec<&'static LanguageIdentifier> {
    let mut result: Vec<_> = TEXTS.locales().collect();
    result.sort();
    result
}

#[allow(dead_code)]
pub fn text_with_args<'a, T: AsRef<str>>(
    locale: &LanguageIdentifier,
    id: &'a str,
    args: &HashMap<T, FluentValue>,
) -> Cow<'a, str> {
    TEXTS
        .try_lookup_with_args(locale, id, args)
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
    context_menu: Option<ContextMenu>,
    dialogs: Dialogs,
    default_player_options: PlayerOptions,
    currently_opened: Option<(Url, PlayerOptions)>,
    was_suspended_before_debug: bool,
    preferences: GlobalPreferences,
}

impl RuffleGui {
    fn new(
        event_loop: EventLoopProxy<RuffleEvent>,
        default_path: Option<Url>,
        default_player_options: PlayerOptions,
        preferences: GlobalPreferences,
    ) -> Self {
        Self {
            was_suspended_before_debug: false,

            context_menu: None,
            dialogs: Dialogs::new(
                preferences.clone(),
                default_player_options.clone(),
                default_path,
                event_loop.clone(),
            ),

            event_loop,
            default_player_options,
            currently_opened: None,
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

        if show_menu {
            self.main_menu_bar(&locale, egui_ctx, player.as_deref_mut());
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
        };

        if let Some(context_menu) = &mut self.context_menu {
            if !context_menu.show(egui_ctx, &self.event_loop) {
                self.context_menu = None;
            }
        }
    }

    pub fn show_context_menu(&mut self, menu: Vec<ruffle_core::ContextMenuItem>) {
        if !menu.is_empty() {
            self.context_menu = Some(ContextMenu::new(menu));
        }
    }

    pub fn is_context_menu_visible(&self) -> bool {
        self.context_menu.is_some()
    }

    /// Notifies the GUI that a new player was created.
    fn on_player_created(
        &mut self,
        opt: PlayerOptions,
        movie_url: Url,
        mut player: MutexGuard<Player>,
    ) {
        self.currently_opened = Some((movie_url.clone(), opt.clone()));
        let recent_limit = self.preferences.recent_limit();
        if let Err(e) = self.preferences.write_recents(|writer| {
            writer.push(
                Recent {
                    url: movie_url.clone(),
                },
                recent_limit,
            )
        }) {
            tracing::warn!("Couldn't update recents: {e}");
        }

        // Update dialog state to reflect the newly-opened movie's options.
        self.dialogs
            .recreate_open_dialog(opt, Some(movie_url), self.event_loop.clone());

        player.set_volume(self.dialogs.volume_controls.get_volume());
    }

    /// Renders the main menu bar at the top of the window.
    fn main_menu_bar(
        &mut self,
        locale: &LanguageIdentifier,
        egui_ctx: &egui::Context,
        mut player: Option<&mut Player>,
    ) {
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
                self.dialogs.open_file_advanced();
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
                menu::menu_button(ui, text(locale, "file-menu"), |ui| {
                    let mut shortcut;

                    shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
                    if Button::new(text(locale, "file-menu-open-quick"))
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui)
                        .clicked()
                    {
                        self.open_file(ui);
                    }

                    shortcut = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::O);
                    if Button::new(text(locale, "file-menu-open-advanced"))
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui).clicked() {
                        ui.close_menu();
                        self.dialogs.open_file_advanced();
                    }

                    if ui.add_enabled(player.is_some(), Button::new(text(locale, "file-menu-reload"))).clicked() {
                        self.reload_movie(ui);
                    }

                    if ui.add_enabled(player.is_some(), Button::new(text(locale, "file-menu-close"))).clicked() {
                        self.close_movie(ui);
                    }
                    ui.separator();

                    ui.menu_button("Recents", |ui| {
                        // Since we store recents from oldest to newest iterate backwards.
                        self.preferences.recents(|recents| {
                            for recent in recents.iter().rev() {
                                if ui.button(recent.url.as_str()).clicked() {
                                    ui.close_menu();
                                    let _ = self.event_loop.send_event(RuffleEvent::OpenURL(recent.url.clone(), Box::new(self.default_player_options.clone())));
                                }
                            }
                        });
                    });

                    ui.separator();
                    if Button::new(text(locale, "file-menu-preferences"))
                        .ui(ui)
                        .clicked()
                    {
                        ui.close_menu();
                        self.dialogs.open_preferences();
                    }
                    ui.separator();

                    shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);
                    if Button::new(text(locale, "file-menu-exit"))
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui)
                        .clicked()
                    {
                        self.request_exit(ui);
                    }
                });
                menu::menu_button(ui, text(locale, "controls-menu"), |ui| {
                    ui.add_enabled_ui(player.is_some(), |ui| {
                        let playing = player.as_ref().map(|p| p.is_playing()).unwrap_or_default();
                        let pause_shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::P);
                        if Button::new(text(locale, if playing { "controls-menu-suspend" } else { "controls-menu-resume" })).shortcut_text(ui.ctx().format_shortcut(&pause_shortcut)).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.set_is_playing(!player.is_playing());
                            }
                        }
                    });
                    if Button::new(text(locale, "controls-menu-volume")).ui(ui).clicked() {
                        self.show_volume_screen(ui);
                    }
                });
                menu::menu_button(ui, text(locale, "bookmarks-menu"), |ui| {
                    if Button::new(text(locale, "bookmarks-menu-add")).ui(ui).clicked() {
                        ui.close_menu();

                        let initial_url = self.currently_opened.as_ref().map(|(url, _)| url.clone());

                        self.dialogs.open_add_bookmark(initial_url);
                    }

                    if Button::new(text(locale, "bookmarks-menu-manage")).ui(ui).clicked() {
                        ui.close_menu();
                        self.dialogs.open_bookmarks();
                    }

                    if self.preferences.have_bookmarks() {
                        ui.separator();
                        self.preferences.bookmarks(|bookmarks| {
                            for bookmark in bookmarks.iter().filter(|x| !x.is_invalid()) {
                                if Button::new(&bookmark.name).ui(ui).clicked() {
                                    ui.close_menu();
                                    let _ = self.event_loop.send_event(RuffleEvent::OpenURL(bookmark.url.clone(), Box::new(self.default_player_options.clone())));
                                }
                            }
                        });
                    }
                });
                menu::menu_button(ui, text(locale, "debug-menu"), |ui| {
                    ui.add_enabled_ui(player.is_some(), |ui| {
                        if Button::new(text(locale, "debug-menu-open-stage")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::TrackStage);
                            }
                        }
                        if Button::new(text(locale, "debug-menu-open-movie")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::TrackTopLevelMovie);
                            }
                        }
                        if Button::new(text(locale, "debug-menu-open-movie-list")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::ShowKnownMovies);
                            }
                        }
                        if Button::new(text(locale, "debug-menu-open-domain-list")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::ShowDomains);
                            }
                        }
                        if Button::new(text(locale, "debug-menu-search-display-objects")).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.debug_ui().queue_message(DebugMessage::SearchForDisplayObject);
                            }
                        }
                    });
                });
                menu::menu_button(ui, text(locale, "help-menu"), |ui| {
                    if ui.button(text(locale, "help-menu-join-discord")).clicked() {
                        self.launch_website(ui, "https://discord.gg/ruffle");
                    }
                    if ui.button(text(locale, "help-menu-report-a-bug")).clicked() {
                        self.launch_website(ui, "https://github.com/ruffle-rs/ruffle/issues/new?assignees=&labels=bug&projects=&template=bug_report.yml");
                    }
                    if ui.button(text(locale, "help-menu-sponsor-development")).clicked() {
                        self.launch_website(ui, "https://opencollective.com/ruffle/");
                    }
                    if ui.button(text(locale, "help-menu-translate-ruffle")).clicked() {
                        self.launch_website(ui, "https://crowdin.com/project/ruffle");
                    }
                    ui.separator();
                    if ui.button(text(locale, "help-menu-about")).clicked() {
                        self.show_about_screen(ui);
                    }
                });
            });
        });
    }

    fn open_file(&mut self, ui: &mut egui::Ui) {
        ui.close_menu();

        let _ = self
            .event_loop
            .send_event(RuffleEvent::BrowseAndOpen(Box::new(
                self.default_player_options.clone(),
            )));
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

    fn request_exit(&mut self, ui: &mut egui::Ui) {
        let _ = self.event_loop.send_event(RuffleEvent::ExitRequested);
        ui.close_menu();
    }

    fn launch_website(&mut self, ui: &mut egui::Ui, url: &str) {
        let _ = webbrowser::open(url);
        ui.close_menu();
    }

    fn show_about_screen(&mut self, ui: &mut egui::Ui) {
        self.dialogs.open_about_screen();
        ui.close_menu();
    }

    fn show_volume_screen(&mut self, ui: &mut egui::Ui) {
        self.dialogs.open_volume_controls();
        ui.close_menu();
    }
}
