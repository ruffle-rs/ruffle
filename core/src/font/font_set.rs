use gc_arena::{Collect, Gc, Mutation};
use swf::Twips;

use crate::font::{Font, FontLike, FontMetrics, FontType, GlyphResolution};

/// Font set contains a set of fonts used to render text.
///
/// It always contains at least one font—the main font. It may also contain
/// fallback fonts, which will be used in case glyphs are missing from the main
/// font. Fallback fonts are always used in order.
///
/// TODO [KJ] We don't know what's the exact behavior when data like kerning,
///   leading, etc. does not match between main and fallback fonts.
#[derive(Debug, Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct FontSet<'gc>(Gc<'gc, FontSetData<'gc>>);

#[derive(Debug, Collect)]
#[collect(no_drop)]
struct FontSetData<'gc> {
    main_font: Font<'gc>,
    fallback_fonts: Vec<Font<'gc>>,
}

impl<'gc> FontSet<'gc> {
    /// Creates a font set from a sorted list of fonts.
    ///
    /// The first font is the main font, the rest are fallbacks.
    ///
    /// Returns None when the list is empty.
    pub fn from_fonts(mc: &Mutation<'gc>, fonts: &[Font<'gc>]) -> Option<Self> {
        let (&main_font, fallback_fonts) = fonts.split_first()?;
        Some(Self(Gc::new(
            mc,
            FontSetData {
                main_font,
                fallback_fonts: fallback_fonts.to_vec(),
            },
        )))
    }

    /// Creates a font set from one font only.
    pub fn from_one_font(mc: &Mutation<'gc>, font: Font<'gc>) -> Self {
        Self(Gc::new(
            mc,
            FontSetData {
                main_font: font,
                fallback_fonts: vec![],
            },
        ))
    }

    pub fn main_font(self) -> Font<'gc> {
        self.0.main_font
    }

    pub fn fallback_fonts(&self) -> &[Font<'gc>] {
        &self.0.fallback_fonts
    }
}

impl<'gc> FontLike<'gc> for FontSet<'gc> {
    fn resolve_glyph(&self, c: char, height: Twips) -> Option<GlyphResolution<'_, 'gc>> {
        if let Some(glyph) = self.0.main_font.get_glyph_for_char(c, height) {
            return Some(GlyphResolution::new(glyph, self.0.main_font));
        }

        for fallback_font in &self.0.fallback_fonts {
            if let Some(glyph) = fallback_font.get_glyph_for_char(c, height) {
                return Some(GlyphResolution::new(glyph, *fallback_font));
            }
        }

        None
    }

    fn has_kerning_info(&self) -> bool {
        self.0.main_font.has_kerning_info()
    }

    fn get_kerning_offset(&self, left: char, right: char, height: Twips) -> Twips {
        self.0.main_font.get_kerning_offset(left, right, height)
    }

    fn metrics(&self) -> FontMetrics {
        self.0.main_font.metrics()
    }

    fn metrics_at(&self, height: Twips) -> FontMetrics {
        self.0.main_font.metrics_at(height)
    }

    fn typo_metrics_at(&self, height: Twips) -> Option<FontMetrics> {
        self.0.main_font.typo_metrics_at(height)
    }

    fn scale(&self) -> f32 {
        self.0.main_font.scale()
    }

    fn font_type(&self) -> FontType {
        self.0.main_font.font_type()
    }
}
