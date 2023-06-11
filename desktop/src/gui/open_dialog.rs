use crate::custom_event::RuffleEvent;
use crate::gui::text;
use crate::player::PlayerOptions;
use crate::util::pick_file;
use egui::{
    Align2, Button, Checkbox, ComboBox, DragValue, Grid, Slider, TextEdit, Ui, Widget, Window,
};
use ruffle_core::backend::navigator::OpenURLMode;
use ruffle_core::{LoadBehavior, StageScaleMode};
use ruffle_render::quality::StageQuality;
use std::path::Path;
use unic_langid::LanguageIdentifier;
use url::Url;
use winit::event_loop::EventLoopProxy;

pub struct OpenDialog {
    options: PlayerOptions,
    event_loop: EventLoopProxy<RuffleEvent>,
    locale: LanguageIdentifier,

    // These are outside of PlayerOptions as it can be an invalid value (ie URL) during typing,
    // and we don't want to clear the value if the user, ie, toggles the checkbox.
    spoof_url: OptionalUrlField,
    base_url: OptionalUrlField,
    proxy_url: OptionalUrlField,
    path: PathOrUrlField,

    framerate: f64,
    framerate_enabled: bool,
}

impl OpenDialog {
    pub fn new(
        defaults: PlayerOptions,
        default_url: Option<Url>,
        event_loop: EventLoopProxy<RuffleEvent>,
        locale: LanguageIdentifier,
    ) -> Self {
        let spoof_url = OptionalUrlField::new(&defaults.spoof_url, "https://example.org/game.swf");
        let base_url = OptionalUrlField::new(&defaults.base, "https://example.org");
        let proxy_url = OptionalUrlField::new(&defaults.proxy, "socks5://localhost:8080");
        let path = PathOrUrlField::new(default_url, "path/to/movie.swf");
        Self {
            options: defaults,
            event_loop,
            locale,
            spoof_url,
            base_url,
            proxy_url,
            path,
            framerate: 30.0,
            framerate_enabled: false,
        }
    }

    fn start(&mut self) -> bool {
        if self.framerate_enabled {
            self.options.frame_rate = Some(self.framerate);
        } else {
            self.options.frame_rate = None;
        }
        if let Some(url) = self.path.value() {
            if self
                .event_loop
                .send_event(RuffleEvent::OpenURL(
                    url.clone(),
                    Box::new(self.options.clone()),
                ))
                .is_ok()
            {
                return true;
            }
        }

        false
    }

    pub fn show(&mut self, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;
        let mut is_valid = true;

        Window::new(text(&self.locale, "open-dialog"))
            .open(&mut keep_open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(egui_ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Grid::new("open-file-options")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(text(&self.locale, "open-dialog-path"));
                            is_valid &= self.path.ui(&self.locale, ui).value().is_some();
                            ui.end_row();
                        });
                });

                ui.collapsing(text(&self.locale, "network-settings"), |ui| {
                    is_valid &= self.network_settings(ui);
                });

                ui.collapsing(text(&self.locale, "player-settings"), |ui| {
                    self.player_settings(ui);
                });

                ui.collapsing(text(&self.locale, "movie-parameters"), |ui| {
                    self.movie_parameters(ui);
                });

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(is_valid, Button::new(text(&self.locale, "start")))
                            .clicked()
                        {
                            should_close = self.start();
                        }
                    })
                });
            });

        keep_open && !should_close
    }

    fn network_settings(&mut self, ui: &mut Ui) -> bool {
        let mut is_valid = true;

        Grid::new("open-file-network-options")
            .num_columns(2)
            .striped(true)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label(text(&self.locale, "custom-base-url"));
                is_valid &= self.base_url.ui(ui, &mut self.options.base).is_valid();
                ui.end_row();

                ui.label(text(&self.locale, "spoof-swf-url"));
                is_valid &= self
                    .spoof_url
                    .ui(ui, &mut self.options.spoof_url)
                    .is_valid();
                ui.end_row();

                ui.label(text(&self.locale, "proxy"));
                is_valid &= self.proxy_url.ui(ui, &mut self.options.proxy).is_valid();
                ui.end_row();

                ui.label(text(&self.locale, "upgrade-http"));
                ui.checkbox(
                    &mut self.options.upgrade_to_https,
                    text(&self.locale, "upgrade-http-check"),
                );
                ui.end_row();

                // TODO: This should probably be a global setting somewhere, not per load
                ui.label(text(&self.locale, "open-url-mode"));
                ComboBox::from_id_source("open-file-advanced-options-open-url-mode")
                    .selected_text(match self.options.open_url_mode {
                        OpenURLMode::Allow => text(&self.locale, "open-url-mode-allow"),
                        OpenURLMode::Confirm => text(&self.locale, "open-url-mode-confirm"),
                        OpenURLMode::Deny => text(&self.locale, "open-url-mode-deny"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.open_url_mode,
                            OpenURLMode::Allow,
                            text(&self.locale, "open-url-mode-allow"),
                        );
                        ui.selectable_value(
                            &mut self.options.open_url_mode,
                            OpenURLMode::Confirm,
                            text(&self.locale, "open-url-mode-confirm"),
                        );
                        ui.selectable_value(
                            &mut self.options.open_url_mode,
                            OpenURLMode::Deny,
                            text(&self.locale, "open-url-mode-deny"),
                        );
                    });
                ui.end_row();

                ui.label(text(&self.locale, "load-behavior"));
                ComboBox::from_id_source("open-file-advanced-options-load-behaviour")
                    .selected_text(match self.options.load_behavior {
                        LoadBehavior::Streaming => text(&self.locale, "load-behavior-streaming"),
                        LoadBehavior::Delayed => text(&self.locale, "load-behavior-delayed"),
                        LoadBehavior::Blocking => text(&self.locale, "load-behavior-blocking"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.load_behavior,
                            LoadBehavior::Streaming,
                            text(&self.locale, "load-behavior-streaming"),
                        );
                        ui.selectable_value(
                            &mut self.options.load_behavior,
                            LoadBehavior::Delayed,
                            text(&self.locale, "load-behavior-delayed"),
                        );
                        ui.selectable_value(
                            &mut self.options.load_behavior,
                            LoadBehavior::Blocking,
                            text(&self.locale, "load-behavior-blocking"),
                        );
                    });
                ui.end_row();
            });

        is_valid
    }

    fn player_settings(&mut self, ui: &mut Ui) {
        Grid::new("open-file-player-options")
            .num_columns(2)
            .striped(true)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label(text(&self.locale, "max-execution-duration"));
                Slider::new(&mut self.options.max_execution_duration, 1.0..=600.0)
                    .suffix(text(&self.locale, "max-execution-duration-suffix"))
                    .ui(ui);
                ui.end_row();

                ui.label(text(&self.locale, "quality"));
                ComboBox::from_id_source("open-file-advanced-options-quality")
                    .selected_text(match self.options.quality {
                        StageQuality::Low => text(&self.locale, "quality-low"),
                        StageQuality::Medium => text(&self.locale, "quality-medium"),
                        StageQuality::High => text(&self.locale, "quality-high"),
                        StageQuality::Best => text(&self.locale, "quality-best"),
                        StageQuality::High8x8 => text(&self.locale, "quality-high8x8"),
                        StageQuality::High8x8Linear => text(&self.locale, "quality-high8x8linear"),
                        StageQuality::High16x16 => text(&self.locale, "quality-high16x16"),
                        StageQuality::High16x16Linear => {
                            text(&self.locale, "quality-high16x16linear")
                        }
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::Low,
                            text(&self.locale, "quality-low"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::Medium,
                            text(&self.locale, "quality-medium"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High,
                            text(&self.locale, "quality-high"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::Best,
                            text(&self.locale, "quality-best"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High8x8,
                            text(&self.locale, "quality-high8x8"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High8x8Linear,
                            text(&self.locale, "quality-high8x8linear"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High16x16,
                            text(&self.locale, "quality-high16x16"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High16x16Linear,
                            text(&self.locale, "quality-high16x16linear"),
                        );
                    });
                ui.end_row();

                ui.label(text(&self.locale, "scale-mode"));
                ComboBox::from_id_source("open-file-advanced-options-scale")
                    .selected_text(match self.options.scale {
                        StageScaleMode::ExactFit => text(&self.locale, "scale-mode-exactfit"),
                        StageScaleMode::NoBorder => text(&self.locale, "scale-mode-noborder"),
                        StageScaleMode::NoScale => text(&self.locale, "scale-mode-noscale"),
                        StageScaleMode::ShowAll => text(&self.locale, "scale-mode-showall"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.scale,
                            StageScaleMode::ExactFit,
                            text(&self.locale, "scale-mode-exactfit"),
                        );
                        ui.selectable_value(
                            &mut self.options.scale,
                            StageScaleMode::NoBorder,
                            text(&self.locale, "scale-mode-noborder"),
                        );
                        ui.selectable_value(
                            &mut self.options.scale,
                            StageScaleMode::NoScale,
                            text(&self.locale, "scale-mode-noscale"),
                        );
                        ui.selectable_value(
                            &mut self.options.scale,
                            StageScaleMode::ShowAll,
                            text(&self.locale, "scale-mode-showall"),
                        );
                    });
                ui.end_row();

                ui.label(text(&self.locale, "force-scale-mode"));
                ui.checkbox(
                    &mut self.options.force_scale,
                    text(&self.locale, "force-scale-mode-check"),
                );
                ui.end_row();

                ui.label(text(&self.locale, "dummy-external-interface"));
                ui.checkbox(
                    &mut self.options.dummy_external_interface,
                    text(&self.locale, "dummy-external-interface-check"),
                );
                ui.end_row();

                // TODO: This should probably be a global setting somewhere, not per load
                ui.label(text(&self.locale, "warn-if-unsupported"));
                ui.checkbox(
                    &mut self.options.warn_on_unsupported_content,
                    text(&self.locale, "warn-if-unsupported-check"),
                );
                ui.end_row();

                ui.label(text(&self.locale, "player-version"));
                DragValue::new(&mut self.options.player_version)
                    .clamp_range(1..=32)
                    .ui(ui);
                ui.end_row();

                ui.label(text(&self.locale, "custom-framerate"));
                ui.horizontal(|ui| {
                    Checkbox::without_text(&mut self.framerate_enabled).ui(ui);
                    ui.add_enabled_ui(self.framerate_enabled, |ui| {
                        ui.add_sized(
                            ui.available_size(),
                            Slider::new(&mut self.framerate, 0.0..=100.0)
                                .clamp_to_range(false)
                                .suffix(text(&self.locale, "custom-framerate-suffix")),
                        );
                    });
                });
                ui.end_row();
            });
    }

    fn movie_parameters(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .button(text(&self.locale, "open-dialog-add-parameter"))
                .clicked()
            {
                self.options
                    .parameters
                    .push((Default::default(), Default::default()));
            }

            if ui
                .add_enabled(
                    !self.options.parameters.is_empty(),
                    Button::new(text(&self.locale, "open-dialog-clear-parameters")),
                )
                .clicked()
            {
                self.options.parameters.clear();
            }
        });

        Grid::new("open-file-params")
            .num_columns(2)
            .spacing([5.0, 4.0])
            .min_col_width(100.0)
            .striped(true)
            .show(ui, |ui| {
                self.options.parameters.retain_mut(|(key, value)| {
                    let mut keep = true;
                    ui.text_edit_singleline(key);
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(value);
                        if ui
                            .button("x")
                            .on_hover_text(text(&self.locale, "open-dialog-delete-parameter"))
                            .clicked()
                        {
                            keep = false;
                        }
                    });
                    ui.end_row();
                    keep
                });
            });
    }
}

struct PathOrUrlField {
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

struct OptionalUrlField {
    value: String,
    error: bool,
    enabled: bool,
    hint: &'static str,
}

impl OptionalUrlField {
    pub fn new(default: &Option<Url>, hint: &'static str) -> Self {
        if let Some(default) = default {
            Self {
                value: default.to_string(),
                error: false,
                enabled: true,
                hint,
            }
        } else {
            Self {
                value: "".to_string(),
                error: false,
                enabled: false,
                hint,
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, result: &mut Option<Url>) -> &mut Self {
        ui.horizontal(|ui| {
            Checkbox::without_text(&mut self.enabled).ui(ui);
            ui.add_enabled_ui(self.enabled, |ui| {
                ui.add_sized(
                    ui.available_size(),
                    TextEdit::singleline(&mut self.value)
                        .hint_text(self.hint)
                        .text_color_opt(if self.error {
                            Some(ui.style().visuals.error_fg_color)
                        } else {
                            None
                        }),
                );
            });
        });

        if self.enabled {
            match Url::parse(&self.value) {
                Ok(url) => {
                    *result = Some(url);
                    self.error = false;
                }
                Err(_) => {
                    self.error = true;
                }
            }
        } else {
            *result = None;
            self.error = false;
        }

        self
    }

    pub fn is_valid(&self) -> bool {
        !self.error
    }
}
