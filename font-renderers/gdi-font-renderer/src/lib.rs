//! Native Node.js addon (napi-rs) that exposes Windows GDI glyph
//! rasterization to the Ruffle WASM core as a pluggable
//! `FontBridge`, selected by the embedding host via
//! `deviceFontRenderer: "custom"` after the bridge object has been
//! assigned to `globalThis.__ruffleCustomFontRenderer`.
//!
//! # Responsibilities
//!
//! Ruffle hands this addon a family / bold / italic triplet and, later,
//! asks for glyph bitmaps at specific pixel sizes. Everything native —
//! HDC / HFONT lifetime, per-size caching, kerning-table extraction —
//! lives here, so the Rust side of Ruffle stays a thin, backend-agnostic
//! proxy.
//!
//! # Shape exposed to JS
//!
//! The module exports:
//!   - `createRenderer(family, bold, italic)` — factory matching the
//!     `FontBridge` contract Ruffle expects.
//!   - `GdiFontRenderer` — a class instance returned by the factory,
//!     also available via `new GdiFontRenderer(...)` for callers that
//!     prefer direct construction.
//!
//! The instance methods form the `CustomFontRenderer` contract: they
//! return all sizes in whole pixels, letting the Rust proxy convert
//! into twips.

#![cfg(windows)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;

use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    CLIP_DEFAULT_PRECIS, CreateCompatibleDC, CreateFontW, DEFAULT_CHARSET, DEFAULT_PITCH,
    DeleteDC, DeleteObject, FF_DONTCARE, FIXED, FW_BOLD, FW_NORMAL, GGO_BITMAP, GGO_METRICS,
    GLYPHMETRICS, GetCharWidth32W, GetGlyphOutlineW, GetKerningPairsW, GetTextMetricsW, HDC,
    HFONT, HGDIOBJ, KERNINGPAIR, MAT2, NONANTIALIASED_QUALITY, OUT_DEFAULT_PRECIS, SelectObject,
    TEXTMETRICW,
};

/// `GetGlyphOutlineW` returns `u32::MAX` (GDI_ERROR) on failure. The
/// Windows crate exposes `GDI_ERROR` under the `GDI_REGION_TYPE` wrapper
/// (i32-based), so we keep a typed u32 constant locally for comparing the
/// raw return value.
const GDI_ERROR: u32 = u32::MAX;

/// Font-wide metrics reported to the bridge, in whole pixels at the
/// requested rasterization size.
#[napi(object)]
pub struct GdiFontMetrics {
    pub ascent: i32,
    pub descent: i32,
    pub leading: i32,
}

/// A rasterized glyph shaped so Ruffle's bitmap layout can place it
/// directly: `width = max(blackBox, 1)`, `height = ascent + descent`,
/// baseline at `y = ascent`.
#[napi(object)]
pub struct GdiGlyphRaster {
    pub width: u32,
    pub height: u32,
    /// Horizontal offset from the current pen position to the bitmap's
    /// left edge, in pixels. Negative when the glyph overhangs to the
    /// left of the origin.
    pub bitmap_tx: i32,
    /// Glyph advance in pixels.
    pub advance: i32,
    /// RGBA bytes, row-major, top-down. Non-ink pixels are
    /// `(0, 0, 0, 0)`.
    pub pixels: Buffer,
}

/// One HDC+HFONT pair matching a specific rasterization size, together
/// with the metrics and kerning table scraped from it once.
struct SizedFont {
    hdc: HDC,
    hfont: HFONT,
    prev_obj: HGDIOBJ,
    ascent: i32,
    descent: i32,
    leading: i32,
    kerning: HashMap<u64, i32>,
}

// GDI handles are per-process; concurrent access is prevented by
// `GdiFontRenderer::inner`'s mutex (and napi-rs dispatches instance
// methods on the Node main thread in any case).
unsafe impl Send for SizedFont {}

impl Drop for SizedFont {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.hdc, self.prev_obj);
            let _ = DeleteObject(HGDIOBJ(self.hfont.0));
            let _ = DeleteDC(self.hdc);
        }
    }
}

/// Mutable state behind `GdiFontRenderer`. Extracted into its own struct
/// so the class can hold it behind a single `Mutex`.
struct Inner {
    /// Null-terminated UTF-16 font family, ready to pass to
    /// `CreateFontW`.
    family_wide: Vec<u16>,
    bold: bool,
    italic: bool,
    /// Lazily-populated pool, one `SizedFont` per requested pixel size.
    sized_fonts: HashMap<u32, SizedFont>,
}

impl Inner {
    fn new(family: String, bold: bool, italic: bool) -> Self {
        let mut family_wide: Vec<u16> = family.encode_utf16().collect();
        family_wide.push(0);
        Self {
            family_wide,
            bold,
            italic,
            sized_fonts: HashMap::new(),
        }
    }

    /// Borrow the `SizedFont` for `size_px`, creating it on first access.
    fn get_or_create(&mut self, size_px: u32) -> Result<&SizedFont> {
        use std::collections::hash_map::Entry;
        if size_px == 0 {
            return Err(Error::from_reason("size_px must be > 0"));
        }
        match self.sized_fonts.entry(size_px) {
            Entry::Occupied(e) => Ok(e.into_mut()),
            Entry::Vacant(e) => {
                let font = create_sized_font(&self.family_wide, self.bold, self.italic, size_px)?;
                Ok(e.insert(font))
            }
        }
    }
}

/// Windows GDI font renderer.
///
/// Exposed to JS as the class returned from
/// [`create_renderer`]; it is also directly constructible as
/// `new GdiFontRenderer(...)` for callers that want to bypass the
/// factory.
///
/// Internally owns a pool of `SizedFont`s, created on demand when Ruffle
/// first asks for a given pixel size. All native resources are released
/// either by an explicit [`destroy`](Self::destroy) call or by the drop
/// of the enclosing JS object.
#[napi]
pub struct GdiFontRenderer {
    inner: Mutex<Inner>,
}

#[napi]
impl GdiFontRenderer {
    /// Construct a renderer for a given font family/style. No native
    /// handle is created here — the first `getFontMetrics` / `renderGlyph`
    /// call spawns the first HFONT for the requested size.
    #[napi(constructor)]
    pub fn new(family: String, bold: bool, italic: bool) -> Self {
        Self {
            inner: Mutex::new(Inner::new(family, bold, italic)),
        }
    }

    /// Whether this renderer supplies kerning pairs. We always report
    /// `true`: pairs without a dedicated entry return 0 from
    /// [`calculate_kerning`](Self::calculate_kerning), which is
    /// indistinguishable from "no kerning info" for that pair, and
    /// answering truthfully would require spawning a native HFONT just
    /// to read the kerning table.
    #[napi]
    pub fn has_kerning_info(&self) -> bool {
        true
    }

    /// Font-wide metrics at `size_px` pixels, in pixel units.
    #[napi]
    pub fn get_font_metrics(&self, size_px: u32) -> Result<GdiFontMetrics> {
        let mut inner = self.lock_inner()?;
        let font = inner.get_or_create(size_px)?;
        Ok(GdiFontMetrics {
            ascent: font.ascent,
            descent: font.descent,
            leading: font.leading,
        })
    }

    /// Rasterize a single glyph at `size_px` pixels. Returns `None` when
    /// GDI recognizes neither a glyph outline nor an advance for the
    /// code point.
    #[napi]
    pub fn render_glyph(&self, code_point: u32, size_px: u32) -> Result<Option<GdiGlyphRaster>> {
        let mut inner = self.lock_inner()?;
        let font = inner.get_or_create(size_px)?;
        render_glyph_with(font, code_point)
    }

    /// Kerning between `left` and `right` at `size_px` pixels, in pixel
    /// units. Unknown pairs return 0.
    #[napi]
    pub fn calculate_kerning(&self, left: u32, right: u32, size_px: u32) -> Result<i32> {
        let mut inner = self.lock_inner()?;
        let font = inner.get_or_create(size_px)?;
        Ok(font
            .kerning
            .get(&kerning_key(left, right))
            .copied()
            .unwrap_or(0))
    }

    /// Release every native resource this renderer owns. Safe to call
    /// more than once; subsequent calls are no-ops until a new size is
    /// requested, which would spawn fresh handles.
    #[napi]
    pub fn destroy(&self) -> Result<()> {
        let mut inner = self.lock_inner()?;
        // Dropping each `SizedFont` releases its HDC/HFONT.
        inner.sized_fonts.clear();
        Ok(())
    }

    fn lock_inner(&self) -> Result<std::sync::MutexGuard<'_, Inner>> {
        self.inner
            .lock()
            .map_err(|_| Error::from_reason("GdiFontRenderer state poisoned"))
    }
}

/// Bridge factory, matching the `FontBridge.createRenderer` contract that
/// Ruffle's Rust-side proxy expects. Ruffle treats the module itself as
/// the bridge object, so a top-level `createRenderer` export is all the
/// bridge contract really needs.
#[napi]
pub fn create_renderer(family: String, bold: bool, italic: bool) -> GdiFontRenderer {
    GdiFontRenderer::new(family, bold, italic)
}

fn identity_mat2() -> MAT2 {
    let one = FIXED { fract: 0, value: 1 };
    let zero = FIXED { fract: 0, value: 0 };
    MAT2 {
        eM11: one,
        eM12: zero,
        eM21: zero,
        eM22: one,
    }
}

fn kerning_key(left: u32, right: u32) -> u64 {
    ((left as u64) << 32) | (right as u64)
}

fn create_sized_font(
    family_wide: &[u16],
    bold: bool,
    italic: bool,
    size_px: u32,
) -> Result<SizedFont> {
    unsafe {
        let hdc = CreateCompatibleDC(None);
        if hdc.is_invalid() {
            return Err(Error::from_reason("CreateCompatibleDC failed"));
        }

        let hfont = CreateFontW(
            -(size_px as i32),
            0,
            0,
            0,
            if bold { FW_BOLD.0 } else { FW_NORMAL.0 } as i32,
            u32::from(italic),
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            NONANTIALIASED_QUALITY.0 as u32,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            PCWSTR(family_wide.as_ptr()),
        );
        if hfont.is_invalid() {
            let _ = DeleteDC(hdc);
            return Err(Error::from_reason("CreateFontW failed"));
        }

        let prev_obj = SelectObject(hdc, HGDIOBJ(hfont.0));

        let mut tm = TEXTMETRICW::default();
        if !GetTextMetricsW(hdc, &mut tm).as_bool() {
            SelectObject(hdc, prev_obj);
            let _ = DeleteObject(HGDIOBJ(hfont.0));
            let _ = DeleteDC(hdc);
            return Err(Error::from_reason("GetTextMetricsW failed"));
        }

        let mut kerning = HashMap::new();
        let n_pairs = GetKerningPairsW(hdc, None);
        if n_pairs > 0 {
            let mut pairs = vec![KERNINGPAIR::default(); n_pairs as usize];
            let got = GetKerningPairsW(hdc, Some(pairs.as_mut_slice()));
            for p in pairs.iter().take(got as usize) {
                kerning.insert(
                    kerning_key(p.wFirst as u32, p.wSecond as u32),
                    p.iKernAmount as i32,
                );
            }
        }

        Ok(SizedFont {
            hdc,
            hfont,
            prev_obj,
            ascent: tm.tmAscent,
            descent: tm.tmDescent,
            leading: tm.tmExternalLeading,
            kerning,
        })
    }
}

fn render_glyph_with(font: &SizedFont, code_point: u32) -> Result<Option<GdiGlyphRaster>> {
    let mat = identity_mat2();
    let mut gm = GLYPHMETRICS::default();
    let bitmap_height = (font.ascent + font.descent).max(1) as u32;

    unsafe {
        let probe = GetGlyphOutlineW(font.hdc, code_point, GGO_METRICS, &mut gm, 0, None, &mat);
        if probe == GDI_ERROR {
            // GDI refused to give us metrics for this code point. This is
            // the common case for whitespace in some fonts: they have no
            // ink but do have an advance, and the layout pipeline still
            // needs a Glyph with the right `advance` so the next
            // character is not glued to the previous one. Try
            // `GetCharWidth32W` as a last resort and synthesize a
            // transparent placeholder if it succeeds.
            return Ok(blank_glyph_with_advance(
                font.hdc,
                code_point,
                bitmap_height,
            ));
        }

        let black_box_w = gm.gmBlackBoxX;
        let black_box_h = gm.gmBlackBoxY;

        let out_width = black_box_w.max(1);
        let out_height = bitmap_height;
        let mut pixels = vec![0u8; (out_width as usize) * (out_height as usize) * 4];

        if black_box_w > 0 && black_box_h > 0 {
            // GGO_BITMAP returns a 1-bit packed bitmap, MSB-first within
            // each byte, with rows padded to a 32-bit boundary.
            let stride = ((black_box_w as usize) + 31) / 32 * 4;
            let mono_size = stride * (black_box_h as usize);
            let mut mono = vec![0u8; mono_size];

            let written = GetGlyphOutlineW(
                font.hdc,
                code_point,
                GGO_BITMAP,
                &mut gm,
                mono_size as u32,
                Some(mono.as_mut_ptr() as *mut _),
                &mat,
            );
            if written == GDI_ERROR {
                return Ok(blank_glyph_with_advance(
                    font.hdc,
                    code_point,
                    bitmap_height,
                ));
            }

            // Baseline sits at y = ascent in the output bitmap.
            // `gmptGlyphOrigin.y` is the distance from the baseline up to
            // the top of the black box, so the top of the black box lands
            // at `(ascent - gmptGlyphOrigin.y)`.
            let top_row = font.ascent - gm.gmptGlyphOrigin.y;

            for row in 0..black_box_h {
                let dst_row = top_row + row as i32;
                if dst_row < 0 || dst_row >= out_height as i32 {
                    continue;
                }
                let src_off = (row as usize) * stride;
                let dst_off = (dst_row as usize) * (out_width as usize) * 4;
                for col in 0..black_box_w {
                    let byte = mono[src_off + (col as usize) / 8];
                    let bit = byte & (0x80 >> (col % 8));
                    if bit == 0 {
                        continue;
                    }
                    // `BitmapFormat::Rgba` on Ruffle's side is
                    // premultiplied, so an opaque white pixel is
                    // `(255, 255, 255, 255)`. In this aliased path every
                    // "ink" bit is fully opaque.
                    let px = dst_off + (col as usize) * 4;
                    pixels[px] = 255;
                    pixels[px + 1] = 255;
                    pixels[px + 2] = 255;
                    pixels[px + 3] = 255;
                }
            }
        }

        Ok(Some(GdiGlyphRaster {
            width: out_width,
            height: out_height,
            bitmap_tx: gm.gmptGlyphOrigin.x,
            advance: gm.gmCellIncX as i32,
            pixels: Buffer::from(pixels),
        }))
    }
}

/// Build a fully-transparent placeholder bitmap whose only purpose is to
/// carry the correct advance for a glyph that GDI refused to rasterize
/// (typically whitespace). Returns `None` if even `GetCharWidth32W`
/// doesn't know about the code point — the layout pipeline then treats
/// it as missing.
unsafe fn blank_glyph_with_advance(
    hdc: HDC,
    code_point: u32,
    bitmap_height: u32,
) -> Option<GdiGlyphRaster> {
    let mut width: i32 = 0;
    let ok = GetCharWidth32W(hdc, code_point, code_point, &mut width as *mut i32);
    if !ok.as_bool() || width <= 0 {
        return None;
    }
    let pixels = vec![0u8; (bitmap_height as usize) * 4];
    Some(GdiGlyphRaster {
        width: 1,
        height: bitmap_height,
        bitmap_tx: 0,
        advance: width,
        pixels: Buffer::from(pixels),
    })
}
