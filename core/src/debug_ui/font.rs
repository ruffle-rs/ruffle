use crate::debug_ui::{ItemToSave, Message};
use crate::font::{Font, FontFace, FontLike, FontRenderer, Glyph, GlyphSource};
use egui::{CollapsingHeader, Grid, TextEdit, Ui, Window};
use fnv::FnvHashMap;
use std::cell::RefCell;
use swf::Twips;

#[derive(Debug, Default)]
pub struct FontWindow {
    glyph_info_text: String,
}

impl FontWindow {
    pub fn show<'gc>(
        &mut self,
        egui_ctx: &egui::Context,
        font: Font<'gc>,
        messages: &mut Vec<Message>,
    ) -> bool {
        let mut keep_open = true;

        Window::new(format!("Font {:p}", font.as_ptr()))
            .open(&mut keep_open)
            .scroll([true, true])
            .show(egui_ctx, |ui| {
                self.show_font_info(ui, font);
                self.show_font_metrics(ui, font);
                self.show_glyph_info(ui, font);
                self.show_glyph_source(ui, font, messages);
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

    fn show_glyph_info(&mut self, ui: &mut Ui, font: Font<'_>) {
        CollapsingHeader::new("Glyph Info")
            .id_salt(ui.id().with("glyph-info"))
            .show(ui, |ui| {
                ui.add(
                    TextEdit::singleline(&mut self.glyph_info_text)
                        .hint_text("Type characters to inspect..."),
                );

                let chars: Vec<char> = self.glyph_info_text.chars().collect();
                if chars.is_empty() {
                    return;
                }

                Grid::new(ui.id().with("glyph-info-table"))
                    .num_columns(4)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Char");
                        ui.label("Has Glyph");
                        ui.label("Advance");
                        ui.label("Kerning to Next");
                        ui.end_row();

                        for (i, &ch) in chars.iter().enumerate() {
                            ui.label(format!("{ch}"));

                            let resolution = font.resolve_glyph(ch);
                            ui.label(if resolution.is_some() { "Yes" } else { "No" });

                            if let Some(resolution) = &resolution {
                                ui.label(format!("{}", resolution.glyph.advance()));
                            } else {
                                ui.weak("-");
                            }

                            if let Some(&next) = chars.get(i + 1) {
                                ui.label(format!("{}", font.get_kerning_offset(ch, next)));
                            } else {
                                ui.weak("-");
                            }

                            ui.end_row();
                        }
                    });
            });
    }

    fn show_glyph_source(&self, ui: &mut Ui, font: Font<'_>, messages: &mut Vec<Message>) {
        let source = font.glyph_source();
        let kind = match source {
            GlyphSource::Memory { .. } => "Memory (embedded shapes)",
            GlyphSource::FontFace { .. } => "Font Face (TTF)",
            GlyphSource::ExternalRenderer { .. } => "External Renderer",
            GlyphSource::Empty => "None",
        };

        CollapsingHeader::new(format!("Glyph Source: {kind}"))
            .id_salt(ui.id().with("glyph-source"))
            .show(ui, |ui| match source {
                GlyphSource::Memory {
                    glyphs,
                    kerning_pairs,
                    ..
                } => self.show_glyph_source_memory(ui, glyphs, kerning_pairs),
                GlyphSource::FontFace { face, .. } => {
                    self.show_glyph_source_font_face(ui, font, face, messages)
                }
                GlyphSource::ExternalRenderer {
                    glyph_cache,
                    kerning_cache,
                    font_renderer,
                } => self.show_glyph_source_external_renderer(
                    ui,
                    glyph_cache,
                    kerning_cache,
                    font_renderer.as_ref(),
                ),
                GlyphSource::Empty => {
                    ui.weak("This font has no glyphs.");
                }
            });
    }

    fn show_glyph_source_memory(
        &self,
        ui: &mut Ui,
        glyphs: &[Glyph],
        kerning_pairs: &FnvHashMap<(u16, u16), Twips>,
    ) {
        Grid::new(ui.id().with("glyph-source-table"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Glyph Count");
                ui.label(format!("{}", glyphs.len()));
                ui.end_row();

                ui.label("Kerning Pairs");
                ui.label(format!("{}", kerning_pairs.len()));
                ui.end_row();
            });
    }

    fn show_glyph_source_font_face(
        &self,
        ui: &mut Ui,
        font: Font<'_>,
        face: &FontFace,
        messages: &mut Vec<Message>,
    ) {
        Grid::new(ui.id().with("glyph-source-table"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Face Index");
                ui.label(format!("{}", face.font_index()));
                ui.end_row();
            });

        if ui.button("Save Font").clicked() {
            messages.push(Message::SaveFile(ItemToSave {
                suggested_name: format!("{}.ttf", font.descriptor().name()),
                data: face.data().to_vec(),
            }));
        }
    }

    fn show_glyph_source_external_renderer(
        &self,
        ui: &mut Ui,
        glyph_cache: &RefCell<FnvHashMap<u16, Option<Glyph>>>,
        kerning_cache: &RefCell<FnvHashMap<(u16, u16), Twips>>,
        font_renderer: &dyn FontRenderer,
    ) {
        Grid::new(ui.id().with("glyph-source-table"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Renderer");
                ui.label(format!("{font_renderer:?}"));
                ui.end_row();

                ui.label("Glyph Cache Size");
                ui.horizontal(|ui| {
                    ui.label(format!("{}", glyph_cache.borrow().len()));
                    if ui.button("Clear").clicked() {
                        glyph_cache.borrow_mut().clear();
                    }
                });
                ui.end_row();

                ui.label("Kerning Cache Size");
                ui.horizontal(|ui| {
                    ui.label(format!("{}", kerning_cache.borrow().len()));
                    if ui.button("Clear").clicked() {
                        kerning_cache.borrow_mut().clear();
                    }
                });
                ui.end_row();
            });
    }
}
