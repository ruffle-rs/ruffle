use crate::custom_event::RuffleEvent;
use crate::gui::dialogs::Dialogs;
use crate::gui::{text, DebugMessage};
use crate::player::LaunchOptions;
use crate::preferences::GlobalPreferences;
use egui::{menu, Button, Key, KeyboardShortcut, Modifiers, Widget};
use ruffle_core::config::Letterbox;
use ruffle_core::{Player, StageScaleMode};
use ruffle_frontend_utils::recents::Recent;
use ruffle_render::quality::StageQuality;
use unic_langid::LanguageIdentifier;
use url::Url;
use winit::event_loop::EventLoopProxy;

pub struct MenuBar {
    event_loop: EventLoopProxy<RuffleEvent>,
    default_launch_options: LaunchOptions,
    preferences: GlobalPreferences,

    cached_recents: Option<Vec<Recent>>,
    pub currently_opened: Option<(Url, LaunchOptions)>,
}

impl MenuBar {
    const SHORTCUT_FULLSCREEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F11);
    const SHORTCUT_FULLSCREEN_WINDOWS: KeyboardShortcut =
        KeyboardShortcut::new(Modifiers::ALT, Key::Enter);
    const SHORTCUT_OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
    const SHORTCUT_OPEN_ADVANCED: KeyboardShortcut =
        KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::O);
    const SHORTCUT_PAUSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::P);
    const SHORTCUT_QUIT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);

    pub fn new(
        event_loop: EventLoopProxy<RuffleEvent>,
        default_launch_options: LaunchOptions,
        preferences: GlobalPreferences,
    ) -> Self {
        Self {
            event_loop,
            default_launch_options,
            cached_recents: None,
            currently_opened: None,
            preferences,
        }
    }

    pub fn consume_shortcuts(
        &mut self,
        egui_ctx: &egui::Context,
        dialogs: &mut Dialogs,
        mut player: Option<&mut Player>,
    ) {
        // TODO(mike): Make some MenuItem struct with shortcut info to handle this more cleanly.
        if egui_ctx.input_mut(|input| input.consume_shortcut(&Self::SHORTCUT_OPEN_ADVANCED)) {
            dialogs.open_file_advanced();
        }
        if egui_ctx.input_mut(|input| input.consume_shortcut(&Self::SHORTCUT_OPEN)) {
            self.open_file();
        }
        if egui_ctx.input_mut(|input| input.consume_shortcut(&Self::SHORTCUT_QUIT)) {
            self.request_exit();
        }
        if egui_ctx.input_mut(|input| input.consume_shortcut(&Self::SHORTCUT_PAUSE)) {
            if let Some(player) = &mut player {
                player.set_is_playing(!player.is_playing());
            }
        }
        let mut fullscreen_pressed =
            egui_ctx.input_mut(|input| input.consume_shortcut(&Self::SHORTCUT_FULLSCREEN));
        if cfg!(windows) && !fullscreen_pressed {
            // TODO We can remove this shortcut when we add some kind of preferences.
            fullscreen_pressed = egui_ctx
                .input_mut(|input| input.consume_shortcut(&Self::SHORTCUT_FULLSCREEN_WINDOWS));
        }
        if fullscreen_pressed {
            if let Some(player) = &mut player {
                let is_fullscreen = player.is_fullscreen();
                player.set_fullscreen(!is_fullscreen);
            }
        }
    }

    pub fn show(
        &mut self,
        locale: &LanguageIdentifier,
        egui_ctx: &egui::Context,
        dialogs: &mut Dialogs,
        mut player: Option<&mut Player>,
    ) {
        egui::TopBottomPanel::top("menu_bar").show(egui_ctx, |ui| {
            menu::bar(ui, |ui| {
                self.file_menu(locale, ui, dialogs, player.is_some());
                self.view_menu(locale, ui, &mut player);

                menu::menu_button(ui, text(locale, "controls-menu"), |ui| {
                    ui.add_enabled_ui(player.is_some(), |ui| {
                        let playing = player.as_ref().map(|p| p.is_playing()).unwrap_or_default();
                        if Button::new(text(locale, if playing { "controls-menu-suspend" } else { "controls-menu-resume" })).shortcut_text(ui.ctx().format_shortcut(&Self::SHORTCUT_PAUSE)).ui(ui).clicked() {
                            ui.close_menu();
                            if let Some(player) = &mut player {
                                player.set_is_playing(!player.is_playing());
                            }
                        }
                    });
                    if Button::new(text(locale, "controls-menu-volume")).ui(ui).clicked() {
                        dialogs.open_volume_controls();
                        ui.close_menu();
                    }
                });
                menu::menu_button(ui, text(locale, "bookmarks-menu"), |ui| {
                    if Button::new(text(locale, "bookmarks-menu-add")).ui(ui).clicked() {
                        ui.close_menu();

                        let initial_url = self.currently_opened.as_ref().map(|(url, _)| url.clone());

                        dialogs.open_add_bookmark(initial_url);
                    }

                    if Button::new(text(locale, "bookmarks-menu-manage")).ui(ui).clicked() {
                        ui.close_menu();
                        dialogs.open_bookmarks();
                    }

                    if self.preferences.have_bookmarks() {
                        ui.separator();
                        self.preferences.bookmarks(|bookmarks| {
                            for bookmark in bookmarks.iter().filter(|x| !x.is_invalid()) {
                                if Button::new(&bookmark.name).ui(ui).clicked() {
                                    ui.close_menu();
                                    let _ = self.event_loop.send_event(RuffleEvent::Open(bookmark.url.clone(), Box::new(self.default_launch_options.clone())));
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
                        dialogs.open_about_screen();
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn file_menu(
        &mut self,
        locale: &LanguageIdentifier,
        ui: &mut egui::Ui,
        dialogs: &mut Dialogs,
        player_exists: bool,
    ) {
        menu::menu_button(ui, text(locale, "file-menu"), |ui| {
            if Button::new(text(locale, "file-menu-open-quick"))
                .shortcut_text(ui.ctx().format_shortcut(&Self::SHORTCUT_OPEN))
                .ui(ui)
                .clicked()
            {
                ui.close_menu();
                self.open_file();
            }

            if Button::new(text(locale, "file-menu-open-advanced"))
                .shortcut_text(ui.ctx().format_shortcut(&Self::SHORTCUT_OPEN_ADVANCED))
                .ui(ui)
                .clicked()
            {
                ui.close_menu();
                dialogs.open_file_advanced();
            }

            if ui
                .add_enabled(player_exists, Button::new(text(locale, "file-menu-reload")))
                .clicked()
            {
                self.reload_movie(ui);
            }

            if ui
                .add_enabled(player_exists, Button::new(text(locale, "file-menu-close")))
                .clicked()
            {
                self.close_movie(ui);
            }
            ui.separator();

            let recent_menu_response = ui
                .menu_button(text(locale, "file-menu-recents"), |ui| {
                    if self
                        .cached_recents
                        .as_ref()
                        .map(|x| x.is_empty())
                        .unwrap_or(true)
                    {
                        ui.label(text(locale, "file-menu-recents-empty"));
                    }

                    if let Some(recents) = &self.cached_recents {
                        for recent in recents {
                            if ui.button(&recent.name).clicked() {
                                ui.close_menu();
                                let _ = self.event_loop.send_event(RuffleEvent::Open(
                                    recent.url.clone(),
                                    Box::new(self.default_launch_options.clone()),
                                ));
                            }
                        }
                    };
                })
                .inner;

            match recent_menu_response {
                // recreate the cache on the first draw.
                Some(_) if self.cached_recents.is_none() => {
                    self.cached_recents = Some(self.preferences.recents(|recents| {
                        recents
                            .iter()
                            .rev()
                            .filter(|x| !x.is_invalid() && x.is_available())
                            .cloned()
                            .collect::<Vec<_>>()
                    }))
                }
                // clear cache, since menu was closed.
                None if self.cached_recents.is_some() => self.cached_recents = None,
                _ => {}
            }

            ui.separator();
            if Button::new(text(locale, "file-menu-preferences"))
                .ui(ui)
                .clicked()
            {
                ui.close_menu();
                dialogs.open_preferences();
            }
            ui.separator();

            if Button::new(text(locale, "file-menu-exit"))
                .shortcut_text(ui.ctx().format_shortcut(&Self::SHORTCUT_QUIT))
                .ui(ui)
                .clicked()
            {
                ui.close_menu();
                self.request_exit();
            }
        });
    }

    fn view_menu(
        &mut self,
        locale: &LanguageIdentifier,
        ui: &mut egui::Ui,
        player: &mut Option<&mut Player>,
    ) {
        menu::menu_button(ui, text(locale, "view-menu"), |ui| {
            ui.add_enabled_ui(player.is_some(), |ui| {
                ui.menu_button(text(locale, "scale-mode"), |ui| {
                    let items = vec![
                        (
                            "scale-mode-noscale",
                            "scale-mode-noscale-tooltip",
                            StageScaleMode::NoScale,
                        ),
                        (
                            "scale-mode-showall",
                            "scale-mode-showall-tooltip",
                            StageScaleMode::ShowAll,
                        ),
                        (
                            "scale-mode-exactfit",
                            "scale-mode-exactfit-tooltip",
                            StageScaleMode::ExactFit,
                        ),
                        (
                            "scale-mode-noborder",
                            "scale-mode-noborder-tooltip",
                            StageScaleMode::NoBorder,
                        ),
                    ];
                    let current_scale_mode = player.as_mut().map(|player| player.scale_mode());
                    for (id, tooltip_id, scale_mode) in items {
                        let response = if Some(scale_mode) == current_scale_mode {
                            ui.checkbox(&mut true, text(locale, id))
                        } else {
                            ui.button(text(locale, id))
                        }
                        .on_hover_text_at_pointer(text(locale, tooltip_id));
                        if response.clicked() {
                            ui.close_menu();
                            if let Some(player) = player {
                                player.set_scale_mode(scale_mode);
                            }
                        }
                    }
                    ui.separator();

                    let original_forced_scale_mode = player
                        .as_mut()
                        .map(|player| player.forced_scale_mode())
                        .unwrap_or_default();
                    let mut forced_scale_mode = original_forced_scale_mode;
                    ui.checkbox(&mut forced_scale_mode, text(locale, "scale-mode-force"))
                        .on_hover_text_at_pointer(text(locale, "scale-mode-force-tooltip"));
                    if forced_scale_mode != original_forced_scale_mode {
                        if let Some(player) = player {
                            player.set_forced_scale_mode(forced_scale_mode);
                        }
                    }
                });

                let original_letterbox = if let Some(player) = player {
                    player.letterbox() == Letterbox::On
                } else {
                    false
                };
                let mut letterbox = original_letterbox;
                ui.checkbox(&mut letterbox, text(locale, "letterbox"));
                if letterbox != original_letterbox {
                    if let Some(player) = player {
                        player.set_letterbox(if letterbox {
                            Letterbox::On
                        } else {
                            Letterbox::Off
                        });
                    }
                }
                ui.separator();

                if Button::new(text(locale, "view-menu-fullscreen"))
                    .shortcut_text(ui.ctx().format_shortcut(&Self::SHORTCUT_FULLSCREEN))
                    .ui(ui)
                    .clicked()
                {
                    ui.close_menu();
                    if let Some(player) = player {
                        player.set_fullscreen(true);
                    }
                }
                ui.separator();

                ui.menu_button(text(locale, "quality"), |ui| {
                    let items = vec![
                        ("quality-low", StageQuality::Low),
                        ("quality-medium", StageQuality::Medium),
                        ("quality-high", StageQuality::High),
                        ("quality-best", StageQuality::Best),
                        ("quality-high8x8", StageQuality::High8x8),
                        ("quality-high8x8linear", StageQuality::High8x8Linear),
                        ("quality-high16x16", StageQuality::High16x16),
                        ("quality-high16x16linear", StageQuality::High16x16Linear),
                    ];
                    let current_quality = player.as_mut().map(|player| player.quality());
                    for (id, quality) in items {
                        let clicked = if Some(quality) == current_quality {
                            ui.checkbox(&mut true, text(locale, id)).clicked()
                        } else {
                            ui.button(text(locale, id)).clicked()
                        };
                        if clicked {
                            ui.close_menu();
                            if let Some(player) = player {
                                player.set_quality(quality);
                            }
                        }
                    }
                });
            });
        });
    }

    fn open_file(&mut self) {
        let _ = self
            .event_loop
            .send_event(RuffleEvent::BrowseAndOpen(Box::new(
                self.default_launch_options.clone(),
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
                .send_event(RuffleEvent::Open(movie_url, opts.into()));
        }
        ui.close_menu();
    }

    fn request_exit(&mut self) {
        let _ = self.event_loop.send_event(RuffleEvent::ExitRequested);
    }

    fn launch_website(&mut self, ui: &mut egui::Ui, url: &str) {
        let _ = webbrowser::open(url);
        ui.close_menu();
    }
}
