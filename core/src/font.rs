use crate::drawing::Drawing;
use crate::html::TextSpan;
use crate::prelude::*;
use crate::string::WStr;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_render::backend::null::NullBitmapSource;
use ruffle_render::backend::{RenderBackend, ShapeHandle};
use ruffle_render::shape_utils::{DrawCommand, FillRule};
use ruffle_render::transform::Transform;
use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};
use std::cmp::max;
use std::hash::{Hash, Hasher};
use swf::FillStyle;

pub use swf::TextGridFit;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum DefaultFont {
    /// `_sans`, a Sans-Serif font (similar to Helvetica or Arial)
    Sans,

    /// `_serif`, a Serif font (similar to Times Roman)
    Serif,

    /// `_typewriter`, a Monospace font (similar to Courier)
    Typewriter,

    /// `_ゴシック`, a Japanese Gothic font
    JapaneseGothic,

    /// `_等幅`, a Japanese Gothic Mono font
    JapaneseGothicMono,

    /// `_明朝`, a Japanese Mincho font
    JapaneseMincho,
}

fn round_to_pixel(t: Twips) -> Twips {
    Twips::from_pixels(t.to_pixels().round())
}

/// Parameters necessary to evaluate a font.
#[derive(Copy, Clone, Debug)]
pub struct EvalParameters {
    /// The height of each glyph, equivalent to a font size.
    height: Twips,

    /// Additional letter spacing to be added to or removed from each glyph
    /// after normal or kerned glyph advances are applied.
    letter_spacing: Twips,

    /// Whether to allow use of font-provided kerning metrics.
    ///
    /// Fonts can optionally add or remove additional spacing between specific
    /// pairs of letters, separate from the ordinary width between glyphs. This
    /// parameter allows enabling or disabling that feature.
    kerning: bool,
}

impl EvalParameters {
    /// Construct eval parameters from their individual parts.
    #[allow(dead_code)]
    fn from_parts(height: Twips, letter_spacing: Twips, kerning: bool) -> Self {
        Self {
            height,
            letter_spacing,
            kerning,
        }
    }

    /// Convert the formatting on a text span over to font evaluation
    /// parameters.
    pub fn from_span(span: &TextSpan) -> Self {
        Self {
            height: Twips::from_pixels(span.font.size),
            letter_spacing: Twips::from_pixels(span.font.letter_spacing),
            kerning: span.font.kerning,
        }
    }

    /// Get the height that the font would be evaluated at.
    pub fn height(&self) -> Twips {
        self.height
    }
}

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

/// Represents a raw font file (ie .ttf).
/// This should be shared and reused where possible, and it's reparsed every time a new glyph is required.
///
/// Parsing of a font is near-free (according to [ttf_parser::Face::parse]), but the storage isn't.
///
/// Font files may contain multiple individual font faces, but those font faces may reuse the same
/// Glyph from the same file. For this reason, glyphs are reused where possible.
#[derive(Debug)]
pub struct FontFace {
    bytes: Cow<'static, [u8]>,
    glyphs: Vec<OnceCell<Option<Glyph>>>,
    font_index: u32,

    ascender: i32,
    descender: i32,
    leading: i16,
    scale: f32,
    might_have_kerning: bool,
}

impl FontFace {
    pub fn new(
        bytes: Cow<'static, [u8]>,
        font_index: u32,
    ) -> Result<Self, ttf_parser::FaceParsingError> {
        // TODO: Support font collections

        // We validate that the font is good here, so we can just `.expect()` it later
        let face = ttf_parser::Face::parse(&bytes, font_index)?;

        let ascender = face.ascender() as i32;
        let descender = -face.descender() as i32;
        let leading = face.line_gap();
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
            bytes,
            font_index,
            glyphs,
            ascender,
            descender,
            leading,
            scale,
            might_have_kerning,
        })
    }

    pub fn get_glyph(&self, character: char) -> Option<&Glyph> {
        let face = ttf_parser::Face::parse(&self.bytes, self.font_index)
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
                            || drawing.self_bounds().width(),
                            |a| Twips::new(a as i32),
                        );
                        Some(Glyph {
                            shape_handle: Default::default(),
                            shape: GlyphShape::Drawing(drawing),
                            advance,
                        })
                    } else {
                        let advance = Twips::new(face.glyph_hor_advance(glyph_id)? as i32);
                        // If we have advance, then this is either an image, SVG or simply missing (ie whitespace)
                        Some(Glyph {
                            shape_handle: Default::default(),
                            shape: GlyphShape::None,
                            advance,
                        })
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
        let face = ttf_parser::Face::parse(&self.bytes, self.font_index)
            .expect("Font was already checked to be valid");

        if let (Some(left_glyph), Some(right_glyph)) =
            (face.glyph_index(left), face.glyph_index(right))
        {
            if let Some(kern) = face.tables().kern {
                for subtable in kern.subtables {
                    if subtable.horizontal {
                        if let Some(value) = subtable.glyphs_kerning(left_glyph, right_glyph) {
                            return Twips::new(value as i32);
                        }
                    }
                }
            }
        }

        Twips::ZERO
    }
}

#[derive(Debug)]
pub enum GlyphSource {
    Memory {
        /// The list of glyphs defined in the font.
        /// Used directly by `DefineText` tags.
        glyphs: Vec<Glyph>,

        /// A map from a Unicode code point to glyph in the `glyphs` array.
        /// Used by `DefineEditText` tags.
        code_point_to_glyph: fnv::FnvHashMap<u16, usize>,

        /// Kerning information.
        /// Maps from a pair of unicode code points to horizontal offset value.
        kerning_pairs: fnv::FnvHashMap<(u16, u16), Twips>,
    },
    FontFace(FontFace),
    Empty,
}

impl GlyphSource {
    pub fn get_by_index(&self, index: usize) -> Option<&Glyph> {
        match self {
            GlyphSource::Memory { glyphs, .. } => glyphs.get(index),
            GlyphSource::FontFace(_) => None, // Unsupported.
            GlyphSource::Empty => None,
        }
    }

    pub fn get_by_code_point(&self, code_point: char) -> Option<&Glyph> {
        match self {
            GlyphSource::Memory {
                glyphs,
                code_point_to_glyph,
                ..
            } => {
                // TODO: Properly handle UTF-16/out-of-bounds code points.
                let code_point = code_point as u16;
                if let Some(index) = code_point_to_glyph.get(&code_point) {
                    glyphs.get(*index)
                } else {
                    None
                }
            }
            GlyphSource::FontFace(face) => face.get_glyph(code_point),
            GlyphSource::Empty => None,
        }
    }

    pub fn has_kerning_info(&self) -> bool {
        match self {
            GlyphSource::Memory { kerning_pairs, .. } => !kerning_pairs.is_empty(),
            GlyphSource::FontFace(face) => face.has_kerning_info(),
            GlyphSource::Empty => false,
        }
    }

    pub fn get_kerning_offset(&self, left: char, right: char) -> Twips {
        match self {
            GlyphSource::Memory { kerning_pairs, .. } => {
                // TODO: Properly handle UTF-16/out-of-bounds code points.
                let left_code_point = left as u16;
                let right_code_point = right as u16;
                kerning_pairs
                    .get(&(left_code_point, right_code_point))
                    .cloned()
                    .unwrap_or_default()
            }
            GlyphSource::FontFace(face) => face.get_kerning_offset(left, right),
            GlyphSource::Empty => Twips::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Collect, Hash)]
#[collect(require_static)]
pub enum FontType {
    Embedded,
    EmbeddedCFF,
    Device,
}

#[derive(Debug, Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Font<'gc>(Gc<'gc, FontData>);

#[derive(Debug, Collect)]
#[collect(require_static)]
struct FontData {
    glyphs: GlyphSource,

    /// The scaling applied to the font height to render at the proper size.
    /// This depends on the DefineFont tag version.
    scale: f32,

    /// The distance from the top of each glyph to the baseline of the font, in
    /// EM-square coordinates.
    ascent: i32,

    /// The distance from the baseline of the font to the bottom of each glyph,
    /// in EM-square coordinates.
    descent: i32,

    /// The distance between the bottom of any one glyph and the top of
    /// another, in EM-square coordinates.
    leading: i16,

    /// The identity of the font.
    #[collect(require_static)]
    descriptor: FontDescriptor,

    font_type: FontType,
}

impl<'gc> Font<'gc> {
    pub fn from_font_file(
        gc_context: &Mutation<'gc>,
        descriptor: FontDescriptor,
        bytes: Cow<'static, [u8]>,
        font_index: u32,
        font_type: FontType,
    ) -> Result<Font<'gc>, ttf_parser::FaceParsingError> {
        let face = FontFace::new(bytes, font_index)?;

        Ok(Font(Gc::new(
            gc_context,
            FontData {
                scale: face.scale,
                ascent: face.ascender,
                descent: face.descender,
                leading: face.leading,
                glyphs: GlyphSource::FontFace(face),
                descriptor,
                font_type,
            },
        )))
    }

    pub fn from_swf_tag(
        gc_context: &Mutation<'gc>,
        renderer: &mut dyn RenderBackend,
        tag: swf::Font,
        encoding: &'static swf::Encoding,
        font_type: FontType,
    ) -> Font<'gc> {
        let mut code_point_to_glyph = fnv::FnvHashMap::default();

        let descriptor = FontDescriptor::from_swf_tag(&tag, encoding);
        let (ascent, descent, leading) = if let Some(layout) = &tag.layout {
            (layout.ascent as i32, layout.descent as i32, layout.leading)
        } else {
            (0, 0, 0)
        };

        let glyphs: Vec<Glyph> = tag
            .glyphs
            .into_iter()
            .enumerate()
            .map(|(index, swf_glyph)| {
                let code = swf_glyph.code;
                code_point_to_glyph.insert(code, index);

                let glyph = Glyph {
                    shape_handle: None.into(),
                    advance: Twips::new(swf_glyph.advance.into()),
                    shape: GlyphShape::Swf(RefCell::new(SwfGlyphOrShape::Glyph(swf_glyph))),
                };

                // Eager-load ASCII characters.
                if code < 128 {
                    glyph.shape_handle(renderer);
                }

                glyph
            })
            .collect();

        let kerning_pairs: fnv::FnvHashMap<(u16, u16), Twips> = if let Some(layout) = &tag.layout {
            layout
                .kerning
                .iter()
                .map(|kerning| ((kerning.left_code, kerning.right_code), kerning.adjustment))
                .collect()
        } else {
            fnv::FnvHashMap::default()
        };

        Font(Gc::new(
            gc_context,
            FontData {
                glyphs: if glyphs.is_empty() {
                    GlyphSource::Empty
                } else {
                    GlyphSource::Memory {
                        glyphs,
                        code_point_to_glyph,
                        kerning_pairs,
                    }
                },

                // DefineFont3 stores coordinates at 20x the scale of DefineFont1/2.
                // (SWF19 p.164)
                scale: if tag.version >= 3 { 20480.0 } else { 1024.0 },
                ascent,
                descent,
                leading,
                descriptor,
                font_type,
            },
        ))
    }

    pub fn from_font4_tag(
        gc_context: &Mutation<'gc>,
        tag: swf::Font4,
        encoding: &'static swf::Encoding,
    ) -> Result<Font<'gc>, ttf_parser::FaceParsingError> {
        let name = tag.name.to_str_lossy(encoding);
        let descriptor = FontDescriptor::from_parts(&name, tag.is_bold, tag.is_italic);

        if let Some(bytes) = tag.data {
            Font::from_font_file(
                gc_context,
                descriptor,
                Cow::Owned(bytes.to_vec()),
                0,
                FontType::EmbeddedCFF,
            )
        } else {
            Ok(Font(Gc::new(
                gc_context,
                FontData {
                    scale: 1.0,
                    ascent: 0,
                    descent: 0,
                    leading: 0,
                    glyphs: GlyphSource::Empty,
                    descriptor,
                    font_type: FontType::EmbeddedCFF,
                },
            )))
        }
    }

    /// Returns whether this font contains glyph shapes.
    /// If not, this font should be rendered as a device font.
    pub fn has_glyphs(&self) -> bool {
        !matches!(self.0.glyphs, GlyphSource::Empty)
    }

    /// Returns a glyph entry by index.
    /// Used by `Text` display objects.
    pub fn get_glyph(&self, i: usize) -> Option<&Glyph> {
        self.0.glyphs.get_by_index(i)
    }

    /// Returns a glyph entry by character.
    /// Used by `EditText` display objects.
    pub fn get_glyph_for_char(&self, c: char) -> Option<&Glyph> {
        self.0.glyphs.get_by_code_point(c)
    }

    /// Determine if this font contains all the glyphs within a given string.
    pub fn has_glyphs_for_str(&self, target_str: &WStr) -> bool {
        for character in target_str.chars() {
            let c = character.unwrap_or(char::REPLACEMENT_CHARACTER);
            if self.get_glyph_for_char(c).is_none() {
                return false;
            }
        }

        true
    }

    /// Returns whether this font contains kerning information.
    pub fn has_kerning_info(&self) -> bool {
        self.0.glyphs.has_kerning_info()
    }

    /// Given a pair of characters, applies the offset that should be applied
    /// to the advance value between these two characters.
    /// Returns 0 twips if no kerning offset exists between these two characters.
    pub fn get_kerning_offset(&self, left: char, right: char) -> Twips {
        self.0.glyphs.get_kerning_offset(left, right)
    }

    /// Return the leading for this font at a given height.
    pub fn get_leading_for_height(&self, height: Twips) -> Twips {
        let scale = height.get() as f32 / self.scale();

        Twips::new((self.0.leading as f32 * scale) as i32)
    }

    /// Get the baseline from the top of the glyph at a given height.
    pub fn get_baseline_for_height(&self, height: Twips) -> Twips {
        let scale = height.get() as f32 / self.scale();

        Twips::new((self.0.ascent as f32 * scale) as i32)
    }

    /// Get the descent from the baseline to the bottom of the glyph at a given height.
    pub fn get_descent_for_height(&self, height: Twips) -> Twips {
        let scale = height.get() as f32 / self.scale();

        Twips::new((self.0.descent as f32 * scale) as i32)
    }

    pub fn scale(&self) -> f32 {
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
        &self,
        text: &WStr, // TODO: take an `IntoIterator<Item=char>`, to not depend on string representation?
        mut transform: Transform,
        params: EvalParameters,
        mut glyph_func: FGlyph,
    ) where
        FGlyph: FnMut(usize, &Transform, &Glyph, Twips, Twips),
    {
        transform.matrix.ty += params.height;
        let scale = params.height.get() as f32 / self.scale();

        transform.matrix.a = scale;
        transform.matrix.d = scale;
        let mut char_indices = text.char_indices().peekable();
        let has_kerning_info = self.has_kerning_info();
        let mut x = Twips::ZERO;
        while let Some((pos, c)) = char_indices.next() {
            let c = c.unwrap_or(char::REPLACEMENT_CHARACTER);
            if let Some(glyph) = self.get_glyph_for_char(c) {
                let mut advance = glyph.advance();
                if has_kerning_info && params.kerning {
                    let next_char = char_indices.peek().cloned().unwrap_or((0, Ok('\0'))).1;
                    let next_char = next_char.unwrap_or(char::REPLACEMENT_CHARACTER);
                    advance += self.get_kerning_offset(c, next_char);
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

                glyph_func(pos, &transform, glyph, twips_advance, x);

                // Step horizontally.
                transform.matrix.tx += twips_advance;
                x += twips_advance;
            }
        }
    }

    /// Measure a particular string's metrics (width and height).
    pub fn measure(&self, text: &WStr, params: EvalParameters) -> (Twips, Twips) {
        let round = false;
        let mut width = Twips::ZERO;
        let mut height = Twips::ZERO;

        self.evaluate(
            text,
            Default::default(),
            params,
            |_pos, transform, _glyph, advance, _x| {
                let tx = transform.matrix.tx;
                let ty = transform.matrix.ty;

                if round {
                    width = width.max((tx + advance).trunc_to_pixel());
                    height = height.max(ty.trunc_to_pixel());
                } else {
                    width = width.max(tx + advance);
                    height = height.max(ty);
                }
            },
        );

        if text.is_empty() {
            height = max(height, params.height);
        }

        (width, height)
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
        &self,
        text: &WStr,
        params: EvalParameters,
        width: Twips,
        offset: Twips,
        mut is_start_of_line: bool,
    ) -> Option<usize> {
        let mut remaining_width = width - offset;
        if remaining_width < Twips::from_pixels(0.0) {
            return Some(0);
        }

        let mut line_end = 0;

        for word in text.split(b' ') {
            let word_start = word.offset_in(text).unwrap();
            let word_end = word_start + word.len();

            let measure = self.measure(
                // +1 is fine because ' ' is 1 unit
                text.slice(word_start..word_end + 1).unwrap_or(word),
                params,
            );

            if is_start_of_line && measure.0 > remaining_width {
                //Failsafe for if we get a word wider than the field.
                let mut last_passing_breakpoint = (Twips::ZERO, Twips::ZERO);

                let cur_slice = &text[word_start..];
                let mut char_iter = cur_slice.char_indices();
                let mut prev_char_index = word_start;
                let mut prev_frag_end = 0;

                char_iter.next(); // No need to check cur_slice[0..0]
                while last_passing_breakpoint.0 < remaining_width {
                    prev_char_index = word_start + prev_frag_end;

                    if let Some((frag_end, _)) = char_iter.next() {
                        last_passing_breakpoint = self.measure(&cur_slice[..frag_end], params);

                        prev_frag_end = frag_end;
                    } else {
                        break;
                    }
                }

                return Some(prev_char_index);
            } else if measure.0 > remaining_width {
                //The word is wider than our remaining width, return the end of
                //the line.
                return Some(line_end);
            } else {
                //Space remains for our current word, move up the word pointer.
                line_end = word_end;
                is_start_of_line = is_start_of_line && text[0..line_end].trim().is_empty();

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

    pub fn descriptor(&self) -> &FontDescriptor {
        &self.0.descriptor
    }

    pub fn font_type(&self) -> FontType {
        self.0.font_type
    }
}

#[derive(Debug, Clone)]
enum SwfGlyphOrShape {
    Glyph(swf::Glyph),
    Shape(swf::Shape),
}

impl SwfGlyphOrShape {
    pub fn shape(&mut self) -> &mut swf::Shape {
        if let SwfGlyphOrShape::Glyph(glyph) = self {
            *self = SwfGlyphOrShape::Shape(ruffle_render::shape_utils::swf_glyph_to_shape(glyph));
        }

        match self {
            SwfGlyphOrShape::Shape(shape) => shape,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
enum GlyphShape {
    Swf(RefCell<SwfGlyphOrShape>),
    Drawing(Drawing),
    None,
}

impl GlyphShape {
    pub fn hit_test(&self, point: Point<Twips>, local_matrix: &Matrix) -> bool {
        match self {
            GlyphShape::Swf(glyph) => {
                let mut glyph = glyph.borrow_mut();
                let shape = glyph.shape();
                shape.shape_bounds.contains(point)
                    && ruffle_render::shape_utils::shape_hit_test(shape, point, local_matrix)
            }
            GlyphShape::Drawing(drawing) => drawing.hit_test(point, local_matrix),
            GlyphShape::None => false,
        }
    }

    pub fn register(&self, renderer: &mut dyn RenderBackend) -> Option<ShapeHandle> {
        match self {
            GlyphShape::Swf(glyph) => {
                let mut glyph = glyph.borrow_mut();
                Some(renderer.register_shape((&*glyph.shape()).into(), &NullBitmapSource))
            }
            GlyphShape::Drawing(drawing) => drawing.register_or_replace(renderer),
            GlyphShape::None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    // Handle to registered shape.
    // If None, it'll be loaded lazily on first render of this glyph.
    // It's a double option; the outer one is "have we registered", the inner one is option because it may not exist
    shape_handle: RefCell<Option<Option<ShapeHandle>>>,

    shape: GlyphShape,
    advance: Twips,
}

impl Glyph {
    pub fn shape_handle(&self, renderer: &mut dyn RenderBackend) -> Option<ShapeHandle> {
        self.shape_handle
            .borrow_mut()
            .get_or_insert_with(|| self.shape.register(renderer))
            .clone()
    }

    pub fn hit_test(&self, point: Point<Twips>, local_matrix: &Matrix) -> bool {
        self.shape.hit_test(point, local_matrix)
    }

    pub fn advance(&self) -> Twips {
        self.advance
    }
}

/// Structure which identifies a particular font by name and properties.
#[derive(Debug, Clone, Ord, PartialOrd, Collect)]
#[collect(require_static)]
pub struct FontDescriptor {
    /// The name of the font.
    /// This is set by the author of the SWF and does not correlate to any opentype names.
    name: String,

    // All name comparisons ignore case, so this is for easy comparisons.
    lowercase_name: String,

    is_bold: bool,
    is_italic: bool,
}

impl PartialEq for FontDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.lowercase_name == other.lowercase_name
            && self.is_italic == other.is_italic
            && self.is_bold == other.is_bold
    }
}

impl Eq for FontDescriptor {}

impl Hash for FontDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lowercase_name.hash(state);
        self.is_bold.hash(state);
        self.is_italic.hash(state);
    }
}

impl FontDescriptor {
    /// Obtain a font descriptor from a SWF font tag.
    pub fn from_swf_tag(val: &swf::Font, encoding: &'static swf::Encoding) -> Self {
        let name = val.name.to_string_lossy(encoding);
        let lowercase_name = name.to_lowercase();

        Self {
            name,
            lowercase_name,
            is_bold: val.flags.contains(swf::FontFlag::IS_BOLD),
            is_italic: val.flags.contains(swf::FontFlag::IS_ITALIC),
        }
    }

    /// Obtain a font descriptor from a name/bold/italic triplet.
    pub fn from_parts(name: &str, is_bold: bool, is_italic: bool) -> Self {
        let mut name = name.to_string();

        if let Some(first_null) = name.find('\0') {
            name.truncate(first_null);
        };
        let lowercase_name = name.to_lowercase();

        Self {
            name,
            lowercase_name,
            is_bold,
            is_italic,
        }
    }

    /// Get the name of the font this descriptor identifies.
    pub fn name(&self) -> &str {
        &self.name
    }

    // Get the lowercase name.
    pub fn lowercase_name(&self) -> &str {
        &self.lowercase_name
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

/// The text rendering engine that a text field should use.
/// This is controlled by the "Anti-alias" setting in the Flash IDE.
/// Using "Anti-alias for readability" switches to the "Advanced" text
/// rendering engine.
#[derive(Debug, PartialEq, Clone)]
pub enum TextRenderSettings {
    /// This text should render with the standard rendering engine.
    /// Set via "Anti-alias for animation" in the Flash IDE.
    ///
    /// The `grid_fit`, `thickness`, and `sharpness` parameters are present
    /// because they are retained when switching from `Advanced` to `Normal`
    /// rendering and vice versa. They are not used in Normal rendering.
    Normal {
        grid_fit: TextGridFit,
        thickness: f32,
        sharpness: f32,
    },

    /// This text should render with the advanced rendering engine.
    /// Set via "Anti-alias for readability" in the Flash IDE.
    /// The parameters are set via the CSMTextSettings SWF tag.
    /// Ruffle does not support this currently, but this also affects
    /// hit-testing behavior.
    Advanced {
        grid_fit: TextGridFit,
        thickness: f32,
        sharpness: f32,
    },
}

impl TextRenderSettings {
    pub fn is_advanced(&self) -> bool {
        matches!(self, TextRenderSettings::Advanced { .. })
    }

    pub fn with_advanced_rendering(self) -> Self {
        match self {
            TextRenderSettings::Advanced { .. } => self,
            TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            } => TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }

    pub fn with_normal_rendering(self) -> Self {
        match self {
            TextRenderSettings::Normal { .. } => self,
            TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            } => TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }

    pub fn sharpness(&self) -> f32 {
        match self {
            TextRenderSettings::Normal { sharpness, .. } => *sharpness,
            TextRenderSettings::Advanced { sharpness, .. } => *sharpness,
        }
    }

    pub fn with_sharpness(self, sharpness: f32) -> Self {
        match self {
            TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness: _,
            } => TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            },
            TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness: _,
            } => TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }

    pub fn thickness(&self) -> f32 {
        match self {
            TextRenderSettings::Normal { thickness, .. } => *thickness,
            TextRenderSettings::Advanced { thickness, .. } => *thickness,
        }
    }

    pub fn with_thickness(self, thickness: f32) -> Self {
        match self {
            TextRenderSettings::Normal {
                grid_fit,
                thickness: _,
                sharpness,
            } => TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            },
            TextRenderSettings::Advanced {
                grid_fit,
                thickness: _,
                sharpness,
            } => TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }

    pub fn grid_fit(&self) -> swf::TextGridFit {
        match self {
            TextRenderSettings::Normal { grid_fit, .. } => *grid_fit,
            TextRenderSettings::Advanced { grid_fit, .. } => *grid_fit,
        }
    }

    pub fn with_grid_fit(self, grid_fit: TextGridFit) -> Self {
        match self {
            TextRenderSettings::Normal {
                grid_fit: _,
                thickness,
                sharpness,
            } => TextRenderSettings::Normal {
                grid_fit,
                thickness,
                sharpness,
            },
            TextRenderSettings::Advanced {
                grid_fit: _,
                thickness,
                sharpness,
            } => TextRenderSettings::Advanced {
                grid_fit,
                thickness,
                sharpness,
            },
        }
    }
}

impl From<swf::CsmTextSettings> for TextRenderSettings {
    fn from(settings: swf::CsmTextSettings) -> Self {
        if settings.use_advanced_rendering {
            TextRenderSettings::Advanced {
                grid_fit: settings.grid_fit,
                thickness: settings.thickness,
                sharpness: settings.sharpness,
            }
        } else {
            TextRenderSettings::default()
        }
    }
}

impl Default for TextRenderSettings {
    fn default() -> Self {
        Self::Normal {
            grid_fit: TextGridFit::Pixel,
            thickness: 0.0,
            sharpness: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::font::{EvalParameters, Font, FontType};
    use crate::string::WStr;
    use gc_arena::{rootless_arena, Mutation};
    use ruffle_render::backend::{null::NullRenderer, ViewportDimensions};
    use swf::Twips;

    const DEVICE_FONT_TAG: &[u8] = include_bytes!("../assets/noto-sans-definefont3.bin");

    fn with_device_font<F>(callback: F)
    where
        F: for<'gc> FnOnce(&Mutation<'gc>, Font<'gc>),
    {
        rootless_arena(|mc| {
            let mut renderer = NullRenderer::new(ViewportDimensions {
                width: 0,
                height: 0,
                scale_factor: 1.0,
            });
            let mut reader = swf::read::Reader::new(DEVICE_FONT_TAG, 8);
            let device_font = Font::from_swf_tag(
                mc,
                &mut renderer,
                reader
                    .read_define_font_2(3)
                    .expect("Built-in font should compile"),
                reader.encoding(),
                FontType::Device,
            );

            callback(mc, device_font);
        })
    }

    #[test]
    fn wrap_line_no_breakpoint() {
        with_device_font(|_mc, df| {
            let params =
                EvalParameters::from_parts(Twips::from_pixels(12.0), Twips::from_pixels(0.0), true);
            let string = WStr::from_units(b"abcdefghijklmnopqrstuv");
            let breakpoint = df.wrap_line(
                string,
                params,
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
            let params =
                EvalParameters::from_parts(Twips::from_pixels(12.0), Twips::from_pixels(0.0), true);
            let string = WStr::from_units(b"abcd efgh ijkl mnop");
            let mut last_bp = 0;
            let breakpoint = df.wrap_line(
                string,
                params,
                Twips::from_pixels(35.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint);

            last_bp += breakpoint.unwrap() + 1;

            let breakpoint2 = df.wrap_line(
                &string[last_bp..],
                params,
                Twips::from_pixels(35.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint2);

            last_bp += breakpoint2.unwrap() + 1;

            let breakpoint3 = df.wrap_line(
                &string[last_bp..],
                params,
                Twips::from_pixels(35.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint3);

            last_bp += breakpoint3.unwrap() + 1;

            let breakpoint4 = df.wrap_line(
                &string[last_bp..],
                params,
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
            let params =
                EvalParameters::from_parts(Twips::from_pixels(12.0), Twips::from_pixels(0.0), true);
            let string = WStr::from_units(b"abcd efgh ijkl mnop");
            let breakpoint = df.wrap_line(
                string,
                params,
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
            let params =
                EvalParameters::from_parts(Twips::from_pixels(12.0), Twips::from_pixels(0.0), true);
            let string = WStr::from_units(b"abcdi j kl mnop q rstuv");
            let mut last_bp = 0;
            let breakpoint = df.wrap_line(
                string,
                params,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(5), breakpoint);

            last_bp += breakpoint.unwrap() + 1;

            let breakpoint2 = df.wrap_line(
                &string[last_bp..],
                params,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint2);

            last_bp += breakpoint2.unwrap() + 1;

            let breakpoint3 = df.wrap_line(
                &string[last_bp..],
                params,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(4), breakpoint3);

            last_bp += breakpoint3.unwrap() + 1;

            let breakpoint4 = df.wrap_line(
                &string[last_bp..],
                params,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(Some(1), breakpoint4);

            last_bp += breakpoint4.unwrap() + 1;

            let breakpoint5 = df.wrap_line(
                &string[last_bp..],
                params,
                Twips::from_pixels(37.0),
                Twips::from_pixels(0.0),
                true,
            );

            assert_eq!(None, breakpoint5);
        });
    }
}
