use swf::Twips;

use crate::font::{FontMetrics, Glyph};

pub trait FontRenderer: std::fmt::Debug {
    fn scale(&self) -> f32;

    fn get_font_metrics(&self) -> FontMetrics;

    fn has_kerning_info(&self) -> bool;

    fn render_glyph(&self, character: char) -> Option<Glyph>;

    fn calculate_kerning(&self, left: char, right: char) -> Twips;
}
