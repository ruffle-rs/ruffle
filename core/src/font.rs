use crate::backend::render::{RenderBackend, ShapeHandle};
use crate::prelude::*;
use crate::transform::Transform;
use gc_arena::{Collect, Gc, MutationContext};

/// Certain Flash routines measure text by rounding down to the nearest whole pixel.
pub fn round_down_to_pixel(t: Twips) -> Twips {
    Twips::from_pixels(t.to_pixels().floor())
}

type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Font<'gc>(Gc<'gc, FontData>);

#[derive(Debug, Clone, Collect)]
#[collect(require_static)]
struct FontData {
    /// The list of glyphs defined in the font.
    /// Used directly by `DefineText` tags.
    glyphs: Vec<Glyph>,

    /// A map from a Unicode code point to glyph in the `glyphs` array.
    /// Used by `DefineEditText` tags.
    code_point_to_glyph: fnv::FnvHashMap<u16, usize>,

    /// The scaling applied to the font height to render at the proper size.
    /// This depends on the DefineFont tag version.
    scale: f32,

    /// Kerning infomration.
    /// Maps from a pair of unicode code points to horizontal offset value.
    kerning_pairs: fnv::FnvHashMap<(u16, u16), Twips>,

    /// The distance from the top of each glyph to the baseline of the font, in
    /// EM-square coordinates.
    ascent: u16,

    /// The distance from the baseline of the font to the bottom of each glyph,
    /// in EM-square coordinates.
    descent: u16,

    /// The distance between the bottom of any one glyph and the top of
    /// another, in EM-square coordinates.
    leading: i16,

    /// The identity of the font.
    descriptor: FontDescriptor,
}

impl<'gc> Font<'gc> {
    pub fn from_swf_tag(
        gc_context: MutationContext<'gc, '_>,
        renderer: &mut dyn RenderBackend,
        tag: &swf::Font,
    ) -> Result<Font<'gc>, Error> {
        let mut glyphs = vec![];
        let mut code_point_to_glyph = fnv::FnvHashMap::default();
        for swf_glyph in &tag.glyphs {
            let glyph = Glyph {
                shape: renderer.register_glyph_shape(swf_glyph),
                advance: swf_glyph.advance.unwrap_or(0),
            };
            let index = glyphs.len();
            glyphs.push(glyph);
            code_point_to_glyph.insert(swf_glyph.code, index);
        }
        let kerning_pairs: fnv::FnvHashMap<(u16, u16), Twips> = if let Some(layout) = &tag.layout {
            layout
                .kerning
                .iter()
                .map(|kerning| ((kerning.left_code, kerning.right_code), kerning.adjustment))
                .collect()
        } else {
            fnv::FnvHashMap::default()
        };

        let descriptor = FontDescriptor::from_swf_tag(tag);
        let (ascent, descent, leading) = if let Some(layout) = &tag.layout {
            (layout.ascent, layout.descent, layout.leading)
        } else {
            (0, 0, 0)
        };

        Ok(Font(Gc::allocate(
            gc_context,
            FontData {
                glyphs,
                code_point_to_glyph,

                /// DefineFont3 stores coordinates at 20x the scale of DefineFont1/2.
                /// (SWF19 p.164)
                scale: if tag.version >= 3 { 20480.0 } else { 1024.0 },
                kerning_pairs,
                ascent,
                descent,
                leading,
                descriptor,
            },
        )))
    }

    /// Returns whether this font contains glyph shapes.
    /// If not, this font should be rendered as a device font.
    pub fn has_glyphs(self) -> bool {
        !self.0.glyphs.is_empty()
    }

    /// Returns a glyph entry by index.
    /// Used by `Text` display objects.
    pub fn get_glyph(self, i: usize) -> Option<Glyph> {
        self.0.glyphs.get(i).cloned()
    }

    /// Returns a glyph entry by character.
    /// Used by `EditText` display objects.
    pub fn get_glyph_for_char(self, c: char) -> Option<Glyph> {
        // TODO: Properly handle UTF-16/out-of-bounds code points.
        let code_point = c as u16;
        if let Some(index) = self.0.code_point_to_glyph.get(&code_point) {
            self.get_glyph(*index)
        } else {
            None
        }
    }

    /// Given a pair of characters, applies the offset that should be applied
    /// to the advance value between these two characters.
    /// Returns 0 twips if no kerning offset exists between these two characters.
    pub fn get_kerning_offset(self, left: char, right: char) -> Twips {
        // TODO: Properly handle UTF-16/out-of-bounds code points.
        let left_code_point = left as u16;
        let right_code_point = right as u16;
        self.0
            .kerning_pairs
            .get(&(left_code_point, right_code_point))
            .cloned()
            .unwrap_or_default()
    }

    /// Return the leading for this font at a given height.
    pub fn get_leading_for_height(self, height: Twips) -> Twips {
        let scale = height.get() as f32 / self.scale();

        Twips::new((self.0.leading as f32 * scale) as i32)
    }

    /// Returns whether this font contains kerning information.
    pub fn has_kerning_info(self) -> bool {
        !self.0.kerning_pairs.is_empty()
    }

    pub fn scale(self) -> f32 {
        self.0.scale
    }

    /// Evaluate this font against a particular string on a glyph-by-glyph
    /// basis.
    ///
    /// This function takes the text string to evaluate against, the base
    /// transform to start from, the height of each glyph, and produces a list
    /// of transforms and glyphs which will be consumed by the `glyph_func`
    /// closure. This corresponds to the series of drawing operations necessary
    /// to render the text on a single horizontal line.
    pub fn evaluate<FGlyph>(
        self,
        text: &str,
        mut transform: Transform,
        height: Twips,
        letter_spacing: Twips,
        kerning: bool,
        mut glyph_func: FGlyph,
    ) where
        FGlyph: FnMut(&Transform, &Glyph, Twips),
    {
        transform.matrix.ty += height;
        let scale = height.get() as f32 / self.scale();

        transform.matrix.a = scale;
        transform.matrix.d = scale;
        let mut chars = text.chars().peekable();
        let has_kerning_info = self.has_kerning_info();
        while let Some(c) = chars.next() {
            if let Some(glyph) = self.get_glyph_for_char(c) {
                let mut advance = Twips::new(glyph.advance);
                if has_kerning_info && kerning {
                    advance += self.get_kerning_offset(c, chars.peek().cloned().unwrap_or('\0'));
                }
                let twips_advance =
                    Twips::new((advance.get() as f32 * scale) as i32) + letter_spacing;

                glyph_func(&transform, &glyph, twips_advance);

                // Step horizontally.
                transform.matrix.tx += twips_advance;
            }
        }
    }

    /// Measure a particular string's metrics (width and height).
    pub fn measure(
        self,
        text: &str,
        font_size: Twips,
        letter_spacing: Twips,
        kerning: bool,
    ) -> (Twips, Twips) {
        let mut size = (Twips::new(0), Twips::new(0));

        self.evaluate(
            text,
            Default::default(),
            font_size,
            letter_spacing,
            kerning,
            |transform, _glyph, advance| {
                let tx = transform.matrix.tx;
                let ty = transform.matrix.ty;
                size.0 = std::cmp::max(size.0, round_down_to_pixel(tx + advance));
                size.1 = std::cmp::max(size.1, round_down_to_pixel(ty));
            },
        );

        size
    }

    /// Given a line of text, find the first breakpoint within the text.
    ///
    /// This function assumes only `" "` is valid whitespace to split words on,
    /// and will not attempt to break words that are longer than `width`, nor
    /// will it break at newlines.
    ///
    /// The given `offset` determines the start of the initial line, while the
    /// `width` indicates how long the line is supposed to be. Be careful to
    /// note that it is possible for this function to return `0`; that
    /// indicates that the string itself cannot fit on the line and should
    /// break onto the next one.
    ///
    /// This function yields `None` if the line is not broken.
    ///
    /// TODO: This function and, more generally, this entire file will need to
    /// be internationalized to implement AS3 `flash.text.engine`.
    pub fn wrap_line(
        self,
        text: &str,
        font_size: Twips,
        letter_spacing: Twips,
        kerning: bool,
        width: Twips,
        offset: Twips,
        mut is_start_of_line: bool,
    ) -> Option<usize> {
        let mut remaining_width = width - offset;
        if remaining_width < Twips::from_pixels(0.0) {
            return Some(0);
        }

        let mut current_word = &text[0..0];

        for word in text.split(' ') {
            let line_start = current_word.as_ptr() as usize - text.as_ptr() as usize;
            let line_end = line_start + current_word.len();
            let word_start = word.as_ptr() as usize - text.as_ptr() as usize;
            let word_end = word_start + word.len();

            let measure = self.measure(
                text.get(word_start..word_end + 1).unwrap_or(word),
                font_size,
                letter_spacing,
                kerning,
            );

            if is_start_of_line && measure.0 > remaining_width {
                //Failsafe for if we get a word wider than the field.
                let mut last_passing_breakpoint = (Twips::new(0), Twips::new(0));
                let mut frag_end = word_start;
                while last_passing_breakpoint.0 < remaining_width {
                    frag_end += 1;
                    last_passing_breakpoint = self.measure(
                        text.get(word_start..frag_end).unwrap(),
                        font_size,
                        letter_spacing,
                        kerning,
                    );
                }

                return Some(frag_end - 1);
            } else if measure.0 > remaining_width {
                //The word is wider than our remaining width, return the end of
                //the line.
                return Some(line_end);
            } else {
                //Space remains for our current word, move up the word pointer.
                current_word = &text[line_start..word_end];
                is_start_of_line = is_start_of_line && current_word.trim().is_empty();

                //If the additional space were to cause an overflow, then
                //return now.
                remaining_width -= measure.0;
                if remaining_width < Twips::from_pixels(0.0) {
                    return Some(word_end);
                }
            }
        }

        None
    }

    pub fn descriptor(self) -> FontDescriptor {
        self.0.descriptor.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub shape: ShapeHandle,
    pub advance: i16,
}

/// Structure which identifies a particular font by name and properties.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Collect)]
#[collect(require_static)]
pub struct FontDescriptor {
    name: String,
    is_bold: bool,
    is_italic: bool,
}

impl FontDescriptor {
    /// Obtain a font descriptor from a SWF font tag.
    pub fn from_swf_tag(val: &swf::Font) -> Self {
        Self {
            name: val.name.clone(),
            is_bold: val.is_bold,
            is_italic: val.is_italic,
        }
    }

    /// Obtain a font descriptor from a name/bold/italic triplet.
    pub fn from_parts(name: &str, is_bold: bool, is_italic: bool) -> Self {
        Self {
            name: name.to_string(),
            is_bold,
            is_italic,
        }
    }

    /// Get the name of the font class this descriptor references.
    pub fn class(&self) -> &str {
        &self.name
    }

    /// Get the boldness of the described font.
    pub fn bold(&self) -> bool {
        self.is_bold
    }

    /// Get the italic-ness of the described font.
    pub fn italic(&self) -> bool {
        self.is_italic
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::render::{NullRenderer, RenderBackend};
    use crate::font::Font;
    use crate::player::{Player, DEVICE_FONT_TAG};
    use gc_arena::{rootless_arena, MutationContext};
    use swf::Twips;

    fn with_device_font<F>(callback: F)
    where
        F: for<'gc> FnOnce(MutationContext<'gc, '_>, Font<'gc>),
    {
        rootless_arena(|mc| {
            let mut renderer: Box<dyn RenderBackend> = Box::new(NullRenderer::new());
            let device_font = Player::load_device_font(mc, DEVICE_FONT_TAG, &mut renderer).unwrap();

            callback(mc, device_font);
        })
    }

    #[test]
    fn wrap_line_no_breakpoint() {
        with_device_font(|_mc, df| {
            let string = "abcdefghijklmnopqrstuv";
            let breakpoint = df.wrap_line(
                &string,
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(200.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(None, breakpoint);
        });
    }

    #[test]
    fn wrap_line_breakpoint_every_word() {
        with_device_font(|_mc, df| {
            let string = "abcd efgh ijkl mnop";
            let mut last_bp = 0;
            let breakpoint = df.wrap_line(
                &string,
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(35.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint);

            last_bp += breakpoint.unwrap() + 1;

            let breakpoint2 = df.wrap_line(
                &string[last_bp..],
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(35.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint2);

            last_bp += breakpoint2.unwrap() + 1;

            let breakpoint3 = df.wrap_line(
                &string[last_bp..],
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(35.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint3);

            last_bp += breakpoint3.unwrap() + 1;

            let breakpoint4 = df.wrap_line(
                &string[last_bp..],
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(35.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(None, breakpoint4);
        });
    }

    #[test]
    fn wrap_line_breakpoint_no_room() {
        with_device_font(|_mc, df| {
            let string = "abcd efgh ijkl mnop";
            let breakpoint = df.wrap_line(
                &string,
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(30.0),
                Twips::from_pixels(29.0),
                false,
            );

            assert_eq!(Some(0), breakpoint);
        });
    }

    #[test]
    fn wrap_line_breakpoint_irregular_sized_words() {
        with_device_font(|_mc, df| {
            let string = "abcdi j kl mnop q rstuv";
            let mut last_bp = 0;
            let breakpoint = df.wrap_line(
                &string,
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(5), breakpoint);

            last_bp += breakpoint.unwrap() + 1;

            let breakpoint2 = df.wrap_line(
                &string[last_bp..],
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint2);

            last_bp += breakpoint2.unwrap() + 1;

            let breakpoint3 = df.wrap_line(
                &string[last_bp..],
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint3);

            last_bp += breakpoint3.unwrap() + 1;

            let breakpoint4 = df.wrap_line(
                &string[last_bp..],
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(1), breakpoint4);

            last_bp += breakpoint4.unwrap() + 1;

            let breakpoint5 = df.wrap_line(
                &string[last_bp..],
                Twips::from_pixels(12.0),
                Twips::from_pixels(0.0),
                true,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(None, breakpoint5);
        });
    }
}
