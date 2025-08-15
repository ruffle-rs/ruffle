use crate::html::{StyleSheet, TextFormat};
use egui::{CollapsingHeader, Grid, Ui};

pub fn show_text_format(ui: &mut Ui, tf: &TextFormat, skip_none: bool) {
    Grid::new(ui.id().with("text_format_table"))
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            for (key, value) in [
                ("Font Face", tf.font.as_ref().map(|v| v.to_string())),
                ("Font Size", tf.size.map(|v| v.to_string())),
                ("Color", tf.color.map(|v| format!("{v:?}"))),
                ("Align", tf.align.map(|v| format!("{v:?}"))),
                ("Bold?", tf.bold.map(|v| v.to_string())),
                ("Italic?", tf.italic.map(|v| v.to_string())),
                ("Underline?", tf.underline.map(|v| v.to_string())),
                ("Left Margin", tf.left_margin.map(|v| v.to_string())),
                ("Right Margin", tf.right_margin.map(|v| v.to_string())),
                ("Indent", tf.indent.map(|v| v.to_string())),
                ("Block Indent", tf.block_indent.map(|v| v.to_string())),
                ("Kerning?", tf.kerning.map(|v| v.to_string())),
                ("Leading", tf.leading.map(|v| v.to_string())),
                ("Letter Spacing", tf.letter_spacing.map(|v| v.to_string())),
                ("Tab Stops", tf.tab_stops.as_ref().map(|v| format!("{v:?}"))),
                ("Bullet?", tf.bullet.map(|v| v.to_string())),
                ("URL", tf.url.as_ref().map(|v| v.to_string())),
                ("Target", tf.target.as_ref().map(|v| v.to_string())),
                ("Display", tf.display.map(|v| format!("{v:?}"))),
            ] {
                if skip_none && value.is_none() {
                    continue;
                }

                ui.label(key);
                if let Some(value) = value {
                    if !value.is_empty() {
                        ui.label(value);
                    } else {
                        ui.weak("Empty");
                    }
                } else {
                    ui.weak("None");
                }
                ui.end_row();
            }
        });
}

pub fn show_style_sheet(ui: &mut Ui, style_sheet: StyleSheet<'_>) {
    let mut selectors = style_sheet.selectors();
    selectors.sort();
    for selector in selectors {
        CollapsingHeader::new(selector.to_utf8_lossy())
            .id_salt(ui.id().with(selector.to_utf8_lossy()))
            .show(ui, |ui| {
                if let Some(tf) = style_sheet.get_style(&selector) {
                    show_text_format(ui, &tf, true);
                } else {
                    ui.weak("No styles");
                }
            });
    }
}
