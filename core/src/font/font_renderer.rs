use std::cell::{Ref, RefCell};

use swf::Twips;

use crate::font::{FontAtlases, FontMetrics, Glyph, glyph::GlyphRef};

pub trait FontRenderer: std::fmt::Debug {
    fn scale(&self) -> f32;

    fn get_font_metrics(&self) -> FontMetrics;

    fn has_kerning_info(&self) -> bool;

    fn render_glyph(&self, character: char) -> Option<Glyph>;

    fn calculate_kerning(&self, left: char, right: char) -> Twips;

    fn atlases(&self) -> Option<&FontAtlases> {
        None
    }
}

#[derive(Debug)]
pub struct FontRendererGlyphSource {
    font_renderer: Box<dyn FontRenderer>,

    /// Maps Unicode code points to glyphs rendered by the renderer.
    glyph_cache: RefCell<fnv::FnvHashMap<u16, Option<Glyph>>>,

    /// Maps Unicode pairs to kerning provided by the renderer.
    kerning_cache: RefCell<fnv::FnvHashMap<(u16, u16), Twips>>,
}

impl FontRendererGlyphSource {
    pub fn new(font_renderer: Box<dyn FontRenderer>) -> Self {
        Self {
            font_renderer,
            glyph_cache: RefCell::new(fnv::FnvHashMap::default()),
            kerning_cache: RefCell::new(fnv::FnvHashMap::default()),
        }
    }

    pub fn glyph_cache_size(&self) -> usize {
        self.glyph_cache.borrow().len()
    }

    pub fn kerning_cache_size(&self) -> usize {
        self.kerning_cache.borrow().len()
    }

    pub fn font_renderer(&self) -> &dyn FontRenderer {
        self.font_renderer.as_ref()
    }

    pub fn get_by_code_point(&self, code_point: char) -> Option<GlyphRef<'_>> {
        let character = code_point;
        let code_point = code_point as u16;

        self.glyph_cache
            .borrow_mut()
            .entry(code_point)
            .or_insert_with(|| self.font_renderer.render_glyph(character));

        let glyph = Ref::filter_map(self.glyph_cache.borrow(), |v| {
            v.get(&code_point).unwrap_or(&None).as_ref()
        })
        .ok();

        glyph.map(GlyphRef::Ref)
    }

    pub fn get_kerning_offset(&self, left: char, right: char) -> Twips {
        let (Ok(left_cp), Ok(right_cp)) = (left.try_into(), right.try_into()) else {
            return Twips::ZERO;
        };

        *self
            .kerning_cache
            .borrow_mut()
            .entry((left_cp, right_cp))
            .or_insert_with(|| self.font_renderer.calculate_kerning(left, right))
    }
}
