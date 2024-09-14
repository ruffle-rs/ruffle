use crate::{gui::text, util::open_url};
use egui::{Align2, Ui, Window};
use unic_langid::LanguageIdentifier;
use url::Url;

pub struct OpenUrlDialog {
    url: Url,
}

impl OpenUrlDialog {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;

        Window::new(text(locale, "open-url-dialog-title"))
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

        ui.label(text(locale, "open-url-dialog-message"));
        ui.label("");
        ui.monospace(self.url.as_str());
        ui.label("");

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(text(locale, "open-url-dialog-open")).clicked() {
                    open_url(&self.url);
                    should_close = true;
                }
                if ui.button(text(locale, "dialog-cancel")).clicked() {
                    should_close = true;
                }
            })
        });

        should_close
    }
}
