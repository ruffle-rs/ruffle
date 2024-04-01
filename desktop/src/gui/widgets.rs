use crate::gui::text;
use crate::util::pick_file;
use egui::{TextEdit, Ui};
use std::path::Path;
use unic_langid::LanguageIdentifier;
use url::Url;

pub struct PathOrUrlField {
    value: String,
    result: Option<Url>,
    hint: &'static str,
}

impl PathOrUrlField {
    pub fn new(default: Option<Url>, hint: &'static str) -> Self {
        if let Some(default) = default {
            if default.scheme() == "file" {
                if let Ok(path) = default.to_file_path() {
                    return Self {
                        value: path.to_string_lossy().to_string(),
                        result: Some(default),
                        hint,
                    };
                }
            }

            return Self {
                value: default.to_string(),
                result: Some(default),
                hint,
            };
        }

        Self {
            value: "".to_string(),
            result: None,
            hint,
        }
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

                if let Some(path) = pick_file(true, dir) {
                    self.value = path.to_string_lossy().to_string();
                }
            }
            ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.value)
                    .hint_text(self.hint)
                    .text_color_opt(if self.result.is_none() {
                        Some(ui.style().visuals.error_fg_color)
                    } else {
                        None
                    }),
            );
        });

        let path = Path::new(&self.value);
        self.result = if path.is_file() {
            Url::from_file_path(path).ok()
        } else {
            Url::parse(&self.value).ok()
        };

        self
    }

    pub fn value(&self) -> Option<&Url> {
        self.result.as_ref()
    }
}
