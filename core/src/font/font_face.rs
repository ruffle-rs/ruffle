use crate::drawing::Drawing;
use crate::font::{FontMetrics, Glyph};
use crate::prelude::*;
use ruffle_render::shape_utils::{DrawCommand, FillRule};

use std::cell::OnceCell;
use std::sync::Arc;
use swf::FillStyle;

struct GlyphToDrawing<'a>(&'a mut Drawing);

/// Convert from a TTF outline, to a flash Drawing.
///
/// Note that the Y axis is flipped. I do not know why, but Flash does this.
impl ttf_parser::OutlineBuilder for GlyphToDrawing<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.draw_command(DrawCommand::MoveTo(Point::new(
            Twips::new(x as i32),
            Twips::new(-y as i32),
        )));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.draw_command(DrawCommand::LineTo(Point::new(
            Twips::new(x as i32),
            Twips::new(-y as i32),
        )));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.draw_command(DrawCommand::QuadraticCurveTo {
            control: Point::new(Twips::new(x1 as i32), Twips::new(-y1 as i32)),
            anchor: Point::new(Twips::new(x as i32), Twips::new(-y as i32)),
        });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.draw_command(DrawCommand::CubicCurveTo {
            control_a: Point::new(Twips::new(x1 as i32), Twips::new(-y1 as i32)),
            control_b: Point::new(Twips::new(x2 as i32), Twips::new(-y2 as i32)),
            anchor: Point::new(Twips::new(x as i32), Twips::new(-y as i32)),
        });
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}

pub struct FontFileData(Arc<dyn AsRef<[u8]>>);

impl FontFileData {
    pub fn new(data: impl AsRef<[u8]> + 'static) -> Self {
        Self(Arc::new(data))
    }

    pub fn new_shared(data: Arc<dyn AsRef<[u8]>>) -> Self {
        Self(data)
    }
}

impl std::ops::Deref for FontFileData {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

impl std::fmt::Debug for FontFileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FontFileData").field(&"<data>").finish()
    }
}

/// Represents a raw font file (ie .ttf).
///
/// This should be shared and reused where possible, and it's reparsed every
/// time a new glyph is required.
///
/// Parsing of a font is near-free (according to [ttf_parser::Face::parse]),
/// but the storage isn't.
///
/// Font files may contain multiple individual font faces, but those font faces
/// may reuse the same Glyph from the same file. For this reason, glyphs are
/// reused where possible.
#[derive(Debug)]
pub struct FontFace {
    data: FontFileData,
    glyphs: Vec<OnceCell<Option<Glyph>>>,
    font_index: u32,

    ascender: i32,
    descender: i32,
    leading: i16,
    /// Typographic ascent/descent from the OS/2 table (`sTypoAscender` /
    /// `sTypoDescender`), if present. Flash Player sizes Flash Text Engine and
    /// Spark text with these, unlike the hhea `ascender`/`descender` above.
    typo_ascender: Option<i32>,
    typo_descender: Option<i32>,
    typo_leading: i16,
    scale: f32,
    might_have_kerning: bool,
}

impl FontFace {
    pub fn new(data: FontFileData, font_index: u32) -> Result<Self, ttf_parser::FaceParsingError> {
        // TODO: Support font collections

        // We validate that the font is good here, so we can just `.expect()` it later
        let face = ttf_parser::Face::parse(&data, font_index)?;

        let ascender = face.ascender() as i32;
        let descender = -face.descender() as i32;
        let leading = face.line_gap();

        // Typographic metrics live in the OS/2 table. Some fonts ship it
        // zeroed, so treat an all-zero sTypo pair as absent.
        let typo = face.tables().os2.and_then(|os2| {
            let a = os2.typographic_ascender() as i32;
            let d = -os2.typographic_descender() as i32;
            if a == 0 && d == 0 {
                None
            } else {
                Some((a, d, os2.typographic_line_gap()))
            }
        });
        let (typo_ascender, typo_descender, typo_leading) = match typo {
            Some((a, d, l)) => (Some(a), Some(d), l),
            None => (None, None, 0),
        };

        let scale = face.units_per_em() as f32;
        let glyphs = vec![OnceCell::new(); face.number_of_glyphs() as usize];

        // [NA] TODO: This is technically correct for just Kerning, but in practice kerning comes in many forms.
        // We need to support GPOS to do better at this, but that's a bigger change to font rendering as a whole.
        let might_have_kerning = face
            .tables()
            .kern
            .map(|k| {
                k.subtables
                    .into_iter()
                    .any(|sub| sub.horizontal && !sub.has_state_machine)
            })
            .unwrap_or_default();

        Ok(Self {
            data,
            font_index,
            glyphs,
            ascender,
            descender,
            leading,
            typo_ascender,
            typo_descender,
            typo_leading,
            scale,
            might_have_kerning,
        })
    }

    pub fn font_index(&self) -> u32 {
        self.font_index
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn metrics(&self) -> FontMetrics {
        FontMetrics {
            scale: self.scale,
            ascent: self.ascender,
            descent: self.descender,
            leading: self.leading,
        }
    }

    /// Typographic metrics (OS/2 `sTypo*`), if this font provides them. Flash
    /// Player sizes Flash Text Engine / Spark text with these, unlike the
    /// hhea/cell metrics returned by [`FontFace::metrics`].
    pub fn typo_metrics(&self) -> Option<FontMetrics> {
        Some(FontMetrics {
            scale: self.scale,
            ascent: self.typo_ascender?,
            descent: self.typo_descender?,
            leading: self.typo_leading,
        })
    }

    pub fn get_glyph(&self, character: char) -> Option<&Glyph> {
        let face = ttf_parser::Face::parse(&self.data, self.font_index)
            .expect("Font was already checked to be valid");
        if let Some(glyph_id) = face.glyph_index(character) {
            return self.glyphs[glyph_id.0 as usize]
                .get_or_init(|| {
                    let mut drawing = Drawing::new();
                    // TTF uses NonZero
                    drawing.new_fill(
                        Some(FillStyle::Color(Color::WHITE)),
                        Some(FillRule::NonZero),
                    );
                    if face
                        .outline_glyph(glyph_id, &mut GlyphToDrawing(&mut drawing))
                        .is_some()
                    {
                        let advance = face.glyph_hor_advance(glyph_id).map_or_else(
                            || drawing.self_bounds(true).width(),
                            |a| Twips::new(a as i32),
                        );
                        Some(Glyph::from_drawing(character, advance, drawing))
                    } else {
                        let advance = Twips::new(face.glyph_hor_advance(glyph_id)? as i32);
                        // If we have advance, then this is either an image, SVG or simply missing (ie whitespace)
                        Some(Glyph::whitespace(character, advance))
                    }
                })
                .as_ref();
        }
        None
    }

    pub fn has_kerning_info(&self) -> bool {
        self.might_have_kerning
    }

    pub fn get_kerning_offset(&self, left: char, right: char) -> Twips {
        let face = ttf_parser::Face::parse(&self.data, self.font_index)
            .expect("Font was already checked to be valid");

        if let Some(kern) = face.tables().kern
            && let (Some(left_glyph), Some(right_glyph)) =
                (face.glyph_index(left), face.glyph_index(right))
        {
            for subtable in kern.subtables {
                if subtable.horizontal
                    && let Some(value) = subtable.glyphs_kerning(left_glyph, right_glyph)
                {
                    return Twips::new(value as i32);
                }
            }
        }

        Twips::ZERO
    }
}
