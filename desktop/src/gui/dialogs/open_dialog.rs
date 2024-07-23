use crate::custom_event::RuffleEvent;
use crate::gui::text;
use crate::gui::widgets::PathOrUrlField;
use crate::player::LaunchOptions;
use egui::{
    emath, Align2, Button, Checkbox, ComboBox, Grid, Layout, Slider, TextEdit, Ui, Widget, Window,
};
use ruffle_core::backend::navigator::{OpenURLMode, SocketMode};
use ruffle_core::config::Letterbox;
use ruffle_core::{LoadBehavior, PlayerRuntime, StageAlign, StageScaleMode};
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
    referer: OptionalField<UrlField>,
    cookie: OptionalField<CookieField>,
    base_url: OptionalField<UrlField>,
    proxy_url: OptionalField<UrlField>,
    path: PathOrUrlField,

    framerate: f64,
    framerate_enabled: bool,

    script_timeout: OptionalField<DurationField>,
    tcp_connections: OptionalField<EnumDropdownField<SocketMode>>,
    quality: OptionalField<EnumDropdownField<StageQuality>>,
    align: OptionalField<FieldWithCheckbox<EnumDropdownField<StageAlign>>>,
    scale_mode: OptionalField<FieldWithCheckbox<EnumDropdownField<StageScaleMode>>>,
    load_behavior: OptionalField<EnumDropdownField<LoadBehavior>>,
    letterbox: OptionalField<EnumDropdownField<Letterbox>>,
    player_version: OptionalField<NumberField<u8>>,
    player_runtime: OptionalField<EnumDropdownField<PlayerRuntime>>,
    dummy_external_interface: OptionalField<BooleanDropdownField>,
    upgrade_to_https: OptionalField<BooleanDropdownField>,
}

impl OpenDialog {
    pub fn new(
        defaults: LaunchOptions,
        default_url: Option<Url>,
        event_loop: EventLoopProxy<RuffleEvent>,
    ) -> Self {
        let spoof_url = OptionalField::new(
            defaults.player.spoof_url.as_ref().map(Url::to_string),
            UrlField::new("https://example.org/game.swf"),
        );
        let referer = OptionalField::new(
            defaults.player.referer.as_ref().map(Url::to_string),
            UrlField::new("https://example.org"),
        );
        let cookie = OptionalField::new(
            defaults.player.cookie.clone(),
            CookieField::new("value1=cookie1; value2=cookie2"),
        );
        let base_url = OptionalField::new(
            defaults.player.base.as_ref().map(Url::to_string),
            UrlField::new("https://example.org"),
        );
        let proxy_url = OptionalField::new(
            defaults.proxy.as_ref().map(Url::to_string),
            UrlField::new("socks5://localhost:8080"),
        );
        let path = PathOrUrlField::new(default_url, "path/to/movie.swf");
        let script_timeout = OptionalField::new(
            defaults
                .player
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
            defaults.player.quality,
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
        let align = OptionalField::new(
            defaults
                .player
                .align
                .map(|a| (a, defaults.player.force_align.unwrap_or_default())),
            FieldWithCheckbox::new(
                EnumDropdownField::new(
                    StageAlign::default(),
                    vec![
                        StageAlign::default(),
                        StageAlign::TOP,
                        StageAlign::BOTTOM,
                        StageAlign::LEFT,
                        StageAlign::RIGHT,
                        StageAlign::TOP | StageAlign::LEFT,
                        StageAlign::TOP | StageAlign::RIGHT,
                        StageAlign::BOTTOM | StageAlign::LEFT,
                        StageAlign::BOTTOM | StageAlign::RIGHT,
                    ],
                    Box::new(|value, locale| match value {
                        StageAlign::TOP => text(locale, "align-top"),
                        StageAlign::BOTTOM => text(locale, "align-bottom"),
                        StageAlign::LEFT => text(locale, "align-left"),
                        StageAlign::RIGHT => text(locale, "align-right"),
                        _ => {
                            if value == StageAlign::TOP | StageAlign::LEFT {
                                text(locale, "align-top-left")
                            } else if value == StageAlign::TOP | StageAlign::RIGHT {
                                text(locale, "align-top-right")
                            } else if value == StageAlign::BOTTOM | StageAlign::LEFT {
                                text(locale, "align-bottom-left")
                            } else if value == StageAlign::BOTTOM | StageAlign::RIGHT {
                                text(locale, "align-bottom-right")
                            } else {
                                text(locale, "align-center")
                            }
                        }
                    }),
                ),
                Box::new(|locale| text(locale, "align-force")),
                false,
            ),
        );
        let scale_mode = OptionalField::new(
            defaults
                .player
                .scale
                .map(|a| (a, defaults.player.force_scale.unwrap_or_default())),
            FieldWithCheckbox::new(
                EnumDropdownField::new(
                    StageScaleMode::default(),
                    vec![
                        StageScaleMode::NoScale,
                        StageScaleMode::ShowAll,
                        StageScaleMode::ExactFit,
                        StageScaleMode::NoBorder,
                    ],
                    Box::new(|value, locale| match value {
                        StageScaleMode::NoScale => text(locale, "scale-mode-noscale"),
                        StageScaleMode::ShowAll => text(locale, "scale-mode-showall"),
                        StageScaleMode::ExactFit => text(locale, "scale-mode-exactfit"),
                        StageScaleMode::NoBorder => text(locale, "scale-mode-noborder"),
                    }),
                )
                .with_tooltips(Box::new(|value, locale| match value {
                    StageScaleMode::NoScale => Some(text(locale, "scale-mode-noscale-tooltip")),
                    StageScaleMode::ShowAll => Some(text(locale, "scale-mode-showall-tooltip")),
                    StageScaleMode::ExactFit => Some(text(locale, "scale-mode-exactfit-tooltip")),
                    StageScaleMode::NoBorder => Some(text(locale, "scale-mode-noborder-tooltip")),
                })),
                Box::new(|locale| text(locale, "scale-mode-force")),
                false,
            )
            .with_checkbox_tooltip(Box::new(|locale| text(locale, "scale-mode-force-tooltip"))),
        );
        let load_behavior = OptionalField::new(
            defaults.player.load_behavior,
            EnumDropdownField::new(
                LoadBehavior::Streaming,
                vec![
                    LoadBehavior::Streaming,
                    LoadBehavior::Delayed,
                    LoadBehavior::Blocking,
                ],
                Box::new(|value, locale| match value {
                    LoadBehavior::Streaming => text(locale, "load-behavior-streaming"),
                    LoadBehavior::Delayed => text(locale, "load-behavior-delayed"),
                    LoadBehavior::Blocking => text(locale, "load-behavior-blocking"),
                }),
            ),
        );
        let letterbox = OptionalField::new(
            defaults.player.letterbox,
            EnumDropdownField::new(
                Letterbox::On,
                vec![Letterbox::On, Letterbox::Fullscreen, Letterbox::Off],
                Box::new(|value, locale| match value {
                    Letterbox::On => text(locale, "letterbox-on"),
                    Letterbox::Fullscreen => text(locale, "letterbox-fullscreen"),
                    Letterbox::Off => text(locale, "letterbox-off"),
                }),
            ),
        );
        let player_version =
            OptionalField::new(defaults.player.player_version, NumberField::new(1..=32, 32));
        let player_runtime = OptionalField::new(
            defaults.player.player_runtime,
            EnumDropdownField::new(
                PlayerRuntime::default(),
                vec![PlayerRuntime::FlashPlayer, PlayerRuntime::AIR],
                Box::new(|value, locale| match value {
                    PlayerRuntime::FlashPlayer => text(locale, "player-runtime-flash"),
                    PlayerRuntime::AIR => text(locale, "player-runtime-air"),
                }),
            ),
        );
        let dummy_external_interface = OptionalField::new(
            defaults.player.dummy_external_interface,
            BooleanDropdownField::new(
                false,
                Box::new(|value, locale| match value {
                    true => text(locale, "enable"),
                    false => text(locale, "disable"),
                }),
            ),
        );
        let upgrade_to_https = OptionalField::new(
            defaults.player.upgrade_to_https,
            BooleanDropdownField::new(
                false,
                Box::new(|value, locale| match value {
                    true => text(locale, "enable"),
                    false => text(locale, "disable"),
                }),
            ),
        );

        Self {
            options: defaults,
            event_loop,
            spoof_url,
            referer,
            cookie,
            base_url,
            proxy_url,
            path,
            framerate: 30.0,
            framerate_enabled: false,
            script_timeout,
            tcp_connections,
            quality,
            align,
            scale_mode,
            load_behavior,
            letterbox,
            player_version,
            player_runtime,
            dummy_external_interface,
            upgrade_to_https,
        }
    }

    fn start(&mut self) -> bool {
        if self.framerate_enabled {
            self.options.player.frame_rate = Some(self.framerate);
        } else {
            self.options.player.frame_rate = None;
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
                    .ui(ui, &mut self.options.player.base, locale)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "spoof-swf-url"));
                is_valid &= self
                    .spoof_url
                    .ui(ui, &mut self.options.player.spoof_url, locale)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "referer-url"));
                is_valid &= self
                    .referer
                    .ui(ui, &mut self.options.player.referer, locale)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "cookie"));
                is_valid &= self
                    .cookie
                    .ui(ui, &mut self.options.player.cookie, locale)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "proxy"));
                is_valid &= self
                    .proxy_url
                    .ui(ui, &mut self.options.proxy, locale)
                    .is_valid();
                ui.end_row();

                ui.label(text(locale, "upgrade-http"));
                self.upgrade_to_https
                    .ui(ui, &mut self.options.player.upgrade_to_https, locale);
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
                self.load_behavior
                    .ui(ui, &mut self.options.player.load_behavior, locale);
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
                    .ui(ui, &mut self.options.player.max_execution_duration, locale);
                ui.end_row();

                ui.label(text(locale, "quality"));
                self.quality
                    .ui(ui, &mut self.options.player.quality, locale);
                ui.end_row();

                ui.label(text(locale, "letterbox"));
                self.letterbox
                    .ui(ui, &mut self.options.player.letterbox, locale);
                ui.end_row();

                ui.label(text(locale, "align"));
                let mut align = self
                    .options
                    .player
                    .align
                    .map(|a| (a, self.options.player.force_align.unwrap_or_default()));
                self.align.ui(ui, &mut align, locale);
                match align {
                    Some((align, force)) => {
                        self.options.player.align = Some(align);
                        self.options.player.force_align = Some(force);
                    }
                    None => {
                        self.options.player.align = None;
                        self.options.player.force_align = None;
                    }
                }
                ui.end_row();

                ui.label(text(locale, "scale-mode"));
                let mut scale_mode = self
                    .options
                    .player
                    .scale
                    .map(|a| (a, self.options.player.force_scale.unwrap_or_default()));
                self.scale_mode.ui(ui, &mut scale_mode, locale);
                match scale_mode {
                    Some((scale, force)) => {
                        self.options.player.scale = Some(scale);
                        self.options.player.force_scale = Some(force);
                    }
                    None => {
                        self.options.player.scale = None;
                        self.options.player.force_scale = None;
                    }
                }
                ui.end_row();

                ui.label(text(locale, "dummy-external-interface"));
                self.dummy_external_interface.ui(
                    ui,
                    &mut self.options.player.dummy_external_interface,
                    locale,
                );
                ui.end_row();

                ui.label(text(locale, "player-version"));
                self.player_version
                    .ui(ui, &mut self.options.player.player_version, locale);
                ui.end_row();

                ui.label(text(locale, "player-runtime"));
                self.player_runtime
                    .ui(ui, &mut self.options.player.player_runtime, locale);
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
                    .player
                    .parameters
                    .push((Default::default(), Default::default()));
            }

            if ui
                .add_enabled(
                    !self.options.player.parameters.is_empty(),
                    Button::new(text(locale, "open-dialog-remove-parameters")),
                )
                .clicked()
            {
                self.options.player.parameters.clear();
            }
        });

        Grid::new("open-file-params")
            .num_columns(2)
            .spacing([5.0, 4.0])
            .min_col_width(100.0)
            .striped(true)
            .show(ui, |ui| {
                self.options.player.parameters.retain_mut(|(key, value)| {
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

struct CookieField {
    hint: &'static str,
}

impl CookieField {
    pub fn new(hint: &'static str) -> Self {
        Self { hint }
    }
}

impl InnerField for CookieField {
    type Value = String;
    type Result = String;

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
        Ok(value.clone())
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

struct NumberField<T: emath::Numeric> {
    range: RangeInclusive<T>,
    default: T,
}

impl<T: emath::Numeric> NumberField<T> {
    pub fn new(range: RangeInclusive<T>, default: T) -> Self {
        Self { range, default }
    }
}

impl<T: emath::Numeric> InnerField for NumberField<T> {
    type Value = T;
    type Result = T;

    fn value_if_missing(&self) -> Self::Value {
        self.default
    }

    fn ui(&self, ui: &mut Ui, value: &mut Self::Value, _error: bool, _locale: &LanguageIdentifier) {
        Slider::new(value, self.range.clone()).ui(ui);
    }

    fn value_to_result(&self, value: &Self::Value) -> Result<Self::Result, ()> {
        Ok(*value)
    }
}

type ValueToTextFn<T> = dyn Fn(T, &LanguageIdentifier) -> Cow<'static, str>;
type ValueToOptTextFn<T> = dyn Fn(T, &LanguageIdentifier) -> Option<Cow<'static, str>>;
type LabelFn = dyn Fn(&LanguageIdentifier) -> Cow<'static, str>;

struct EnumDropdownField<T: Copy> {
    id: egui::Id,
    default: T,
    value_to_name: Box<ValueToTextFn<T>>,
    value_to_tooltip: Box<ValueToOptTextFn<T>>,
    possible_values: Vec<T>,
}

impl<T: Copy> EnumDropdownField<T> {
    pub fn new(default: T, possible_values: Vec<T>, value_to_name: Box<ValueToTextFn<T>>) -> Self {
        Self {
            id: egui::Id::new(rand::random::<u64>()),
            default,
            value_to_name,
            possible_values,
            value_to_tooltip: Box::new(|_, _| None),
        }
    }

    pub fn with_tooltips(mut self, value_to_tooltip: Box<ValueToOptTextFn<T>>) -> Self {
        self.value_to_tooltip = value_to_tooltip;
        self
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
                    let response = ui.selectable_value(
                        value,
                        *possible_value,
                        (self.value_to_name)(*possible_value, locale),
                    );

                    if let Some(tooltip) = (self.value_to_tooltip)(*possible_value, locale) {
                        response.on_hover_text_at_pointer(tooltip);
                    }
                }
            });
    }

    fn value_to_result(&self, value: &Self::Value) -> Result<Self::Result, ()> {
        Ok(*value)
    }
}

struct BooleanDropdownField {
    id: egui::Id,
    default: bool,
    value_to_name: Box<ValueToTextFn<bool>>,
}

impl BooleanDropdownField {
    pub fn new(default: bool, value_to_name: Box<ValueToTextFn<bool>>) -> Self {
        Self {
            id: egui::Id::new(rand::random::<u64>()),
            default,
            value_to_name,
        }
    }
}

impl InnerField for BooleanDropdownField {
    type Value = bool;
    type Result = bool;

    fn value_if_missing(&self) -> Self::Value {
        self.default
    }

    fn ui(&self, ui: &mut Ui, value: &mut Self::Value, _error: bool, locale: &LanguageIdentifier) {
        ComboBox::from_id_source(self.id)
            .selected_text((self.value_to_name)(*value, locale))
            .show_ui(ui, |ui| {
                ui.selectable_value(value, false, (self.value_to_name)(false, locale));
                ui.selectable_value(value, true, (self.value_to_name)(true, locale));
            });
    }

    fn value_to_result(&self, value: &Self::Value) -> Result<Self::Result, ()> {
        Ok(*value)
    }
}

struct FieldWithCheckbox<T: InnerField> {
    field: T,
    checkbox_label: Box<LabelFn>,
    checkbox_default: bool,
    tooltip_label: Option<Box<LabelFn>>,
}

impl<T: InnerField> FieldWithCheckbox<T> {
    pub fn new(field: T, checkbox_label: Box<LabelFn>, checkbox_default: bool) -> Self {
        Self {
            field,
            checkbox_label,
            checkbox_default,
            tooltip_label: None,
        }
    }

    pub fn with_checkbox_tooltip(mut self, tooltip_label: Box<LabelFn>) -> Self {
        self.tooltip_label = Some(tooltip_label);
        self
    }
}

impl<T: InnerField> InnerField for FieldWithCheckbox<T> {
    type Value = (T::Value, bool);
    type Result = (T::Result, bool);

    fn value_if_missing(&self) -> Self::Value {
        (self.field.value_if_missing(), self.checkbox_default)
    }

    fn ui(&self, ui: &mut Ui, value: &mut Self::Value, error: bool, locale: &LanguageIdentifier) {
        self.field.ui(ui, &mut value.0, error, locale);
        let response = ui.checkbox(&mut value.1, (self.checkbox_label)(locale));
        if let Some(ref tooltip_label) = self.tooltip_label {
            response.on_hover_text_at_pointer(tooltip_label(locale));
        }
    }

    fn value_to_result(&self, value: &Self::Value) -> Result<Self::Result, ()> {
        Ok((self.field.value_to_result(&value.0)?, value.1))
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
