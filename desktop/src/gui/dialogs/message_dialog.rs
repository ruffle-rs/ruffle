use crate::gui::{text, LocalizableText};
use egui::{Align2, Ui, Window};
use unic_langid::LanguageIdentifier;

pub struct MessageDialogConfiguration {
    title: LocalizableText,
    body: LocalizableText,
}

impl MessageDialogConfiguration {
    pub fn new(title: LocalizableText, body: LocalizableText) -> Self {
        Self { title, body }
    }
}

pub struct MessageDialog {
    config: MessageDialogConfiguration,
}

impl MessageDialog {
    pub fn new(config: MessageDialogConfiguration) -> Self {
        Self { config }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;

        Window::new(self.config.title.localize(locale))
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

        ui.vertical_centered(|ui| {
            ui.label("");
            ui.label(self.config.body.localize(locale));
            ui.label("");
        });

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(text(locale, "dialog-ok")).clicked() {
                    should_close = true;
                }
            })
        });

        should_close
    }
}
