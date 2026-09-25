use crate::font::glyph::GlyphRef;
use crate::font::{Font, FontMetrics, FontType, Glyph};
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

    /// The color to render the font with.
    pub color: swf::Color,
}

impl EvalParameters {
    /// Convert the formatting on a text span over to font evaluation
    /// parameters.
    pub fn from_span(span: &TextSpan) -> Self {
        let color = swf::Color::from_rgb(span.font.color.to_rgb(), 0xFF);
        Self {
            height: Twips::from_pixels(span.font.size),
            letter_spacing: Twips::from_pixels(span.font.letter_spacing),
            kerning: span.font.kerning,
            color,
        }
    }

    /// Get the height that the font would be evaluated at.
    pub fn height(&self) -> Twips {
        self.height
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
    /// Resolve a glyph for a char.
    ///
    /// The resolution contains information about the glyph and the font that
    /// provided the glyph.
    fn resolve_glyph(&self, c: char) -> Option<GlyphResolution<'_, 'gc>>;

    /// Returns whether this font contains kerning information.
    fn has_kerning_info(&self) -> bool;

    /// Given a pair of characters, applies the offset that should be applied
    /// to the advance value between these two characters.
    /// Returns 0 twips if no kerning offset exists between these two characters.
    fn get_kerning_offset(&self, left: char, right: char) -> Twips;

    fn metrics(&self) -> FontMetrics;

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
        params: EvalParameters,
        mut glyph_func: impl FnMut(usize, &Transform, GlyphRef, Twips, Twips),
    ) {
        let mut transform: Transform = Default::default();
        transform.color_transform.set_mult_color(params.color);

        let baseline = self.metrics().ascent(params.height);

        // TODO [KJ] I'm not sure whether we should iterate over characters here or over code units.
        //   I suspect Flash Player does not support full UTF-16 when displaying and laying out text.
        let mut char_indices = text
            .char_indices()
            .map(|(pos, c)| (pos, c.unwrap_or(char::REPLACEMENT_CHARACTER)))
            .peekable();

        let kerning_enabled =
            self.has_kerning_info() && (self.font_type().is_device() || params.kerning);

        let mut x = Twips::ZERO;
        while let Some((pos, c)) = char_indices.next() {
            if let Some(resolution) = self.resolve_glyph(c) {
                let glyph = resolution.glyph;
                let scale = params.height.get() as f32 / resolution.font.scale();
                let mut advance = glyph.advance();
                if kerning_enabled {
                    let next_char = char_indices.peek().map(|(_, ch)| *ch);
                    let kerning = next_char
                        .map(|ch| self.get_kerning_offset(c, ch))
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

                glyph_func(pos, &transform, glyph, twips_advance, x);

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

        self.evaluate(text, params, |_pos, _transform, _glyph, advance, x| {
            width = width.max(x + advance);
        });

        width
    }
}

fn round_to_pixel(t: Twips) -> Twips {
    Twips::from_pixels(t.to_pixels().round())
}
