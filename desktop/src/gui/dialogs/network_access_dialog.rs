use crate::gui::{text, text_with_args};
use egui::{Align2, Ui, Window};
use fluent_templates::fluent_bundle::FluentValue;
use std::collections::HashMap;
use tokio::sync::oneshot::Sender;
use unic_langid::LanguageIdentifier;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum NetworkAccessDialogResult {
    Allow,
    Cancel,
}

pub struct NetworkAccessDialogConfiguration {
    notifier: Option<Sender<NetworkAccessDialogResult>>,
    host: String,
    port: u16,
}

impl NetworkAccessDialogConfiguration {
    pub fn new(
        notifier: Sender<NetworkAccessDialogResult>,
        host: impl Into<String>,
        port: u16,
    ) -> Self {
        Self {
            notifier: Some(notifier),
            host: host.into(),
            port,
        }
    }
}

pub struct NetworkAccessDialog {
    config: NetworkAccessDialogConfiguration,
}

impl Drop for NetworkAccessDialog {
    fn drop(&mut self) {
        self.respond(NetworkAccessDialogResult::Cancel);
    }
}

impl NetworkAccessDialog {
    pub fn new(config: NetworkAccessDialogConfiguration) -> Self {
        Self { config }
    }

    fn respond(&mut self, result: NetworkAccessDialogResult) {
        if let Some(notifier) = std::mem::take(&mut self.config.notifier) {
            let _ = notifier.send(result);
        }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;

        Window::new(text(locale, "network-access-dialog-title"))
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

        ui.label(text(locale, "network-access-dialog-message"));
        ui.label("");
        ui.horizontal(|ui| {
            ui.monospace(&self.config.host);
            ui.label(text_with_args(
                locale,
                "network-access-dialog-port",
                &HashMap::from([(
                    "port",
                    FluentValue::String(self.config.port.to_string().into()),
                )]),
            ));
        });
        ui.label("");

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(text(locale, "network-access-dialog-allow"))
                    .clicked()
                {
                    self.respond(NetworkAccessDialogResult::Allow);
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
