use crate::gui::text;
use chrono::DateTime;
use egui::{Align2, Grid, Image};
use unic_langid::LanguageIdentifier;

const VERGEN_UNKNOWN: &str = "VERGEN_IDEMPOTENT_OUTPUT";

/// Renders the About Ruffle dialog.
pub fn show_about_dialog(locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
    let mut keep_open = true;

    egui::Window::new(text(locale, "about-ruffle"))
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut keep_open)
        .show(egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(
                    Image::new(egui::include_image!("../../../assets/about_logo.png"))
                        .max_width(350.0),
                );
                Grid::new("about_ruffle_version_info")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(text(locale, "about-ruffle-version"));
                        ui.label(env!("CARGO_PKG_VERSION"));
                        ui.end_row();

                        ui.label(text(locale, "about-ruffle-channel"));
                        ui.label(env!("CFG_RELEASE_CHANNEL"));
                        ui.end_row();

                        let build_time = env!("VERGEN_BUILD_TIMESTAMP");
                        if build_time != VERGEN_UNKNOWN {
                            ui.label(text(locale, "about-ruffle-build-time"));
                            ui.label(
                                DateTime::parse_from_rfc3339(build_time)
                                    .map(|t| t.format("%c").to_string())
                                    .unwrap_or_else(|_| build_time.to_string()),
                            );
                            ui.end_row();
                        }

                        let sha = env!("VERGEN_GIT_SHA");
                        if sha != VERGEN_UNKNOWN {
                            ui.label(text(locale, "about-ruffle-commit-ref"));
                            ui.hyperlink_to(
                                sha,
                                format!("https://github.com/ruffle-rs/ruffle/commit/{}", sha),
                            );
                            ui.end_row();
                        }

                        let commit_time = env!("VERGEN_GIT_COMMIT_TIMESTAMP");
                        if sha != VERGEN_UNKNOWN {
                            ui.label(text(locale, "about-ruffle-commit-time"));
                            ui.label(
                                DateTime::parse_from_rfc3339(commit_time)
                                    .map(|t| t.format("%c").to_string())
                                    .unwrap_or_else(|_| commit_time.to_string()),
                            );
                            ui.end_row();
                        }

                        ui.label(text(locale, "about-ruffle-build-features"));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(env!("VERGEN_CARGO_FEATURES").replace(',', ", "));
                        });
                        ui.end_row();
                    });

                ui.horizontal(|ui| {
                    ui.hyperlink_to(
                        text(locale, "about-ruffle-visit-website"),
                        "https://ruffle.rs",
                    );
                    ui.hyperlink_to(
                        text(locale, "about-ruffle-visit-github"),
                        "https://github.com/ruffle-rs/ruffle/",
                    );
                    ui.hyperlink_to(
                        text(locale, "about-ruffle-visit-discord"),
                        "https://discord.gg/ruffle",
                    );
                    ui.hyperlink_to(
                        text(locale, "about-ruffle-visit-sponsor"),
                        "https://opencollective.com/ruffle/",
                    );
                    ui.shrink_width_to_current();
                });
            })
        });

    keep_open
}
