use crate::gui::text;
use crate::preferences::GlobalPreferences;
use egui::{Align2, Button, ComboBox, Grid, Ui, Widget, Window};
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use ruffle_render_wgpu::descriptors::Descriptors;
use std::borrow::Cow;
use unic_langid::LanguageIdentifier;

pub struct PreferencesDialog {
    available_backends: wgpu::Backends,
    preferences: GlobalPreferences,
    locale: LanguageIdentifier,

    graphics_backend: GraphicsBackend,
    graphics_backend_readonly: bool,

    power_preference: PowerPreference,
    power_preference_readonly: bool,
}

impl PreferencesDialog {
    pub fn new(
        descriptors: &Descriptors,
        preferences: GlobalPreferences,
        locale: LanguageIdentifier,
    ) -> Self {
        let mut available_backends = wgpu::Backends::empty();

        available_backends |= backend_availability(descriptors, wgpu::Backends::VULKAN);
        available_backends |= backend_availability(descriptors, wgpu::Backends::GL);
        available_backends |= backend_availability(descriptors, wgpu::Backends::METAL);
        available_backends |= backend_availability(descriptors, wgpu::Backends::DX12);

        Self {
            available_backends,
            graphics_backend: preferences.graphics_backends(),
            graphics_backend_readonly: preferences.cli.graphics.is_some(),

            power_preference: preferences.graphics_power_preference(),
            power_preference_readonly: preferences.cli.power.is_some(),

            preferences,
            locale,
        }
    }

    pub fn show(&mut self, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;
        let locked_text = text(&self.locale, "preference-locked-by-cli");

        Window::new(text(&self.locale, "preferences-dialog"))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(egui_ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Grid::new("preferences-dialog-graphics")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            self.show_graphics_preferences(&locked_text, ui);
                        });

                    if self.restart_required() {
                        ui.colored_label(
                            ui.style().visuals.error_fg_color,
                            "A restart is required to apply the selected changes",
                        );
                    }

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if Button::new(text(&self.locale, "save")).ui(ui).clicked() {
                                self.save();
                                should_close = true;
                            }
                        })
                    });
                });
            });

        keep_open && !should_close
    }

    fn restart_required(&self) -> bool {
        self.graphics_backend != self.preferences.graphics_backends()
            || self.power_preference != self.preferences.graphics_power_preference()
    }

    fn show_graphics_preferences(&mut self, locked_text: &str, ui: &mut Ui) {
        ui.label(text(&self.locale, "graphics-backend"));
        if self.graphics_backend_readonly {
            ui.label(graphics_backend_name(&self.locale, self.graphics_backend))
                .on_hover_text(locked_text);
        } else {
            ComboBox::from_id_source("graphics-backend")
                .selected_text(graphics_backend_name(&self.locale, self.graphics_backend))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.graphics_backend,
                        GraphicsBackend::Default,
                        text(&self.locale, "graphics-backend-default"),
                    );
                    if self.available_backends.contains(wgpu::Backends::VULKAN) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Vulkan,
                            text(&self.locale, "graphics-backend-vulkan"),
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::METAL) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Metal,
                            text(&self.locale, "graphics-backend-metal"),
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::DX12) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Dx12,
                            text(&self.locale, "graphics-backend-dx12"),
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::GL) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Gl,
                            text(&self.locale, "graphics-backend-gl"),
                        );
                    }
                });
        }
        ui.end_row();

        ui.label(text(&self.locale, "graphics-power"));
        if self.power_preference_readonly {
            ui.label(graphics_power_name(&self.locale, self.power_preference))
                .on_hover_text(locked_text);
        } else {
            ComboBox::from_id_source("graphics-power")
                .selected_text(graphics_power_name(&self.locale, self.power_preference))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.power_preference,
                        PowerPreference::Low,
                        text(&self.locale, "graphics-power-low"),
                    );
                    ui.selectable_value(
                        &mut self.power_preference,
                        PowerPreference::High,
                        text(&self.locale, "graphics-power-high"),
                    );
                });
        }
        ui.end_row();
    }

    fn save(&mut self) {
        if let Err(e) = self.preferences.write_preferences(|preferences| {
            if !self.graphics_backend_readonly {
                preferences.graphics_backend = self.graphics_backend;
            }
            if !self.power_preference_readonly {
                preferences.graphics_power_preference = self.power_preference;
            }
        }) {
            // [NA] TODO: Better error handling... everywhere in desktop, really
            tracing::error!("Could not save preferences: {e}");
        }
    }
}

fn graphics_backend_name(locale: &LanguageIdentifier, backend: GraphicsBackend) -> Cow<str> {
    match backend {
        GraphicsBackend::Default => text(locale, "graphics-backend-default"),
        GraphicsBackend::Vulkan => text(locale, "graphics-backend-vulkan"),
        GraphicsBackend::Metal => text(locale, "graphics-backend-metal"),
        GraphicsBackend::Dx12 => text(locale, "graphics-backend-dx12"),
        GraphicsBackend::Gl => text(locale, "graphics-backend-gl"),
    }
}

fn graphics_power_name(locale: &LanguageIdentifier, power_preference: PowerPreference) -> Cow<str> {
    match power_preference {
        PowerPreference::Low => text(locale, "graphics-power-low"),
        PowerPreference::High => text(locale, "graphics-power-high"),
    }
}

fn backend_availability(descriptors: &Descriptors, backend: wgpu::Backends) -> wgpu::Backends {
    if descriptors
        .wgpu_instance
        .enumerate_adapters(backend)
        .is_empty()
    {
        wgpu::Backends::empty()
    } else {
        backend
    }
}
