use freetype::bitmap::PixelMode;
use freetype::face::KerningMode;
use freetype::face::LoadFlag;
use std::ffi::OsStr;
use thiserror::Error;

use ruffle_core::font::FontMetrics;
use ruffle_core::font::FontRenderer;
use ruffle_core::font::Glyph;
use ruffle_core::swf::Twips;
use ruffle_render::bitmap::Bitmap;
use ruffle_render::bitmap::BitmapFormat;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Missing glyph: {0}")]
    MissingGlyph(char),

    #[error("FreeType error: {0}")]
    FreetypeError(#[from] freetype::Error),
}

#[derive(Debug)]
pub struct FreetypeFontRenderer {
    face: freetype::Face,
}

impl FreetypeFontRenderer {
    /// Render fonts with size 64px. It affects the bitmap size.
    const SIZE_PX: f64 = 64.0;

    /// Divide each pixel into 20 (use twips precision). It affects metrics.
    const SCALE: f64 = 20.0;

    pub fn new<P>(path: P, face_index: u32) -> Result<Self, Error>
    where
        P: AsRef<OsStr>,
    {
        let ft = freetype::Library::init()?;
        let face = ft.new_face(path, face_index as isize)?;
        face.set_char_size((Self::SIZE_PX * 64.0) as isize, 0, 0, 0)?;
        Ok(Self { face })
    }

    fn size_metrics(&self) -> freetype::ffi::FT_Size_Metrics {
        self.face
            .size_metrics()
            .expect("face should have a char size set")
    }

    fn ascent(&self) -> Twips {
        convert_26_6_to_twips(self.size_metrics().ascender)
    }

    fn descent(&self) -> Twips {
        // FreeType's `descender` is a signed distance from the baseline
        // (negative since it points below it); this returns the positive
        // magnitude expected by `FontMetrics::descent`.
        convert_26_6_to_twips(-self.size_metrics().descender)
    }

    fn render_glyph_internal(&self, character: char) -> Result<Glyph, Error> {
        let index = self
            .face
            .get_char_index(character as usize)
            .ok_or(Error::MissingGlyph(character))?;

        self.face
            .load_glyph(index, LoadFlag::NO_BITMAP | LoadFlag::RENDER)?;
        let glyph = self.face.glyph();
        let bitmap = glyph.bitmap();
        let advance = convert_26_6_to_twips(glyph.advance().x);
        let tx = Twips::from_pixels(glyph.bitmap_left() as f64);

        // `bitmap_top` is the distance from the baseline up to the top of
        // the bitmap. Glyph bitmaps are positioned relative to the top of
        // the line, not the baseline, so convert it into a downward offset
        // from the top of the line.
        let bitmap_top = Twips::from_pixels(glyph.bitmap_top() as f64);
        let ty = self.ascent() - bitmap_top;

        // Glyphs with no ink (e.g. space) have an empty bitmap, but a
        // zero-sized texture isn't allowed, so clamp to at least 1x1.
        let bitmap = Bitmap::new(
            (bitmap.width() as u32).max(1),
            (bitmap.rows() as u32).max(1),
            BitmapFormat::Rgba,
            convert_bitmap(&bitmap)?,
        );

        Ok(Glyph::from_bitmap(character, bitmap, advance, tx, ty))
    }

    fn calculate_kerning_internal(&self, left: char, right: char) -> Result<Twips, Error> {
        if !self.face.has_kerning() {
            return Ok(Twips::ZERO);
        }

        let left_index = self
            .face
            .get_char_index(left as usize)
            .ok_or(Error::MissingGlyph(left))?;
        let right_index = self
            .face
            .get_char_index(right as usize)
            .ok_or(Error::MissingGlyph(right))?;

        let kerning =
            self.face
                .get_kerning(left_index, right_index, KerningMode::KerningDefault)?;

        Ok(convert_26_6_to_twips(kerning.x))
    }
}

impl FontRenderer for FreetypeFontRenderer {
    fn scale(&self) -> f32 {
        (Self::SIZE_PX * Self::SCALE) as f32
    }

    fn get_font_metrics(&self) -> FontMetrics {
        FontMetrics {
            scale: self.scale(),
            ascent: self.ascent().get(),
            descent: self.descent().get(),
            leading: 0,
        }
    }

    fn has_kerning_info(&self) -> bool {
        self.face.has_kerning()
    }

    fn render_glyph(&self, character: char) -> Option<Glyph> {
        self.render_glyph_internal(character)
            .map_err(|err| tracing::error!("Failed to render a glyph: {err:?}"))
            .ok()
    }

    fn calculate_kerning(&self, left: char, right: char) -> Twips {
        self.calculate_kerning_internal(left, right)
            .map_err(|err| tracing::error!("Failed to calculate kerning: {err:?}"))
            .unwrap_or(Twips::ZERO)
    }
}

/// Converts a FreeType 26.6 fixed-point value (1/64th of a pixel) to Twips
/// (1/20th of a pixel), rounding to the nearest twip.
fn convert_26_6_to_twips(value_26_6: i64) -> Twips {
    let whole_pixels = (value_26_6 / 64) as i32;

    let fractional_twips = (value_26_6 % 64) as f64 / 64.0 * 20.0;
    let fractional_twips = fractional_twips.round_ties_even() as i32;

    Twips::new(fractional_twips + whole_pixels * 20)
}

fn convert_bitmap(bitmap: &freetype::Bitmap) -> Result<Vec<u8>, Error> {
    let pixel_mode = bitmap.pixel_mode()?;
    let buffer = bitmap.buffer();

    match pixel_mode {
        PixelMode::Gray => {
            let mut result = Vec::with_capacity(buffer.len() * 4);

            // TODO Add support for num_grays.
            for &pixel in buffer {
                result.push(pixel);
                result.push(pixel);
                result.push(pixel);
                result.push(pixel);
            }

            Ok(result)
        }
        m => panic!("FreeType pixel mode unsupported: {m:?}"),
    }
}
