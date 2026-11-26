use crate::font::{FontAtlases, FontMetrics, Glyph, glyph::GlyphRef};
use std::cell::{Cell, Ref, RefCell};
use swf::Twips;

pub trait FontRenderer: std::fmt::Debug {
    fn scale(&self) -> f32;

    fn get_font_metrics(&self) -> FontMetrics;

    /// Typographic metrics (OS/2 `sTypoAscender`/`sTypoDescender`) at a raster
    /// size, if the renderer can provide them. The Flash Text Engine reports
    /// these to ActionScript (matching Flash Player), so a renderer with access
    /// to the font's OS/2 table should override this; raster renderers work at a
    /// concrete pixel size, hence `height_px`. The default returns `None`, which
    /// falls back to the hhea/cell metrics from
    /// [`FontRenderer::get_font_metrics`].
    fn get_typo_font_metrics(&self, _height_px: u32) -> Option<FontMetrics> {
        None
    }

    fn has_kerning_info(&self) -> bool;

    fn render_glyph(&self, character: char) -> Option<Glyph>;

    fn calculate_kerning(&self, left: char, right: char) -> Twips;

    fn atlases(&self) -> Option<&FontAtlases> {
        None
    }
}

#[derive(Debug)]
pub struct CachedValue<T> {
    value: T,
    used: bool,
}

type CacheMap<K, V> = RefCell<fnv::FnvHashMap<K, CachedValue<V>>>;

#[derive(Debug)]
pub struct FontRendererGlyphSource {
    font_renderer: Box<dyn FontRenderer>,

    /// Maps Unicode code points to glyphs rendered by the renderer.
    glyph_cache: CacheMap<u16, Option<Glyph>>,

    /// Maps Unicode pairs to kerning provided by the renderer.
    kerning_cache: CacheMap<(u16, u16), Twips>,

    sweep_caches: Cell<bool>,
    sweep_count: Cell<usize>,
    swept_glyphs_count: Cell<usize>,
    swept_kerning_count: Cell<usize>,
}

impl FontRendererGlyphSource {
    pub fn new(font_renderer: Box<dyn FontRenderer>) -> Self {
        Self {
            font_renderer,
            glyph_cache: RefCell::new(fnv::FnvHashMap::default()),
            kerning_cache: RefCell::new(fnv::FnvHashMap::default()),
            sweep_caches: Cell::new(false),
            sweep_count: Cell::new(0),
            swept_glyphs_count: Cell::new(0),
            swept_kerning_count: Cell::new(0),
        }
    }

    pub fn glyph_cache_size(&self) -> usize {
        self.glyph_cache.borrow().len()
    }

    pub fn kerning_cache_size(&self) -> usize {
        self.kerning_cache.borrow().len()
    }

    pub fn sweep_count(&self) -> usize {
        self.sweep_count.get()
    }

    pub fn swept_glyphs_count(&self) -> usize {
        self.swept_glyphs_count.get()
    }

    pub fn swept_kerning_count(&self) -> usize {
        self.swept_kerning_count.get()
    }

    pub fn font_renderer(&self) -> &dyn FontRenderer {
        self.font_renderer.as_ref()
    }

    pub fn get_by_code_point(&self, code_point: char) -> Option<GlyphRef<'_>> {
        let character = code_point;
        let code_point = code_point as u16;

        let mut cache = self.glyph_cache.borrow_mut();
        let entry = cache.entry(code_point).or_insert_with(|| {
            self.sweep_caches.set(true);
            CachedValue {
                value: self.font_renderer.render_glyph(character),
                used: false,
            }
        });
        entry.used = true;
        drop(cache);

        let glyph = Ref::filter_map(self.glyph_cache.borrow(), |v| {
            v.get(&code_point).and_then(|entry| entry.value.as_ref())
        })
        .ok();

        glyph.map(GlyphRef::Ref)
    }

    pub fn get_kerning_offset(&self, left: char, right: char) -> Twips {
        let (Ok(left_cp), Ok(right_cp)) = (left.try_into(), right.try_into()) else {
            return Twips::ZERO;
        };

        let mut cache = self.kerning_cache.borrow_mut();
        let entry = cache.entry((left_cp, right_cp)).or_insert_with(|| {
            self.sweep_caches.set(true);
            CachedValue {
                value: self.font_renderer.calculate_kerning(left, right),
                used: false,
            }
        });
        entry.used = true;
        entry.value
    }

    pub fn sweep_caches(&self, force: bool) {
        if force || self.sweep_caches.replace(false) {
            self.sweep_count.set(self.sweep_count.get().wrapping_add(1));

            let swept_glyphs = retain_used(&self.glyph_cache);
            self.swept_glyphs_count
                .set(self.swept_glyphs_count.get().wrapping_add(swept_glyphs));

            let swept_kerning = retain_used(&self.kerning_cache);
            self.swept_kerning_count
                .set(self.swept_kerning_count.get().wrapping_add(swept_kerning));
        }
    }
}

fn retain_used<K, V>(cache: &CacheMap<K, V>) -> usize {
    let mut cache = cache.borrow_mut();
    let before = cache.len();
    cache.retain(|_, entry| std::mem::replace(&mut entry.used, false));
    before - cache.len()
}
