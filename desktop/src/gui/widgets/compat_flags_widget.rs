use std::borrow::Cow;

use egui::{ComboBox, Id, Ui};
use egui_extras::{Column, TableBuilder};
use ruffle_core::compat_flags::CompatFlags;
use unic_langid::LanguageIdentifier;

use crate::gui::{text, LocalizableText};

pub struct CompatFlagsWidget {
    locked_flags: CompatFlags,
    locked_text: LocalizableText,
    flags: CompatFlags,
    flags_changed: bool,
}

impl CompatFlagsWidget {
    pub fn new(
        flags: CompatFlags,
        locked_flags: CompatFlags,
        locked_text: LocalizableText,
    ) -> Self {
        Self {
            locked_flags,
            locked_text,
            flags,
            flags_changed: false,
        }
    }

    pub fn changed(&self) -> bool {
        self.flags_changed
    }

    pub fn flags(&self) -> &CompatFlags {
        &self.flags
    }

    pub fn show(&mut self, locale: &LanguageIdentifier, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
                .column(Column::exact(12.0))
                .column(Column::auto())
                .column(Column::remainder().resizable(true))
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .header(20.0, |mut header| {
                    header.col(|_| {});
                    header.col(|ui| {
                        ui.strong(text(locale, "preferences-panel-flag-id"));
                    });
                    header.col(|ui| {
                        ui.strong(text(locale, "preferences-panel-flag-description"));
                    });
                    header.col(|ui| {
                        ui.strong(text(locale, "preferences-panel-flag-enabled"));
                    });
                    header.col(|ui| {
                        ui.strong(text(locale, "preferences-panel-flag-default"))
                            .on_hover_text_at_pointer(text(
                                locale,
                                "preferences-panel-flag-default-tooltip",
                            ));
                    });
                    header.col(|ui| {
                        ui.strong(text(locale, "preferences-panel-flag-fp"))
                            .on_hover_text_at_pointer(text(
                                locale,
                                "preferences-panel-flag-fp-tooltip",
                            ));
                    });
                })
                .body(|mut body| {
                    for &flag in CompatFlags::all_known_flags() {
                        let def = flag.definition();
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                if self.locked_flags.enabled(flag).is_ok() {
                                    ui.label("🔒").on_hover_text_at_pointer(
                                        self.locked_text.localize(locale),
                                    );
                                } else if self.flags.enabled(flag).is_ok() {
                                    ui.label("⚠").on_hover_text_at_pointer(text(
                                        locale,
                                        "preferences-panel-flag-overridden",
                                    ));
                                }
                            });
                            row.col(|ui| {
                                ui.label(def.id);
                            });
                            row.col(|ui| {
                                ui.label(def.name(locale))
                                    .on_hover_text_at_pointer(def.description(locale));
                            });
                            row.col(|ui| {
                                if let Ok(locked_value) = self.locked_flags.enabled(flag) {
                                    if locked_value {
                                        ui.weak(text(locale, "enable"));
                                    } else {
                                        ui.weak(text(locale, "disable"));
                                    }
                                } else {
                                    let orig_value = self.flags.enabled(flag).ok();
                                    let mut current_value = orig_value;
                                    ComboBox::from_id_salt(Id::new("flag-switch").with(flag))
                                        .selected_text(Self::compat_flag_switch_name(
                                            locale, orig_value,
                                        ))
                                        .show_ui(ui, |ui| {
                                            let values = [None, Some(true), Some(false)];
                                            for value in values {
                                                ui.selectable_value(
                                                    &mut current_value,
                                                    value,
                                                    Self::compat_flag_switch_name(locale, value),
                                                );
                                            }
                                        });
                                    if current_value != orig_value {
                                        match current_value {
                                            Some(enabled) => self.flags.set(flag, enabled),
                                            None => self.flags.reset(flag),
                                        }
                                        self.flags_changed = true;
                                    }
                                }
                            });
                            row.col(|ui| {
                                if flag.definition().default_value {
                                    ui.weak(text(locale, "enable"));
                                } else {
                                    ui.weak(text(locale, "disable"));
                                }
                            });
                            row.col(|ui| {
                                if flag.definition().flash_player_value {
                                    ui.weak(text(locale, "enable"));
                                } else {
                                    ui.weak(text(locale, "disable"));
                                }
                            });
                        });
                    }
                });
        });
    }

    fn compat_flag_switch_name(locale: &LanguageIdentifier, enabled: Option<bool>) -> Cow<'_, str> {
        match enabled {
            None => text(locale, "preferences-panel-flag-default-value"),
            Some(true) => text(locale, "enable"),
            Some(false) => text(locale, "disable"),
        }
    }
}
