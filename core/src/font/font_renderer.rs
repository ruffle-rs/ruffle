use swf::Twips;

use crate::font::{FontAtlases, FontMetrics, Glyph};

pub trait FontRenderer: std::fmt::Debug {
    fn scale(&self) -> f32;

    fn get_font_metrics(&self) -> FontMetrics;

    fn has_kerning_info(&self) -> bool;

    fn render_glyph(&self, character: char) -> Option<Glyph>;

    fn calculate_kerning(&self, left: char, right: char) -> Twips;

    fn atlases(&self) -> Option<&FontAtlases> {
        None
    }

    /// Like `render_glyph`, but rasterizes the glyph at exactly the requested
    /// pixel size. Pixel-locked renderers (e.g. GDI) override this and produce
    /// glyphs whose bitmaps map 1:1 to display pixels at that size, avoiding
    /// the blur introduced by scaling a single canonical-size raster up or
    /// down. The returned `Glyph` should set its `intrinsic_scale` so that the
    /// layout uses scale=1.0 at the requested size.
    ///
    /// Default delegates to size-agnostic `render_glyph`; size-unaware
    /// renderers (e.g. canvas) keep their existing single-cache behavior.
    fn render_glyph_at_size(&self, character: char, _height_px: u32) -> Option<Glyph> {
        self.render_glyph(character)
    }

    /// Like `calculate_kerning` but for a specific raster size. Defaults to
    /// the size-agnostic version.
    fn calculate_kerning_at_size(&self, left: char, right: char, _height_px: u32) -> Twips {
        self.calculate_kerning(left, right)
    }

    /// Font-wide metrics measured at a specific raster size, in twips, with
    /// `scale` equal to that size so the layout consumes them 1:1.
    ///
    /// Size-locked renderers (e.g. GDI) return metrics that are exact at the
    /// requested pixel size — whole device pixels, matching the glyph
    /// bitmaps — so every layout position derived from them (line heights,
    /// baselines, the text measurements reported to ActionScript) lands on
    /// the pixel grid, like Flash Player device fonts did on Windows.
    ///
    /// `None` (the default) means the renderer has no per-size metrics: the
    /// consumer falls back to `get_font_metrics`, i.e. canonical-size
    /// metrics scaled linearly by the caller.
    fn get_font_metrics_at_size(&self, _height_px: u32) -> Option<FontMetrics> {
        None
    }

    /// Typographic font metrics (OS/2 `sTypoAscender`/`sTypoDescender`) at a
    /// raster size, in twips. The Flash Text Engine reports these to
    /// ActionScript for measuring text, matching Flash Player; classic text
    /// fields keep the taller cell metrics from `get_font_metrics_at_size`.
    /// `None` (the default) means the renderer exposes no typographic metrics
    /// and FTE falls back to the cell metrics.
    fn get_typo_font_metrics(&self, _height_px: u32) -> Option<FontMetrics> {
        None
    }

    /// Whether `render_glyph_at_size` produces a different result for each
    /// size (and therefore deserves a separate cache entry per size).
    /// Defaults to `false`: the consumer caches one entry per code point.
    fn is_size_aware(&self) -> bool {
        false
    }
}
