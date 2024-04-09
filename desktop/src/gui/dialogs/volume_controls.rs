use crate::gui::text;
use crate::preferences::GlobalPreferences;
use egui::{Align2, Slider};
use ruffle_core::Player;
use unic_langid::LanguageIdentifier;

/// The volume controls of the Ruffle GUI.
pub struct VolumeControls {
    is_muted: bool,
    volume: f32,
}

impl VolumeControls {
    pub fn new(preferences: &GlobalPreferences) -> Self {
        Self {
            is_muted: preferences.mute(),
            volume: preferences.preferred_volume() * 100.0,
        }
    }

    pub fn show(
        &mut self,
        locale: &LanguageIdentifier,
        egui_ctx: &egui::Context,
        player: Option<&mut Player>,
        preferences: &GlobalPreferences,
    ) -> bool {
        let mut keep_open = true;

        egui::Window::new(text(locale, "volume-controls"))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut keep_open)
            .show(egui_ctx, |ui| {
                let mut changed_slider = false;

                let changed_checkbox = ui
                    .checkbox(&mut self.is_muted, text(locale, "volume-controls-mute"))
                    .changed();

                ui.add_enabled_ui(!self.is_muted, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(text(locale, "volume-controls-volume"));
                        changed_slider =
                            ui.add(Slider::new(&mut self.volume, 0.0..=100.0)).changed();
                    });
                });

                if changed_checkbox || changed_slider {
                    if let Some(player) = player {
                        player.set_volume(self.get_volume());
                    }
                    // Don't update persisted volume if the CLI set it
                    if preferences.cli.volume.is_none() {
                        if let Err(e) = preferences.write_preferences(|writer| {
                            if changed_checkbox {
                                writer.set_mute(self.is_muted);
                            }
                            if changed_slider {
                                writer.set_volume(self.volume / 100.0);
                            }
                        }) {
                            tracing::warn!("Couldn't update volume preferences: {e}");
                        }
                    }
                }
            });

        keep_open
    }

    /// Returns the volume between 0 and 1 (calculated out of the
    /// checkbox and the slider).
    pub fn get_volume(&self) -> f32 {
        if !self.is_muted {
            self.volume / 100.0
        } else {
            0.0
        }
    }
}
