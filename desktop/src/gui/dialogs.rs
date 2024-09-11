mod about_dialog;
mod bookmarks_dialog;
pub mod filesystem_access_dialog;
pub mod message_dialog;
pub mod network_access_dialog;
mod open_dialog;
mod open_url_dialog;
mod preferences_dialog;
mod volume_controls;

use crate::custom_event::RuffleEvent;
use crate::player::LaunchOptions;
use crate::preferences::GlobalPreferences;
use bookmarks_dialog::{BookmarkAddDialog, BookmarksDialog};
use filesystem_access_dialog::{FilesystemAccessDialog, FilesystemAccessDialogConfiguration};
use message_dialog::{MessageDialog, MessageDialogConfiguration};
use network_access_dialog::{NetworkAccessDialog, NetworkAccessDialogConfiguration};
use open_dialog::OpenDialog;
use open_url_dialog::OpenUrlDialog;
use preferences_dialog::PreferencesDialog;
use ruffle_core::Player;
use std::{collections::VecDeque, sync::Weak};
use unic_langid::LanguageIdentifier;
use url::Url;
use volume_controls::VolumeControls;
use winit::event_loop::EventLoopProxy;

use super::FilePicker;

pub struct Dialogs {
    event_loop: EventLoopProxy<RuffleEvent>,

    picker: FilePicker,
    preferences_dialog: Option<PreferencesDialog>,
    bookmarks_dialog: Option<BookmarksDialog>,
    bookmark_add_dialog: Option<BookmarkAddDialog>,
    open_url_dialog: Option<OpenUrlDialog>,
    message_dialog: Option<MessageDialog>,

    // Use a queue for the following dialogs in order to:
    //  1. support handling multiple instances of them,
    //  2. prevent new instances popping up over existing ones
    //     (and possibly stealing a click).
    network_access_dialog_queue: VecDeque<NetworkAccessDialog>,
    filesystem_access_dialog: Option<FilesystemAccessDialog>,
    filesystem_access_dialog_queue: VecDeque<FilesystemAccessDialogConfiguration>,

    open_dialog: OpenDialog,
    is_open_dialog_visible: bool,

    pub volume_controls: VolumeControls,
    is_volume_visible: bool,

    is_about_visible: bool,

    preferences: GlobalPreferences,
}

pub enum DialogDescriptor {
    OpenUrl(url::Url),
    ShowMessage(MessageDialogConfiguration),
    NetworkAccess(NetworkAccessDialogConfiguration),
    FilesystemAccess(FilesystemAccessDialogConfiguration),
}

impl Dialogs {
    pub fn new(
        preferences: GlobalPreferences,
        player_options: LaunchOptions,
        default_path: Option<Url>,
        window: Weak<winit::window::Window>,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> Self {
        let picker = FilePicker::new(window, preferences.clone());
        Self {
            preferences_dialog: None,
            bookmarks_dialog: None,
            bookmark_add_dialog: None,
            open_url_dialog: None,
            message_dialog: None,

            network_access_dialog_queue: VecDeque::new(),
            filesystem_access_dialog: None,
            filesystem_access_dialog_queue: VecDeque::new(),

            open_dialog: OpenDialog::new(
                player_options,
                default_path,
                picker.clone(),
                event_loop.clone(),
            ),
            is_open_dialog_visible: false,

            volume_controls: VolumeControls::new(&preferences),
            is_volume_visible: false,

            is_about_visible: false,

            event_loop,
            picker,
            preferences,
        }
    }

    pub fn file_picker(&self) -> FilePicker {
        self.picker.clone()
    }

    pub fn recreate_open_dialog(
        &mut self,
        opt: LaunchOptions,
        url: Option<Url>,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) {
        self.is_open_dialog_visible = false;
        self.open_dialog = OpenDialog::new(opt, url, self.picker.clone(), event_loop);
    }

    pub fn open_file_advanced(&mut self) {
        self.is_open_dialog_visible = true;
    }

    pub fn open_preferences(&mut self) {
        self.preferences_dialog = Some(PreferencesDialog::new(self.preferences.clone()));
    }

    pub fn open_bookmarks(&mut self) {
        self.bookmarks_dialog = Some(BookmarksDialog::new(
            self.preferences.clone(),
            self.picker.clone(),
            self.event_loop.clone(),
        ));
    }

    pub fn open_add_bookmark(&mut self, initial_url: Option<url::Url>) {
        self.bookmark_add_dialog = Some(BookmarkAddDialog::new(
            self.preferences.clone(),
            initial_url,
            self.picker.clone(),
        ))
    }

    pub fn open_volume_controls(&mut self) {
        self.is_volume_visible = true;
    }

    pub fn open_about_screen(&mut self) {
        self.is_about_visible = true;
    }

    pub fn open_dialog(&mut self, event: DialogDescriptor) {
        match event {
            DialogDescriptor::OpenUrl(url) => {
                self.open_url_dialog = Some(OpenUrlDialog::new(url));
            }
            DialogDescriptor::ShowMessage(config) => {
                self.message_dialog = Some(MessageDialog::new(config));
            }
            DialogDescriptor::NetworkAccess(config) => self
                .network_access_dialog_queue
                .push_back(NetworkAccessDialog::new(config)),
            DialogDescriptor::FilesystemAccess(config) => {
                self.filesystem_access_dialog_queue.push_back(config)
            }
        }
    }

    pub fn show(
        &mut self,
        locale: &LanguageIdentifier,
        egui_ctx: &egui::Context,
        player: Option<&mut Player>,
    ) {
        self.show_open_dialog(locale, egui_ctx);
        self.show_preferences_dialog(locale, egui_ctx);
        self.show_bookmarks_dialog(locale, egui_ctx);
        self.show_bookmark_add_dialog(locale, egui_ctx);
        self.show_volume_controls(locale, egui_ctx, player);
        self.show_about_dialog(locale, egui_ctx);
        self.show_open_url_dialog(locale, egui_ctx);
        self.show_message_dialog(locale, egui_ctx);
        self.show_network_access_dialog(locale, egui_ctx);
        self.show_filesystem_access_dialog(locale, egui_ctx);
    }

    fn show_open_dialog(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) {
        if self.is_open_dialog_visible {
            let keep_open = self.open_dialog.show(locale, egui_ctx);
            self.is_open_dialog_visible = keep_open;
        }
    }

    fn show_preferences_dialog(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) {
        let keep_open = if let Some(dialog) = &mut self.preferences_dialog {
            dialog.show(locale, egui_ctx)
        } else {
            true
        };
        if !keep_open {
            self.preferences_dialog = None;
        }
    }

    fn show_bookmarks_dialog(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) {
        let keep_open = if let Some(dialog) = &mut self.bookmarks_dialog {
            dialog.show(locale, egui_ctx)
        } else {
            true
        };
        if !keep_open {
            self.bookmarks_dialog = None;
        }
    }

    fn show_bookmark_add_dialog(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) {
        let keep_open = if let Some(dialog) = &mut self.bookmark_add_dialog {
            dialog.show(locale, egui_ctx)
        } else {
            true
        };
        if !keep_open {
            self.bookmark_add_dialog = None;
        }
    }

    fn show_volume_controls(
        &mut self,
        locale: &LanguageIdentifier,
        egui_ctx: &egui::Context,
        player: Option<&mut Player>,
    ) {
        if self.is_volume_visible {
            let keep_open = self
                .volume_controls
                .show(locale, egui_ctx, player, &self.preferences);
            self.is_volume_visible = keep_open;
        }
    }

    fn show_about_dialog(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) {
        if self.is_about_visible {
            let keep_open = about_dialog::show_about_dialog(locale, egui_ctx);
            self.is_about_visible = keep_open;
        }
    }

    fn show_open_url_dialog(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) {
        let keep_open = if let Some(dialog) = &mut self.open_url_dialog {
            dialog.show(locale, egui_ctx)
        } else {
            true
        };
        if !keep_open {
            self.open_url_dialog = None;
        }
    }

    fn show_message_dialog(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) {
        let keep_open = if let Some(dialog) = &mut self.message_dialog {
            dialog.show(locale, egui_ctx)
        } else {
            true
        };
        if !keep_open {
            self.message_dialog = None;
        }
    }

    fn show_network_access_dialog(
        &mut self,
        locale: &LanguageIdentifier,
        egui_ctx: &egui::Context,
    ) {
        let keep_open = if let Some(dialog) = &mut self.network_access_dialog_queue.front_mut() {
            dialog.show(locale, egui_ctx)
        } else {
            true
        };
        if !keep_open {
            self.network_access_dialog_queue.pop_front();
        }
    }

    fn show_filesystem_access_dialog(
        &mut self,
        locale: &LanguageIdentifier,
        egui_ctx: &egui::Context,
    ) {
        let keep_open = if let Some(dialog) = &mut self.filesystem_access_dialog {
            dialog.show(locale, egui_ctx)
        } else {
            true
        };
        if !keep_open {
            self.filesystem_access_dialog = None;
        }

        if self.filesystem_access_dialog.is_none() {
            self.filesystem_access_dialog = self
                .filesystem_access_dialog_queue
                .pop_front()
                .map(FilesystemAccessDialog::new);
        }
    }
}
