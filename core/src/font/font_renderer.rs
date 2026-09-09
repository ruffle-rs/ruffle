use crate::font::{FontAtlases, FontMetrics, Glyph, glyph::GlyphRef};
use std::cell::{Cell, Ref, RefCell};
use swf::Twips;

pub trait FontRenderer: std::fmt::Debug {
    fn scale(&self) -> f32;

    fn get_font_metrics(&self) -> FontMetrics;

    fn has_kerning_info(&self) -> bool;

    fn render_glyph(&self, character: char) -> Option<Glyph>;

    fn calculate_kerning(&self, left: char, right: char) -> Twips;

    fn atlases(&self) -> Option<&FontAtlases> {
        None
    }

    /// Like `render_glyph`, but rasterizes the glyph at exactly the requested
    /// pixel size. Pixel-locked renderers (e.g. GDI) override this and produce
    /// glyphs whose bitmaps map 1:1 to display pixels at that size, avoiding
    /// the blur introduced by scaling a single canonical-size raster up or
    /// down. The returned `Glyph` should set its `intrinsic_scale` so that the
    /// layout uses scale=1.0 at the requested size.
    ///
    /// Default delegates to size-agnostic `render_glyph`; size-unaware
    /// renderers (e.g. canvas) keep their existing single-cache behavior.
    fn render_glyph_at_size(&self, character: char, _height_px: u32) -> Option<Glyph> {
        self.render_glyph(character)
    }

    /// Like `calculate_kerning` but for a specific raster size. Defaults to
    /// the size-agnostic version.
    fn calculate_kerning_at_size(&self, left: char, right: char, _height_px: u32) -> Twips {
        self.calculate_kerning(left, right)
    }

    /// Font-wide metrics measured at a specific raster size, in twips, with
    /// `scale` equal to that size so the layout consumes them 1:1.
    ///
    /// Size-locked renderers (e.g. GDI) return metrics that are exact at the
    /// requested pixel size — whole device pixels, matching the glyph
    /// bitmaps — so every layout position derived from them (line heights,
    /// baselines, the text measurements reported to ActionScript) lands on
    /// the pixel grid, like Flash Player device fonts did on Windows.
    ///
    /// `None` (the default) means the renderer has no per-size metrics: the
    /// consumer falls back to `get_font_metrics`, i.e. canonical-size
    /// metrics scaled linearly by the caller.
    fn get_font_metrics_at_size(&self, _height_px: u32) -> Option<FontMetrics> {
        None
    }

    /// Typographic font metrics (OS/2 `sTypoAscender`/`sTypoDescender`) at a
    /// raster size, in twips. The Flash Text Engine reports these to
    /// ActionScript for measuring text, matching Flash Player; classic text
    /// fields keep the taller cell metrics from `get_font_metrics_at_size`.
    /// `None` (the default) means the renderer exposes no typographic metrics
    /// and FTE falls back to the cell metrics.
    fn get_typo_font_metrics(&self, _height_px: u32) -> Option<FontMetrics> {
        None
    }

    /// Whether `render_glyph_at_size` produces a different result for each
    /// size (and therefore deserves a separate cache entry per size).
    /// Defaults to `false`: the consumer caches one entry per code point.
    fn is_size_aware(&self) -> bool {
        false
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

    /// Maps `(code point, height_px)` to glyphs rendered by the renderer.
    /// Size-unaware renderers share the `0` slot; size-aware ones (e.g. GDI)
    /// cache a distinct raster per requested pixel size.
    glyph_cache: CacheMap<(u16, u32), Option<Glyph>>,

    /// Maps `(left, right, height_px)` to kerning provided by the renderer.
    kerning_cache: CacheMap<(u16, u16, u32), Twips>,

    /// Maps `height_px` to per-size font metrics (size-aware renderers only).
    metrics_cache: RefCell<fnv::FnvHashMap<u32, FontMetrics>>,

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
            metrics_cache: RefCell::new(fnv::FnvHashMap::default()),
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

    pub fn get_by_code_point(&self, code_point: char, height_px: u32) -> Option<GlyphRef<'_>> {
        let character = code_point;
        let code_point = code_point as u16;
        // Size-unaware renderers share the `0` slot so we don't render the same
        // glyph at every size; size-aware ones cache per requested pixel size.
        let cache_size = if self.font_renderer.is_size_aware() {
            height_px
        } else {
            0
        };
        let key = (code_point, cache_size);

        let mut cache = self.glyph_cache.borrow_mut();
        let entry = cache.entry(key).or_insert_with(|| {
            self.sweep_caches.set(true);
            CachedValue {
                value: self
                    .font_renderer
                    .render_glyph_at_size(character, cache_size),
                used: false,
            }
        });
        entry.used = true;
        drop(cache);

        let glyph = Ref::filter_map(self.glyph_cache.borrow(), |v| {
            v.get(&key).and_then(|entry| entry.value.as_ref())
        })
        .ok();

        glyph.map(GlyphRef::Ref)
    }

    pub fn get_kerning_offset(&self, left: char, right: char, height_px: u32) -> Twips {
        let (Ok(left_cp), Ok(right_cp)) = (left.try_into(), right.try_into()) else {
            return Twips::ZERO;
        };
        let cache_size = if self.font_renderer.is_size_aware() {
            height_px
        } else {
            0
        };

        let mut cache = self.kerning_cache.borrow_mut();
        let entry = cache
            .entry((left_cp, right_cp, cache_size))
            .or_insert_with(|| {
                self.sweep_caches.set(true);
                CachedValue {
                    value: self
                        .font_renderer
                        .calculate_kerning_at_size(left, right, cache_size),
                    used: false,
                }
            });
        entry.used = true;
        entry.value
    }

    /// Font-wide metrics at a requested pixel size. Size-aware renderers return
    /// per-size metrics (cached per size); others fall back to the canonical
    /// `get_font_metrics`.
    pub fn metrics_at(&self, height_px: u32) -> FontMetrics {
        if height_px > 0 && self.font_renderer.is_size_aware() {
            return *self
                .metrics_cache
                .borrow_mut()
                .entry(height_px)
                .or_insert_with(|| {
                    self.font_renderer
                        .get_font_metrics_at_size(height_px)
                        .unwrap_or_else(|| self.font_renderer.get_font_metrics())
                });
        }
        self.font_renderer.get_font_metrics()
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
