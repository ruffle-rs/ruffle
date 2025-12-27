use crate::gui::{LocalizableText, text};
use egui::{Align2, Ui, Window};
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::sync::oneshot::Sender;
use unic_langid::LanguageIdentifier;
use walkdir::WalkDir;

#[derive(PartialEq, Eq, Clone)]
pub enum SelectPathDialogResult {
    PathSelected(PathBuf),
    Canceled,
}

pub struct SelectPathDialogConfiguration {
    notifier: Option<Sender<SelectPathDialogResult>>,
    directory: PathBuf,
    title: LocalizableText,
    message: Option<LocalizableText>,
    label: LocalizableText,
    extension: Option<&'static str>,
}

impl SelectPathDialogConfiguration {
    pub fn new(
        notifier: Sender<SelectPathDialogResult>,
        directory: PathBuf,
        title: LocalizableText,
        message: Option<LocalizableText>,
        label: LocalizableText,
        extension: Option<&'static str>,
    ) -> Self {
        Self {
            notifier: Some(notifier),
            directory,
            title,
            message,
            label,
            extension,
        }
    }
}

pub struct SelectPathDialog {
    config: SelectPathDialogConfiguration,
    files: Vec<(PathBuf, String)>,
    selected_file: Option<PathBuf>,
    show_all_files: bool,
}

impl Drop for SelectPathDialog {
    fn drop(&mut self) {
        self.respond(SelectPathDialogResult::Canceled);
    }
}

impl SelectPathDialog {
    pub fn new(config: SelectPathDialogConfiguration) -> Self {
        let mut files: Vec<(PathBuf, String)> = WalkDir::new(&config.directory)
            .into_iter()
            .filter_map(Result::ok)
            .map(|e| e.path().to_path_buf())
            .filter(|e| e.is_file())
            .filter_map(|path| {
                let relative_path = path.strip_prefix(&config.directory).ok()?;
                let label = relative_path.to_string_lossy().to_string();
                Some((path, label))
            })
            .collect();
        files.sort_unstable();
        Self {
            config,
            files,
            selected_file: None,
            show_all_files: false,
        }
    }

    /// Respond with a result, do nothing when already responded.
    fn respond(&mut self, result: SelectPathDialogResult) {
        if let Some(notifier) = std::mem::take(&mut self.config.notifier) {
            let _ = notifier.send(result);
        }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;

        Window::new(self.config.title.localize(locale))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .default_height(300.0)
            .show(egui_ctx, |ui| {
                should_close = self.render_window_contents(locale, ui)
            });

        keep_open && !should_close
    }

    pub fn render_window_contents(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) -> bool {
        let mut should_close = false;

        egui::TopBottomPanel::top("top").show_inside(ui, |ui| {
            if let Some(ref message) = self.config.message {
                ui.label(message.localize(locale));
            }
        });

        egui::TopBottomPanel::bottom("bottom").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if self.config.extension.is_some() {
                    ui.checkbox(
                        &mut self.show_all_files,
                        text(locale, "dialog-show-all-files"),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_enabled_ui(self.selected_file.is_some(), |ui| {
                        if ui.button(self.config.label.localize(locale)).clicked()
                            && let Some(path) = std::mem::take(&mut self.selected_file)
                        {
                            self.respond(SelectPathDialogResult::PathSelected(path));
                            should_close = true;
                        }
                    });
                    if ui.button(text(locale, "dialog-cancel")).clicked() {
                        should_close = true;
                    }
                })
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(ui.layout().with_cross_justify(true), |ui| {
                    for (path, label) in &self.files {
                        if let Some(extension) = self.config.extension
                            && !self.show_all_files
                            && path.extension() != Some(OsStr::new(extension))
                        {
                            continue;
                        }

                        let original_selected = self.selected_file.as_ref() == Some(path);
                        let mut selected = original_selected;
                        ui.toggle_value(&mut selected, label.as_str());
                        if selected && !original_selected {
                            self.selected_file = Some(path.clone());
                        } else if !selected && original_selected {
                            self.selected_file = None;
                        }
                    }
                });
            });
        });

        should_close
    }
}
