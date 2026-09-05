use crate::font::glyph::GlyphRef;
use crate::font::{Font, FontMetrics, FontType, Glyph, twips_to_px_for_cache};
use crate::html::TextSpan;
use crate::prelude::*;
use crate::string::WStr;
use ruffle_render::transform::Transform;

/// Parameters necessary to evaluate a font.
#[derive(Copy, Clone, Debug)]
pub struct EvalParameters {
    /// The height of each glyph, equivalent to a font size.
    pub height: Twips,

    /// Additional letter spacing to be added to or removed from each glyph
    /// after normal or kerned glyph advances are applied.
    pub letter_spacing: Twips,

    /// Whether to allow use of font-provided kerning metrics.
    ///
    /// Fonts can optionally add or remove additional spacing between specific
    /// pairs of letters, separate from the ordinary width between glyphs. This
    /// parameter allows enabling or disabling that feature.
    pub kerning: bool,

    /// Ratio between device pixels and stage pixels at draw time (viewport
    /// DPI scale, page zoom, uniform ancestor scaling). Rendering callers may
    /// set it so size-aware renderers rasterize presentation bitmaps at the
    /// effective on-screen size; it never affects layout, which always
    /// measures at `height` with the neutral `1.0`.
    pub display_scale: f32,
}

impl EvalParameters {
    /// Convert the formatting on a text span over to font evaluation
    /// parameters.
    pub fn from_span(span: &TextSpan) -> Self {
        Self {
            height: Twips::from_pixels(span.font.size),
            letter_spacing: Twips::from_pixels(span.font.letter_spacing),
            kerning: span.font.kerning,
            display_scale: 1.0,
        }
    }

    /// Get the height that the font would be evaluated at.
    pub fn height(&self) -> Twips {
        self.height
    }

    /// Returns a copy tuned for rendering under the given display scale
    /// (device pixels per stage pixel). Layout callers keep the neutral
    /// default.
    pub fn with_display_scale(mut self, display_scale: f32) -> Self {
        self.display_scale = display_scale;
        self
    }
}

pub struct GlyphResolution<'a, 'gc> {
    pub glyph: GlyphRef<'a>,
    pub font: Font<'gc>,
}

impl<'a, 'gc> GlyphResolution<'a, 'gc> {
    pub fn new(glyph: GlyphRef<'a>, font: Font<'gc>) -> Self {
        Self { glyph, font }
    }
}

pub trait FontLike<'gc> {
    /// Resolve a glyph for a char at a particular requested height in twips.
    ///
    /// The resolution contains information about the glyph and the font that
    /// provided the glyph. `height` lets size-aware renderers (e.g. GDI)
    /// rasterize at the matching pixel size. Pass `Twips::ZERO` if you don't
    /// know or care about the size — for SWF/embedded fonts the value is
    /// ignored anyway.
    fn resolve_glyph(&self, c: char, height: Twips) -> Option<GlyphResolution<'_, 'gc>>;

    /// Returns whether this font contains kerning information.
    fn has_kerning_info(&self) -> bool;

    /// Given a pair of characters, applies the offset that should be applied
    /// to the advance value between these two characters.
    /// Returns 0 twips if no kerning offset exists between these two characters.
    /// `height` follows the same convention as `resolve_glyph`.
    fn get_kerning_offset(&self, left: char, right: char, height: Twips) -> Twips;

    fn metrics(&self) -> FontMetrics;

    /// Font-wide metrics tuned to a specific requested glyph height.
    ///
    /// For size-aware renderers (e.g. GDI) the returned metrics are exact at
    /// the matching raster size — whole device pixels, consistent with the
    /// glyph bitmaps — so line heights and the text measurements reported to
    /// ActionScript land on the pixel grid. Everywhere else this is the same
    /// as `metrics()`.
    fn metrics_at(&self, _height: Twips) -> FontMetrics {
        self.metrics()
    }

    /// Typographic metrics for the Flash Text Engine (see
    /// [`crate::font::FontRenderer::get_typo_font_metrics`]). `None` (the
    /// default) means the font has no separate typographic metrics and callers
    /// use `metrics_at`.
    fn typo_metrics_at(&self, _height: Twips) -> Option<FontMetrics> {
        None
    }

    fn scale(&self) -> f32;

    fn font_type(&self) -> FontType;

    /// Evaluate this font against a particular string on a glyph-by-glyph
    /// basis.
    ///
    /// This function takes the text string to evaluate against, the base
    /// transform to start from, the height of each glyph, and produces a list
    /// of transforms and glyphs which will be consumed by the `glyph_func`
    /// closure. This corresponds to the series of drawing operations necessary
    /// to render the text on a single horizontal line.
    ///
    /// It's guaranteed that this function will iterate over all characters
    /// from the text, irrespectively of whether they have a glyph or not.
    fn evaluate(
        &self,
        text: &WStr, // TODO: take an `IntoIterator<Item=char>`, to not depend on string representation?
        mut transform: Transform,
        params: EvalParameters,
        mut glyph_func: impl FnMut(usize, &Transform, GlyphRef, Twips, Twips),
    ) {
        let baseline = self.metrics_at(params.height).ascent(params.height);

        // TODO [KJ] I'm not sure whether we should iterate over characters here or over code units.
        //   I suspect Flash Player does not support full UTF-16 when displaying and laying out text.
        let mut char_indices = text
            .char_indices()
            .map(|(pos, c)| (pos, c.unwrap_or(char::REPLACEMENT_CHARACTER)))
            .peekable();

        let kerning_enabled =
            self.has_kerning_info() && (self.font_type().is_device() || params.kerning);

        let mut x = Twips::ZERO;
        // Accumulated difference between the presentation pen (device-
        // resolution raster advances, matching Flash Player's spacing under
        // zoom) and the layout pen. Affects drawn pixels only; the layout
        // pen below stays strictly logical.
        let mut presentation_drift = Twips::ZERO;
        while let Some((pos, c)) = char_indices.next() {
            if let Some(resolution) = self.resolve_glyph(c, params.height) {
                let font = resolution.font;
                let glyph = resolution.glyph;
                // Pixel-locked glyphs (e.g. GDI rasterized at the requested
                // size) carry their own intrinsic em-scale; using it instead
                // of the font's canonical scale collapses the layout-time
                // scale to 1.0 so the bitmap is drawn 1:1 to display pixels.
                let glyph_scale_basis = glyph.intrinsic_scale().unwrap_or_else(|| font.scale());
                let scale = params.height.get() as f32 / glyph_scale_basis;
                let pixel_locked = glyph.intrinsic_scale().is_some();
                let glyph_advance = glyph.advance();
                let mut advance = glyph_advance;
                if kerning_enabled {
                    let next_char = char_indices.peek().map(|(_, ch)| *ch);
                    let kerning = next_char
                        .map(|ch| self.get_kerning_offset(c, ch, params.height))
                        .unwrap_or_default();
                    advance += kerning;
                }
                let twips_advance = if self.font_type() == FontType::Device {
                    let unspaced_advance =
                        round_to_pixel(Twips::new((advance.get() as f32 * scale) as i32));
                    let spaced_advance =
                        unspaced_advance + params.letter_spacing.round_to_pixel_ties_even();
                    if spaced_advance > Twips::ZERO {
                        spaced_advance
                    } else {
                        unspaced_advance
                    }
                } else {
                    Twips::new((advance.get() as f32 * scale) as i32) + params.letter_spacing
                };

                transform.matrix.a = scale;
                transform.matrix.d = scale;
                transform.matrix.ty = if glyph.rendered_at_baseline() {
                    baseline
                } else {
                    Twips::ZERO
                };

                // Under a scaling display transform (page zoom, viewport DPI)
                // a pixel-locked glyph would be GPU-resampled from its
                // logical-size raster and turn blurry. Swap in a presentation
                // bitmap rasterized at the effective on-screen size, drawn
                // with a compensating matrix, so layout geometry (advances,
                // line breaks, caret) is untouched while the pixels map 1:1
                // to the device grid.
                let raster_px =
                    presentation_raster_px(&params, pixel_locked && font.has_size_aware_renderer());

                if let Some(raster_px) = raster_px {
                    // Release the logical glyph's cache borrow before
                    // resolving another size on the same font.
                    drop(glyph);
                    let raster_height = Twips::new(raster_px as i32 * 20);
                    let mut presented = false;
                    if let Some(raster_resolution) = font.resolve_glyph(c, raster_height) {
                        let raster_glyph = raster_resolution.glyph;
                        if raster_glyph.intrinsic_scale().is_some() {
                            // Map the raster 1:1 to device pixels, exactly
                            // like Flash Player: at fractional zooms the glyph
                            // may run a fraction of a pixel larger or smaller
                            // than the ideal size, but it is never resampled.
                            // The ~1.0 device scale also re-engages
                            // PixelSnapping::Auto, so positions land on the
                            // device grid.
                            let draw_scale = 1.0 / params.display_scale;
                            // Per-size integer metrics don't scale exactly
                            // linearly: align the raster bitmap's baseline
                            // with the logical baseline layout positioned.
                            let logical_ascent =
                                font.metrics_at(params.height).ascent(params.height);
                            let raster_ascent =
                                font.metrics_at(raster_height).ascent(raster_height);
                            // Device-resolution advance of the raster glyph,
                            // expressed in stage twips.
                            let raster_advance = Twips::new(
                                (raster_glyph.advance().get() as f32 * draw_scale) as i32,
                            );
                            let mut draw_transform = transform.clone();
                            draw_transform.matrix.a = draw_scale;
                            draw_transform.matrix.d = draw_scale;
                            draw_transform.matrix.tx += presentation_drift;
                            draw_transform.matrix.ty = logical_ascent
                                - Twips::new((raster_ascent.get() as f32 * draw_scale) as i32);
                            glyph_func(pos, &draw_transform, raster_glyph, twips_advance, x);
                            // Device-resolution advances differ slightly from
                            // the scaled logical ones (integer hinting at each
                            // size): track the difference so glyph spacing
                            // matches Flash Player while the layout pen stays
                            // logical.
                            presentation_drift += raster_advance
                                - round_to_pixel(Twips::new(
                                    (glyph_advance.get() as f32 * scale) as i32,
                                ));
                            presented = true;
                        }
                    }
                    if !presented && let Some(fallback) = self.resolve_glyph(c, params.height) {
                        // Keep mid-run continuity with the presentation pen.
                        let mut draw_transform = transform.clone();
                        draw_transform.matrix.tx += presentation_drift;
                        glyph_func(pos, &draw_transform, fallback.glyph, twips_advance, x);
                    }
                } else {
                    glyph_func(pos, &transform, glyph, twips_advance, x);
                }

                // Step horizontally.
                transform.matrix.tx += twips_advance;
                x += twips_advance;
            } else {
                // No glyph, zero advance.  This makes it possible to use this method for purposes
                // other than rendering the font, e.g. measurement, iterating over characters.
                glyph_func(pos, &transform, Glyph::empty(c).as_ref(), Twips::ZERO, x);
            }
        }
    }

    /// Measure a particular string's width.
    fn measure(&self, text: &WStr, params: EvalParameters) -> Twips {
        let mut width = Twips::ZERO;

        self.evaluate(
            text,
            Default::default(),
            params,
            |_pos, _transform, _glyph, advance, x| {
                width = width.max(x + advance);
            },
        );

        width
    }
}

fn round_to_pixel(t: Twips) -> Twips {
    Twips::from_pixels(t.to_pixels().round())
}

/// Pick the raster size, in pixels, for the presentation bitmap of a glyph
/// drawn under a scaling display transform. Returns `None` when the ordinary
/// path is already right (neutral scale, non-size-aware source, or unknown
/// size). The raster may round to the logical size at near-1 scales: the
/// presentation path still applies, drawing it 1:1 to device pixels instead
/// of resampling it.
fn presentation_raster_px(params: &EvalParameters, size_aware: bool) -> Option<u32> {
    let scale = params.display_scale;
    if !size_aware || !scale.is_finite() || scale <= 0.0 || (scale - 1.0).abs() < 0.01 {
        return None;
    }
    let logical_px = twips_to_px_for_cache(params.height);
    if logical_px == 0 {
        return None;
    }
    // Cap the raster size so degenerate transforms can't conjure huge rasters.
    Some(((logical_px as f32 * scale).round() as u32).clamp(1, 1024))
}
