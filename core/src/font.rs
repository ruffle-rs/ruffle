use crate::backend::render::{RenderBackend, ShapeHandle};
use crate::prelude::*;
use crate::transform::Transform;
use gc_arena::{Collect, Gc, MutationContext};

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
        Ok(Font(Gc::allocate(
            gc_context,
            FontData {
                glyphs,
                code_point_to_glyph,

                /// DefineFont3 stores coordinates at 20x the scale of DefineFont1/2.
                /// (SWF19 p.164)
                scale: if tag.version >= 3 { 20480.0 } else { 1024.0 },
                kerning_pairs,
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
        mut glyph_func: FGlyph,
    ) where
        FGlyph: FnMut(&Transform, &Glyph),
    {
        transform.matrix.ty += height;
        let scale = height.get() as f32 / self.scale();

        transform.matrix.a = scale;
        transform.matrix.d = scale;
        let mut chars = text.chars().peekable();
        let has_kerning_info = self.has_kerning_info();
        while let Some(c) = chars.next() {
            if let Some(glyph) = self.get_glyph_for_char(c) {
                glyph_func(&transform, &glyph);
                // Step horizontally.
                let mut advance = Twips::new(glyph.advance);
                if has_kerning_info {
                    advance += self.get_kerning_offset(c, chars.peek().cloned().unwrap_or('\0'));
                }
                transform.matrix.tx += Twips::new((advance.get() as f32 * scale) as i32);
            }
        }
    }

    /// Measure a particular string's metrics (width and height).
    pub fn measure(self, text: &str, font_size: Twips) -> (Twips, Twips) {
        let mut size = (Twips::new(0), Twips::new(0));

        self.evaluate(text, Default::default(), font_size, |transform, _glyph| {
            let tx = transform.matrix.tx;
            let ty = transform.matrix.ty;
            size.0 = std::cmp::max(size.0, tx);
            size.1 = std::cmp::max(size.1, ty);
        });

        size
    }

    /// Given a line of text, split it into the shortest number of lines that
    /// are shorter than `width`.
    ///
    /// This function assumes only `" "` is valid whitespace to split words on,
    /// and will not attempt to break words that are longer than `width`.
    ///
    /// The given `offset` determines the start of the initial line.
    ///
    /// TODO: This function and, more generally, this entire file will need to
    /// be internationalized to implement AS3 `flash.text.engine`.
    pub fn split_wrapped_lines(
        self,
        text: &str,
        font_size: Twips,
        width: Twips,
        offset: Twips,
    ) -> Vec<usize> {
        let mut result = vec![];
        let mut current_width = width
            .checked_sub(offset)
            .unwrap_or_else(|| Twips::from_pixels(0.0));
        let mut current_word = &text[0..0];

        for word in text.split(' ') {
            let measure = self.measure(word, font_size);
            let line_start = current_word.as_ptr() as usize - text.as_ptr() as usize;
            let line_end = if (line_start + current_word.len() + 1) < text.len() {
                line_start + current_word.len() + 1
            } else {
                line_start + current_word.len()
            };
            let word_start = word.as_ptr() as usize - text.as_ptr() as usize;
            let word_end = if (word_start + word.len() + 1) < text.len() {
                word_start + word.len() + 1
            } else {
                word_start + word.len()
            };

            if measure.0 > current_width && measure.0 > width {
                //Failsafe for if we get a word wider than the field.
                if !current_word.is_empty() {
                    result.push(line_end);
                }
                result.push(word_end);
                current_word = &text[word_end..word_end];
                current_width = width;
            } else if measure.0 > current_width {
                if !current_word.is_empty() {
                    result.push(line_end);
                }

                current_word = &text[word_start..word_end];
                current_width = width;
            } else {
                current_word = &text[line_start..word_end];
                current_width -= measure.0;
            }
        }

        result
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub shape: ShapeHandle,
    pub advance: i16,
}
