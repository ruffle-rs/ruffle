use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::atlas::{BitmapAtlas, BitmapAtlasRegion};
use ruffle_render::bitmap::{Bitmap, BitmapFormat, BitmapHandle, BitmapSize, PixelRegion};
use std::cell::RefCell;
use std::rc::Rc;
use swf::Twips;

struct FontAtlasData {
    size: BitmapSize,
    format: BitmapFormat,

    /// All atlas pages created so far, in the order they were created. A fresh
    /// page is only added once none of the existing ones fit.
    pages: Vec<BitmapAtlas>,
}

/// Font atlas takes care of allocating bitmaps and managing glyphs on them.
///
/// It supports unbounded, variable-size glyphs spread across atlas pages with
/// an option to deallocate them when they are not used.
#[derive(Clone)]
pub struct FontAtlas(Rc<RefCell<FontAtlasData>>);

impl FontAtlas {
    pub fn new(size: BitmapSize, format: BitmapFormat) -> Self {
        Self(Rc::new(RefCell::new(FontAtlasData {
            size,
            format,
            pages: Vec::new(),
        })))
    }

    /// A pointer uniquely identifying this atlas instance, for debug/inspection
    /// purposes.
    pub fn as_ptr(&self) -> *const () {
        Rc::as_ptr(&self.0) as *const ()
    }

    /// Allocates space for `bitmap` in the atlas and copies it in, starting a
    /// fresh atlas page if none of the existing ones have room left.
    pub fn new_glyph(&self, bitmap: Bitmap<'_>, tx: Twips, ty: Twips) -> FontAtlasGlyph {
        // Do not allocate space in font atlases for empty bitmaps, use empty glyphs instead.
        assert!(bitmap.width() != 0 && bitmap.height() != 0);

        let mut data = self.0.borrow_mut();

        let atlas_region = match data.pages.iter().find_map(|page| allocate(page, &bitmap)) {
            Some(atlas_region) => atlas_region,
            None => {
                // None of the existing pages have room; start a fresh one, sized to
                // guarantee it fits `bitmap` (plus its margin) even if it's larger
                // than the configured size.
                let width = data.size.width.max(bitmap.width() + GLYPH_MARGIN * 2);
                let height = data.size.height.max(bitmap.height() + GLYPH_MARGIN * 2);
                let new_page = BitmapAtlas::new(width, height, data.format);
                let atlas_region = allocate(&new_page, &bitmap)
                    .expect("a fresh page sized to fit `bitmap` should fit `bitmap`");
                data.pages.push(new_page);
                atlas_region
            }
        };

        let region = trim_margin(&atlas_region);

        FontAtlasGlyph(Rc::new(FontAtlasGlyphData {
            atlas_region,
            region,
            tx,
            ty,
        }))
    }

    /// All atlas pages, for debug/inspection purposes.
    pub fn pages(&self) -> Vec<BitmapAtlas> {
        self.0.borrow().pages.clone()
    }
}

impl std::fmt::Debug for FontAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontAtlas")
            .field("pages", &self.0.borrow().pages)
            .finish()
    }
}

/// Empty space reserved around each glyph's pixels in the atlas, so that
/// texture filtering at the glyph's edge can't sample an unrelated
/// neighboring glyph packed right next to it.
const GLYPH_MARGIN: u32 = 1;

fn allocate(atlas: &BitmapAtlas, bitmap: &Bitmap<'_>) -> Option<BitmapAtlasRegion> {
    let padded_width = bitmap.width() + GLYPH_MARGIN * 2;
    let padded_height = bitmap.height() + GLYPH_MARGIN * 2;

    let region = atlas.allocate(padded_width, padded_height, true)?;
    region.set_region(bitmap, GLYPH_MARGIN, GLYPH_MARGIN);
    Some(region)
}

/// The glyph's own pixels within its atlas allocation, excluding the margin
/// reserved around them.
fn trim_margin(atlas_region: &BitmapAtlasRegion) -> PixelRegion {
    let region = atlas_region.region();
    PixelRegion {
        x_min: region.x_min + GLYPH_MARGIN,
        y_min: region.y_min + GLYPH_MARGIN,
        x_max: region.x_max - GLYPH_MARGIN,
        y_max: region.y_max - GLYPH_MARGIN,
    }
}

struct FontAtlasGlyphData {
    atlas_region: BitmapAtlasRegion,

    /// The glyph's own pixels within the atlas, excluding the margin
    /// reserved around them. Held in an `Rc` so it can be handed out as a
    /// `Weak` alongside render commands: once this glyph is dropped, that
    /// `Weak` can no longer be upgraded, even if the atlas later reuses the
    /// same coordinates for an unrelated glyph. Backends use this to know
    /// when a resource they cached keyed on the rectangle (e.g. a bind
    /// group) has gone stale.
    region: PixelRegion,

    tx: Twips,
    ty: Twips,
}

#[derive(Clone)]
pub struct FontAtlasGlyph(Rc<FontAtlasGlyphData>);

impl FontAtlasGlyph {
    pub fn tx(&self) -> Twips {
        self.0.tx
    }

    pub fn ty(&self) -> Twips {
        self.0.ty
    }

    pub fn atlas_handle(&self, renderer: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        self.0.atlas_region.atlas().handle(renderer)
    }

    /// The glyph's own pixels within the atlas, excluding the margin reserved
    /// around them.
    pub fn atlas_region(&self) -> PixelRegion {
        self.0.region
    }
}

impl std::fmt::Debug for FontAtlasGlyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontAtlasGlyph")
            .field("atlas", self.0.atlas_region.atlas())
            .field("atlas_region", &self.atlas_region())
            .field("tx", &self.0.tx)
            .field("ty", &self.0.ty)
            .finish()
    }
}

/// The page size used for atlases created through [`FontAtlases`].
const SHARED_ATLAS_PAGE_SIZE: BitmapSize = BitmapSize {
    width: 1024,
    height: 1024,
};

/// A small, shared, global set of font atlases.
///
/// There's one atlas per pixel format rather than per font, since different
/// fonts producing glyphs of the same format can happily share pages. Each
/// atlas starts out empty, so eagerly creating all of them costs nothing
/// until a glyph actually gets placed in one.
#[derive(Debug, Clone)]
pub struct FontAtlases {
    rgba: FontAtlas,
    // TODO Add grayscale atlas?
}

impl FontAtlases {
    pub fn new() -> Self {
        Self {
            rgba: FontAtlas::new(SHARED_ATLAS_PAGE_SIZE, BitmapFormat::Rgba),
        }
    }

    /// The shared atlas for RGBA glyphs.
    pub fn rgba(&self) -> FontAtlas {
        self.rgba.clone()
    }
}

impl Default for FontAtlases {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ruffle_render::backend::ViewportDimensions;
    use ruffle_render::backend::null::NullRenderer;

    fn null_renderer() -> NullRenderer {
        NullRenderer::new(ViewportDimensions {
            width: 100,
            height: 80,
            scale_factor: 1.0,
        })
    }

    fn font_atlas(width: u32, height: u32) -> FontAtlas {
        FontAtlas::new(BitmapSize { width, height }, BitmapFormat::Rgba)
    }

    fn solid_bitmap(width: u32, height: u32, format: BitmapFormat, fill: u8) -> Bitmap<'static> {
        let len = format.length_for_size(width as usize, height as usize);
        Bitmap::new(width, height, format, vec![fill; len])
    }

    /// Adds a glyph of solid `fill` at the origin, the shape most of these
    /// tests care about.
    fn add_glyph(atlas: &FontAtlas, width: u32, height: u32, fill: u8) -> FontAtlasGlyph {
        atlas.new_glyph(
            solid_bitmap(width, height, BitmapFormat::Rgba, fill),
            Twips::ZERO,
            Twips::ZERO,
        )
    }

    /// Expands one value per pixel into RGBA data. All test glyphs are solid
    /// fills, so a whole page can be written out one value per pixel.
    fn expand_pixels(pixels: &[u8]) -> Vec<u8> {
        pixels.iter().flat_map(|&value| [value; 4]).collect()
    }

    /// Index of the page holding `glyph`, identified by its texture handle.
    fn page_index_of(atlas: &FontAtlas, glyph: &FontAtlasGlyph) -> usize {
        let mut renderer = null_renderer();
        let handle = glyph
            .atlas_handle(&mut renderer)
            .expect("glyph's page should register");
        atlas
            .pages()
            .iter()
            .position(|page| page.handle(&mut renderer).as_ref() == Some(&handle))
            .expect("glyph should live on one of the atlas's pages")
    }

    #[test]
    fn new_atlas_has_no_pages() {
        let atlas = font_atlas(8, 8);
        assert!(atlas.pages().is_empty());
    }

    #[test]
    fn first_glyph_creates_a_page() {
        let atlas = FontAtlas::new(
            BitmapSize {
                width: 16,
                height: 8,
            },
            BitmapFormat::Rgb,
        );
        let _glyph = atlas.new_glyph(
            solid_bitmap(2, 2, BitmapFormat::Rgb, 0x33),
            Twips::ZERO,
            Twips::ZERO,
        );

        let pages = atlas.pages();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].format(), BitmapFormat::Rgb);

        let page = pages[0].to_rgba();
        assert_eq!(page.width(), 16);
        assert_eq!(page.height(), 8);
    }

    #[test]
    fn glyph_region_matches_the_bitmap_size() {
        let atlas = font_atlas(64, 64);
        let glyph = add_glyph(&atlas, 5, 3, 0xAB);

        let region = glyph.atlas_region();
        assert_eq!(region.width(), 5);
        assert_eq!(region.height(), 3);
    }

    #[test]
    #[should_panic(expected = "Can't copy a region between bitmaps of different formats")]
    fn bad_bitmap_format() {
        let atlas = font_atlas(8, 8);

        // The atlas is RGBA, so a glyph in another format can't be copied in.
        atlas.new_glyph(
            solid_bitmap(2, 2, BitmapFormat::Rgb, 0xAB),
            Twips::ZERO,
            Twips::ZERO,
        );
    }

    #[test]
    fn glyph_placement() {
        let atlas = font_atlas(10, 5);
        let first = add_glyph(&atlas, 3, 3, 0x11);
        let second = add_glyph(&atlas, 3, 3, 0x22);

        assert_eq!(atlas.pages().len(), 1, "both glyphs should share one page");
        assert_eq!(
            first.atlas_region(),
            PixelRegion {
                x_min: 1,
                y_min: 1,
                x_max: 4,
                y_max: 4,
            }
        );
        assert_eq!(
            second.atlas_region(),
            PixelRegion {
                x_min: 6,
                y_min: 1,
                x_max: 9,
                y_max: 4,
            }
        );

        // The whole page: both glyphs intact, with blank margin between them
        // and along the page edges, so neither glyph bleeds into the other's
        // filtering neighborhood.
        let page = atlas.pages()[0].to_rgba();
        #[rustfmt::skip]
        let expected = expand_pixels(&[
            0x00, 0x00, 0x00, 0x00, 0x00,   0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x11, 0x11, 0x11, 0x00,   0x00, 0x22, 0x22, 0x22, 0x00,
            0x00, 0x11, 0x11, 0x11, 0x00,   0x00, 0x22, 0x22, 0x22, 0x00,
            0x00, 0x11, 0x11, 0x11, 0x00,   0x00, 0x22, 0x22, 0x22, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,   0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(page.data(), expected);
    }

    #[test]
    fn margins_are_cleared() {
        let atlas = font_atlas(6, 4);

        // Fills the whole page, leaving its pixels behind when dropped.
        let first = add_glyph(&atlas, 4, 2, 0x11);
        assert_eq!(
            first.atlas_region(),
            PixelRegion {
                x_min: 1,
                y_min: 1,
                x_max: 5,
                y_max: 3,
            }
        );
        drop(first);

        #[rustfmt::skip]
        let expected = expand_pixels(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x11, 0x11, 0x11, 0x11, 0x00,
            0x00, 0x11, 0x11, 0x11, 0x11, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(atlas.pages()[0].to_rgba().data(), expected);

        // Reuses the left part of the space the first glyph occupied, so its
        // margin falls on pixels the first glyph had written.
        let second = add_glyph(&atlas, 2, 2, 0x22);
        assert_eq!(
            second.atlas_region(),
            PixelRegion {
                x_min: 1,
                y_min: 1,
                x_max: 3,
                y_max: 3,
            }
        );

        // The margin around the second glyph is blank despite the first
        // glyph's pixels having been there. Only the reused space is cleared
        // though; the rest of the page keeps the leftovers.
        #[rustfmt::skip]
        let expected = expand_pixels(&[
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x22, 0x22, 0x00, 0x11, 0x00,
            0x00, 0x22, 0x22, 0x00, 0x11, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(atlas.pages()[0].to_rgba().data(), expected);
    }

    #[test]
    fn new_page_created_when_no_space() {
        let atlas = font_atlas(8, 8);

        let first = add_glyph(&atlas, 5, 5, 0x11);
        assert_eq!(atlas.pages().len(), 1);

        let second = add_glyph(&atlas, 1, 1, 0x22);
        assert_eq!(atlas.pages().len(), 2);
        assert_eq!(page_index_of(&atlas, &first), 0);
        assert_eq!(page_index_of(&atlas, &second), 1);
    }

    #[test]
    fn glyphs_go_to_the_earliest_page() {
        let atlas = font_atlas(16, 16);
        let first = add_glyph(&atlas, 4, 4, 0x11);

        // Too large for the first page, so it gets a page of its own.
        let oversized = add_glyph(&atlas, 14, 14, 0x22);
        assert_eq!(atlas.pages().len(), 2);

        // The first page still has plenty of room, so this goes back to it
        // rather than onto the most recently created page.
        let third = add_glyph(&atlas, 2, 2, 0x33);

        assert_eq!(page_index_of(&atlas, &first), 0);
        assert_eq!(page_index_of(&atlas, &oversized), 1);
        assert_eq!(page_index_of(&atlas, &third), 0);
        assert_eq!(atlas.pages().len(), 2, "no third page should be needed");
    }

    #[test]
    fn glyph_larger_than_page() {
        let atlas = font_atlas(4, 4);
        let first_big = add_glyph(&atlas, 3, 3, 0x11);
        let second = add_glyph(&atlas, 1, 1, 0x22);
        let third_big = add_glyph(&atlas, 3, 3, 0x33);

        assert_eq!(atlas.pages().len(), 3);

        assert_eq!(page_index_of(&atlas, &first_big), 0);
        assert_eq!(page_index_of(&atlas, &second), 1);
        assert_eq!(page_index_of(&atlas, &third_big), 2);

        let page0 = atlas.pages()[0].to_rgba();
        let page1 = atlas.pages()[1].to_rgba();
        let page2 = atlas.pages()[2].to_rgba();
        assert_eq!(page0.width(), 5);
        assert_eq!(page0.height(), 5);
        assert_eq!(page1.width(), 4);
        assert_eq!(page1.height(), 4);
        assert_eq!(page2.width(), 5);
        assert_eq!(page2.height(), 5);

        #[rustfmt::skip]
        let expected0 = expand_pixels(&[
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x11, 0x11, 0x11, 0x00,
            0x00, 0x11, 0x11, 0x11, 0x00,
            0x00, 0x11, 0x11, 0x11, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(page0.data(), expected0);

        #[rustfmt::skip]
        let expected1 = expand_pixels(&[
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x22, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(page1.data(), expected1);

        #[rustfmt::skip]
        let expected2 = expand_pixels(&[
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x33, 0x33, 0x33, 0x00,
            0x00, 0x33, 0x33, 0x33, 0x00,
            0x00, 0x33, 0x33, 0x33, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(page2.data(), expected2);
    }

    #[test]
    #[should_panic]
    fn zero_sized_glyph() {
        let atlas = font_atlas(8, 8);
        atlas.new_glyph(Bitmap::empty(BitmapFormat::Rgba), Twips::ZERO, Twips::ZERO);
    }

    #[test]
    fn dropping_a_glyph_frees_its_space_for_reuse() {
        let atlas = font_atlas(4, 4);
        let glyph = add_glyph(&atlas, 2, 2, 0x11);
        let region = glyph.atlas_region();

        drop(glyph);

        let reused = add_glyph(&atlas, 2, 2, 0x22);
        assert_eq!(
            atlas.pages().len(),
            1,
            "the freed space should be reused instead of starting a new page"
        );
        assert_eq!(reused.atlas_region(), region);

        // Reusing the space overwrites the dropped glyph's pixels entirely,
        // leaving no trace of it anywhere on the page.
        #[rustfmt::skip]
        let expected = expand_pixels(&[
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x22, 0x22, 0x00,
            0x00, 0x22, 0x22, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(atlas.pages()[0].to_rgba().data(), expected);
    }

    #[test]
    fn glyph_translation() {
        let atlas = font_atlas(8, 8);
        let glyph = atlas.new_glyph(
            solid_bitmap(2, 2, BitmapFormat::Rgba, 0xAB),
            Twips::from_pixels(1.5),
            Twips::new(-40),
        );

        assert_eq!(glyph.tx(), Twips::from_pixels(1.5));
        assert_eq!(glyph.ty(), Twips::new(-40));
    }

    #[test]
    fn cloned_atlas_shares_pages_with_the_original() {
        let atlas = font_atlas(4, 4);
        let clone = atlas.clone();

        let glyph = add_glyph(&clone, 2, 2, 0xAB);

        // The glyph added through the clone is visible on the original's page.
        assert_eq!(atlas.pages().len(), 1);
        assert_eq!(
            glyph.atlas_region(),
            PixelRegion {
                x_min: 1,
                y_min: 1,
                x_max: 3,
                y_max: 3,
            }
        );
        #[rustfmt::skip]
        let expected = expand_pixels(&[
            0x00, 0x00, 0x00, 0x00,
            0x00, 0xAB, 0xAB, 0x00,
            0x00, 0xAB, 0xAB, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(atlas.pages()[0].to_rgba().data(), expected);
    }
}
