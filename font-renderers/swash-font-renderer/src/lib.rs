//! Pluggable swash-based device font renderer for Ruffle.
//!
//! This crate is compiled to a standalone WebAssembly module via
//! `wasm-pack` and plugged into Ruffle through the
//! `deviceFontRenderer: "custom"` configuration option, with the
//! module's default export assigned to the well-known global slot
//! `globalThis.__ruffleCustomFontRenderer` before Ruffle requests
//! its first device font.
//!
//! # Contract
//!
//! The module top-level exports match the Ruffle `FontBridge`
//! TypeScript contract:
//!
//! - `createRenderer(family, bold, italic) -> CustomFontRenderer | null`
//! - `registerFontData(source, bytes)` — receives the raw bytes of
//!   every entry in the Ruffle `fontSources` config option.
//!
//! The returned `CustomFontRenderer` mirrors the renderer half of the
//! contract: `hasKerningInfo`, `getFontMetrics`, `renderGlyph`,
//! `calculateKerning`, `destroy`.
//!
//! # Architecture
//!
//! 1. `registerFontData` parses each incoming TTF/OTF/TTC/OTC, extracts
//!    the `(family, bold, italic)` key for every face inside, and
//!    stores a shared handle to the bytes + the face offset in a global
//!    registry.
//! 2. `createRenderer` looks up the registry and, on a hit, spawns a
//!    per-instance renderer that owns its own per-pixel-size
//!    rasterization cache. On a miss it returns `null`, letting Ruffle
//!    fall back to the default canvas device-font renderer.
//! 3. All bytes are kept alive via `Rc<Vec<u8>>` so that swash's
//!    zero-copy `FontRef` and ttf-parser's `Face` remain valid for as
//!    long as any renderer needs them.
//!
//! # Rasterization parameters
//!
//! Glyphs are produced as grayscale alpha masks and then fanned out to
//! RGBA premultiplied white (the text is tinted by Ruffle downstream).
//! Hinting is enabled (`Scaler::hint(true)`) for sharper small sizes,
//! matching the quality target of a GDI-style device-font renderer.
//!
//! Ruffle's proxy always calls us with integer pixel sizes (the
//! contract method signatures take `number`, but the Rust proxy
//! converts to `u32` before crossing the wasm-bindgen boundary), so
//! per-size cache entries are keyed by `u32` without any extra
//! quantization step.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use js_sys::{Object, Reflect, Uint8Array};
use swash::FontRef;
use swash::proxy::{CharmapProxy, MetricsProxy};
use swash::scale::image::{Content, Image};
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::Format;
use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------

/// Key used to look up a face inside the registry. Family name is
/// lower-cased so lookup is case-insensitive, matching how Ruffle
/// resolves device fonts.
#[derive(Clone, Eq, Hash, PartialEq)]
struct FaceKey {
    family_lower: String,
    bold: bool,
    italic: bool,
}

/// A single face inside the registry. The byte buffer is shared across
/// every face extracted from the same TTC/OTC file via `Rc`, so each
/// collection pays for its bytes exactly once regardless of how many
/// faces it contributes.
#[derive(Clone)]
struct RegisteredFace {
    /// Raw font bytes. Shared so multiple renderers (and the kern-table
    /// scan path) can borrow from the same allocation.
    data: Rc<Vec<u8>>,
    /// Offset inside `data` at which this face begins. Always 0 for
    /// plain TTF/OTF; non-zero for entries inside a TTC/OTC.
    offset: u32,
    /// Precomputed from the face at registration time so that
    /// `hasKerningInfo` is a cheap boolean read with no re-parse.
    has_kern_table: bool,
    /// Design units per em. Cached so that metric scaling in
    /// `getFontMetrics` doesn't touch the font tables on every call.
    units_per_em: u16,
}

thread_local! {
    static REGISTRY: RefCell<HashMap<FaceKey, RegisteredFace>> =
        RefCell::new(HashMap::new());
}

fn registry_insert(key: FaceKey, face: RegisteredFace) {
    REGISTRY.with(|r| {
        r.borrow_mut().insert(key, face);
    });
}

fn registry_lookup(key: &FaceKey) -> Option<RegisteredFace> {
    REGISTRY.with(|r| r.borrow().get(key).cloned())
}

// ---------------------------------------------------------------------
// Public API — `FontBridge` contract
// ---------------------------------------------------------------------

/// Parse every face inside the provided TTF/OTF/TTC/OTC blob and index
/// it by `(family, bold, italic)` into the module-global registry.
///
/// Invoked by Ruffle once per entry in the `fontSources` config option
/// before any call to `createRenderer`. Never throws: parse failures
/// are logged to the browser console and the call returns silently,
/// which matches Ruffle's behaviour for its own embedded path.
#[wasm_bindgen(js_name = registerFontData)]
pub fn register_font_data(source: &str, bytes: Uint8Array) {
    // Copy the bytes out of the JS ArrayBuffer once. Every face we
    // extract from this blob will share the resulting Rc.
    let data: Rc<Vec<u8>> = Rc::new(bytes.to_vec());

    let face_count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);

    let mut registered = 0u32;
    for index in 0..face_count {
        let face = match ttf_parser::Face::parse(&data, index) {
            Ok(face) => face,
            Err(err) => {
                web_log_warn(&format!(
                    "swash-font-renderer: failed to parse face #{index} from \"{source}\": {err}",
                ));
                continue;
            }
        };

        let Some(family) = pick_family_name(&face) else {
            web_log_warn(&format!(
                "swash-font-renderer: face #{index} from \"{source}\" has no usable family name, skipping"
            ));
            continue;
        };
        let bold = face.is_bold();
        let italic = face.is_italic();
        let has_kern_table = face.tables().kern.is_some();
        let units_per_em = face.units_per_em();

        let key = FaceKey {
            family_lower: family.to_lowercase(),
            bold,
            italic,
        };
        registry_insert(
            key,
            RegisteredFace {
                data: Rc::clone(&data),
                offset: index,
                has_kern_table,
                units_per_em,
            },
        );
        registered += 1;
    }

    if registered == 0 {
        web_log_warn(&format!(
            "swash-font-renderer: no usable faces found in \"{source}\" ({face_count} candidates)",
        ));
    }
}

/// Create a renderer for `(family, bold, italic)`. Returns `None` (i.e.
/// JS `null`) when the family was never registered, so Ruffle can fall
/// back to its canvas device-font path without surfacing an error.
#[wasm_bindgen(js_name = createRenderer)]
pub fn create_renderer(
    family: &str,
    bold: bool,
    italic: bool,
) -> Option<CustomFontRenderer> {
    let key = FaceKey {
        family_lower: family.to_lowercase(),
        bold,
        italic,
    };
    let face = registry_lookup(&key)?;
    Some(CustomFontRenderer::new(face))
}

// ---------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------

/// Metrics for a specific pixel size, computed once per size and
/// reused across every subsequent `getFontMetrics` call at that size.
struct SizedMetrics {
    ascent: f32,
    descent: f32,
    leading: f32,
}

/// A rasterized glyph held in the per-size cache. Stored pre-fanned out
/// to RGBA premultiplied white so that every subsequent
/// `renderGlyph` call for the same (codepoint, size) is a straight
/// Uint8Array copy.
struct CachedGlyph {
    width: u32,
    height: u32,
    bitmap_tx: f32,
    advance: f32,
    pixels: Vec<u8>,
}

/// Per-size cache entry. Glyph entries of `None` represent
/// "rendered-but-empty" glyphs — we keep them so we don't retry
/// rasterization on every call.
#[derive(Default)]
struct SizedFont {
    metrics: Option<SizedMetrics>,
    glyphs: HashMap<u32, Option<CachedGlyph>>,
}

/// Per-family/style renderer instance, as returned by
/// `createRenderer`. Owns a shared handle to the font bytes plus the
/// per-size rasterization cache.
#[wasm_bindgen]
pub struct CustomFontRenderer {
    face: RegisteredFace,
    /// Precomputed charmap descriptor. Cheap to materialize against a
    /// fresh `FontRef`, so we don't need to pin the `FontRef` itself.
    charmap_proxy: CharmapProxy,
    /// Same idea as `charmap_proxy`, for `getFontMetrics`.
    metrics_proxy: MetricsProxy,
    /// Reused across every rasterization to avoid rebuilding the
    /// hinting cache from scratch on every glyph.
    scale_context: ScaleContext,
    by_size: HashMap<u32, SizedFont>,
}

impl CustomFontRenderer {
    fn new(face: RegisteredFace) -> Self {
        // These proxies are lightweight: they capture a handful of
        // table offsets and can be materialized against any matching
        // `FontRef` later.
        let (charmap_proxy, metrics_proxy) = {
            let font_ref = FontRef::from_offset(&face.data, face.offset)
                .expect("face was validated during registerFontData");
            (
                CharmapProxy::from_font(&font_ref),
                MetricsProxy::from_font(&font_ref),
            )
        };
        Self {
            face,
            charmap_proxy,
            metrics_proxy,
            scale_context: ScaleContext::new(),
            by_size: HashMap::new(),
        }
    }

    /// Rebuild a `FontRef` from the stored bytes. Cheap — just parses
    /// a directory header.
    fn font_ref(&self) -> FontRef<'_> {
        FontRef::from_offset(&self.face.data, self.face.offset)
            .expect("face was validated during registerFontData")
    }

    /// Render a glyph at `size_px` and populate the cache entry. Does
    /// nothing if the entry already exists. Missing glyphs (no such
    /// codepoint in the charmap) are cached as `None`.
    fn ensure_glyph(&mut self, codepoint: u32, size_px: u32) {
        // Fast path: nothing to do if the entry already exists.
        if let Some(sized) = self.by_size.get(&size_px)
            && sized.glyphs.contains_key(&codepoint)
        {
            return;
        }

        // Make sure we have ascent/descent at this size — the
        // rasterizer needs them to pad the tight swash bitmap up to
        // Ruffle's expected line-box convention.
        self.ensure_metrics(size_px);
        let (ascent_px, descent_px) = {
            let m = self
                .by_size
                .get(&size_px)
                .and_then(|sf| sf.metrics.as_ref())
                .expect("ensure_metrics populated the entry");
            (m.ascent.round() as i32, m.descent.round() as i32)
        };

        // Build the FontRef via direct field access rather than
        // `self.font_ref()` so that the borrow checker sees disjoint
        // borrows of `self.face.data` (shared) and
        // `self.scale_context` (mutable) further down.
        let font_ref = FontRef::from_offset(&self.face.data, self.face.offset)
            .expect("face was validated during registerFontData");
        let glyph_id = self.charmap_proxy.materialize(&font_ref).map(codepoint);
        // `map(..)` returns 0 for unmapped codepoints; in TrueType that
        // glyph id is the `.notdef` glyph. We forward that to Ruffle as
        // `None` so that the caller can fall back to another font.
        let entry = if glyph_id == 0 {
            None
        } else {
            render_glyph_entry(
                &mut self.scale_context,
                &font_ref,
                glyph_id,
                size_px as f32,
                ascent_px,
                descent_px,
            )
        };

        self.by_size
            .entry(size_px)
            .or_default()
            .glyphs
            .insert(codepoint, entry);
    }

    /// Materialize metrics for `size_px` exactly once, then reuse.
    fn ensure_metrics(&mut self, size_px: u32) {
        if self
            .by_size
            .get(&size_px)
            .map(|sf| sf.metrics.is_some())
            .unwrap_or(false)
        {
            return;
        }
        let font_ref = self.font_ref();
        let metrics = self
            .metrics_proxy
            .materialize_metrics(&font_ref, &[])
            .scale(size_px as f32);
        self.by_size.entry(size_px).or_default().metrics = Some(SizedMetrics {
            ascent: metrics.ascent,
            descent: metrics.descent,
            leading: metrics.leading,
        });
    }
}

#[wasm_bindgen]
impl CustomFontRenderer {
    #[wasm_bindgen(js_name = hasKerningInfo)]
    pub fn has_kerning_info(&self) -> bool {
        self.face.has_kern_table
    }

    #[wasm_bindgen(js_name = getFontMetrics)]
    pub fn get_font_metrics(&mut self, size_px: f64) -> JsValue {
        let size_px = sanitize_size(size_px);
        self.ensure_metrics(size_px);

        let m = self
            .by_size
            .get(&size_px)
            .and_then(|sf| sf.metrics.as_ref())
            .expect("ensure_metrics populated the entry");

        let obj = Object::new();
        set_number(&obj, "ascent", m.ascent as f64);
        set_number(&obj, "descent", m.descent as f64);
        set_number(&obj, "leading", m.leading as f64);
        obj.into()
    }

    #[wasm_bindgen(js_name = renderGlyph)]
    pub fn render_glyph(&mut self, code_point: u32, size_px: f64) -> JsValue {
        let size_px = sanitize_size(size_px);
        self.ensure_glyph(code_point, size_px);

        let Some(sized) = self.by_size.get(&size_px) else {
            return JsValue::NULL;
        };
        let Some(Some(glyph)) = sized.glyphs.get(&code_point) else {
            return JsValue::NULL;
        };

        // Build the plain JS object the FontBridge contract expects.
        // `Uint8Array::from(&[u8])` performs the wasm-memory-to-JS copy
        // in a single shot.
        let pixels = Uint8Array::from(glyph.pixels.as_slice());
        let obj = Object::new();
        set_number(&obj, "width", glyph.width as f64);
        set_number(&obj, "height", glyph.height as f64);
        set_number(&obj, "bitmapTx", glyph.bitmap_tx as f64);
        set_number(&obj, "advance", glyph.advance as f64);
        let _ = Reflect::set(&obj, &JsValue::from_str("pixels"), &pixels.into());
        obj.into()
    }

    #[wasm_bindgen(js_name = calculateKerning)]
    pub fn calculate_kerning(&self, left: u32, right: u32, size_px: f64) -> f64 {
        if !self.face.has_kern_table {
            return 0.0;
        }
        let size_px = sanitize_size(size_px);

        // Kerning lookup uses ttf-parser directly because swash only
        // exposes pair kerning through its shaper, which is overkill
        // for a single pair query.
        let Ok(face) = ttf_parser::Face::parse(&self.face.data, self.face.offset) else {
            return 0.0;
        };

        let left_gid = match face.glyph_index(match char::from_u32(left) {
            Some(c) => c,
            None => return 0.0,
        }) {
            Some(g) => g,
            None => return 0.0,
        };
        let right_gid = match face.glyph_index(match char::from_u32(right) {
            Some(c) => c,
            None => return 0.0,
        }) {
            Some(g) => g,
            None => return 0.0,
        };

        // Iterate subtables: the first one that reports a value wins,
        // same convention Ruffle uses in the embedded path.
        let Some(kern) = face.tables().kern else {
            return 0.0;
        };
        for subtable in kern.subtables {
            if !subtable.horizontal || subtable.variable {
                continue;
            }
            if let Some(units) = subtable.glyphs_kerning(left_gid, right_gid) {
                // Design-units to pixels: value * (ppem / units_per_em).
                let upem = self.face.units_per_em as f32;
                if upem <= 0.0 {
                    return 0.0;
                }
                let scale = size_px as f32 / upem;
                return (units as f32 * scale) as f64;
            }
        }
        0.0
    }

    #[wasm_bindgen(js_name = destroy)]
    pub fn destroy(&mut self) {
        // Drop every cached glyph and metrics entry immediately so the
        // memory is returned to the WASM allocator without waiting for
        // JS to finalize the handle.
        self.by_size.clear();
    }
}

// ---------------------------------------------------------------------
// Rasterization helpers
// ---------------------------------------------------------------------

/// Rasterize a single glyph via swash and produce a bitmap that
/// matches the convention Ruffle's canvas device-font renderer uses:
/// the output is `ink_width × (ascent + descent)` and the glyph is
/// positioned so that its baseline lands on row `ascent` from the top.
///
/// This is necessary because Ruffle's `GlyphRaster` contract has no
/// vertical-offset field: the core pipeline assumes the bitmap already
/// encodes the full line-box and pastes it at `(pen_x + bitmapTx,
/// line_top)` without further vertical adjustment. Swash by default
/// returns a *tight* bitmap containing only the ink pixels plus a
/// separate `placement.top` offset, so we have to pad it ourselves
/// here.
///
/// Returns `None` when swash refuses to produce an image for this
/// glyph (typically the `.notdef` hit above, or a degenerate outline).
fn render_glyph_entry(
    scale_context: &mut ScaleContext,
    font_ref: &FontRef<'_>,
    glyph_id: u16,
    ppem: f32,
    ascent_px: i32,
    descent_px: i32,
) -> Option<CachedGlyph> {
    let mut scaler = scale_context
        .builder(*font_ref)
        .size(ppem)
        .hint(true)
        .build();

    let mut image = Image::new();
    let rendered = Render::new(&[
        Source::Outline,
        Source::Bitmap(StrikeWith::BestFit),
    ])
    .format(Format::Alpha)
    .render_into(&mut scaler, glyph_id, &mut image);
    if !rendered {
        return None;
    }

    // Horizontal advance in pixels. `GlyphMetrics::advance_width` is
    // in design units; `linear_scale(ppem / upem)` gets us pixels.
    let upem = font_ref.metrics(&[]).units_per_em as f32;
    let advance = if upem > 0.0 {
        font_ref
            .glyph_metrics(&[])
            .linear_scale(ppem / upem)
            .advance_width(glyph_id)
    } else {
        0.0
    };

    let placement = image.placement;
    // `placement.left` is the signed horizontal offset from the pen
    // position to the left edge of the ink bitmap, exactly what
    // Ruffle's `bitmapTx` wants.
    let bitmap_tx = placement.left as f32;

    // Output dimensions. Width is the tight ink width (matches what
    // canvas returns via `actual_bounding_box`); height is the full
    // line box so Ruffle's vertical-offset-less bitmap placement lands
    // the baseline in the right spot.
    let out_w = placement.width.max(1);
    let line_height = (ascent_px + descent_px).max(1) as u32;
    let out_h = line_height;

    // Row in the output where input row 0 lands. Positive when the
    // glyph's top sits below the ascent line (the common case);
    // becomes negative only for glyphs that extend above the ascent,
    // in which case those top rows are clipped by the loop below.
    let dst_y_offset = ascent_px - placement.top;

    let mut pixels = vec![0u8; (out_w as usize) * (out_h as usize) * 4];

    if matches!(image.content, Content::Mask) && !image.data.is_empty() {
        // Fan out alpha → RGBA premultiplied white, placing each input
        // row at the correct row in the padded output.
        let src_w = placement.width as usize;
        let src_h = placement.height as i32;
        let out_w_usize = out_w as usize;
        let out_h_i32 = out_h as i32;
        let copy_w = src_w.min(out_w_usize);

        for src_y in 0..src_h {
            let dst_y = src_y + dst_y_offset;
            if dst_y < 0 || dst_y >= out_h_i32 {
                continue;
            }
            let src_row_start = (src_y as usize) * src_w;
            let dst_row_start = (dst_y as usize) * out_w_usize * 4;
            for x in 0..copy_w {
                let src_idx = src_row_start + x;
                if src_idx >= image.data.len() {
                    break;
                }
                let a = image.data[src_idx];
                let base = dst_row_start + x * 4;
                pixels[base] = a;
                pixels[base + 1] = a;
                pixels[base + 2] = a;
                pixels[base + 3] = a;
            }
        }
    }

    Some(CachedGlyph {
        width: out_w,
        height: out_h,
        bitmap_tx,
        advance,
        pixels,
    })
}

// ---------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------

/// Pick the best family name out of a ttf-parser face. We prefer the
/// typographic family name (name id 16) when present because it groups
/// all weight/italic variants under a single name, matching how users
/// (and Flash) reference families. Fall back to the regular family
/// name (id 1) otherwise.
fn pick_family_name(face: &ttf_parser::Face<'_>) -> Option<String> {
    const TYPOGRAPHIC_FAMILY_ID: u16 = 16;
    const FAMILY_ID: u16 = 1;

    let mut family: Option<String> = None;
    let mut typographic: Option<String> = None;

    for name in face.names() {
        if !name.is_unicode() {
            continue;
        }
        let Some(text) = name.to_string() else {
            continue;
        };
        match name.name_id {
            TYPOGRAPHIC_FAMILY_ID => {
                typographic.get_or_insert(text);
            }
            FAMILY_ID => {
                family.get_or_insert(text);
            }
            _ => continue,
        }
    }
    typographic.or(family)
}

/// Clamp and round an incoming pixel size to a sane `u32`. Sizes
/// arriving from Ruffle's proxy are already integer in practice, but
/// the contract exposes them as `number`, so we defend against NaN,
/// negative and fractional values here.
fn sanitize_size(size_px: f64) -> u32 {
    if !size_px.is_finite() || size_px <= 0.0 {
        return 1;
    }
    let rounded = size_px.round();
    if rounded <= 1.0 {
        1
    } else if rounded >= u32::MAX as f64 {
        u32::MAX
    } else {
        rounded as u32
    }
}

fn set_number(obj: &Object, key: &str, value: f64) {
    let _ = Reflect::set(obj, &JsValue::from_str(key), &JsValue::from_f64(value));
}

fn web_log_warn(msg: &str) {
    // Avoid dragging in the whole `web-sys` crate just for a console
    // log — we use the js-sys reflection path already pulled in.
    let console = match Reflect::get(&js_sys::global(), &JsValue::from_str("console")) {
        Ok(c) => c,
        Err(_) => return,
    };
    let warn_fn = match Reflect::get(&console, &JsValue::from_str("warn")) {
        Ok(f) => f,
        Err(_) => return,
    };
    if let Some(f) = warn_fn.dyn_ref::<js_sys::Function>() {
        let _ = f.call1(&console, &JsValue::from_str(msg));
    }
}
