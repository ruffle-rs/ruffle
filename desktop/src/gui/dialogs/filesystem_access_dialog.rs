use crate::gui::text;
use egui::{Align2, ComboBox, Ui, Window};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::oneshot::Sender;
use unic_langid::LanguageIdentifier;

#[derive(PartialEq, Eq, Clone)]
pub enum FilesystemAccessDialogResult {
    Allow,
    Cancel,
}

pub struct FilesystemAccessDialogConfiguration {
    notifier: Option<Sender<FilesystemAccessDialogResult>>,

    /// Collection of already allowed paths that can be updated.
    ///
    /// TODO Make this more generic, maybe a manager?
    allowed_paths: Arc<Mutex<Vec<PathBuf>>>,

    /// Path of the file to access.
    path: PathBuf,
}

impl FilesystemAccessDialogConfiguration {
    pub fn new(
        notifier: Sender<FilesystemAccessDialogResult>,
        allowed_paths: Arc<Mutex<Vec<PathBuf>>>,
        path: PathBuf,
    ) -> Self {
        Self {
            notifier: Some(notifier),
            allowed_paths,
            path,
        }
    }
}

pub struct FilesystemAccessDialog {
    config: FilesystemAccessDialogConfiguration,

    /// Whether the user already allowed access to this path.
    allowed: bool,

    /// Whether the user wants to allow permanent access.
    remember_access: bool,

    /// What path the user wants to allow permanent access to.
    selected_path: PathBuf,

    /// Available paths to allow permanent access to.
    selectable_paths: Vec<PathBuf>,
}

impl Drop for FilesystemAccessDialog {
    fn drop(&mut self) {
        self.respond(FilesystemAccessDialogResult::Cancel);
    }
}

impl FilesystemAccessDialog {
    pub fn new(config: FilesystemAccessDialogConfiguration) -> Self {
        let allowed = Self::is_path_allowed(&config);
        let selectable_paths = Self::get_selectable_paths(&config);
        let selected_path = selectable_paths
            .first()
            .cloned()
            .unwrap_or_else(PathBuf::new);

        Self {
            config,
            allowed,
            remember_access: false,
            selected_path,
            selectable_paths,
        }
    }

    /// Returns paths that will be shown in the dropdown menu.
    ///
    /// It returns parents of the file that the movie wants to access.
    /// For instance, for the file `/a/b/c/d/e`, this method will return:
    /// * `/a/b/c/d`,
    /// * `/a/b/c`,
    /// * `/a/b`.
    fn get_selectable_paths(config: &FilesystemAccessDialogConfiguration) -> Vec<PathBuf> {
        let mut selectable_paths = Vec::new();
        let mut current_path = config.path.as_path().parent();
        while let Some(path) = current_path {
            // Do not return paths shorter than 2 components,
            // that's usually a bad idea to grant such wide permissions.
            // Windows does not have a unified filesystem so that does not apply there.
            if !cfg!(windows) && path.components().count() <= 2 {
                break;
            }
            selectable_paths.push(path.to_path_buf());
            current_path = path.parent();
        }
        selectable_paths
    }

    /// Checks whether the user already allowed access to the file.
    ///
    /// This check needs to be done as late as possible, because we want the
    /// user's decision to apply to every future dialog,
    /// not only those requested after the decision.
    fn is_path_allowed(config: &FilesystemAccessDialogConfiguration) -> bool {
        for path_prefix in config
            .allowed_paths
            .lock()
            .expect("Non-poisoned lock")
            .as_slice()
        {
            if config.path.starts_with(path_prefix) {
                return true;
            }
        }
        false
    }

    fn respond(&mut self, result: FilesystemAccessDialogResult) {
        if let Some(notifier) = std::mem::take(&mut self.config.notifier) {
            let _ = notifier.send(result);
        }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        if self.allowed {
            self.respond(FilesystemAccessDialogResult::Allow);
            return false;
        }

        let mut keep_open = true;
        let mut should_close = false;

        Window::new(text(locale, "filesystem-access-dialog-title"))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    should_close = self.render_window_contents(locale, ui)
                });
            });

        keep_open && !should_close
    }

    pub fn render_window_contents(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) -> bool {
        let mut should_close = false;

        ui.label(text(locale, "filesystem-access-dialog-message"));
        ui.label("");
        ui.monospace(self.config.path.to_string_lossy());
        ui.label("");

        if !self.selectable_paths.is_empty() {
            ui.horizontal(|ui| {
                self.render_checkbox(locale, ui);
            });
            ui.label("");
        }

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let primary_text = if self.remember_access {
                    text(locale, "filesystem-access-dialog-allow-remember")
                } else {
                    text(locale, "filesystem-access-dialog-allow")
                };
                if ui.button(primary_text).clicked() {
                    if self.remember_access {
                        self.config
                            .allowed_paths
                            .lock()
                            .expect("Non-poisoned lock")
                            .push(self.selected_path.clone());
                    }
                    self.respond(FilesystemAccessDialogResult::Allow);
                    should_close = true;
                }
                if ui.button(text(locale, "dialog-cancel")).clicked() {
                    should_close = true;
                }
            })
        });

        should_close
    }

    fn render_checkbox(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.checkbox(&mut self.remember_access, "");
        ui.vertical(|ui| {
            ui.label(text(
                locale,
                "filesystem-access-dialog-allow-remember-message",
            ));
            ComboBox::from_id_salt("allow-remember-path-combobox")
                .selected_text(self.selected_path.to_string_lossy())
                .show_ui(ui, |ui| {
                    for path in self.selectable_paths.as_slice() {
                        ui.selectable_value(
                            &mut self.selected_path,
                            path.clone(),
                            path.to_string_lossy(),
                        );
                    }
                });
        });
    }
}
