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
use rfd::FileDialog;
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
        self.dialogs
            .recreate_open_dialog(opt, Some(movie_url), self.event_loop.clone());

        player.set_volume(self.dialogs.volume_controls.get_volume());
    }
}
