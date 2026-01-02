use crate::gui::{FilePicker, LocalizableText, text};
use egui::{TextEdit, Ui};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use unic_langid::LanguageIdentifier;
use url::Url;

#[derive(Default)]
struct PathOrUrlFieldValue {
    url: Option<Url>,
    representation: String,
}

impl PathOrUrlFieldValue {
    fn from_url(url: Url) -> Self {
        Self {
            representation: if url.scheme() == "file"
                && let Ok(path) = url.to_file_path()
            {
                path.to_string_lossy().to_string()
            } else {
                url.to_string()
            },
            url: Some(url),
        }
    }

    fn from_path(path: PathBuf) -> Self {
        Self {
            representation: Self::path_to_representation(&path),
            url: Url::from_file_path(path).ok(),
        }
    }

    fn from_string(string: String) -> Self {
        let path = Path::new(&string);
        if path.is_file() {
            Self::from_path(path.to_path_buf())
        } else {
            Self {
                url: Url::parse(&string).ok(),
                representation: string,
            }
        }
    }

    /// When the user picked a directory, we want to show the directory, but
    /// encode the real path we want to play.
    fn from_picked_directory(directory: PathBuf, content: PathBuf) -> Self {
        Self {
            url: Url::from_file_path(content).ok(),
            representation: Self::path_to_representation(&directory),
        }
    }

    fn path_to_representation(path: &Path) -> String {
        if let Some(name) = path.file_name() {
            name.to_string_lossy().into_owned()
        } else {
            path.to_string_lossy().into_owned()
        }
    }
}

pub struct PathOrUrlField {
    picker: FilePicker,
    value: Arc<Mutex<PathOrUrlFieldValue>>,
    hint: LocalizableText,
}

impl PathOrUrlField {
    pub fn new(default: Option<Url>, hint: LocalizableText, picker: FilePicker) -> Self {
        Self {
            picker,
            value: Arc::new(Mutex::new(
                default
                    .map(PathOrUrlFieldValue::from_url)
                    .unwrap_or_default(),
            )),
            hint,
        }
    }

    fn lock_value(value: &Arc<Mutex<PathOrUrlFieldValue>>) -> MutexGuard<'_, PathOrUrlFieldValue> {
        value.lock().expect("Non-poisoned value")
    }

    pub fn ui(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) -> &mut Self {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let dir = Self::lock_value(&self.value)
                .url
                .as_ref()
                .filter(|url| url.scheme() == "file")
                .and_then(|url| url.to_file_path().ok())
                .map(|mut path| {
                    path.pop();
                    path
                });

            if ui
                .button(text(locale, "path-or-url-field-open-directory"))
                .clicked()
            {
                let dir = dir.clone();
                let value = self.value.clone();
                let picker = self.picker.clone();
                tokio::spawn(async move {
                    if let Some((directory, content)) =
                        picker.pick_ruffle_directory_and_content(dir).await
                    {
                        *Self::lock_value(&value) =
                            PathOrUrlFieldValue::from_picked_directory(directory, content);
                    }
                });
            }

            if ui
                .button(text(locale, "path-or-url-field-open-file"))
                .clicked()
            {
                let value = self.value.clone();
                let picker = self.picker.clone();
                tokio::spawn(async move {
                    if let Some(path) = picker.pick_ruffle_file(dir).await {
                        *Self::lock_value(&value) = PathOrUrlFieldValue::from_path(path);
                    }
                });
            }

            let mut value = Self::lock_value(&self.value);
            let old_representation = value.representation.clone();
            let mut new_representation = old_representation.clone();
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut new_representation)
                    .hint_text(self.hint.localize(locale))
                    .text_color_opt(if value.url.is_none() {
                        Some(ui.style().visuals.error_fg_color)
                    } else {
                        None
                    }),
            );
            if new_representation != old_representation {
                *value = PathOrUrlFieldValue::from_string(new_representation);
            }
        });

        self
    }

    pub fn result(&self) -> Option<Url> {
        Self::lock_value(&self.value).url.clone()
    }
}
