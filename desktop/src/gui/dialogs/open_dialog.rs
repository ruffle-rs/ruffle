use crate::custom_event::RuffleEvent;
use crate::gui::text;
use crate::gui::widgets::PathOrUrlField;
use crate::player::LaunchOptions;
use egui::{
    Align2, Button, Checkbox, ComboBox, DragValue, Grid, Layout, Slider, TextEdit, Ui, Widget,
    Window,
};
use ruffle_core::backend::navigator::{OpenURLMode, SocketMode};
use ruffle_core::config::Letterbox;
use ruffle_core::{LoadBehavior, StageAlign, StageScaleMode};
use ruffle_render::quality::StageQuality;
use std::borrow::Cow;
use std::ops::RangeInclusive;
use std::time::Duration;
use unic_langid::LanguageIdentifier;
use url::Url;
use winit::event_loop::EventLoopProxy;

pub struct OpenDialog {
    options: LaunchOptions,
    event_loop: EventLoopProxy<RuffleEvent>,

    // These are outside of PlayerOptions as it can be an invalid value (ie URL) during typing,
    // and we don't want to clear the value if the user, ie, toggles the checkbox.
    spoof_url: OptionalField<UrlField>,
    base_url: OptionalField<UrlField>,
    proxy_url: OptionalField<UrlField>,
    path: PathOrUrlField,

    framerate: f64,
    framerate_enabled: bool,

    script_timeout: OptionalField<DurationField>,
    tcp_connections: OptionalField<EnumDropdownField<SocketMode>>,
    quality: OptionalField<EnumDropdownField<StageQuality>>,
}

impl OpenDialog {
    pub fn new(
        defaults: LaunchOptions,
        default_url: Option<Url>,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> Self {
        let spoof_url = OptionalField::new(
            defaults.spoof_url.as_ref().map(Url::to_string),
            UrlField::new("https://example.org/game.swf"),
        );
        let base_url = OptionalField::new(
            defaults.base.as_ref().map(Url::to_string),
            UrlField::new("https://example.org"),
        );
        let proxy_url = OptionalField::new(
            defaults.proxy.as_ref().map(Url::to_string),
            UrlField::new("socks5://localhost:8080"),
        );
        let path = PathOrUrlField::new(default_url, "path/to/movie.swf");
        let script_timeout = OptionalField::new(
            defaults
                .max_execution_duration
                .as_ref()
                .map(Duration::as_secs_f64),
            DurationField::new(1.0..=600.0, 30.0),
        );
        let tcp_connections = OptionalField::new(
            defaults.tcp_connections,
            EnumDropdownField::new(
                SocketMode::Ask,
                vec![SocketMode::Allow, SocketMode::Ask, SocketMode::Deny],
                Box::new(|value, locale| match value {
                    SocketMode::Allow => text(locale, "tcp-connections-allow"),
                    SocketMode::Ask => text(locale, "tcp-connections-ask"),
                    SocketMode::Deny => text(locale, "tcp-connections-deny"),
                }),
            ),
        );
        let quality = OptionalField::new(
            defaults.quality,
            EnumDropdownField::new(
                StageQuality::High,
                vec![
                    StageQuality::Low,
                    StageQuality::Medium,
                    StageQuality::High,
                    StageQuality::Best,
                    StageQuality::High8x8,
                    StageQuality::High8x8Linear,
                    StageQuality::High16x16,
                    StageQuality::High16x16Linear,
                ],
                Box::new(|value, locale| match value {
                    StageQuality::Low => text(locale, "quality-low"),
                    StageQuality::Medium => text(locale, "quality-medium"),
                    StageQuality::High => text(locale, "quality-high"),
                    StageQuality::Best => text(locale, "quality-best"),
                    StageQuality::High8x8 => text(locale, "quality-high8x8"),
                    StageQuality::High8x8Linear => text(locale, "quality-high8x8linear"),
                    StageQuality::High16x16 => text(locale, "quality-high16x16"),
                    StageQuality::High16x16Linear => text(locale, "quality-high16x16linear"),
                }),
            ),
        );

        Self {
            options: defaults,
            event_loop,
            spoof_url,
            base_url,
            proxy_url,
            path,
            framerate: 30.0,
            framerate_enabled: false,
            script_timeout,
            tcp_connections,
            quality,
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
                is_valid &= self
                    .base_url
                    .ui(ui, &mut self.options.base, locale)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "spoof-swf-url"));
                is_valid &= self
                    .spoof_url
                    .ui(ui, &mut self.options.spoof_url, locale)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "proxy"));
                is_valid &= self
                    .proxy_url
                    .ui(ui, &mut self.options.proxy, locale)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "upgrade-http"));
                ui.checkbox(
                    &mut self.options.upgrade_to_https,
                    text(locale, "upgrade-http-check"),
                );
                ui.end_row();

                ui.label(text(locale, "tcp-connections"));
                self.tcp_connections
                    .ui(ui, &mut self.options.tcp_connections, locale);
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
                self.script_timeout
                    .ui(ui, &mut self.options.max_execution_duration, locale);
                ui.end_row();

                ui.label(text(locale, "quality"));
                self.quality.ui(ui, &mut self.options.quality, locale);
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

trait InnerField {
    type Value;
    type Result;

    fn value_if_missing(&self) -> Self::Value;

    fn ui(&self, ui: &mut Ui, value: &mut Self::Value, error: bool, locale: &LanguageIdentifier);

    fn value_to_result(&self, value: &Self::Value) -> Result<Self::Result, ()>;
}

struct UrlField {
    hint: &'static str,
}

impl UrlField {
    pub fn new(hint: &'static str) -> Self {
        Self { hint }
    }
}

impl InnerField for UrlField {
    type Value = String;
    type Result = Url;

    fn value_if_missing(&self) -> Self::Value {
        String::new()
    }

    fn ui(&self, ui: &mut Ui, value: &mut Self::Value, error: bool, _locale: &LanguageIdentifier) {
        TextEdit::singleline(value)
            .hint_text(self.hint)
            .text_color_opt(if error {
                Some(ui.style().visuals.error_fg_color)
            } else {
                None
            })
            .ui(ui);
    }

    fn value_to_result(&self, value: &Self::Value) -> Result<Self::Result, ()> {
        Url::parse(value).map_err(|_| ())
    }
}

struct DurationField {
    range: RangeInclusive<f64>,
    default_seconds: f64,
}

impl DurationField {
    pub fn new(range: RangeInclusive<f64>, default_seconds: f64) -> Self {
        Self {
            range,
            default_seconds,
        }
    }
}

impl InnerField for DurationField {
    type Value = f64;
    type Result = Duration;

    fn value_if_missing(&self) -> Self::Value {
        self.default_seconds
    }

    fn ui(&self, ui: &mut Ui, value: &mut Self::Value, _error: bool, locale: &LanguageIdentifier) {
        Slider::new(value, self.range.clone())
            .suffix(text(locale, "max-execution-duration-suffix"))
            .ui(ui);
    }

    fn value_to_result(&self, value: &Self::Value) -> Result<Self::Result, ()> {
        Ok(if value.is_finite() {
            Duration::from_secs_f64(*value)
        } else {
            Duration::MAX
        })
    }
}

type ValueToTextFn<T> = dyn Fn(T, &LanguageIdentifier) -> Cow<'static, str>;

struct EnumDropdownField<T: Copy> {
    id: egui::Id,
    default: T,
    value_to_name: Box<ValueToTextFn<T>>,
    possible_values: Vec<T>,
}

impl<T: Copy> EnumDropdownField<T> {
    pub fn new(default: T, possible_values: Vec<T>, value_to_name: Box<ValueToTextFn<T>>) -> Self {
        Self {
            id: egui::Id::new(rand::random::<u64>()),
            default,
            value_to_name,
            possible_values,
        }
    }
}

impl<T: Copy + PartialEq> InnerField for EnumDropdownField<T> {
    type Value = T;
    type Result = T;

    fn value_if_missing(&self) -> Self::Value {
        self.default
    }

    fn ui(&self, ui: &mut Ui, value: &mut Self::Value, _error: bool, locale: &LanguageIdentifier) {
        ComboBox::from_id_source(self.id)
            .selected_text((self.value_to_name)(*value, locale))
            .show_ui(ui, |ui| {
                for possible_value in &self.possible_values {
                    ui.selectable_value(
                        value,
                        *possible_value,
                        (self.value_to_name)(*possible_value, locale),
                    );
                }
            });
    }

    fn value_to_result(&self, value: &Self::Value) -> Result<Self::Result, ()> {
        Ok(*value)
    }
}

struct OptionalField<Inner: InnerField> {
    value: Inner::Value,
    error: bool,
    enabled: bool,
    inner: Inner,
}

impl<Inner: InnerField> OptionalField<Inner> {
    pub fn new(default: Option<Inner::Value>, inner: Inner) -> Self {
        if let Some(default) = default {
            Self {
                value: default,
                error: false,
                enabled: true,
                inner,
            }
        } else {
            Self {
                value: inner.value_if_missing(),
                error: false,
                enabled: false,
                inner,
            }
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        result: &mut Option<Inner::Result>,
        locale: &LanguageIdentifier,
    ) -> &mut Self {
        ui.horizontal(|ui| {
            Checkbox::without_text(&mut self.enabled).ui(ui);
            ui.add_enabled_ui(self.enabled, |ui| {
                let layout = Layout::centered_and_justified(ui.layout().main_dir());
                ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
                    self.inner.ui(ui, &mut self.value, self.error, locale);
                });
            });
        });

        if self.enabled {
            match self.inner.value_to_result(&self.value) {
                Ok(value) => {
                    *result = Some(value);
                    self.error = false;
                }
                Err(()) => {
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
