use crate::custom_event::RuffleEvent;
use crate::gui::text;
use crate::gui::widgets::PathOrUrlField;
use crate::player::PlayerOptions;
use egui::{
    Align2, Button, Checkbox, ComboBox, DragValue, Grid, Slider, TextEdit, Ui, Widget, Window,
};
use ruffle_core::backend::navigator::{OpenURLMode, SocketMode};
use ruffle_core::config::Letterbox;
use ruffle_core::{LoadBehavior, StageAlign, StageScaleMode};
use ruffle_render::quality::StageQuality;
use unic_langid::LanguageIdentifier;
use url::Url;
use winit::event_loop::EventLoopProxy;

pub struct OpenDialog {
    options: PlayerOptions,
    event_loop: EventLoopProxy<RuffleEvent>,

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
    ) -> Self {
        let spoof_url = OptionalUrlField::new(&defaults.spoof_url, "https://example.org/game.swf");
        let base_url = OptionalUrlField::new(&defaults.base, "https://example.org");
        let proxy_url = OptionalUrlField::new(&defaults.proxy, "socks5://localhost:8080");
        let path = PathOrUrlField::new(default_url, "path/to/movie.swf");
        Self {
            options: defaults,
            event_loop,
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

    pub fn show(&mut self, locale: &LanguageIdentifier, egui_ctx: &egui::Context) -> bool {
        let mut keep_open = true;
        let mut should_close = false;
        let mut is_valid = true;

        Window::new(text(locale, "open-dialog"))
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
                            ui.label(text(locale, "open-dialog-path"));
                            is_valid &= self.path.ui(locale, ui).value().is_some();
                            ui.end_row();
                        });
                });

                ui.collapsing(text(locale, "network-settings"), |ui| {
                    is_valid &= self.network_settings(locale, ui);
                });

                ui.collapsing(text(locale, "player-settings"), |ui| {
                    self.player_settings(locale, ui);
                });

                ui.collapsing(text(locale, "movie-parameters"), |ui| {
                    self.movie_parameters(locale, ui);
                });

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(is_valid, Button::new(text(locale, "start")))
                            .clicked()
                        {
                            should_close = self.start();
                        }
                    })
                });
            });

        keep_open && !should_close
    }

    fn network_settings(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) -> bool {
        let mut is_valid = true;

        Grid::new("open-file-network-options")
            .num_columns(2)
            .striped(true)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label(text(locale, "custom-base-url"));
                is_valid &= self.base_url.ui(ui, &mut self.options.base).is_valid();
                ui.end_row();

                ui.label(text(locale, "spoof-swf-url"));
                is_valid &= self
                    .spoof_url
                    .ui(ui, &mut self.options.spoof_url)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "proxy"));
                is_valid &= self.proxy_url.ui(ui, &mut self.options.proxy).is_valid();
                ui.end_row();

                ui.label(text(locale, "upgrade-http"));
                ui.checkbox(
                    &mut self.options.upgrade_to_https,
                    text(locale, "upgrade-http-check"),
                );
                ui.end_row();

                ui.label(text(locale, "tcp-connections"));
                ComboBox::from_id_source("open-file-advanced-options-tcp-connections")
                    .selected_text(match self.options.tcp_connections {
                        SocketMode::Allow => text(locale, "tcp-connections-allow"),
                        SocketMode::Ask => text(locale, "tcp-connections-ask"),
                        SocketMode::Deny => text(locale, "tcp-connections-deny"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.tcp_connections,
                            SocketMode::Allow,
                            text(locale, "tcp-connections-allow"),
                        );
                        ui.selectable_value(
                            &mut self.options.tcp_connections,
                            SocketMode::Ask,
                            text(locale, "tcp-connections-ask"),
                        );
                        ui.selectable_value(
                            &mut self.options.tcp_connections,
                            SocketMode::Deny,
                            text(locale, "tcp-connections-deny"),
                        );
                    });
                ui.end_row();

                // TODO: This should probably be a global setting somewhere, not per load
                ui.label(text(locale, "open-url-mode"));
                ComboBox::from_id_source("open-file-advanced-options-open-url-mode")
                    .selected_text(match self.options.open_url_mode {
                        OpenURLMode::Allow => text(locale, "open-url-mode-allow"),
                        OpenURLMode::Confirm => text(locale, "open-url-mode-confirm"),
                        OpenURLMode::Deny => text(locale, "open-url-mode-deny"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.open_url_mode,
                            OpenURLMode::Allow,
                            text(locale, "open-url-mode-allow"),
                        );
                        ui.selectable_value(
                            &mut self.options.open_url_mode,
                            OpenURLMode::Confirm,
                            text(locale, "open-url-mode-confirm"),
                        );
                        ui.selectable_value(
                            &mut self.options.open_url_mode,
                            OpenURLMode::Deny,
                            text(locale, "open-url-mode-deny"),
                        );
                    });
                ui.end_row();

                ui.label(text(locale, "load-behavior"));
                ComboBox::from_id_source("open-file-advanced-options-load-behaviour")
                    .selected_text(match self.options.load_behavior {
                        LoadBehavior::Streaming => text(locale, "load-behavior-streaming"),
                        LoadBehavior::Delayed => text(locale, "load-behavior-delayed"),
                        LoadBehavior::Blocking => text(locale, "load-behavior-blocking"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.load_behavior,
                            LoadBehavior::Streaming,
                            text(locale, "load-behavior-streaming"),
                        );
                        ui.selectable_value(
                            &mut self.options.load_behavior,
                            LoadBehavior::Delayed,
                            text(locale, "load-behavior-delayed"),
                        );
                        ui.selectable_value(
                            &mut self.options.load_behavior,
                            LoadBehavior::Blocking,
                            text(locale, "load-behavior-blocking"),
                        );
                    });
                ui.end_row();
            });

        is_valid
    }

    fn player_settings(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        Grid::new("open-file-player-options")
            .num_columns(2)
            .striped(true)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label(text(locale, "max-execution-duration"));
                Slider::new(&mut self.options.max_execution_duration, 1.0..=600.0)
                    .suffix(text(locale, "max-execution-duration-suffix"))
                    .ui(ui);
                ui.end_row();

                ui.label(text(locale, "quality"));
                ComboBox::from_id_source("open-file-advanced-options-quality")
                    .selected_text(match self.options.quality {
                        StageQuality::Low => text(locale, "quality-low"),
                        StageQuality::Medium => text(locale, "quality-medium"),
                        StageQuality::High => text(locale, "quality-high"),
                        StageQuality::Best => text(locale, "quality-best"),
                        StageQuality::High8x8 => text(locale, "quality-high8x8"),
                        StageQuality::High8x8Linear => text(locale, "quality-high8x8linear"),
                        StageQuality::High16x16 => text(locale, "quality-high16x16"),
                        StageQuality::High16x16Linear => text(locale, "quality-high16x16linear"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::Low,
                            text(locale, "quality-low"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::Medium,
                            text(locale, "quality-medium"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High,
                            text(locale, "quality-high"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::Best,
                            text(locale, "quality-best"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High8x8,
                            text(locale, "quality-high8x8"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High8x8Linear,
                            text(locale, "quality-high8x8linear"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High16x16,
                            text(locale, "quality-high16x16"),
                        );
                        ui.selectable_value(
                            &mut self.options.quality,
                            StageQuality::High16x16Linear,
                            text(locale, "quality-high16x16linear"),
                        );
                    });
                ui.end_row();

                ui.label(text(locale, "letterbox"));
                ComboBox::from_id_source("open-file-advanced-options-letterbox")
                    .selected_text(match self.options.letterbox {
                        Letterbox::On => text(locale, "letterbox-on"),
                        Letterbox::Fullscreen => text(locale, "letterbox-fullscreen"),
                        Letterbox::Off => text(locale, "letterbox-off"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.options.letterbox,
                            Letterbox::On,
                            text(locale, "letterbox-on"),
                        );
                        ui.selectable_value(
                            &mut self.options.letterbox,
                            Letterbox::Fullscreen,
                            text(locale, "letterbox-fullscreen"),
                        );
                        ui.selectable_value(
                            &mut self.options.letterbox,
                            Letterbox::Off,
                            text(locale, "letterbox-off"),
                        );
                    });
                ui.end_row();

                ui.label(text(locale, "align"));
                ui.horizontal(|ui| {
                    ComboBox::from_id_source("open-file-advanced-options-align")
                        .selected_text(match self.options.align {
                            StageAlign::TOP => text(locale, "align-top"),
                            StageAlign::BOTTOM => text(locale, "align-bottom"),
                            StageAlign::LEFT => text(locale, "align-left"),
                            StageAlign::RIGHT => text(locale, "align-right"),
                            _ => {
                                let align = self.options.align;
                                if align == StageAlign::TOP | StageAlign::LEFT {
                                    text(locale, "align-top-left")
                                } else if align == StageAlign::TOP | StageAlign::RIGHT {
                                    text(locale, "align-top-right")
                                } else if align == StageAlign::BOTTOM | StageAlign::LEFT {
                                    text(locale, "align-bottom-left")
                                } else if align == StageAlign::BOTTOM | StageAlign::RIGHT {
                                    text(locale, "align-bottom-right")
                                } else {
                                    text(locale, "align-center")
                                }
                            }
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::default(),
                                text(locale, "align-center"),
                            );
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::TOP,
                                text(locale, "align-top"),
                            );
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::BOTTOM,
                                text(locale, "align-bottom"),
                            );
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::LEFT,
                                text(locale, "align-left"),
                            );
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::RIGHT,
                                text(locale, "align-right"),
                            );
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::TOP | StageAlign::LEFT,
                                text(locale, "align-top-left"),
                            );
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::TOP | StageAlign::RIGHT,
                                text(locale, "align-top-right"),
                            );
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::BOTTOM | StageAlign::LEFT,
                                text(locale, "align-bottom-left"),
                            );
                            ui.selectable_value(
                                &mut self.options.align,
                                StageAlign::BOTTOM | StageAlign::RIGHT,
                                text(locale, "align-bottom-right"),
                            );
                        });
                    ui.checkbox(&mut self.options.force_align, text(locale, "align-force"));
                });
                ui.end_row();

                ui.label(text(locale, "scale-mode"));
                ui.horizontal(|ui| {
                    ComboBox::from_id_source("open-file-advanced-options-scale")
                        .selected_text(match self.options.scale {
                            StageScaleMode::ExactFit => text(locale, "scale-mode-exactfit"),
                            StageScaleMode::NoBorder => text(locale, "scale-mode-noborder"),
                            StageScaleMode::NoScale => text(locale, "scale-mode-noscale"),
                            StageScaleMode::ShowAll => text(locale, "scale-mode-showall"),
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.options.scale,
                                StageScaleMode::ExactFit,
                                text(locale, "scale-mode-exactfit"),
                            );
                            ui.selectable_value(
                                &mut self.options.scale,
                                StageScaleMode::NoBorder,
                                text(locale, "scale-mode-noborder"),
                            );
                            ui.selectable_value(
                                &mut self.options.scale,
                                StageScaleMode::NoScale,
                                text(locale, "scale-mode-noscale"),
                            );
                            ui.selectable_value(
                                &mut self.options.scale,
                                StageScaleMode::ShowAll,
                                text(locale, "scale-mode-showall"),
                            );
                        });
                    ui.checkbox(
                        &mut self.options.force_scale,
                        text(locale, "scale-mode-force"),
                    );
                });
                ui.end_row();

                ui.label(text(locale, "dummy-external-interface"));
                ui.checkbox(
                    &mut self.options.dummy_external_interface,
                    text(locale, "dummy-external-interface-check"),
                );
                ui.end_row();

                ui.label(text(locale, "player-version"));
                DragValue::new(&mut self.options.player_version)
                    .clamp_range(1..=32)
                    .ui(ui);
                ui.end_row();

                ui.label(text(locale, "custom-framerate"));
                ui.horizontal(|ui| {
                    Checkbox::without_text(&mut self.framerate_enabled).ui(ui);
                    ui.add_enabled_ui(self.framerate_enabled, |ui| {
                        ui.add_sized(
                            ui.available_size(),
                            Slider::new(&mut self.framerate, 0.0..=100.0)
                                .clamp_to_range(false)
                                .suffix(text(locale, "custom-framerate-suffix")),
                        );
                    });
                });
                ui.end_row();
            });
    }

    fn movie_parameters(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .button(text(locale, "open-dialog-add-parameter"))
                .clicked()
            {
                self.options
                    .parameters
                    .push((Default::default(), Default::default()));
            }

            if ui
                .add_enabled(
                    !self.options.parameters.is_empty(),
                    Button::new(text(locale, "open-dialog-remove-parameters")),
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
                            .on_hover_text(text(locale, "open-dialog-remove-parameter"))
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
