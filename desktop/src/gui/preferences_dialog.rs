use crate::gui::{available_languages, optional_text, text};
use crate::log::FilenamePattern;
use crate::preferences::{storage::StorageBackend, GlobalPreferences};
use cpal::traits::{DeviceTrait, HostTrait};
use egui::{Align2, Button, ComboBox, Grid, Ui, Widget, Window};
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use std::borrow::Cow;
use unic_langid::LanguageIdentifier;

pub struct PreferencesDialog {
    available_backends: wgpu::Backends,
    preferences: GlobalPreferences,

    graphics_backend: GraphicsBackend,
    graphics_backend_readonly: bool,
    graphics_backend_changed: bool,

    power_preference: PowerPreference,
    power_preference_readonly: bool,
    power_preference_changed: bool,

    language: LanguageIdentifier,
    language_changed: bool,

    output_device: Option<String>,
    available_output_devices: Vec<String>,
    output_device_changed: bool,

    log_filename_pattern: FilenamePattern,
    log_filename_pattern_changed: bool,

    storage_backend: StorageBackend,
    storage_backend_readonly: bool,
    storage_backend_changed: bool,
}

impl PreferencesDialog {
    pub fn new(preferences: GlobalPreferences) -> Self {
        let available_backends = find_available_graphics_backends();

        let audio_host = cpal::default_host();
        let mut available_output_devices = Vec::new();
        if let Ok(devices) = audio_host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    available_output_devices.push(name);
                }
            }
        }

        Self {
            available_backends,
            graphics_backend: preferences.graphics_backends(),
            graphics_backend_readonly: preferences.cli.graphics.is_some(),
            graphics_backend_changed: false,

            power_preference: preferences.graphics_power_preference(),
            power_preference_readonly: preferences.cli.power.is_some(),
            power_preference_changed: false,

            language: preferences.language(),
            language_changed: false,

            output_device: preferences.output_device_name(),
            available_output_devices,
            output_device_changed: false,

            log_filename_pattern: preferences.log_filename_pattern(),
            log_filename_pattern_changed: false,

            storage_backend: preferences.storage_backend(),
            storage_backend_readonly: preferences.cli.storage.is_some(),
            storage_backend_changed: false,

            preferences,
        }
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;
        let locked_text = text(locale, "preference-locked-by-cli");

        Window::new(text(locale, "preferences-dialog"))
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
                            self.show_graphics_preferences(locale, &locked_text, ui);

                            self.show_language_preferences(locale, ui);

                            self.show_audio_preferences(locale, ui);

                            self.show_log_preferences(locale, ui);

                            self.show_storage_preferences(locale, &locked_text, ui);
                        });

                    if self.restart_required() {
                        ui.colored_label(
                            ui.style().visuals.error_fg_color,
                            "A restart is required to apply the selected changes",
                        );
                    }

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if Button::new(text(locale, "save")).ui(ui).clicked() {
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
            || self.output_device != self.preferences.output_device_name()
            || self.log_filename_pattern != self.preferences.log_filename_pattern()
            || self.storage_backend != self.preferences.storage_backend()
    }

    fn show_graphics_preferences(
        &mut self,
        locale: &LanguageIdentifier,
        locked_text: &str,
        ui: &mut Ui,
    ) {
        ui.label(text(locale, "graphics-backend"));
        if self.graphics_backend_readonly {
            ui.label(graphics_backend_name(locale, self.graphics_backend))
                .on_hover_text(locked_text);
        } else {
            let previous = self.graphics_backend;
            ComboBox::from_id_source("graphics-backend")
                .selected_text(graphics_backend_name(locale, self.graphics_backend))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.graphics_backend,
                        GraphicsBackend::Default,
                        text(locale, "graphics-backend-default"),
                    );
                    if self.available_backends.contains(wgpu::Backends::VULKAN) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Vulkan,
                            "Vulkan",
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::METAL) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Metal,
                            "Metal",
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::DX12) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Dx12,
                            "DirectX 12",
                        );
                    }
                    if self.available_backends.contains(wgpu::Backends::GL) {
                        ui.selectable_value(
                            &mut self.graphics_backend,
                            GraphicsBackend::Gl,
                            "OpenGL",
                        );
                    }
                });
            if self.graphics_backend != previous {
                self.graphics_backend_changed = true;
            }
        }
        ui.end_row();

        ui.label(text(locale, "graphics-power"));
        if self.power_preference_readonly {
            ui.label(graphics_power_name(locale, self.power_preference))
                .on_hover_text(locked_text);
        } else {
            let previous = self.power_preference;
            ComboBox::from_id_source("graphics-power")
                .selected_text(graphics_power_name(locale, self.power_preference))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.power_preference,
                        PowerPreference::Low,
                        text(locale, "graphics-power-low"),
                    );
                    ui.selectable_value(
                        &mut self.power_preference,
                        PowerPreference::High,
                        text(locale, "graphics-power-high"),
                    );
                });
            if self.power_preference != previous {
                self.power_preference_changed = true;
            }
        }
        ui.end_row();
    }

    fn show_language_preferences(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.label(text(locale, "language"));
        let previous = self.language.clone();
        ComboBox::from_id_source("language")
            .selected_text(language_name(&self.language))
            .show_ui(ui, |ui| {
                for language in available_languages() {
                    ui.selectable_value(
                        &mut self.language,
                        language.clone(),
                        language_name(language),
                    );
                }
            });
        if self.language != previous {
            self.language_changed = true;
        }
        ui.end_row();
    }

    fn show_audio_preferences(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.label(text(locale, "audio-output-device"));

        let previous = self.output_device.clone();
        let default = text(locale, "audio-output-device-default");
        ComboBox::from_id_source("audio-output-device")
            .selected_text(self.output_device.as_deref().unwrap_or(default.as_ref()))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.output_device, None, default);
                for device in &self.available_output_devices {
                    ui.selectable_value(&mut self.output_device, Some(device.to_string()), device);
                }
            });
        if self.output_device != previous {
            self.output_device_changed = true;
        }
        ui.end_row();
    }

    fn show_log_preferences(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.label(text(locale, "log-filename-pattern"));

        let previous = self.log_filename_pattern;
        ComboBox::from_id_source("log-filename-pattern")
            .selected_text(filename_pattern_name(locale, self.log_filename_pattern))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.log_filename_pattern,
                    FilenamePattern::SingleFile,
                    filename_pattern_name(locale, FilenamePattern::SingleFile),
                );
                ui.selectable_value(
                    &mut self.log_filename_pattern,
                    FilenamePattern::WithTimestamp,
                    filename_pattern_name(locale, FilenamePattern::WithTimestamp),
                );
            });
        if self.log_filename_pattern != previous {
            self.log_filename_pattern_changed = true;
        }
        ui.end_row();
    }

    fn show_storage_preferences(
        &mut self,
        locale: &LanguageIdentifier,
        locked_text: &str,
        ui: &mut Ui,
    ) {
        ui.label(text(locale, "storage-backend"));

        if self.storage_backend_readonly {
            ui.label(storage_backend_name(locale, self.storage_backend))
                .on_hover_text(locked_text);
        } else {
            let previous = self.storage_backend;
            ComboBox::from_id_source("storage-backend")
                .selected_text(storage_backend_name(locale, self.storage_backend))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.storage_backend,
                        StorageBackend::Disk,
                        storage_backend_name(locale, StorageBackend::Disk),
                    );
                    ui.selectable_value(
                        &mut self.storage_backend,
                        StorageBackend::Memory,
                        storage_backend_name(locale, StorageBackend::Memory),
                    );
                });

            if self.storage_backend != previous {
                self.storage_backend_changed = true;
            }
        }

        ui.end_row();
    }

    fn save(&mut self) {
        if let Err(e) = self.preferences.write_preferences(|preferences| {
            if self.graphics_backend_changed {
                preferences.set_graphics_backend(self.graphics_backend);
            }
            if self.power_preference_changed {
                preferences.set_graphics_power_preference(self.power_preference);
            }
            if self.language_changed {
                preferences.set_language(self.language.clone());
            }
            if self.output_device_changed {
                preferences.set_output_device(self.output_device.clone());
                // [NA] TODO: Inform the running player that the device changed
            }
            if self.log_filename_pattern_changed {
                preferences.set_log_filename_pattern(self.log_filename_pattern);
            }
            if self.storage_backend_changed {
                preferences.set_storage_backend(self.storage_backend);
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
        GraphicsBackend::Vulkan => Cow::Borrowed("Vulkan"),
        GraphicsBackend::Metal => Cow::Borrowed("Metal"),
        GraphicsBackend::Dx12 => Cow::Borrowed("DirectX 12"),
        GraphicsBackend::Gl => Cow::Borrowed("OpenGL"),
    }
}

fn graphics_power_name(locale: &LanguageIdentifier, power_preference: PowerPreference) -> Cow<str> {
    match power_preference {
        PowerPreference::Low => text(locale, "graphics-power-low"),
        PowerPreference::High => text(locale, "graphics-power-high"),
    }
}

fn language_name(language: &LanguageIdentifier) -> String {
    optional_text(language, "language-name")
        .map(|s| s.to_string())
        .unwrap_or_else(|| language.to_string())
}

fn filename_pattern_name(locale: &LanguageIdentifier, pattern: FilenamePattern) -> Cow<str> {
    match pattern {
        FilenamePattern::SingleFile => text(locale, "log-filename-pattern-single-file"),
        FilenamePattern::WithTimestamp => text(locale, "log-filename-pattern-with-timestamp"),
    }
}

fn storage_backend_name(locale: &LanguageIdentifier, backend: StorageBackend) -> Cow<str> {
    match backend {
        StorageBackend::Disk => text(locale, "storage-backend-disk"),
        StorageBackend::Memory => text(locale, "storage-backend-memory"),
    }
}

fn backend_availability(instance: &wgpu::Instance, backend: wgpu::Backends) -> wgpu::Backends {
    if instance.enumerate_adapters(backend).is_empty() {
        wgpu::Backends::empty()
    } else {
        backend
    }
}

fn find_available_graphics_backends() -> wgpu::Backends {
    let mut available_backends = wgpu::Backends::empty();

    // We have to make a new instance here, as the one created for the entire application may not have
    // all backends enabled
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default().with_env(),
        ..Default::default()
    });

    available_backends |= backend_availability(&instance, wgpu::Backends::VULKAN);
    available_backends |= backend_availability(&instance, wgpu::Backends::GL);
    available_backends |= backend_availability(&instance, wgpu::Backends::METAL);
    available_backends |= backend_availability(&instance, wgpu::Backends::DX12);

    available_backends
}
