use crate::gui::{FilePicker, text};
use egui::{Align2, Ui, Window};
use rfd::FileHandle;
use ruffle_frontend_utils::bundle::exporter::FilesystemHelper;
use ruffle_frontend_utils::bundle::exporter::FilesystemHelperError;
use ruffle_frontend_utils::bundle::info::BundleInformation;
use ruffle_frontend_utils::content::ContentDescriptor;
use ruffle_frontend_utils::player_options::PlayerOptions;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use unic_langid::LanguageIdentifier;
use url::Url;

pub struct ExportBundleDialogConfiguration {
    content_descriptor: ContentDescriptor,
    player_options: PlayerOptions,
}

impl ExportBundleDialogConfiguration {
    pub fn new(content_descriptor: ContentDescriptor, player_options: PlayerOptions) -> Self {
        Self {
            content_descriptor,
            player_options,
        }
    }
}

struct LocalFileToExport {
    export: bool,
    path: PathBuf,
    displayed_path: String,
}

impl LocalFileToExport {
    fn new(path: PathBuf, base: Option<&Path>) -> Self {
        Self {
            export: true,
            displayed_path: Self::to_displayed_path(&path, base),
            path,
        }
    }

    fn to_displayed_path(path: &Path, base: Option<&Path>) -> String {
        let relative_path = if let Some(base) = base {
            path.strip_prefix(base).ok().unwrap_or(path)
        } else {
            path.file_name().map(Path::new).unwrap_or(path)
        };

        relative_path.to_string_lossy().to_string()
    }
}

#[derive(Clone, Copy, Debug)]
enum ExportStatus {
    Idle,
    Exporting,

    Success,

    // Failures
    FailedUserCanceled,
    FailedToShowPicker,
    FailedIoError,
    FailedOtherError,
}

impl ExportStatus {
    fn ui_enabled(self) -> bool {
        !matches!(self, ExportStatus::Exporting)
    }
}

#[derive(Clone)]
struct AtomicExportStatus(Arc<Mutex<ExportStatus>>);

#[expect(clippy::unwrap_used)]
impl AtomicExportStatus {
    fn new(status: ExportStatus) -> Self {
        Self(Arc::new(Mutex::new(status)))
    }

    fn get(&self) -> ExportStatus {
        *self.0.lock().unwrap()
    }

    fn set(&self, status: ExportStatus) {
        *self.0.lock().unwrap() = status;
    }
}

pub struct ExportBundleDialog {
    config: ExportBundleDialogConfiguration,
    picker: FilePicker,
    bundle_name: String,

    bundle_local_files: bool,
    local_files: Vec<LocalFileToExport>,

    export_status: AtomicExportStatus,
}

impl ExportBundleDialog {
    pub fn new(config: ExportBundleDialogConfiguration, picker: FilePicker) -> Self {
        let mut bundle_local_files = false;
        let mut local_files = Vec::new();
        if let Ok(root_movie) = config.content_descriptor.url.to_file_path() {
            bundle_local_files = true;
            if let Some(root_content_path) = &config.content_descriptor.root_content_path {
                for entry in walkdir::WalkDir::new(root_content_path)
                    .follow_links(false)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    let path = entry.path().to_owned();
                    local_files.push(LocalFileToExport::new(path, Some(root_content_path)));
                }
            } else {
                local_files.push(LocalFileToExport::new(root_movie, None));
            }
        }

        let suggested_name = Self::suggested_name(&config.content_descriptor.url);

        Self {
            config,
            picker,
            bundle_name: suggested_name,
            bundle_local_files,
            local_files,
            export_status: AtomicExportStatus::new(ExportStatus::Idle),
        }
    }

    fn suggested_name(url: &Url) -> String {
        let file_name = url
            .path_segments()
            .and_then(|mut ps| ps.next_back())
            .filter(|name| !name.is_empty())
            .unwrap_or("exported");

        file_name
            .rsplit_once('.')
            .map(|(name, _)| name)
            .unwrap_or(file_name)
            .to_owned()
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;

        Window::new(text(locale, "export-bundle-dialog-title"))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .show(egui_ctx, |ui| {
                should_close = self.render_window_contents(locale, ui)
            });

        keep_open && !should_close
    }

    fn render_window_contents(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) -> bool {
        let mut should_close = false;

        let export_status = self.export_status.get();
        if matches!(export_status, ExportStatus::Success) {
            // Close automatically on successful export.
            should_close = true;
        }

        self.render_info(locale, ui);

        ui.separator();

        self.render_status(export_status, locale, ui);

        let enabled = export_status.ui_enabled();
        ui.add_enabled_ui(enabled, |ui| {
            ui.horizontal(|ui| {
                ui.label(text(locale, "export-bundle-dialog-bundle-name"));
                ui.text_edit_singleline(&mut self.bundle_name);
            });

            if self.bundle_local_files {
                self.render_local_files(locale, ui);
            }

            ui.separator();

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(text(locale, "export-bundle-dialog-export"))
                        .clicked()
                    {
                        let export_status = self.trigger_export();
                        self.export_status.set(export_status);
                    }
                    if ui.button(text(locale, "dialog-cancel")).clicked() {
                        should_close = true;
                    }
                })
            });
        });

        should_close
    }

    fn render_info(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.collapsing(text(locale, "export-bundle-dialog-info-title"), |ui| {
            ui.label(text(locale, "export-bundle-dialog-info-description"));
        });
    }

    fn render_status(
        &mut self,
        export_status: ExportStatus,
        locale: &LanguageIdentifier,
        ui: &mut Ui,
    ) {
        let error_message = match export_status {
            ExportStatus::Idle | ExportStatus::Success => return,
            ExportStatus::Exporting => {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label(text(locale, "export-bundle-dialog-exporting"));
                });
                return;
            }
            ExportStatus::FailedUserCanceled => {
                Some(text(locale, "export-bundle-dialog-error-user-canceled"))
            }
            ExportStatus::FailedToShowPicker => Some(text(
                locale,
                "export-bundle-dialog-error-failed-to-show-picker",
            )),
            ExportStatus::FailedIoError => {
                Some(text(locale, "export-bundle-dialog-error-io-error"))
            }
            ExportStatus::FailedOtherError => None,
        };

        ui.horizontal(|ui| {
            ui.label(text(locale, "export-bundle-dialog-exporting-failed"));
            if let Some(error_message) = error_message {
                ui.label(error_message);
            }
        });
    }

    fn render_local_files(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.label(text(locale, "export-bundle-dialog-files-description"));

        let all_checked = self.local_files.iter().all(|f| f.export);
        let all_unchecked = self.local_files.iter().all(|f| !f.export);

        ui.horizontal(|ui| {
            let mut checked = all_checked;
            ui.add(
                egui::Checkbox::new(
                    &mut checked,
                    text(locale, "export-bundle-dialog-files-select-all"),
                )
                .indeterminate(!all_checked && !all_unchecked),
            );
            if checked != all_checked {
                for file in self.local_files.iter_mut() {
                    file.export = checked;
                }
            }
        });
        egui::ScrollArea::both().max_height(160.0).show(ui, |ui| {
            for file in self.local_files.iter_mut() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut file.export, &file.displayed_path);
                });
            }
        });
    }

    fn trigger_export(&mut self) -> ExportStatus {
        let dialog = rfd::AsyncFileDialog::new().set_file_name(self.bundle_name.clone() + ".ruf");
        let selected_file = self.picker.show_dialog(dialog, |d| d.save_file());
        let Some(selected_file) = selected_file else {
            return ExportStatus::FailedToShowPicker;
        };

        let bundle_name = self.bundle_name.clone();
        let player_options = self.config.player_options.clone();
        let movie_url = self.config.content_descriptor.url.clone();
        let local_files: Vec<PathBuf> = self
            .local_files
            .iter()
            .filter(|file| file.export)
            .map(|file| file.path.clone())
            .collect();

        let export_status = self.export_status.clone();
        tokio::spawn(async move {
            let status = Self::perform_export(
                selected_file.await,
                bundle_name,
                player_options,
                movie_url,
                local_files,
            );
            export_status.set(status);
        });
        ExportStatus::Exporting
    }

    fn perform_export(
        selected_file: Option<FileHandle>,
        bundle_name: String,
        player_options: PlayerOptions,
        movie_url: Url,
        exported_files: Vec<PathBuf>,
    ) -> ExportStatus {
        let Some(handle) = selected_file else {
            return ExportStatus::FailedUserCanceled;
        };
        let output = handle.path();

        let info = BundleInformation {
            name: bundle_name,
            url: movie_url,
            player: player_options,
        };

        match FilesystemHelper::new(exported_files).and_then(|h| h.export_bundle(info, output)) {
            Ok(_) => ExportStatus::Success,
            Err(err) => {
                tracing::error!("Failed exporting bundle: {err}");
                match err {
                    FilesystemHelperError::IoError(_) => ExportStatus::FailedIoError,
                    _ => ExportStatus::FailedOtherError,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    fn url(url: &str) -> Url {
        Url::parse(url).expect("url in test should parse")
    }

    #[test]
    fn suggested_name_https() {
        assert_eq!(
            ExportBundleDialog::suggested_name(&url("https://example.com/abc.xyz")),
            "abc"
        );
        assert_eq!(
            ExportBundleDialog::suggested_name(&url("https://example.com/dir/file.xyz")),
            "file"
        );
        assert_eq!(
            ExportBundleDialog::suggested_name(&url("https://example.com/dir/file")),
            "file"
        );
        assert_eq!(
            ExportBundleDialog::suggested_name(&url("https://example.com/dir/")),
            "exported"
        );
    }

    #[test]
    fn suggested_name_file() {
        assert_eq!(
            ExportBundleDialog::suggested_name(&url("file:///")),
            "exported"
        );
        assert_eq!(ExportBundleDialog::suggested_name(&url("file:///a")), "a");
        assert_eq!(ExportBundleDialog::suggested_name(&url("file:///a.b")), "a");
        assert_eq!(
            ExportBundleDialog::suggested_name(&url("file:///d/a.b")),
            "a"
        );
    }

    #[test]
    fn file_to_export_displayed_path_no_base() {
        assert_eq!(
            LocalFileToExport::to_displayed_path(Path::new(""), None),
            ""
        );

        assert_eq!(
            LocalFileToExport::to_displayed_path(Path::new("file.txt"), None),
            "file.txt"
        );

        assert_eq!(
            LocalFileToExport::to_displayed_path(Path::new("a/b/c"), None),
            "c"
        );

        assert_eq!(
            LocalFileToExport::to_displayed_path(Path::new("/a/b/c"), None),
            "c"
        );
    }

    #[test]
    fn file_to_export_displayed_path_base() {
        assert_eq!(
            LocalFileToExport::to_displayed_path(
                Path::new("/a/b/c"),
                Some(Path::new("/unrelated_path"))
            ),
            "/a/b/c"
        );

        assert_eq!(
            LocalFileToExport::to_displayed_path(Path::new("/file.txt"), Some(Path::new("/"))),
            "file.txt"
        );

        assert_eq!(
            LocalFileToExport::to_displayed_path(Path::new("/a/b/c"), Some(Path::new("/a"))),
            "b/c"
        );

        assert_eq!(
            LocalFileToExport::to_displayed_path(Path::new("/a/b/c"), Some(Path::new("/a/b"))),
            "c"
        );

        assert_eq!(
            LocalFileToExport::to_displayed_path(Path::new("/a/b/c"), Some(Path::new("/a/"))),
            "b/c"
        );
    }
}
