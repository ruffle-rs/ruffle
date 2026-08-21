use crate::font::{Font, FontLike};
use egui::{CollapsingHeader, Grid, Ui, Window};

#[derive(Debug, Default)]
pub struct FontWindow {}

impl FontWindow {
    pub fn show<'gc>(&mut self, egui_ctx: &egui::Context, font: Font<'gc>) -> bool {
        let mut keep_open = true;

        Window::new(format!("Font {:p}", font.as_ptr()))
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                self.show_font_info(ui, font);
                self.show_font_metrics(ui, font);
            });

        keep_open
    }

    fn show_font_info(&self, ui: &mut Ui, font: Font<'_>) {
        Grid::new(ui.id().with("font-info"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                let desc = font.descriptor();

                ui.label("Name");
                ui.label(desc.name());
                ui.end_row();

                ui.label("Font Type");
                ui.label(format!("{:?}", font.font_type()));
                ui.end_row();

                ui.label("Bold");
                ui.label(if desc.bold() { "Yes" } else { "No" });
                ui.end_row();

                ui.label("Italic");
                ui.label(if desc.italic() { "Yes" } else { "No" });
                ui.end_row();

                ui.label("Has Layout");
                ui.label(if font.has_layout() { "Yes" } else { "No" });
                ui.end_row();

                ui.label("Has Glyphs");
                ui.label(if font.has_glyphs() { "Yes" } else { "No" });
                ui.end_row();

                ui.label("Has Kerning Info");
                ui.label(if font.has_kerning_info() { "Yes" } else { "No" });
                ui.end_row();
            });
    }

    fn show_font_metrics(&self, ui: &mut Ui, font: Font<'_>) {
        CollapsingHeader::new("Font Metrics")
            .id_salt(ui.id().with("metrics"))
            .show(ui, |ui| {
                let metrics = font.metrics();

                Grid::new(ui.id().with("font-metrics"))
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Scale");
                        ui.label(format!("{:.2}", metrics.scale));
                        ui.end_row();

                        ui.label("Ascent");
                        ui.label(format!("{}", metrics.ascent));
                        ui.end_row();

                        ui.label("Descent");
                        ui.label(format!("{}", metrics.descent));
                        ui.end_row();

                        ui.label("Leading");
                        ui.label(format!("{}", metrics.leading));
                        ui.end_row();
                    });
            });
    }
}
