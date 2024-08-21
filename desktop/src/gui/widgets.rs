use crate::gui::text;
use egui::{TextEdit, Ui};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use unic_langid::LanguageIdentifier;
use url::Url;

use super::FilePicker;

pub struct PathOrUrlField {
    picker: FilePicker,
    value: Arc<Mutex<String>>,
    result: Option<Url>,
    hint: &'static str,
}

impl PathOrUrlField {
    pub fn new(default: Option<Url>, hint: &'static str, picker: FilePicker) -> Self {
        if let Some(default) = default {
            if default.scheme() == "file" {
                if let Ok(path) = default.to_file_path() {
                    return Self {
                        picker,
                        value: Arc::new(Mutex::new(path.to_string_lossy().to_string())),
                        result: Some(default),
                        hint,
                    };
                }
            }

            return Self {
                picker,
                value: Arc::new(Mutex::new(default.to_string())),
                result: Some(default),
                hint,
            };
        }

        Self {
            picker,
            value: Arc::new(Mutex::new("".to_string())),
            result: None,
            hint,
        }
    }

    fn lock_value(value: &Arc<Mutex<String>>) -> MutexGuard<'_, String> {
        value.lock().expect("Non-poisoned value")
    }

    pub fn ui(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) -> &mut Self {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(text(locale, "browse")).clicked() {
                let dir = self
                    .result
                    .as_ref()
                    .filter(|url| url.scheme() == "file")
                    .and_then(|url| url.to_file_path().ok())
                    .map(|mut path| {
                        path.pop();
                        path
                    });

                let value = self.value.clone();
                let picker = self.picker.clone();
                tokio::spawn(async move {
                    if let Some(path) = picker.pick_file(dir).await {
                        let mut value_lock = Self::lock_value(&value);
                        *value_lock = path.to_string_lossy().to_string();
                    }
                });
            }

            let mut value_locked = Self::lock_value(&self.value);
            let mut value = value_locked.clone();
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut value)
                    .hint_text(self.hint)
                    .text_color_opt(if self.result.is_none() {
                        Some(ui.style().visuals.error_fg_color)
                    } else {
                        None
                    }),
            );
            *value_locked = value;
        });

        let value = Self::lock_value(&self.value).clone();
        let path = Path::new(&value);
        self.result = if path.is_file() {
            Url::from_file_path(path).ok()
        } else {
            Url::parse(&value).ok()
        };

        self
    }

    pub fn result(&self) -> Option<&Url> {
        self.result.as_ref()
    }
}
