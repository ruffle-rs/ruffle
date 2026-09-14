mod atlas;
mod font_descriptor;
mod font_face;
mod font_like;
mod font_renderer;
mod font_set;
mod glyph;
mod text_render_settings;

pub use atlas::{FontAtlas, FontAtlasGlyph, FontAtlases};
pub use font_descriptor::FontDescriptor;
pub use font_face::{FontFace, FontFileData};
pub use font_like::{EvalParameters, FontLike, GlyphResolution};
pub use font_renderer::FontRenderer;
pub use font_renderer::FontRendererGlyphSource;
pub use font_set::FontSet;
pub use glyph::Glyph;
pub use text_render_settings::TextRenderSettings;

use crate::font::glyph::GlyphRef;
use crate::prelude::*;
use crate::string::WStr;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_render::backend::RenderBackend;
use std::hash::{Hash, Hasher};

pub use swf::TextGridFit;

#[derive(Clone, Eq, Collect, Debug)]
#[collect(require_static)]
pub struct FontQuery {
    pub font_type: FontType,
    pub name: String,
    pub lowercase_name: String,
    pub is_bold: bool,
    pub is_italic: bool,
}

impl FontQuery {
    pub fn new(font_type: FontType, name: String, is_bold: bool, is_italic: bool) -> Self {
        Self {
            font_type,
            lowercase_name: name.to_lowercase(),
            name,
            is_bold,
            is_italic,
        }
    }

    pub fn from_descriptor(font_type: FontType, descriptor: &FontDescriptor) -> Self {
        Self {
            font_type,
            name: descriptor.name().to_owned(),
            lowercase_name: descriptor.lowercase_name().to_owned(),
            is_bold: descriptor.bold(),
            is_italic: descriptor.italic(),
        }
    }
}

impl PartialEq for FontQuery {
    fn eq(&self, other: &Self) -> bool {
        self.font_type == other.font_type
            && self.lowercase_name == other.lowercase_name
            && self.is_bold == other.is_bold
            && self.is_italic == other.is_italic
    }
}

impl Hash for FontQuery {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_type.hash(state);
        self.lowercase_name.hash(state);
        self.is_bold.hash(state);
        self.is_italic.hash(state);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Collect)]
#[collect(require_static)]
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

impl DefaultFont {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "_serif" => DefaultFont::Serif,
            "_sans" => DefaultFont::Sans,
            "_typewriter" => DefaultFont::Typewriter,
            "_ゴシック" => DefaultFont::JapaneseGothic,
            "_等幅" => DefaultFont::JapaneseGothicMono,
            "_明朝" => DefaultFont::JapaneseMincho,
            _ => return None,
        })
    }
}

#[derive(Debug)]
pub(crate) enum GlyphSource {
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

        metrics: FontMetrics,
    },
    FontFace {
        face: FontFace,
        metrics: FontMetrics,
    },
    ExternalRenderer(FontRendererGlyphSource),
    Empty,
}

impl GlyphSource {
    pub fn get_by_index(&self, index: usize) -> Option<GlyphRef<'_>> {
        match self {
            GlyphSource::Memory { glyphs, .. } => glyphs.get(index).map(GlyphRef::Direct),
            GlyphSource::FontFace { .. } => None, // Unsupported.
            GlyphSource::ExternalRenderer(_) => None, // Unsupported.
            GlyphSource::Empty => None,
        }
    }

    pub fn get_by_code_point(&self, code_point: char) -> Option<GlyphRef<'_>> {
        match self {
            GlyphSource::Memory {
                glyphs,
                code_point_to_glyph,
                ..
            } => {
                // TODO: Properly handle UTF-16/out-of-bounds code points.
                let code_point = code_point as u16;
                if let Some(index) = code_point_to_glyph.get(&code_point) {
                    glyphs.get(*index).map(GlyphRef::Direct)
                } else {
                    None
                }
            }
            GlyphSource::FontFace { face, .. } => face.get_glyph(code_point).map(GlyphRef::Direct),
            GlyphSource::ExternalRenderer(glyph_source) => {
                glyph_source.get_by_code_point(code_point)
            }
            GlyphSource::Empty => None,
        }
    }

    pub fn has_kerning_info(&self) -> bool {
        match self {
            GlyphSource::Memory { kerning_pairs, .. } => !kerning_pairs.is_empty(),
            GlyphSource::FontFace { face, .. } => face.has_kerning_info(),
            GlyphSource::ExternalRenderer(glyph_source) => {
                glyph_source.font_renderer().has_kerning_info()
            }
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
            GlyphSource::FontFace { face, .. } => face.get_kerning_offset(left, right),
            GlyphSource::ExternalRenderer(glyph_source) => {
                glyph_source.get_kerning_offset(left, right)
            }
            GlyphSource::Empty => Twips::ZERO,
        }
    }

    pub fn metrics(&self) -> FontMetrics {
        match self {
            GlyphSource::Memory { metrics, .. } => *metrics,
            GlyphSource::FontFace { metrics, .. } => *metrics,
            GlyphSource::ExternalRenderer(glyph_source) => {
                glyph_source.font_renderer().get_font_metrics()
            }
            GlyphSource::Empty => FontMetrics::ZERO,
        }
    }

    pub fn sweep_caches(&self) {
        if let GlyphSource::ExternalRenderer(glyph_source) = self {
            glyph_source.sweep_caches(false);
        }
    }

    /// Typographic metrics (OS/2 `sTypo*`) at a requested height, if available.
    /// For fonts Ruffle parses itself they are read straight from the font data
    /// (no renderer involved, height ignored); a font supplied by an external
    /// renderer is asked for them at that height. `None` (e.g. a SWF glyph font,
    /// or a renderer that can't provide them) falls back to the hhea/cell
    /// metrics.
    pub fn typo_metrics_at(&self, height: Twips) -> Option<FontMetrics> {
        match self {
            GlyphSource::FontFace { face, .. } => face.typo_metrics(),
            GlyphSource::ExternalRenderer(glyph_source) => {
                let height_px = height.to_pixels().round() as u32;
                if height_px == 0 {
                    return None;
                }
                glyph_source
                    .font_renderer()
                    .get_typo_font_metrics(height_px)
            }
            GlyphSource::Memory { .. } | GlyphSource::Empty => None,
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

impl FontType {
    pub fn is_device(self) -> bool {
        self == Self::Device
    }

    pub fn is_embedded(self) -> bool {
        self != Self::Device
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// The scaling applied to the font height to render at the proper size.
    /// This depends on the DefineFont tag version.
    pub scale: f32,

    /// The distance from the top of each glyph to the baseline of the font, in
    /// EM-square coordinates.
    pub ascent: i32,

    /// The distance from the baseline of the font to the bottom of each glyph,
    /// in EM-square coordinates.
    pub descent: i32,

    /// The distance between the bottom of any one glyph and the top of
    /// another, in EM-square coordinates.
    #[allow(dead_code)] // Web build falsely claims it's unused
    pub leading: i16,
}

impl FontMetrics {
    /// Zero metrics, used when e.g. there's no font.
    pub const ZERO: FontMetrics = Self {
        scale: 1.0,
        ascent: 0,
        descent: 0,
        leading: 0,
    };

    /// Get the baseline (ascent) from the top of the glyph at a given height.
    #[must_use]
    pub fn ascent(&self, height: Twips) -> Twips {
        let scale = height.get() as f32 / self.scale;
        Twips::new((self.ascent as f32 * scale) as i32)
    }

    /// Get the descent from the baseline to the bottom of the glyph at a given height.
    #[must_use]
    pub fn descent(&self, height: Twips) -> Twips {
        let scale = height.get() as f32 / self.scale;
        Twips::new((self.descent as f32 * scale) as i32)
    }

    /// Return the leading for this font at a given height.
    #[allow(dead_code)] // TODO Do we need this method at all?
    #[must_use]
    pub fn leading(&self, height: Twips) -> Twips {
        let scale = height.get() as f32 / self.scale;
        Twips::new((self.leading as f32 * scale) as i32)
    }
}

#[derive(Debug, Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Font<'gc>(Gc<'gc, FontData>);

#[derive(Debug, Collect)]
#[collect(require_static)]
struct FontData {
    glyphs: GlyphSource,

    scale: f32,

    /// The identity of the font.
    #[collect(require_static)]
    descriptor: FontDescriptor,

    font_type: FontType,

    /// Whether this font has a layout defined.
    ///
    /// Fonts without a layout are used only to describe a font,
    /// not to provide glyphs.
    has_layout: bool,
}

impl<'gc> Font<'gc> {
    pub fn from_font_file(
        gc_context: &Mutation<'gc>,
        descriptor: FontDescriptor,
        data: FontFileData,
        font_index: u32,
        font_type: FontType,
    ) -> Result<Font<'gc>, ttf_parser::FaceParsingError> {
        let face = FontFace::new(data, font_index)?;
        let metrics = face.metrics();

        Ok(Font(Gc::new(
            gc_context,
            FontData {
                scale: metrics.scale,
                glyphs: GlyphSource::FontFace { metrics, face },
                descriptor,
                font_type,
                has_layout: true,
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
                // TODO: Flash doesn't care whether it's a surrogate code point or not.
                //   We should probably rethink using Rust's char for Flash characters.
                let character = char::from_u32(code as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
                code_point_to_glyph.insert(code, index);

                let glyph = Glyph::from_swf(character, swf_glyph);

                // Eager-load ASCII characters.
                if code < 128 {
                    glyph.glyph_render_data(renderer);
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

        // DefineFont3 stores coordinates at 20x the scale of DefineFont1/2.
        // (SWF19 p.164)
        let scale = if tag.version >= 3 { 20480.0 } else { 1024.0 };
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
                        metrics: FontMetrics {
                            scale,
                            ascent,
                            descent,
                            leading,
                        },
                    }
                },
                scale,
                descriptor,
                font_type,
                has_layout: tag.layout.is_some(),
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
                // TODO remove when https://github.com/rust-lang/rust-clippy/issues/15252 is fixed
                #[expect(clippy::unnecessary_to_owned)]
                FontFileData::new(bytes.to_vec()),
                0,
                FontType::EmbeddedCFF,
            )
        } else {
            Ok(Self::empty_font(
                gc_context,
                &name,
                tag.is_bold,
                tag.is_italic,
                FontType::EmbeddedCFF,
            ))
        }
    }

    pub fn from_renderer(
        gc_context: &Mutation<'gc>,
        descriptor: FontDescriptor,
        font_renderer: Box<dyn FontRenderer>,
    ) -> Self {
        let scale = font_renderer.scale();
        Font(Gc::new(
            gc_context,
            FontData {
                glyphs: GlyphSource::ExternalRenderer(FontRendererGlyphSource::new(font_renderer)),
                scale,
                descriptor,
                font_type: FontType::Device,
                has_layout: true,
            },
        ))
    }

    pub fn empty_font(
        gc_context: &Mutation<'gc>,
        name: &str,
        is_bold: bool,
        is_italic: bool,
        font_type: FontType,
    ) -> Font<'gc> {
        let descriptor = FontDescriptor::from_parts(name, is_bold, is_italic);

        Font(Gc::new(
            gc_context,
            FontData {
                glyphs: GlyphSource::Empty,
                scale: 1.0,
                descriptor,
                font_type,
                has_layout: true,
            },
        ))
    }

    pub fn as_ptr(self) -> *const () {
        Gc::as_ptr(self.0).cast()
    }

    #[cfg(feature = "egui")]
    pub(crate) fn glyph_source(&self) -> &GlyphSource {
        &self.0.glyphs
    }

    /// Returns whether this font contains glyph shapes.
    /// If not, this font should be rendered as a device font.
    pub fn has_glyphs(self) -> bool {
        !matches!(self.0.glyphs, GlyphSource::Empty)
    }

    /// Returns a glyph entry by index.
    /// Used by `Text` display objects.
    pub fn get_glyph(&self, i: usize) -> Option<GlyphRef<'_>> {
        self.0.glyphs.get_by_index(i)
    }

    /// Returns a glyph entry by character.
    /// Used by `EditText` display objects.
    pub fn get_glyph_for_char(&self, c: char) -> Option<GlyphRef<'_>> {
        self.0.glyphs.get_by_code_point(c)
    }

    /// Determine if this font contains all the glyphs within a given string.
    pub fn has_glyphs_for_str(self, target_str: &WStr) -> bool {
        for character in target_str.chars() {
            let c = character.unwrap_or(char::REPLACEMENT_CHARACTER);
            if self.get_glyph_for_char(c).is_none() {
                return false;
            }
        }

        true
    }

    pub fn descriptor(&self) -> &FontDescriptor {
        &self.0.descriptor
    }

    pub fn has_layout(self) -> bool {
        self.0.has_layout
    }

    pub fn sweep_caches(&self) {
        self.0.glyphs.sweep_caches();
    }
}

impl<'gc> FontLike<'gc> for Font<'gc> {
    fn resolve_glyph(&self, c: char) -> Option<GlyphResolution<'_, 'gc>> {
        self.get_glyph_for_char(c)
            .map(|glyph| GlyphResolution::new(glyph, *self))
    }

    fn has_kerning_info(&self) -> bool {
        self.0.glyphs.has_kerning_info()
    }

    fn get_kerning_offset(&self, left: char, right: char) -> Twips {
        self.0.glyphs.get_kerning_offset(left, right)
    }

    fn metrics(&self) -> FontMetrics {
        self.0.glyphs.metrics()
    }

    fn typo_metrics_at(&self, height: Twips) -> Option<FontMetrics> {
        self.0.glyphs.typo_metrics_at(height)
    }

    fn scale(&self) -> f32 {
        self.0.scale
    }

    fn font_type(&self) -> FontType {
        self.0.font_type
    }
}
